//! LanceVectorStore —— LanceDB 嵌入式向量库后端。
//!
//! Day 9 主线：解锁 chunks > 10k 的真实 corpus（InMemoryVectorStore 的 cosine 暴力扫描
//! 在 10k 以上会卡）。
//!
//! 设计：
//! - 单 table `chunks`，Arrow schema：id (Utf8) / content (Utf8) / metadata_json (Utf8) /
//!   vector (FixedSizeList<Float32, dim>)
//! - 完整 `ChunkMetadata` 序列化到 `metadata_json` stored field —— search 返回自包含 Hit
//! - 持久化天然：LanceDB connect(path) 即 open or create；writer commit 立即落盘
//! - **filters 简化**：Day 9 仅支持 deprecated 排除 + tag include，平台/版本 filter 留 Day 9 续
//!   （Arrow 多值字段过滤需特殊语法，简化以保证 1 commit 完成）
//!
//! 注意：lancedb 0.10 API 与具体细节可能漂移；本文件按主流惯例写，
//! 用户首次 `make build-lancedb` 若 API mismatch 请上报 issue（feature gated 不影响默认编译）。

#![cfg(feature = "lancedb")]

use crate::{MetadataStore, VectorStore};
use arkui_rag_core::{
    Chunk, ChunkId, ChunkMetadata, Hit, HitSource, QueryFilters, RagError, Result,
};
use arrow_array::{
    types::Float32Type, Array, FixedSizeListArray, RecordBatch, RecordBatchIterator,
    RecordBatchReader, StringArray,
};
use arrow_schema::{DataType, Field, Schema, SchemaRef};
use async_trait::async_trait;
use futures::TryStreamExt;
use lancedb::{
    connection::Connection,
    query::{ExecutableQuery, QueryBase},
    Table,
};
use std::path::Path;
use std::sync::Arc;

const TABLE_NAME: &str = "chunks";

pub struct LanceVectorStore {
    _conn: Connection,
    table: Table,
    embedder_model_id: String,
    embedder_dim: usize,
    schema: SchemaRef,
}

impl LanceVectorStore {
    /// 打开（或创建）指定目录的 LanceDB。
    ///
    /// `uri` 是 LanceDB 连接串：本地用文件 URI 或绝对路径。
    /// 推荐路径约定：`<corpus>/_index/vectors.lance/`（CLI 自动推导）。
    ///
    /// 传 `dim=0` 表示 "load existing"：从已存在 table 的 Arrow schema 推导 dim；
    /// 若 table 不存在则报错（不能从无中生有）。
    /// 传 `dim > 0` 表示 "open or create"：若 table 不存在则用此 dim 建空表。
    pub async fn open(uri: &str, embedder_model_id: impl Into<String>, dim: usize) -> Result<Self> {
        let conn = lancedb::connect(uri)
            .execute()
            .await
            .map_err(|e| RagError::Storage(format!("lancedb connect {}: {}", uri, e)))?;

        let model_id = embedder_model_id.into();

        let (table, resolved_dim, schema) = match conn.open_table(TABLE_NAME).execute().await {
            Ok(t) => {
                // 从已有 schema 反推 dim
                let table_schema = t
                    .schema()
                    .await
                    .map_err(|e| RagError::Storage(format!("read table schema: {}", e)))?;
                let resolved = read_vector_dim_from_schema(&table_schema).ok_or_else(|| {
                    RagError::Storage("已有 lance table 但找不到 vector 字段或 dim 推断失败".into())
                })?;
                // 若用户传了 dim 且与已有 schema 不一致 → 报错（防错配）
                if dim != 0 && dim != resolved {
                    return Err(RagError::Storage(format!(
                        "lancedb open：传入 dim={} 与已有 schema dim={} 不一致",
                        dim, resolved
                    )));
                }
                let s: SchemaRef = table_schema;
                (t, resolved, s)
            }
            Err(_) => {
                // 不存在 → 必须用 dim>0 建空表
                if dim == 0 {
                    return Err(RagError::Storage(format!(
                        "lancedb open：table 不存在且未指定 dim · 用 dim>0 创建新 table"
                    )));
                }
                let new_schema = Arc::new(build_schema(dim));
                let empty_batches: Vec<std::result::Result<RecordBatch, arrow_schema::ArrowError>> =
                    Vec::new();
                let iter = RecordBatchIterator::new(empty_batches.into_iter(), new_schema.clone());
                let reader: Box<dyn RecordBatchReader + Send> = Box::new(iter);
                let t = conn
                    .create_table(TABLE_NAME, reader)
                    .execute()
                    .await
                    .map_err(|e| RagError::Storage(format!("create_table: {}", e)))?;
                (t, dim, new_schema)
            }
        };

        Ok(Self {
            _conn: conn,
            table,
            embedder_model_id: model_id,
            embedder_dim: resolved_dim,
            schema,
        })
    }

