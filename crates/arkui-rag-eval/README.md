# arkui-rag-eval

**定位**：检索质量评估。建立可量化的 RAG 质量基线，让"Hybrid + Rerank"的效果可被回归测试。

技术方案对应：
- §1.5 RAG 评估体系
- §2.7 评估指标设计
- §8.5 共识 4："评估先行，Eval-Driven Development"

## Day 6 提供

| 模块 | 功能 |
|---|---|
| `EvalQuery` / `EvalGroundTruth` | 评估集 schema（YAML 序列化） |
| `EvalResult` / `EvalSummary` | 单 query 结果 + 全集汇总 |
| `Evaluator::run()` | 串行跑全部 query + 算指标 + 收集延迟 |
| `report::render_markdown()` | 生成 markdown 报告（顶部 summary + per-query 详情） |
| 指标 | `recall@k` / `MRR@k` / `latency p50/p99` |

## 评估集格式（YAML）

```yaml
# corpus/_eval/queries.yaml
- id: q1
  query: "ArkUI-X 如何实现下拉刷新"
  relevant:
    - "list.md#List/下拉刷新@10"
    - "list.md#List/Refresh-API@25"
  notes: "下拉刷新场景"
- id: q2
  query: "router.pushUrl 怎么传参数"
  relevant:
    - "router.md#Router/pushUrl@9"
```

`relevant` 是 ground truth 的 chunk_id 列表，可标多个。Chunk id 见 `MarkdownChunker::make_id`：
`<source>#<heading_path>@<start_line>` 格式。

## 用法（库）

```rust,ignore
use std::sync::Arc;
use std::path::Path;
use arkui_rag_eval::{Evaluator, load_queries};

# tokio_test::block_on(async {
let queries = load_queries(Path::new("corpus/_eval/queries.yaml")).unwrap();
let retriever: Arc<dyn arkui_rag_core::Retriever> = /* 构造 */;
let evaluator = Evaluator::new(retriever).with_k(5);
let summary = evaluator.run(&queries).await.unwrap();
println!("avg recall@5 = {:.3}", summary.avg_recall_at_k);
println!("avg MRR@5    = {:.3}", summary.avg_mrr_at_k);
# });
```

## 用法（CLI）

```bash
# 跑评估
cargo run -p arkui-rag-cli -- eval \
    --queries corpus/_eval/queries.yaml \
    --index-path corpus/_index/index.json \
    --k 5 \
    --report-path reports/eval.md \
    --embedder mock --bm25 memory --rerank none
```

输出 stdout summary + 写完整报告到 `reports/eval.md`。

## 指标定义

- **recall@k**: `|gt ∩ top_k| / |gt|`，越高越好（理想 1.0）
- **MRR@k**: `1 / rank_of_first_gt_in_top_k`，没命中 = 0；越高越好（理想 1.0）
- **latency_ms**: 单 query 检索 + rerank 总耗时；汇总取 avg / p50 / p99

不实现 **faithfulness / answer relevancy**（需要 LLM 调用），Week 3 接 RAGAS 时补。

## 评估集标注指南

1. **覆盖核心场景**：每个主要功能（路由 / 状态 / 列表 / 网络 / 一多）至少 2 个 query
2. **不同 query 风格**：精确 API 名 / 自然语言场景 / 残缺代码片段 / 错误信息修复
3. **多 GT 容忍**：一个 query 允许多个相关 chunk（OR 关系）
4. **不要全是精确匹配**：mock embedder 阶段会让"语义相似"测试退化为"逐字匹配"，混合 query 风格能暴露这个

## 不做（Day 6 明确边界）

- ❌ RAGAS（Python 子进程）→ Week 3
- ❌ Faithfulness / Answer Relevancy（需 LLM）→ Week 3
- ❌ 自动生成 ground truth（人工 + LLM 半自动）→ 长期演进
- ❌ 多维度对比 dashboard（HTML）→ Week 4+
