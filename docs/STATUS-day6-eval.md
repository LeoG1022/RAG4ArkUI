# STATUS · Day 6 · 检索质量评估

> 日期：2026-05-27
> 对应 commit：[本 commit · Day 6 评估闭环]
> 对应 feature log：[`feedback/features/rag4arkui-core/10-2026-05-27-day6-eval.md`](../feedback/features/rag4arkui-core/10-2026-05-27-day6-eval.md)
> 上一阶段：[`STATUS-day5-reranker.md`](STATUS-day5-reranker.md)（Day 5 Hybrid + Rerank 业界基线完整）
> 下一阶段：`STATUS-day7-hyde.md` 或 `STATUS-day10-tree-sitter.md`（按用户选择）

> 🎯 **里程碑**：§8.5 共识 4 "Eval-Driven Development" 落地 — 检索质量从此可量化、可回归。

---

## 当前状态

新增第 9 个 crate `arkui-rag-eval`，提供完整的检索质量评估闭环：

- **评估集（YAML）**：ground truth 标注格式 + 8 条 sample query 覆盖 4 类风格
- **Evaluator**：跑全集 + 算 recall@k / MRR@k / latency p50/p99
- **报告**：markdown 格式三节（整体指标 / 每 query 详情 / 失败 query 详情）
- **CLI**：`arkui-rag eval` 子命令复用 query 全部参数（embedder/bm25/rerank）

### 交付清单

| 模块 | 变化 |
|---|---|
| `arkui-rag-eval` crate（**新增**） | 5 src + 1 tests + Cargo.toml + README ≈ 600 行代码 |
| `EvalQuery` / `EvalResult` / `EvalSummary` / `EvalConfig` | YAML 反序列化 + JSON serializable |
| `Evaluator` builder（`with_k` / `with_reranker` / `with_pre_rerank_k` / `with_config`） | 与现有流水线 trait 无缝集成 |
| `render_markdown(summary, run_at)` | 报告与数据分离，可单测 |
| `corpus/_eval/queries.yaml` | 8 sample queries + 详细标注指南 |
| `arkui-rag-cli` 加 `Eval` subcommand | 7 个新参数 + cmd_eval 约 120 行 |
| `docs/ADR-002` + `crates/README.md` | 速查表 Day 6 进度 |

### 测试覆盖增量

| 测试 | 数量 |
|---|---|
| `Evaluator` 单测（recall/MRR 算法 + 空集容错） | 3 |
| `render_markdown` 单测（完整 + 全通过场景） | 2 |
| 端到端集成测（fixture corpus → index → eval → 报告） | 1 |
| **本轮新增小计** | **6** |
| **累计** | 25 → **31** |

---

## 输入契约

### 评估集格式

```yaml
- id: q1
  query: "ArkUI-X 如何实现下拉刷新"
  relevant:
    - "list.md#List/下拉刷新@10"
    - "list.md#List/Refresh-API@25"
  notes: "可选备注"
```

- **id**：短标识，便于报告引用
- **query**：用户输入文本（任意风格）
- **relevant**：ground truth chunk_id 列表（OR 关系，任一命中算召回）
- **notes**：可选，不参与计算

Chunk id 格式严格对齐 `MarkdownChunker::make_id`：`<source_relative_path>#<heading_path>@<start_line>`。

### CLI 输入

```bash
arkui-rag eval [OPTIONS]

主要参数：
  --queries <FILE>           评估集 YAML (默认 corpus/_eval/queries.yaml)
  --index-path <FILE>        索引产物路径
  --k <N>                    评估的 top-k (默认 5)
  --report-path <FILE>       报告输出（默认 reports/eval-<ts>-<config>.md）

流水线参数（与 query 子命令完全一致）：
  --embedder mock|onnx
  --bm25 memory|tantivy
  --rerank none|mock|onnx
  --pre-rerank-k <N>
  --model-path / --reranker-model-path / --reranker-model-id
```

### 库 API 输入

```rust
let queries = load_queries(Path::new("queries.yaml"))?;        // YAML → Vec<EvalQuery>
let evaluator = Evaluator::new(retriever)
    .with_k(5)
    .with_reranker(reranker)        // 可选
    .with_pre_rerank_k(50)
    .with_config(config);
let summary = evaluator.run(&queries).await?;                  // → EvalSummary
```

---

## 输出契约

### 控制台输出（stdout）