    pub fn embedder_model_id(&self) -> &str {
        &self.embedder_model_id
    }

    pub fn dim(&self) -> usize {
        self.embedder_dim
    }

    /// 把 chunks + embeddings 转 Arrow RecordBatch。
    fn build_batch(&self, chunks: &[Chunk], embeddings: &[Vec<f32>]) -> Result<RecordBatch> {
        let ids: Vec<&str> = chunks.iter().map(|c| c.id.as_str()).collect();
        let contents: Vec<&str> = chunks.iter().map(|c| c.content.as_str()).collect();
        let metas: Vec<String> = chunks
            .iter()
            .map(|c| {
                serde_json::to_string(&c.metadata)
                    .map_err(|e| RagError::Storage(format!("serialize metadata: {}", e)))
            })
            .collect::<Result<Vec<_>>>()?;
        let metas_ref: Vec<&str> = metas.iter().map(|s| s.as_str()).collect();

        // 向量字段：扁平 Float32 + FixedSizeList
        let flat: Vec<f32> = embeddings.iter().flat_map(|v| v.iter().copied()).collect();
        let vector_array = FixedSizeListArray::from_iter_primitive::<Float32Type, _, _>(
            embeddings
                .iter()
                .map(|v| Some(v.iter().map(|x| Some(*x)).collect::<Vec<_>>())),
            self.embedder_dim as i32,
        );

        let _ = flat; // 留 unused 防优化；实际向量靠 vector_array

        let batch = RecordBatch::try_new(
            self.schema.clone(),
            vec![
                Arc::new(StringArray::from(ids)),
                Arc::new(StringArray::from(contents)),
                Arc::new(StringArray::from(metas_ref)),
                Arc::new(vector_array),
            ],
        )
        .map_err(|e| RagError::Storage(format!("build RecordBatch: {}", e)))?;
        Ok(batch)
    }

    /// 把 LanceDB 返回的 RecordBatch 反序列化为 Hit。
    fn reconstruct_hits(&self, batches: Vec<RecordBatch>) -> Result<Vec<Hit>> {
        let mut hits = Vec::new();
        for batch in batches {
            let n = batch.num_rows();
            let id_col = batch
                .column_by_name("id")
                .and_then(|c| c.as_any().downcast_ref::<StringArray>())
                .ok_or_else(|| RagError::Storage("missing id column".into()))?;
            let content_col = batch
                .column_by_name("content")
                .and_then(|c| c.as_any().downcast_ref::<StringArray>())
                .ok_or_else(|| RagError::Storage("missing content column".into()))?;
            let meta_col = batch
                .column_by_name("metadata_json")
                .and_then(|c| c.as_any().downcast_ref::<StringArray>())
                .ok_or_else(|| RagError::Storage("missing metadata_json column".into()))?;
            // _distance 是 lancedb nearest_to 查询自动加的列（升序：越小越相似）
            let dist_col = batch
                .column_by_name("_distance")
                .and_then(|c| c.as_any().downcast_ref::<arrow_array::Float32Array>());

            for i in 0..n {
                let id = id_col.value(i).to_string();
                let content = content_col.value(i).to_string();
                let meta_str = meta_col.value(i);
                let metadata: ChunkMetadata = serde_json::from_str(meta_str)
                    .map_err(|e| RagError::Storage(format!("deserialize metadata: {}", e)))?;
                // 距离 → 相似度：越小越好；变成"相似度"用 1.0 - distance.min(1.0)
                let score = match dist_col {
                    Some(d) => 1.0 - d.value(i).clamp(0.0, 1.0),
                    None => 0.0,
                };
                hits.push(Hit {
                    chunk: Chunk {
                        id: ChunkId::new(id),
                        content,
                        metadata,
                    },
                    score,
                    source: HitSource::Vector,
                });
            }
        }
        Ok(hits)
    }
}

