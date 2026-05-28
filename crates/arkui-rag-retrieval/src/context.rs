//! ContextAssembler —— 把 Hit 扩展为含父级 chunk 的 Context。
//!
//! 方案 §1.4 "Parent-Child 索引"标准：
//! - 检索粒度：小 chunk（精准命中）
//! - 生成粒度：父 chunk（上下文完整）
//!
//! Day 11 实现：给定 `MetadataStore`，对每个 Hit 查找 `parent_id`，
//! 返回 `ExpandedHit { original, parent: Option<Chunk> }`。
//! 调用方（CLI / Server）决定如何展示（原 hit + 可选父 context）。

use arkui_rag_core::{Chunk, Hit, RagError, Result};
use arkui_rag_storage::MetadataStore;
use serde::Serialize;
use std::sync::Arc;

/// 扩展后的 Hit：原 Hit + 可选父 chunk。
#[derive(Debug, Clone, Serialize)]
pub struct ExpandedHit {
    pub original: Hit,
    pub parent: Option<Chunk>,
}

pub struct ContextAssembler {
    store: Arc<dyn MetadataStore>,
}

impl ContextAssembler {
    pub fn new(store: Arc<dyn MetadataStore>) -> Self {
        Self { store }
    }

    /// 把 hits 扩展为 ExpandedHit（含父 chunk 查找）。
    ///
    /// 行为：
    /// - 每个 hit 用其 `chunk.metadata.parent_id` 查 MetadataStore
    /// - 找到 → ExpandedHit.parent = Some(...)
    /// - 没 parent_id 或找不到 → ExpandedHit.parent = None
    /// - parent_id 链接到的 chunk 自身 `id == hit.chunk.id`（自指）→ 视为 None
    pub async fn expand_to_parent(&self, hits: Vec<Hit>) -> Result<Vec<ExpandedHit>> {
        let mut out = Vec::with_capacity(hits.len());
        for hit in hits {
            let parent = match &hit.chunk.metadata.parent_id {
                Some(pid) if *pid != hit.chunk.id => self.store.get(pid).await?,
                _ => None,
            };
            out.push(ExpandedHit {
                original: hit,
                parent,
            });
        }
        Ok(out)
    }

    /// 把 ExpandedHit 列表"压平"为单纯 Hit 列表，
    /// 用父 chunk 的 content 替换原 chunk content（如果有 parent）。
    /// 元数据保持原 hit 的（保留 heading_path 等 small-chunk 信息）。
    ///
    /// **用途**：当下游消费者（如 LLM prompt 拼装）只接受 Vec<Hit> 时，
    /// 调用此方法获得 "small id + big content" 的结果。
    pub fn flatten_with_parent_content(expanded: Vec<ExpandedHit>) -> Vec<Hit> {
        expanded
            .into_iter()
            .map(|e| {
                let mut hit = e.original;
                if let Some(parent) = e.parent {
                    hit.chunk.content = parent.content;
                }
                hit
            })
            .collect()
    }

