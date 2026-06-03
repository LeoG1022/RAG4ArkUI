//! TantivyBM25Index —— 基于 Tantivy 0.22 的真实 BM25 倒排索引。
//!
//! 设计：
//! - `id`、`content`、`heading_path`：精确/全文 BM25 主体
//! - `source` / `platforms` / `api_version` / `chunk_type` / `tags` / `deprecated`：
//!   STORED + INDEXED 用于元数据过滤
//! - `meta_json`：完整 `ChunkMetadata` 的 JSON 序列化，让 search 返回完整 Hit
//! - **中文分词**：注册 `ngram(2,3)` tokenizer。这是 Day 4 简化方案；
//!   生产建议接 `tantivy-jieba` 第三方（feature 留口子）。
//!
//! 跨进程持久化：Tantivy 写入是 `commit()` 即落盘的，索引目录可被任意进程 `open`，
//! 与 `InMemoryVectorStore::save_to/load_from` 的 JSON 方案天然对齐。

use crate::BM25Index;
use arkui_rag_core::{
    chunk::Platform, Chunk, ChunkId, ChunkMetadata, ChunkType, Hit, HitSource, QueryFilters,
    RagError, Result,
};
use async_trait::async_trait;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tantivy::{
    collector::TopDocs,
    directory::MmapDirectory,
    query::{BooleanQuery, Occur, Query, QueryParser, TermQuery},
    schema::{
        Field, IndexRecordOption, Schema, TextFieldIndexing, TextOptions, FAST, INDEXED, STORED,
        STRING,
    },
    tokenizer::{LowerCaser, NgramTokenizer, RemoveLongFilter, TextAnalyzer},
    Index, IndexReader, IndexWriter, ReloadPolicy, TantivyDocument, Term,
};

/// 注册的中文友好 tokenizer 名。
const TOKENIZER_NAME: &str = "ngram_cn";
const WRITER_HEAP_BYTES: usize = 50_000_000; // 50MB，单进程足够

#[derive(Clone)]
struct Fields {
    id: Field,
    content: Field,
    heading_path: Field,
    source: Field,
    platforms: Field,
    api_version: Field,
    chunk_type: Field,
    tags: Field,
    deprecated: Field,
    meta_json: Field,
}

pub struct TantivyBM25Index {
    index: Index,
    reader: IndexReader,
    /// `None` = 只读模式（`open_read_only` 打开 · 不持 IndexWriter 锁 · 多 instance 共存）
    /// `Some` = 写模式（`open` 打开 · 持独占写锁 · `upsert` 可用）
    writer: Option<Arc<Mutex<IndexWriter>>>,
    fields: Fields,
}

impl TantivyBM25Index {
    /// 打开（或创建）指定目录下的 Tantivy 索引 · **写模式**（持独占 IndexWriter 锁）。
    ///
    /// 用于 `arkui-rag index` 子命令。路径约定：`<corpus>/_index/bm25/`。
    /// 目录不存在会自动创建。同一目录任何时刻只能有一个写者。
    pub fn open(dir: &Path) -> Result<Self> {
        Self::open_with_mode(dir, true)
    }

    /// 只读打开（不持写锁） · 多 instance 共存。
    ///
    /// 用于 `arkui-rag serve --mcp/--http/--lsp` / `query` / `eval` 等只检索路径。
    /// Claude Code (CLI) 和 Claude Desktop 可同时连同一个 binary 而不冲突。
    /// `upsert` 在该模式下返回错误。索引必须已存在（不创建）。
    pub fn open_read_only(dir: &Path) -> Result<Self> {
        Self::open_with_mode(dir, false)
    }

    fn open_with_mode(dir: &Path, writable: bool) -> Result<Self> {
        let schema = build_schema();
        let fields = extract_fields(&schema);

        std::fs::create_dir_all(dir)
            .map_err(|e| RagError::Storage(format!("create bm25 dir {}: {}", dir.display(), e)))?;
        let mmap = MmapDirectory::open(dir)
            .map_err(|e| RagError::Storage(format!("open bm25 dir {}: {}", dir.display(), e)))?;
        let index = Index::open_or_create(mmap, schema)
            .map_err(|e| RagError::Storage(format!("open_or_create index: {}", e)))?;

        // 注册中文友好的 ngram(2,3) tokenizer
        let analyzer = TextAnalyzer::builder(
            NgramTokenizer::new(2, 3, false)
                .map_err(|e| RagError::Storage(format!("ngram tokenizer init: {}", e)))?,
        )
        .filter(RemoveLongFilter::limit(40))
        .filter(LowerCaser)
        .build();
        index.tokenizers().register(TOKENIZER_NAME, analyzer);

        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommitWithDelay)
            .try_into()
            .map_err(|e| RagError::Storage(format!("reader build: {}", e)))?;