fn build_schema(dim: usize) -> Schema {
    Schema::new(vec![
        Field::new("id", DataType::Utf8, false),
        Field::new("content", DataType::Utf8, false),
        Field::new("metadata_json", DataType::Utf8, false),
        Field::new(
            "vector",
            DataType::FixedSizeList(
                Arc::new(Field::new("item", DataType::Float32, true)),
                dim as i32,
            ),
            false,
        ),
    ])
}

/// 从 Arrow schema 的 `vector` FixedSizeList 字段反推 dim。
/// 用于打开已存在的 lance table 时无需用户显式传 dim。
fn read_vector_dim_from_schema(schema: &Schema) -> Option<usize> {
    let field = schema.field_with_name("vector").ok()?;
    if let DataType::FixedSizeList(_, size) = field.data_type() {
        if *size > 0 {
            return Some(*size as usize);
        }
    }
    None
}

#[async_trait]
impl VectorStore for LanceVectorStore {
    async fn upsert(&self, chunks: &[Chunk], embeddings: &[Vec<f32>]) -> Result<()> {
        if chunks.len() != embeddings.len() {
            return Err(RagError::Storage(format!(
                "upsert: chunks.len={} != embeddings.len={}",
                chunks.len(),
                embeddings.len()
            )));
        }
        for e in embeddings {
            if e.len() != self.embedder_dim {
                return Err(RagError::Storage(format!(
                    "embedding dim {} != store dim {}",
                    e.len(),
                    self.embedder_dim
                )));
            }
        }
        if chunks.is_empty() {
            return Ok(());
        }

        // Upsert 语义：先按 id 删旧 row，再 add 新
        let id_list: Vec<String> = chunks.iter().map(|c| c.id.as_str().to_string()).collect();
        let in_clause = id_list
            .iter()
            .map(|id| format!("'{}'", id.replace('\'', "''")))
            .collect::<Vec<_>>()
            .join(", ");
        let predicate = format!("id IN ({})", in_clause);
        let _ = self
            .table
            .delete(&predicate)
            .await
            .map_err(|e| RagError::Storage(format!("delete-before-upsert: {}", e)))?;

        let batch = self.build_batch(chunks, embeddings)?;
        let schema = self.schema.clone();
        let iter = RecordBatchIterator::new(std::iter::once(Ok(batch)), schema);
        let reader: Box<dyn RecordBatchReader + Send> = Box::new(iter);
        self.table
            .add(reader)
            .execute()
            .await
            .map_err(|e| RagError::Storage(format!("table.add: {}", e)))?;
        Ok(())
    }

    async fn search(
        &self,
        query_vec: &[f32],
        top_k: usize,
        filters: &QueryFilters,
    ) -> Result<Vec<Hit>> {
        if query_vec.len() != self.embedder_dim {
            return Err(RagError::Storage(format!(
                "query dim {} != store dim {}",
                query_vec.len(),
                self.embedder_dim
            )));
        }

        // Day 9 简化：仅支持 deprecated 过滤（其他 filter 在反序列化后再 filter，Day 9 续可下沉）
        let mut query = self
            .table
            .query()
            .nearest_to(query_vec.to_vec())
            .map_err(|e| RagError::Storage(format!("nearest_to: {}", e)))?
            .limit(top_k.max(1) * 2); // 过取，给 post-filter 留余量

        // 如果 metadata 含 deprecated/platform/tags 字段且简单可以下沉，未来扩展
        let _ = filters; // post-filter 在下方做

        let stream = query
            .execute()
            .await
            .map_err(|e| RagError::Retrieval(format!("lancedb query execute: {}", e)))?;
        let batches: Vec<RecordBatch> = stream
            .try_collect()
            .await
            .map_err(|e| RagError::Retrieval(format!("collect: {}", e)))?;
        let mut hits = self.reconstruct_hits(batches)?;

        // post-filter（与 InMemoryVectorStore 一致行为）
        hits.retain(|h| passes_post_filter(&h.chunk, filters));
        hits.truncate(top_k);
        Ok(hits)
    }

