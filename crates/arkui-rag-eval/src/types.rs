//! 评估相关的类型定义。
//!
//! `EvalQuery` 反序列化自 `corpus/_eval/queries.yaml`；
//! `EvalResult` / `EvalSummary` 是评估器产出。

use serde::{Deserialize, Serialize};

/// 单条评估 query（含 ground truth）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalQuery {
    /// 短标识，便于报告引用（如 "q1"）。
    pub id: String,
    /// 用户输入文本。
    pub query: String,
    /// 相关 chunk_id 列表（ground truth）。任意命中算召回成功。
    pub relevant: Vec<String>,
    /// 可选备注（不参与计算）。
    #[serde(default)]
    pub notes: Option<String>,
}

/// 单条 query 的评估结果。
#[derive(Debug, Clone, Serialize)]
pub struct EvalResult {
    pub query_id: String,
    pub query_text: String,
    /// recall@k：命中 ground truth 的 chunk 数 / ground truth 总数
    pub recall_at_k: f32,
    /// MRR@k：first hit rank 的倒数（0 表示前 k 没命中）
    pub mrr_at_k: f32,
    /// 检索 + rerank 总耗时（毫秒）
    pub latency_ms: u128,
    /// 实际返回的 top-k chunk_id（顺序保留，便于人工核对）
    pub returned: Vec<String>,
    /// ground truth 中**未**被命中的 chunk_id
    pub missed: Vec<String>,
}

/// 全集汇总。
#[derive(Debug, Clone, Serialize)]
pub struct EvalSummary {
    pub config: EvalConfig,
    pub k: usize,
    pub total_queries: usize,
    pub avg_recall_at_k: f32,
    pub avg_mrr_at_k: f32,
    pub avg_latency_ms: f32,
    pub p50_latency_ms: f32,
    pub p99_latency_ms: f32,
    pub per_query: Vec<EvalResult>,
}

/// 评估配置标识（写入报告头部，便于多次跑对比）。
#[derive(Debug, Clone, Default, Serialize)]
pub struct EvalConfig {
    pub embedder: String,
    pub bm25: String,
    pub rerank: String,
    pub pre_rerank_k: usize,
    pub index_path: String,
    pub queries_path: String,
}