```
📊 跑评估：8 个 query · embedder=mock-384 · bm25=memory · rerank=none · k=5

✅ 评估完成
   total queries  : 8
   avg recall@5   : 0.625
   avg MRR@5      : 0.583
   avg latency    : 12.3 ms
   p50 latency    : 11.0 ms
   p99 latency    : 18.0 ms
   report saved   : reports/eval-1716800000-mock-384-memory-none-5.md
```

### Markdown 报告（reports/）

```
# RAG4ArkUI 检索质量评估报告
- 跑评时间 / 评估集 / 索引 / k / 配置

## 整体指标
| 指标 | 值 |
| 总 query 数 | 8 |
| **平均 recall@5** | **0.625** |
| ...

## 每 query 详情
| id | query | recall@5 | MRR@5 | latency | 命中 GT | 漏命中 |

## 失败 query 详情（recall@5 < 1.0）
### q-state-mgmt · @State 装饰器怎么用
- recall@5: 0.000
- 返回 top-5:
  - [1] `router.md#Router/pushUrl@9`
  - [2] ❌ `list.md#List/下拉刷新@10`
  ...
- 漏命中 GT: `state.md#State/装饰器@5`
```

### 文件命名约定

`reports/eval-<unix_ts>-<embedder_model_id>-<bm25_name>-<rerank_name>-<k>.md`

便于：
- grep 找特定配置的报告
- 对比 mock vs onnx（同 timestamp 范围）
- 长期 trend 分析（按 ts 排序）

### Hit.source 与评估的对应

| HitSource | 评估算法看到 |
|---|---|
| `Vector` / `Bm25` / `Hybrid` / `Reranked` | 不区分（只看 chunk_id） |

评估器**只看 chunk_id 是否在 ground truth 里**，不区分召回路径。多路径都召回同一 chunk → 只计 1 次（HashSet 去重）。

---

## 验证手段

### 用户手动

```bash
# 1. 编译验证（9 crate 全过）
make check
make test                                  # 全 workspace 测试，含 6 个 eval crate 新测

# 2. 单 crate 测试
cd crates && cargo test -p arkui-rag-eval

# 3. 端到端：建索引 → 跑评估 → 看报告
# 前提：corpus/ 下有对应文档；queries.yaml 的 chunk_id 已校准
cd ..
cargo run --manifest-path crates/Cargo.toml -p arkui-rag-cli -- \
    index --source corpus
cargo run --manifest-path crates/Cargo.toml -p arkui-rag-cli -- \
    eval --queries corpus/_eval/queries.yaml --k 5

# 4. 配置矩阵对比（多次跑 → 多个报告）
# Mock + Memory + None（基线）
cargo run -- eval --embedder mock --bm25 memory --rerank none
# Mock + Tantivy + None（加 BM25）
cargo run --features tantivy -- eval --bm25 tantivy
# Onnx + Tantivy + Onnx Rerank（业界基线）
cargo run --features full -- eval --embedder onnx --model-path ... \
    --bm25 tantivy --rerank onnx --reranker-model-path ...
# 对比 reports/eval-*.md，肉眼看 recall@5 / MRR@5 趋势
```

### 自动化

| 手段 | 范围 | 来源 |
|---|---|---|
| 单元测试 × 5 | recall/MRR 算法 + 报告渲染 | `arkui-rag-eval` |
| 端到端集成测 × 1 | 完整 index → eval 闭环 | `arkui-rag-eval/tests/eval_end_to_end.rs` |
| pre-commit `M-STATUS-PER-ROUND` | 本 commit 强制：Round 10 + STATUS-day6-eval.md 配套 | ✅ |
| pre-commit `M-FEATURE-PLAN` | feature log 必含 `## Plan` + `## 对话摘要` | ✅ |

### 暂未自动化（明确缺口）

- ❌ CI 跑评估（待 GitHub CI 启用，Day 3.5 已搁置）
- ❌ 评估趋势线（多 commit 对比，待 Day 6 续 / Week 4）
- ❌ 评估自动入 `make smoke`（待校准真实评估集后再加，避免阻塞）

---

## 与上一阶段（STATUS-day5-reranker）的关联性

### 增量