        let writer = if writable {
            let w = index
                .writer(WRITER_HEAP_BYTES)
                .map_err(|e| RagError::Storage(format!("writer init: {}", e)))?;
            Some(Arc::new(Mutex::new(w)))
        } else {
            None
        };

        Ok(Self {
            index,
            reader,
            writer,
            fields,
        })
    }

    /// 当前已索引的 doc 数。
    pub fn len(&self) -> u64 {
        self.reader.searcher().num_docs()
    }

    /// 索引是否为空（与 `len` 配套 · 满足 clippy `len_without_is_empty`）。
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

fn build_schema() -> Schema {
    let mut sb = Schema::builder();

    sb.add_text_field("id", STRING | STORED);

    let text_opts = TextOptions::default()
        .set_indexing_options(
            TextFieldIndexing::default()
                .set_tokenizer(TOKENIZER_NAME)
                .set_index_option(IndexRecordOption::WithFreqsAndPositions),
        )
        .set_stored();
    sb.add_text_field("content", text_opts.clone());
    sb.add_text_field("heading_path", text_opts);

    sb.add_text_field("source", STRING | STORED);
    sb.add_text_field("platforms", STRING | STORED); // 多值靠 add_text 多次调用
    sb.add_text_field("api_version", STRING | STORED);
    sb.add_text_field("chunk_type", STRING | STORED);
    sb.add_text_field("tags", STRING | STORED);
    sb.add_bool_field("deprecated", INDEXED | STORED | FAST);
    sb.add_text_field("meta_json", STORED);

    sb.build()
}

fn extract_fields(schema: &Schema) -> Fields {
    Fields {
        id: schema.get_field("id").unwrap(),
        content: schema.get_field("content").unwrap(),
        heading_path: schema.get_field("heading_path").unwrap(),
        source: schema.get_field("source").unwrap(),
        platforms: schema.get_field("platforms").unwrap(),
        api_version: schema.get_field("api_version").unwrap(),
        chunk_type: schema.get_field("chunk_type").unwrap(),
        tags: schema.get_field("tags").unwrap(),
        deprecated: schema.get_field("deprecated").unwrap(),
        meta_json: schema.get_field("meta_json").unwrap(),
    }
}

fn platform_str(p: Platform) -> &'static str {
    match p {
        Platform::HarmonyOs => "harmonyos",
        Platform::Android => "android",
        Platform::Ios => "ios",
    }
}

fn chunk_type_str(t: ChunkType) -> &'static str {
    match t {
        ChunkType::ApiDoc => "api_doc",
        ChunkType::CodeExample => "code_example",
        ChunkType::MigrationRule => "migration_rule",
        ChunkType::ErrorFix => "error_fix",
        ChunkType::Generic => "generic",
    }
}

