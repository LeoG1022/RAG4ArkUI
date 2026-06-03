//! Evaluator —— 跑 ground truth 评估集 + 汇总指标。

use crate::types::{EvalConfig, EvalQuery, EvalResult, EvalSummary};
use arkui_rag_core::{
    EnhancedQuery, PassthroughEnhancer, QueryEnhancer, RagError, Result, Retriever,
};
use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

/// 反序列化 YAML 评估集。
pub fn load_queries(path: &Path) -> Result<Vec<EvalQuery>> {
    let s = std::fs::read_to_string(path)
        .map_err(|e| RagError::Config(format!("读评估集 {} 失败: {}", path.display(), e)))?;
    serde_yaml::from_str(&s).map_err(|e| RagError::Config(format!("解析评估集 YAML 失败: {}", e)))
}

pub struct Evaluator {
    retriever: Arc<dyn Retriever>,
    k: usize,
    /// 可选 reranker，evaluator 内部链式调用（用户传入已构造好的）
    reranker: Option<Arc<dyn arkui_rag_core::Reranker>>,
    /// Query enhancer（默认 PassthroughEnhancer）
    enhancer: Arc<dyn QueryEnhancer>,
    pre_rerank_k: usize,
    config: EvalConfig,
}

impl Evaluator {
    pub fn new(retriever: Arc<dyn Retriever>) -> Self {
        Self {
            retriever,
            k: 5,
            reranker: None,
            enhancer: Arc::new(PassthroughEnhancer),
            pre_rerank_k: 50,
            config: EvalConfig::default(),
        }
    }

    pub fn with_k(mut self, k: usize) -> Self {
        self.k = k.max(1);
        self
    }

    pub fn with_reranker(mut self, r: Arc<dyn arkui_rag_core::Reranker>) -> Self {
        self.reranker = Some(r);
        self
    }

    /// Day 7：注入 QueryEnhancer（默认 PassthroughEnhancer，不改写）
    pub fn with_enhancer(mut self, e: Arc<dyn QueryEnhancer>) -> Self {
        self.enhancer = e;
        self
    }

    pub fn with_pre_rerank_k(mut self, n: usize) -> Self {
        self.pre_rerank_k = n.max(self.k);
        self
    }

    pub fn with_config(mut self, c: EvalConfig) -> Self {
        self.config = c;
        self
    }

    /// 跑全部 query。
    pub async fn run(&self, queries: &[EvalQuery]) -> Result<EvalSummary> {
        if queries.is_empty() {
            return Err(RagError::Config("评估集为空".into()));
        }
        let mut results = Vec::with_capacity(queries.len());
        for q in queries {
            results.push(self.eval_one(q).await?);
        }

        let n = results.len() as f32;
        let avg_recall = results.iter().map(|r| r.recall_at_k).sum::<f32>() / n;
        let avg_mrr = results.iter().map(|r| r.mrr_at_k).sum::<f32>() / n;

        let mut lats: Vec<u128> = results.iter().map(|r| r.latency_ms).collect();
        lats.sort_unstable();
        let avg_lat = lats.iter().sum::<u128>() as f32 / n;
        let p50 = lats[(lats.len() / 2).min(lats.len() - 1)] as f32;
        let p99_idx = ((lats.len() as f32 * 0.99).ceil() as usize).saturating_sub(1);
        let p99 = lats[p99_idx.min(lats.len() - 1)] as f32;

        Ok(EvalSummary {
            config: self.config.clone(),
            k: self.k,
            total_queries: results.len(),
            avg_recall_at_k: avg_recall,
            avg_mrr_at_k: avg_mrr,
            avg_latency_ms: avg_lat,
            p50_latency_ms: p50,
            p99_latency_ms: p99,
            per_query: results,
        })
    }