| 维度 | Day 5 (Reranker 真活) | Day 6 (Eval 真活) |
|---|---|---|
| Crate 数 | 8 | **9** (+ arkui-rag-eval) |
| 流水线能力 | retrieve → rerank（无评估） | + **量化评估**（recall/MRR/延迟） |
| CLI subcommand 数 | 4 (serve/index/query/corpus) | **5** (+ eval) |
| 测试数 | 25 + onnx 1 + tantivy 7 + reranker 2 = 35 极限 | 31 + onnx 1 + tantivy 7 + reranker 2 = **41 极限** |
| 业界基线 | §1.6 + §8.5 共识 2 | + **§8.5 共识 4（Eval-Driven）** |

### 兼容性

- ✅ 无破坏性变更
- ✅ `Eval` subcommand 完全新增（不影响 query / index / serve）
- ✅ Evaluator 接口接收 `Arc<dyn Retriever>` + 可选 `Arc<dyn Reranker>`，不要求改 trait
- ✅ 报告路径默认到 `reports/`（已存在的目录），与 agent harness 自动产物目录约定一致

### 与规则 #17 的协同

本 STATUS 是规则 #17 立起来后**第 2 份**按规则走的产物（第 1 份是 STATUS-day5-reranker）。流程一致：

```
agent 写 10-*.md → 同 commit 写 docs/STATUS-day6-eval.md
                → commit.sh touch .agent-pending
                → pre-commit M-STATUS-PER-ROUND PASS
                → 入库
```

规则连续生效 ✓

---

## 完成度 / 下一阶段

### Day 6 完成度

| 项 | 状态 |
|---|---|
| `arkui-rag-eval` crate 接口完整 | ✅ |
| recall@k / MRR@k / latency 三大指标 | ✅ |
| Markdown 报告（3 节） | ✅ |
| CLI `eval` subcommand | ✅ |
| 8 条 sample 评估集 + 标注指南 | ✅ |
| 端到端集成测试 | ✅ |
| **用户真实校准评估集** | ⏳ 用户责任（投真实 corpus 后） |
| **多配置矩阵对比报告** | ⏳ 待用户跑 |

**对照 6 周 MVP 路线图**：

| 章节 | 状态 |
|---|---|
| §1.5 RAG 评估体系 | ✅ recall + MRR（基线） |
| §2.7 评估指标设计 | 部分 ✅（recall/MRR） · 代码专属（编译通过率/一多合规）⏳ Week 4 |
| §8.5 共识 4 Eval-Driven | ✅ |
| 附录 A.4 核心 KPI | 部分 ✅（延迟 + recall） · faithfulness/answer relevancy ⏳ Week 3 RAGAS |

### 下一阶段建议（按优先级 + 价值）

| 候选 | 价值 | 预估工作量 |
|---|---|---|
| 🟢 **Day 7 HyDE 改写器** | 用现有评估集量化 HyDE 对自然语言 query 的提升 | 1-2 commit |
| 🟢 Day 10 tree-sitter 代码切分 | 代码 corpus 真活（解锁 ArkTS/Kotlin/Swift） | 2 commit |
| 🟡 Day 8 tantivy-jieba | 中文 BM25 精度（评估集应能直接量化提升） | 0.5 commit |
| 🟡 Day 9 LanceDB | 大规模 corpus（>10k chunks） | 1-2 commit |
| 🟡 Day 11 Parent-Child 索引 | 检索小返回大（方案 §1.4 标准） | 1 commit |
| ⚪ Week 3 RAGAS 接入 | faithfulness + answer relevancy | 2-3 commit |
| ⚪ Week 4 HTTP/MCP | 协议层 | 3-5 commit |

**Agent 推荐**：**Day 7 HyDE 改写器**。理由：
1. HyDE 是方案 §2.4 "检索流水线"的关键中间件，能让自然语言 query 转代码风格再检索
2. 现在有评估集了，HyDE 接入后**第一时间能量化效果**（mock retrieval 阶段就能看出 query 改写对 chunk 命中率的影响）
3. 与 §1.2 Advanced RAG 范式对齐
4. 工作量较小（1-2 commit），快速迭代

### 重要的"非完成"项

- ❌ Sample 评估集的 chunk_id 是**占位**（如 `migration/router.md#Router/pushUrl@9`），用户投放真实 corpus 后必须校准
- ❌ 当前评估不区分 hit.source（向量 vs BM25 vs Reranked）—— Week 3 可加按 source 分组的子指标
- ❌ 没有"配置矩阵自动跑"工具 —— 用户需手工连跑多次（Day 6 续可加 `eval --matrix`）
- ❌ 评估集生成自动化（LLM 半自动 + 人工校验）→ 长期演进