    /// 与 expand_to_parent 类似但有限制：如果任何 hit 找不到父，返回错误。
    /// 适合严格场景（如评估父级覆盖率）。
    pub async fn expand_strict(&self, hits: Vec<Hit>) -> Result<Vec<ExpandedHit>> {
        let total = hits.len();
        let expanded = self.expand_to_parent(hits).await?;
        let with_parent = expanded.iter().filter(|e| e.parent.is_some()).count();
        if with_parent < total {
            return Err(RagError::Retrieval(format!(
                "expand_strict: {} / {} hits 缺父级（chunker 可能未生成 parent_id）",
                total - with_parent,
                total
            )));
        }
        Ok(expanded)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arkui_rag_core::{Chunk, ChunkId, ChunkMetadata, ChunkType, HitSource};
    use async_trait::async_trait;
    use std::collections::HashMap;
    use std::sync::RwLock;

    struct InMemMeta {
        store: RwLock<HashMap<String, Chunk>>,
    }

    #[async_trait]
    impl MetadataStore for InMemMeta {
        async fn get(&self, id: &ChunkId) -> Result<Option<Chunk>> {
            Ok(self.store.read().unwrap().get(id.as_str()).cloned())
        }
        async fn parent_of(&self, id: &ChunkId) -> Result<Option<Chunk>> {
            let store = self.store.read().unwrap();
            let me = store.get(id.as_str());
            let parent_id = me.and_then(|c| c.metadata.parent_id.clone());
            Ok(parent_id.and_then(|pid| store.get(pid.as_str()).cloned()))
        }
    }

    fn mk_chunk(id: &str, content: &str, parent: Option<&str>) -> Chunk {
        Chunk {
            id: ChunkId::new(id),
            content: content.to_string(),
            metadata: ChunkMetadata {
                source: "t.md".into(),
                heading_path: vec![id.to_string()],
                r#type: ChunkType::Generic,
                parent_id: parent.map(ChunkId::new),
                ..ChunkMetadata::default()
            },
        }
    }

    fn mk_hit(chunk: Chunk) -> Hit {
        Hit {
            chunk,
            score: 1.0,
            source: HitSource::Vector,
        }
    }

    fn store_with(chunks: Vec<Chunk>) -> Arc<dyn MetadataStore> {
        let map: HashMap<String, Chunk> = chunks
            .into_iter()
            .map(|c| (c.id.as_str().to_string(), c))
            .collect();
        Arc::new(InMemMeta {
            store: RwLock::new(map),
        })
    }

    #[tokio::test]
    async fn expand_finds_parent() {
        let parent = mk_chunk("p1", "parent body", None);
        let child = mk_chunk("c1", "child body", Some("p1"));
        let store = store_with(vec![parent.clone(), child.clone()]);
        let assembler = ContextAssembler::new(store);
        let hits = vec![mk_hit(child)];
        let expanded = assembler.expand_to_parent(hits).await.unwrap();
        assert_eq!(expanded.len(), 1);
        assert!(expanded[0].parent.is_some());
        assert_eq!(expanded[0].parent.as_ref().unwrap().id.as_str(), "p1");
    }

    #[tokio::test]
    async fn no_parent_id_returns_none() {
        let orphan = mk_chunk("orphan", "no parent", None);
        let store = store_with(vec![orphan.clone()]);
        let hits = vec![mk_hit(orphan)];
        let exp = ContextAssembler::new(store)
            .expand_to_parent(hits)
            .await
            .unwrap();
        assert_eq!(exp.len(), 1);
        assert!(exp[0].parent.is_none());
    }

    #[tokio::test]
    async fn parent_not_in_store_returns_none() {
        let child = mk_chunk("c", "child", Some("missing_parent"));
        let store = store_with(vec![child.clone()]); // 父不在 store
        let exp = ContextAssembler::new(store)
            .expand_to_parent(vec![mk_hit(child)])
            .await
            .unwrap();
        assert!(exp[0].parent.is_none());
    }

    #[tokio::test]
    async fn flatten_uses_parent_content() {
        let parent = mk_chunk("p1", "I AM PARENT", None);
        let child = mk_chunk("c1", "i am child", Some("p1"));
        let store = store_with(vec![parent, child.clone()]);
        let exp = ContextAssembler::new(store)
            .expand_to_parent(vec![mk_hit(child)])
            .await
            .unwrap();
        let flat = ContextAssembler::flatten_with_parent_content(exp);
        assert_eq!(flat[0].chunk.id.as_str(), "c1"); // id 保留小 chunk
        assert_eq!(flat[0].chunk.content, "I AM PARENT"); // content 用父
        assert_eq!(flat[0].chunk.metadata.heading_path, vec!["c1"]); // metadata 保留小 chunk
    }

    #[tokio::test]
    async fn expand_strict_fails_on_missing_parent() {
        let orphan = mk_chunk("orphan", "no parent", None);
        let store = store_with(vec![orphan.clone()]);
        let r = ContextAssembler::new(store)
            .expand_strict(vec![mk_hit(orphan)])
            .await;
        assert!(r.is_err());
    }
}
