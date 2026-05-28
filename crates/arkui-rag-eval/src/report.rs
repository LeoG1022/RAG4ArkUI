//! 评估报告（markdown）生成。

use crate::types::EvalSummary;
use std::fmt::Write as _;

/// 把 EvalSummary 渲染为可读的 markdown 报告。
pub fn render_markdown(summary: &EvalSummary, run_at: &str) -> String {
    let mut out = String::with_capacity(8192);

    let _ = writeln!(out, "# RAG4ArkUI 检索质量评估报告");
    let _ = writeln!(out);
    let _ = writeln!(out, "- **跑评时间**: {}", run_at);
    let _ = writeln!(out, "- **评估集**: `{}`", summary.config.queries_path);
    let _ = writeln!(out, "- **索引**: `{}`", summary.config.index_path);
    let _ = writeln!(out, "- **k**: {}", summary.k);
    let _ = writeln!(out, "- **配置**: embedder=`{}` · bm25=`{}` · rerank=`{}` · hyde=`{}` · pre_rerank_k=`{}`",
        summary.config.embedder, summary.config.bm25, summary.config.rerank, summary.config.hyde, summary.config.pre_rerank_k);
    let _ = writeln!(out);

    let _ = writeln!(out, "## 整体指标");
    let _ = writeln!(out);
    let _ = writeln!(out, "| 指标 | 值 |");
    let _ = writeln!(out, "|---|---|");
    let _ = writeln!(out, "| 总 query 数 | {} |", summary.total_queries);
    let _ = writeln!(out, "| **平均 recall@{}** | **{:.3}** |", summary.k, summary.avg_recall_at_k);
    let _ = writeln!(out, "| **平均 MRR@{}** | **{:.3}** |", summary.k, summary.avg_mrr_at_k);
    let _ = writeln!(out, "| 平均延迟 | {:.1} ms |", summary.avg_latency_ms);
    let _ = writeln!(out, "| p50 延迟 | {:.1} ms |", summary.p50_latency_ms);
    let _ = writeln!(out, "| p99 延迟 | {:.1} ms |", summary.p99_latency_ms);
    let _ = writeln!(out);

    let _ = writeln!(out, "## 每 query 详情");
    let _ = writeln!(out);
    let _ = writeln!(out, "| id | query | recall@{} | MRR@{} | latency | 命中 GT | 漏命中 |", summary.k, summary.k);
    let _ = writeln!(out, "|---|---|---|---|---|---|---|");
    for r in &summary.per_query {
        let q_short = if r.query_text.chars().count() > 40 {
            let head: String = r.query_text.chars().take(38).collect();
            format!("{}…", head)
        } else {
            r.query_text.clone()
        };
        let q_short = q_short.replace('|', "\\|");
        let hit = r.returned.iter().filter(|id| !r.missed.contains(id)).count();
        let missed_short = if r.missed.is_empty() {
            "—".to_string()
        } else {
            r.missed
                .iter()
                .map(|s| format!("`{}`", s))
                .collect::<Vec<_>>()
                .join("<br>")
        };
        let _ = writeln!(
            out,
            "| {} | {} | {:.3} | {:.3} | {} ms | {}/{} | {} |",
            r.query_id,
            q_short,
            r.recall_at_k,
            r.mrr_at_k,
            r.latency_ms,
            hit,
            r.returned.len(),
            missed_short
        );
    }
    let _ = writeln!(out);

    let _ = writeln!(out, "## 失败 query 详情（recall@{} < 1.0）", summary.k);
    let _ = writeln!(out);
    let failed: Vec<_> = summary
        .per_query
        .iter()
        .filter(|r| r.recall_at_k < 1.0)
        .collect();
    if failed.is_empty() {
        let _ = writeln!(out, "无 —— 全部 query 在 top-{} 内召回了所有 ground truth。", summary.k);
    } else {
        for r in failed {
            let _ = writeln!(out, "### {} · {}", r.query_id, r.query_text.replace('\n', " "));
            let _ = writeln!(out);
            let _ = writeln!(out, "- **recall@{}**: {:.3}", summary.k, r.recall_at_k);
            let _ = writeln!(out, "- **MRR@{}**: {:.3}", summary.k, r.mrr_at_k);
            let _ = writeln!(out, "- **返回 top-{}**:", summary.k);
            for (rank, id) in r.returned.iter().enumerate() {
                let marker = if r.missed.iter().any(|m| m == id) {
                    " ❌ "
                } else {
                    " "
                };
                let _ = writeln!(out, "  - [{}]{}`{}`", rank + 1, marker, id);
            }
            if !r.missed.is_empty() {
                let _ = writeln!(out, "- **漏命中 GT**: {}", r.missed.iter().map(|s| format!("`{}`", s)).collect::<Vec<_>>().join(", "));
            }
            let _ = writeln!(out);
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{EvalConfig, EvalResult};

    #[test]
    fn renders_markdown_with_all_sections() {
        let s = EvalSummary {
            config: EvalConfig {
                embedder: "mock-64".into(),
                bm25: "memory".into(),
                rerank: "none".into(),
                pre_rerank_k: 50,
                index_path: "/tmp/idx.json".into(),
                queries_path: "/tmp/q.yaml".into(),
                hyde: "none".into(),
            },
            k: 5,
            total_queries: 2,
            avg_recall_at_k: 0.5,
            avg_mrr_at_k: 0.5,
            avg_latency_ms: 10.0,
            p50_latency_ms: 10.0,
            p99_latency_ms: 12.0,
            per_query: vec![
                EvalResult {
                    query_id: "q1".into(),
                    query_text: "test 1".into(),
                    recall_at_k: 1.0,
                    mrr_at_k: 1.0,
                    latency_ms: 8,
                    returned: vec!["a".into(), "b".into()],
                    missed: vec![],
                },
                EvalResult {
                    query_id: "q2".into(),
                    query_text: "test 2".into(),
                    recall_at_k: 0.0,
                    mrr_at_k: 0.0,
                    latency_ms: 12,
                    returned: vec!["x".into(), "y".into()],
                    missed: vec!["miss".into()],
                },
            ],
        };
        let md = render_markdown(&s, "2026-05-27 10:00:00");
        assert!(md.contains("整体指标"));
        assert!(md.contains("每 query 详情"));
        assert!(md.contains("失败 query 详情"));
        assert!(md.contains("q1"));
        assert!(md.contains("q2"));
        assert!(md.contains("漏命中 GT")); // 失败详情
        assert!(md.contains("0.500")); // avg recall
    }

    #[test]
    fn renders_all_pass_message_when_no_failure() {
        let s = EvalSummary {
            config: EvalConfig::default(),
            k: 3,
            total_queries: 1,
            avg_recall_at_k: 1.0,
            avg_mrr_at_k: 1.0,
            avg_latency_ms: 5.0,
            p50_latency_ms: 5.0,
            p99_latency_ms: 5.0,
            per_query: vec![EvalResult {
                query_id: "q1".into(),
                query_text: "x".into(),
                recall_at_k: 1.0,
                mrr_at_k: 1.0,
                latency_ms: 5,
                returned: vec!["a".into()],
                missed: vec![],
            }],
        };
        let md = render_markdown(&s, "now");
        assert!(md.contains("全部 query 在 top-3 内召回"));
    }
}