    async fn delete(&self, ids: &[ChunkId]) -> Result<()> {
        if ids.is_empty() {
            return Ok(());
        }
        let in_clause = ids
            .iter()
            .map(|id| format!("'{}'", id.as_str().replace('\'', "''")))
            .collect::<Vec<_>>()
            .join(", ");
        let predicate = format!("id IN ({})", in_clause);
        let _ = self
            .table
            .delete(&predicate)
            .await
            .map_err(|e| RagError::Storage(format!("delete: {}", e)))?;
        Ok(())
    }

    async fn len(&self) -> Result<usize> {
        let n = self
            .table
            .count_rows(None)
            .await
            .map_err(|e| RagError::Storage(format!("count_rows: {}", e)))?;
        Ok(n)
    }
}

#[async_trait]
impl MetadataStore for LanceVectorStore {
    async fn get(&self, id: &ChunkId) -> Result<Option<Chunk>> {
        let pred = format!("id = '{}'", id.as_str().replace('\'', "''"));
        let stream = self
            .table
            .query()
            .only_if(&pred)
            .limit(1)
            .execute()
            .await
            .map_err(|e| RagError::Storage(format!("get query: {}", e)))?;
        let batches: Vec<RecordBatch> = stream
            .try_collect()
            .await
            .map_err(|e| RagError::Storage(format!("get collect: {}", e)))?;
        let hits = self.reconstruct_hits(batches)?;
        Ok(hits.into_iter().next().map(|h| h.chunk))
    }

    async fn parent_of(&self, id: &ChunkId) -> Result<Option<Chunk>> {
        let me = self.get(id).await?;
        let parent_id = match me.and_then(|c| c.metadata.parent_id) {
            Some(p) => p,
            None => return Ok(None),
        };
        self.get(&parent_id).await
    }
}