    async fn eval_one(&self, q: &EvalQuery) -> Result<EvalResult> {
        let start = Instant::now();
        let eq: EnhancedQuery = self.enhancer.enhance(&q.query).await?;

        let retrieve_k = if self.reranker.is_some() {
            self.pre_rerank_k.max(self.k)
        } else {
            self.k
        };
        let hits = self.retriever.retrieve(&eq, retrieve_k).await?;
        let hits = if let Some(rr) = &self.reranker {
            rr.rerank(&q.query, hits, self.k).await?
        } else {
            hits.into_iter().take(self.k).collect()
        };
        let latency_ms = start.elapsed().as_millis();

        let returned: Vec<String> = hits
            .iter()
            .map(|h| h.chunk.id.as_str().to_string())
            .collect();
        let returned_set: HashSet<&str> = returned.iter().map(|s| s.as_str()).collect();
        let gt_set: HashSet<&str> = q.relevant.iter().map(|s| s.as_str()).collect();

        let hit_count = returned_set.intersection(&gt_set).count();
        let recall = if gt_set.is_empty() {
            0.0
        } else {
            hit_count as f32 / gt_set.len() as f32
        };
        let mrr = returned
            .iter()
            .position(|id| gt_set.contains(id.as_str()))
            .map(|rank| 1.0 / (rank as f32 + 1.0))
            .unwrap_or(0.0);
        let missed: Vec<String> = q
            .relevant
            .iter()
            .filter(|gt| !returned_set.contains(gt.as_str()))
            .cloned()
            .collect();

        Ok(EvalResult {
            query_id: q.id.clone(),
            query_text: q.query.clone(),
            recall_at_k: recall,
            mrr_at_k: mrr,
            latency_ms,
            returned,
            missed,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arkui_rag_core::{Chunk, ChunkId, ChunkMetadata, Hit, HitSource};
    use async_trait::async_trait;

    /// 受控 mock retriever：按预设顺序返回固定 chunk
    struct StubRetriever {
        scripted: Vec<Vec<&'static str>>,
        idx: std::sync::Mutex<usize>,
    }
    #[async_trait]
    impl Retriever for StubRetriever {
        fn name(&self) -> &str {
            "stub"
        }
        async fn retrieve(&self, _q: &EnhancedQuery, top_k: usize) -> Result<Vec<Hit>> {
            let mut i = self.idx.lock().unwrap();
            let ids = self.scripted[*i].clone();
            *i = (*i + 1) % self.scripted.len();
            Ok(ids
                .into_iter()
                .take(top_k)
                .enumerate()
                .map(|(rank, id)| Hit {
                    chunk: Chunk {
                        id: ChunkId::new(id),
                        content: format!("content for {}", id),
                        metadata: ChunkMetadata::default(),
                    },
                    score: 1.0 - 0.1 * rank as f32,
                    source: HitSource::Hybrid,
                    vector_score: None,
                    bm25_score: None,
                })
                .collect())
        }
    }

    #[tokio::test]
    async fn recall_and_mrr_basic() {
        let retriever = Arc::new(StubRetriever {
            scripted: vec![vec!["a", "b", "c"], vec!["x", "y", "z"]],
            idx: std::sync::Mutex::new(0),
        });
        let queries = vec![
            EvalQuery {
                id: "q1".into(),
                query: "first".into(),
                relevant: vec!["a".into(), "b".into()],
                notes: None,
            },
            EvalQuery {
                id: "q2".into(),
                query: "second".into(),
                relevant: vec!["miss1".into(), "miss2".into()],
                notes: None,
            },
        ];
        let eval = Evaluator::new(retriever).with_k(3);
        let s = eval.run(&queries).await.unwrap();
        assert_eq!(s.total_queries, 2);
        // q1: recall=2/2=1.0, mrr=1/1=1.0
        // q2: recall=0/2=0.0, mrr=0
        // avg recall=0.5, avg mrr=0.5
        assert!((s.avg_recall_at_k - 0.5).abs() < 1e-5);
        assert!((s.avg_mrr_at_k - 0.5).abs() < 1e-5);
        assert_eq!(s.per_query[0].recall_at_k, 1.0);
        assert_eq!(s.per_query[1].recall_at_k, 0.0);
    }

    #[tokio::test]
    async fn mrr_decays_with_rank() {
        let retriever = Arc::new(StubRetriever {
            scripted: vec![vec!["x", "y", "a"]],
            idx: std::sync::Mutex::new(0),
        });
        let queries = vec![EvalQuery {
            id: "q1".into(),
            query: "test".into(),
            relevant: vec!["a".into()],
            notes: None,
        }];
        let s = Evaluator::new(retriever)
            .with_k(3)
            .run(&queries)
            .await
            .unwrap();
        // "a" 在 rank 3 → mrr = 1/3 ≈ 0.333
        assert!((s.per_query[0].mrr_at_k - 0.3333).abs() < 0.01);
    }

    #[tokio::test]
    async fn empty_queries_returns_err() {
        let retriever = Arc::new(StubRetriever {
            scripted: vec![vec!["a"]],
            idx: std::sync::Mutex::new(0),
        });
        let eval = Evaluator::new(retriever);
        let r = eval.run(&[]).await;
        assert!(r.is_err());
    }
}
