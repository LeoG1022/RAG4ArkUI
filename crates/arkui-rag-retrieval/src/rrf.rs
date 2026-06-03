//! RRF (Reciprocal Rank Fusion)。
//!
//! 论文：Cormack, Clarke, Buettcher 2009《Reciprocal Rank Fusion outperforms Condorcet ...》
//! 默认 k=60；这是经验值（业界几乎所有实现都用 60）。

use arkui_rag_core::Hit;
use std::collections::HashMap;

pub const RRF_DEFAULT_K: f32 = 60.0;

/// 把多路检索结果按 RRF 融合后排序。
///
/// 每路输入按 score 降序（即 rank 从 1 开始）。融合 score 累加；最终按融合 score 降序返回。
pub fn rrf_fuse(rankings: Vec<Vec<Hit>>, k: f32) -> Vec<Hit> {
    let mut accum: HashMap<String, (f32, Hit)> = HashMap::new();

    for hits in rankings {
        for (rank, hit) in hits.into_iter().enumerate() {
            let rank_f = (rank + 1) as f32;
            let contrib = 1.0 / (k + rank_f);
            let key = hit.chunk.id.as_str().to_string();
            // Round 52: 把 Copy 字段提前取出 · 防止 hit move 后 closure 借不到
            let new_vec_score = hit.vector_score;
            let new_bm25_score = hit.bm25_score;
            accum
                .entry(key)
                .and_modify(|(s, existing)| {
                    *s += contrib;
                    // 合并：保留任一路径已有的非 None raw score
                    if existing.vector_score.is_none() && new_vec_score.is_some() {
                        existing.vector_score = new_vec_score;
                    }
                    if existing.bm25_score.is_none() && new_bm25_score.is_some() {
                        existing.bm25_score = new_bm25_score;
                    }
                })
                .or_insert_with(|| (contrib, hit));
        }
    }

    let mut out: Vec<Hit> = accum
        .into_iter()
        .map(|(_, (score, mut hit))| {
            hit.score = score;
            hit
        })
        .collect();
    out.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use arkui_rag_core::{Chunk, ChunkId, ChunkMetadata};

    fn mk(id: &str, score: f32) -> Hit {
        Hit {
            chunk: Chunk {
                id: ChunkId::new(id),
                content: String::new(),
                metadata: ChunkMetadata::default(),
            },
            score,
            source: Default::default(),
            vector_score: None,
            bm25_score: None,
        }
    }

    #[test]
    fn doc_in_both_ranks_higher() {
        // Doc A 在两路都排第 1，Doc B 只在第一路排第 2，Doc C 只在第二路排第 2
        let r1 = vec![mk("A", 1.0), mk("B", 0.9)];
        let r2 = vec![mk("A", 1.0), mk("C", 0.8)];
        let fused = rrf_fuse(vec![r1, r2], RRF_DEFAULT_K);
        assert_eq!(fused[0].chunk.id.as_str(), "A"); // A 双重命中 → 第 1
                                                     // B 和 C 都只一路命中且同 rank → 分数相等
        assert!(fused.iter().any(|h| h.chunk.id.as_str() == "B"));
        assert!(fused.iter().any(|h| h.chunk.id.as_str() == "C"));
    }

    #[test]
    fn empty_input_empty_output() {
        let fused = rrf_fuse(vec![], RRF_DEFAULT_K);
        assert!(fused.is_empty());
    }
}