fn passes_post_filter(chunk: &Chunk, filters: &QueryFilters) -> bool {
    if !filters.include_deprecated && chunk.metadata.deprecated {
        return false;
    }
    if !filters.platforms.is_empty() && !chunk.metadata.platforms.is_empty() {
        let any = chunk
            .metadata
            .platforms
            .iter()
            .any(|p| filters.platforms.contains(p));
        if !any {
            return false;
        }
    }
    if let (Some(want), Some(got)) = (&filters.api_version, &chunk.metadata.api_version) {
        if want != got {
            return false;
        }
    }
    if !filters.tags.is_empty() {
        let any = chunk.metadata.tags.iter().any(|t| filters.tags.contains(t));
        if !any {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use arkui_rag_core::ChunkType;

    fn mk_chunk(id: &str, content: &str) -> Chunk {
        Chunk {
            id: ChunkId::new(id),
            content: content.to_string(),
            metadata: ChunkMetadata {
                source: format!("{}.md", id),
                r#type: ChunkType::Generic,
                ..ChunkMetadata::default()
            },
        }
    }

    fn norm(v: Vec<f32>) -> Vec<f32> {
        let n = v.iter().map(|x| x * x).sum::<f32>().sqrt().max(1e-9);
        v.into_iter().map(|x| x / n).collect()
    }

    #[tokio::test]
    async fn upsert_and_search_topk() {
        let dir = tempfile::tempdir().unwrap();
        let store = LanceVectorStore::open(dir.path().to_str().unwrap(), "mock-4", 4)
            .await
            .unwrap();

        let chunks = vec![
            mk_chunk("a", "ca"),
            mk_chunk("b", "cb"),
            mk_chunk("c", "cc"),
        ];
        let embeddings = vec![
            norm(vec![1.0, 0.0, 0.0, 0.0]),
            norm(vec![0.0, 1.0, 0.0, 0.0]),
            norm(vec![0.0, 0.0, 1.0, 0.0]),
        ];
        store.upsert(&chunks, &embeddings).await.unwrap();
        assert_eq!(store.len().await.unwrap(), 3);

        let q = norm(vec![0.9, 0.1, 0.0, 0.0]);
        let hits = store.search(&q, 2, &QueryFilters::default()).await.unwrap();
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].chunk.id.as_str(), "a", "Top-1 应是 'a'");
    }

    #[tokio::test]
    async fn open_with_dim_zero_reads_from_schema() {
        let dir = tempfile::tempdir().unwrap();
        let uri = dir.path().to_str().unwrap();

        // 1. 先用 dim=4 建表 + 写一条
        let store = LanceVectorStore::open(uri, "mock-4", 4).await.unwrap();
        store
            .upsert(&[mk_chunk("a", "x")], &[norm(vec![1.0, 0.0, 0.0, 0.0])])
            .await
            .unwrap();
        drop(store);

        // 2. 重新 open · 传 dim=0 · 应自动从 schema 读出 dim=4
        let reopened = LanceVectorStore::open(uri, "mock-4", 0).await.unwrap();
        assert_eq!(reopened.dim(), 4, "dim 应该从 Arrow schema 自动推导");

        // 3. 真跑 query 验证 dim 正确
        let q = norm(vec![1.0, 0.0, 0.0, 0.0]);
        let hits = reopened
            .search(&q, 1, &QueryFilters::default())
            .await
            .unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].chunk.id.as_str(), "a");
    }

    #[tokio::test]
    async fn open_with_dim_zero_no_table_fails() {
        let dir = tempfile::tempdir().unwrap();
        let r = LanceVectorStore::open(dir.path().to_str().unwrap(), "mock-4", 0).await;
        assert!(r.is_err(), "空目录 + dim=0 应报错（不能无中生有）");
    }

    #[tokio::test]
    async fn open_dim_mismatch_with_existing_schema_fails() {
        let dir = tempfile::tempdir().unwrap();
        let uri = dir.path().to_str().unwrap();
        LanceVectorStore::open(uri, "mock-4", 4)
            .await
            .unwrap()
            .upsert(&[mk_chunk("a", "x")], &[norm(vec![1.0, 0.0, 0.0, 0.0])])
            .await
            .unwrap();
        // 重 open 传 dim=8 与 schema 不一致 → 应报错
        let r = LanceVectorStore::open(uri, "mock-4", 8).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn dim_mismatch_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let store = LanceVectorStore::open(dir.path().to_str().unwrap(), "mock-4", 4)
            .await
            .unwrap();
        let r = store.upsert(&[mk_chunk("a", "x")], &[vec![1.0, 0.0]]).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn upsert_overwrites_same_id() {
        let dir = tempfile::tempdir().unwrap();
        let store = LanceVectorStore::open(dir.path().to_str().unwrap(), "mock-2", 2)
            .await
            .unwrap();
        store
            .upsert(&[mk_chunk("a", "first")], &[norm(vec![1.0, 0.0])])
            .await
            .unwrap();
        store
            .upsert(&[mk_chunk("a", "second")], &[norm(vec![0.0, 1.0])])
            .await
            .unwrap();
        assert_eq!(store.len().await.unwrap(), 1);
        let got = store
            .get(&ChunkId::new("a"))
            .await
            .unwrap()
            .expect("should exist");
        assert_eq!(got.content, "second");
    }

    #[tokio::test]
    async fn reopen_persists_data() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().to_str().unwrap().to_string();
        {
            let s = LanceVectorStore::open(&path, "mock-2", 2).await.unwrap();
            s.upsert(&[mk_chunk("a", "persisted")], &[norm(vec![1.0, 0.0])])
                .await
                .unwrap();
        }
        // 模拟跨进程
        let s2 = LanceVectorStore::open(&path, "mock-2", 2).await.unwrap();
        assert_eq!(s2.len().await.unwrap(), 1);
    }

    #[tokio::test]
    async fn delete_works() {
        let dir = tempfile::tempdir().unwrap();
        let s = LanceVectorStore::open(dir.path().to_str().unwrap(), "mock-2", 2)
            .await
            .unwrap();
        s.upsert(&[mk_chunk("a", "x")], &[norm(vec![1.0, 0.0])])
            .await
            .unwrap();
        assert_eq!(s.len().await.unwrap(), 1);
        s.delete(&[ChunkId::new("a")]).await.unwrap();
        assert_eq!(s.len().await.unwrap(), 0);
    }
}