#[async_trait]
impl BM25Index for TantivyBM25Index {
    async fn upsert(&self, chunks: &[Chunk]) -> Result<()> {
        let writer = self.writer.as_ref().ok_or_else(|| {
            RagError::Storage(
                "TantivyBM25Index 以 open_read_only 模式打开 · upsert 不可用 · \
                 重建索引请用 `arkui-rag index`（持写锁）"
                    .into(),
            )
        })?;
        let mut w = writer.lock().unwrap();
        for chunk in chunks {
            // 先按 id 删旧 doc（保证 upsert 语义）
            let id_term = Term::from_field_text(self.fields.id, chunk.id.as_str());
            w.delete_term(id_term);

            let mut doc = TantivyDocument::default();
            doc.add_text(self.fields.id, chunk.id.as_str());
            doc.add_text(self.fields.content, &chunk.content);
            doc.add_text(
                self.fields.heading_path,
                chunk.metadata.heading_path.join(" / "),
            );
            doc.add_text(self.fields.source, &chunk.metadata.source);
            for p in &chunk.metadata.platforms {
                doc.add_text(self.fields.platforms, platform_str(*p));
            }
            if let Some(v) = &chunk.metadata.api_version {
                doc.add_text(self.fields.api_version, v);
            }
            doc.add_text(
                self.fields.chunk_type,
                chunk_type_str(chunk.metadata.r#type),
            );
            for t in &chunk.metadata.tags {
                doc.add_text(self.fields.tags, t);
            }
            doc.add_bool(self.fields.deprecated, chunk.metadata.deprecated);
            let meta_json = serde_json::to_string(&chunk.metadata)
                .map_err(|e| RagError::Storage(format!("serialize metadata: {}", e)))?;
            doc.add_text(self.fields.meta_json, &meta_json);

            w.add_document(doc)
                .map_err(|e| RagError::Storage(format!("add_document: {}", e)))?;
        }
        w.commit()
            .map_err(|e| RagError::Storage(format!("commit: {}", e)))?;
        // OnCommitWithDelay 是 background reload · 测试 / 立即 query 场景需要同步刷一次
        self.reader
            .reload()
            .map_err(|e| RagError::Storage(format!("reader reload: {}", e)))?;
        Ok(())
    }

    async fn search(&self, query: &str, top_k: usize, filters: &QueryFilters) -> Result<Vec<Hit>> {
        if query.trim().is_empty() {
            return Ok(Vec::new());
        }

        // 重新触发 reload 让最近 commit 可见
        self.reader
            .reload()
            .map_err(|e| RagError::Storage(format!("reader reload: {}", e)))?;
        let searcher = self.reader.searcher();

        let mut parser = QueryParser::for_index(
            &self.index,
            vec![self.fields.content, self.fields.heading_path],
        );
        parser.set_conjunction_by_default(); // 多 token 走 AND，提高精度
        let main_q: Box<dyn Query> = match parser.parse_query(query) {
            Ok(q) => q,
            Err(e) => {
                tracing::warn!("BM25 query parse failed ({}), fallback to lenient", e);
                parser.set_conjunction_by_default();
                Box::new(parser.parse_query_lenient(query).0)
            }
        };

        let mut clauses: Vec<(Occur, Box<dyn Query>)> = vec![(Occur::Must, main_q)];
        for p in &filters.platforms {
            clauses.push((
                Occur::Must,
                Box::new(TermQuery::new(
                    Term::from_field_text(self.fields.platforms, platform_str(*p)),
                    IndexRecordOption::Basic,
                )),
            ));
        }
        if let Some(v) = &filters.api_version {
            clauses.push((
                Occur::Must,
                Box::new(TermQuery::new(
                    Term::from_field_text(self.fields.api_version, v),
                    IndexRecordOption::Basic,
                )),
            ));
        }
        if !filters.include_deprecated {
            clauses.push((
                Occur::MustNot,
                Box::new(TermQuery::new(
                    Term::from_field_bool(self.fields.deprecated, true),
                    IndexRecordOption::Basic,
                )),
            ));
        }
        for t in &filters.tags {
            clauses.push((
                Occur::Must,
                Box::new(TermQuery::new(
                    Term::from_field_text(self.fields.tags, t),
                    IndexRecordOption::Basic,
                )),
            ));
        }
        let boolean = BooleanQuery::new(clauses);

        let top = searcher
            .search(&boolean, &TopDocs::with_limit(top_k))
            .map_err(|e| RagError::Retrieval(format!("BM25 search: {}", e)))?;

        let mut hits = Vec::with_capacity(top.len());
        for (score, addr) in top {
            let doc: TantivyDocument = searcher
                .doc(addr)
                .map_err(|e| RagError::Retrieval(format!("BM25 doc fetch: {}", e)))?;
            let chunk = reconstruct_chunk(&doc, &self.fields)?;
            hits.push(Hit {
                chunk,
                score,
                source: HitSource::Bm25,
                vector_score: None,
                bm25_score: None,
            });
        }
        Ok(hits)
    }

    async fn delete(&self, ids: &[ChunkId]) -> Result<()> {
        let writer = self.writer.as_ref().ok_or_else(|| {
            RagError::Storage("TantivyBM25Index 以 open_read_only 模式打开 · delete 不可用".into())
        })?;
        let mut w = writer.lock().unwrap();
        for id in ids {
            let term = Term::from_field_text(self.fields.id, id.as_str());
            w.delete_term(term);
        }
        w.commit()
            .map_err(|e| RagError::Storage(format!("commit (delete): {}", e)))?;
        self.reader
            .reload()
            .map_err(|e| RagError::Storage(format!("reader reload (delete): {}", e)))?;
        Ok(())
    }
}

fn first_text(doc: &TantivyDocument, field: Field) -> Option<String> {
    use tantivy::schema::Value;
    doc.get_first(field)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

fn reconstruct_chunk(doc: &TantivyDocument, fields: &Fields) -> Result<Chunk> {
    let id =
        first_text(doc, fields.id).ok_or_else(|| RagError::Storage("missing id field".into()))?;
    let content = first_text(doc, fields.content)
        .ok_or_else(|| RagError::Storage("missing content field".into()))?;
    let meta_json = first_text(doc, fields.meta_json)
        .ok_or_else(|| RagError::Storage("missing meta_json field".into()))?;
    let metadata: ChunkMetadata = serde_json::from_str(&meta_json)
        .map_err(|e| RagError::Storage(format!("deserialize metadata: {}", e)))?;
    Ok(Chunk {
        id: ChunkId::new(id),
        content,
        metadata,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use arkui_rag_core::ChunkMetadata;

    fn mk_chunk(id: &str, content: &str, source: &str) -> Chunk {
        Chunk {
            id: ChunkId::new(id),
            content: content.to_string(),
            metadata: ChunkMetadata {
                source: source.to_string(),
                r#type: ChunkType::Generic,
                ..ChunkMetadata::default()
            },
        }
    }

    #[tokio::test]
    async fn upsert_and_search_basic() {
        let dir = tempfile::tempdir().unwrap();
        let bm = TantivyBM25Index::open(dir.path()).unwrap();

        let chunks = vec![
            mk_chunk("a", "ArkUI-X 用 Refresh 组件实现下拉刷新", "list.md"),
            mk_chunk("b", "Kotlin Coroutine launch viewModelScope", "kmp.md"),
            mk_chunk("c", "Android Activity lifecycle onCreate", "android.md"),
        ];
        bm.upsert(&chunks).await.unwrap();
        assert_eq!(bm.len(), 3);

        let hits = bm
            .search("下拉刷新", 5, &QueryFilters::default())
            .await
            .unwrap();
        assert!(!hits.is_empty(), "BM25 应能命中 '下拉刷新'");
        assert_eq!(hits[0].chunk.id.as_str(), "a");
        assert!(matches!(hits[0].source, HitSource::Bm25));
    }

    #[tokio::test]
    async fn upsert_overwrites_same_id() {
        let dir = tempfile::tempdir().unwrap();
        let bm = TantivyBM25Index::open(dir.path()).unwrap();

        bm.upsert(&[mk_chunk("a", "original content", "a.md")])
            .await
            .unwrap();
        bm.upsert(&[mk_chunk("a", "completely different text", "a.md")])
            .await
            .unwrap();

        // 用旧内容查不到
        let hits_old = bm
            .search("original", 5, &QueryFilters::default())
            .await
            .unwrap();
        assert!(
            hits_old.is_empty(),
            "旧 doc 应已被覆盖，期望 0 个命中，实际 {}",
            hits_old.len()
        );

        let hits_new = bm
            .search("different", 5, &QueryFilters::default())
            .await
            .unwrap();
        assert_eq!(hits_new.len(), 1);
    }

    #[tokio::test]
    async fn delete_works() {
        let dir = tempfile::tempdir().unwrap();
        let bm = TantivyBM25Index::open(dir.path()).unwrap();
        bm.upsert(&[mk_chunk("a", "removeme", "x.md")])
            .await
            .unwrap();
        assert_eq!(bm.len(), 1);
        bm.delete(&[ChunkId::new("a")]).await.unwrap();
        assert_eq!(bm.len(), 0);
    }

    #[tokio::test]
    async fn filter_by_platform() {
        let dir = tempfile::tempdir().unwrap();
        let bm = TantivyBM25Index::open(dir.path()).unwrap();

        let mut harmony = mk_chunk("h", "shared body text", "h.md");
        harmony.metadata.platforms = vec![Platform::HarmonyOs];
        let mut android = mk_chunk("a", "shared body text", "a.md");
        android.metadata.platforms = vec![Platform::Android];

        bm.upsert(&[harmony, android]).await.unwrap();

        let filters = QueryFilters {
            platforms: vec![Platform::HarmonyOs],
            ..Default::default()
        };
        let hits = bm.search("body", 10, &filters).await.unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].chunk.id.as_str(), "h");
    }

    #[tokio::test]
    async fn deprecated_filtered_by_default() {
        let dir = tempfile::tempdir().unwrap();
        let bm = TantivyBM25Index::open(dir.path()).unwrap();

        let active = mk_chunk("active", "router pushUrl new page", "a.md");
        let mut old = mk_chunk("old", "router pushUrl new page", "b.md");
        old.metadata.deprecated = true;

        bm.upsert(&[active, old]).await.unwrap();

        let hits = bm
            .search("router", 5, &QueryFilters::default())
            .await
            .unwrap();
        assert_eq!(hits.len(), 1, "默认应排除 deprecated");
        assert_eq!(hits[0].chunk.id.as_str(), "active");

        let filters = QueryFilters {
            include_deprecated: true,
            ..Default::default()
        };
        let hits_all = bm.search("router", 5, &filters).await.unwrap();
        assert_eq!(hits_all.len(), 2);
    }

    #[tokio::test]
    async fn empty_query_returns_empty() {
        let dir = tempfile::tempdir().unwrap();
        let bm = TantivyBM25Index::open(dir.path()).unwrap();
        bm.upsert(&[mk_chunk("a", "content", "a.md")])
            .await
            .unwrap();
        let hits = bm.search("", 5, &QueryFilters::default()).await.unwrap();
        assert!(hits.is_empty());
    }

    #[tokio::test]
    async fn reopen_persists_data() {
        let dir = tempfile::tempdir().unwrap();
        {
            let bm = TantivyBM25Index::open(dir.path()).unwrap();
            bm.upsert(&[mk_chunk("a", "Persisted across processes", "p.md")])
                .await
                .unwrap();
        }
        // 新实例 = 模拟跨进程 reopen
        let bm2 = TantivyBM25Index::open(dir.path()).unwrap();
        assert_eq!(bm2.len(), 1);
        let hits = bm2
            .search("Persisted", 5, &QueryFilters::default())
            .await
            .unwrap();
        assert_eq!(hits.len(), 1);
    }

    /// Round 36 · 多 instance 共存（多 server reader 不冲突）
    /// 修复前：第二个 `open()` 拿 IndexWriter 锁失败 · LockBusy。
    /// 修复后：`open_read_only()` 不持写锁 · 任意多个 reader 可同时活。
    #[tokio::test]
    async fn read_only_allows_concurrent_multi_instance() {
        let dir = tempfile::tempdir().unwrap();
        // 先用 writer 写入数据，关闭 writer
        {
            let w = TantivyBM25Index::open(dir.path()).unwrap();
            w.upsert(&[mk_chunk("x", "shared multi-reader content", "x.md")])
                .await
                .unwrap();
        }

        // 三个 reader 同时活 · 模拟 Claude Code + Claude Desktop + manual stdio 三处接同一 binary
        let r1 = TantivyBM25Index::open_read_only(dir.path()).unwrap();
        let r2 = TantivyBM25Index::open_read_only(dir.path()).unwrap();
        let r3 = TantivyBM25Index::open_read_only(dir.path()).unwrap();
        for r in [&r1, &r2, &r3] {
            let hits = r
                .search("shared", 5, &QueryFilters::default())
                .await
                .unwrap();
            assert_eq!(hits.len(), 1, "每个 reader 都应能查到");
            assert_eq!(hits[0].chunk.id.as_str(), "x");
        }
    }

    /// Round 36 · read_only 模式下 upsert 必须拒绝（fail-fast 防误用）
    #[tokio::test]
    async fn read_only_upsert_returns_error() {
        let dir = tempfile::tempdir().unwrap();
        // 先建索引
        {
            let w = TantivyBM25Index::open(dir.path()).unwrap();
            w.upsert(&[mk_chunk("seed", "any", "s.md")]).await.unwrap();
        }
        let r = TantivyBM25Index::open_read_only(dir.path()).unwrap();
        let err = r
            .upsert(&[mk_chunk("new", "should fail", "n.md")])
            .await
            .expect_err("read_only upsert 必须报错");
        assert!(
            err.to_string().contains("read_only") || err.to_string().contains("不可用"),
            "错误提示应指引用户用 `arkui-rag index` · 实际: {}",
            err
        );
    }
}
