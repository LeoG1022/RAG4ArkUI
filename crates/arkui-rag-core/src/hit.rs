//! 检索命中与引用类型。
//!
//! `Hit` 是检索流水线的统一货币——向量检索、BM25、Reranker、Context Assembler
//! 全部输入输出 `Vec<Hit>`，便于链式组合。

use crate::chunk::{Chunk, ChunkId};
use serde::{Deserialize, Serialize};

/// 单条检索结果。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hit {
    /// 命中的 chunk（已扩展到父粒度时，content 是父 chunk）。
    pub chunk: Chunk,
    /// 检索器给出的分数（含义因检索器不同：vector=cosine / bm25=BM25 raw / hybrid=RRF）。
    pub score: f32,
    /// 哪个检索器召回（用于 RRF 融合 / 调试）。
    #[serde(default)]
    pub source: HitSource,
    /// Round 52: 原始向量 cosine 相似度（HybridRetriever fuse 前保留 · 用于真实相似度展示 + 阈值过滤）。
    /// None 表示这条 hit 不来自 vector 路径，或者来自老版 index 反序列化。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vector_score: Option<f32>,
    /// Round 52: 原始 BM25 score（fuse 前保留）。None 表示不来自 BM25 路径。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bm25_score: Option<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum HitSource {
    Vector,
    Bm25,
    #[default]
    Hybrid,
    Reranked,
}

/// 给 LLM / UI 用的引用单元。一个 Hit → 一个 Citation。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Citation {
    pub chunk_id: ChunkId,
    pub source: String,
    pub heading_path: Vec<String>,
    pub line_range: Option<(u32, u32)>,
    pub score: f32,
}

impl From<&Hit> for Citation {
    fn from(h: &Hit) -> Self {
        Self {
            chunk_id: h.chunk.id.clone(),
            source: h.chunk.metadata.source.clone(),
            heading_path: h.chunk.metadata.heading_path.clone(),
            line_range: h.chunk.metadata.line_range,
            score: h.score,
        }
    }
}
