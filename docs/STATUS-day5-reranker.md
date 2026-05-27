# STATUS · Day 5 · Reranker 真活

> 日期：2026-05-27
> 对应 commit：[本 commit · Day 5 Reranker 主线]
> 对应 feature log：[`feedback/features/rag4arkui-core/9-2026-05-27-day5-reranker.md`](../feedback/features/rag4arkui-core/9-2026-05-27-day5-reranker.md)
> 上一阶段：[`STATUS-bootstrap-status-rule.md`](STATUS-bootstrap-status-rule.md)（规则 #17 立起来，本 STATUS 是首份按新规则走的产物）
> 下一阶段：`STATUS-day6-eval.md`（计划：检索质量评估集 · recall@k + RAGAS）

> 🎯 **里程碑**：技术方案 §1.6 第 1 条 + §8.5 共识 2 双达成 —— "Hybrid + Rerank" 业界基线完整。

---

## 当前状态

把 `CrossEncoderReranker` 从 Day 1 的 identity stub 升级为可工作的 cross-encoder 精排：

- **MockReranker 路径**：用 Day 1 的 `CrossEncoderReranker`（identity + truncate）→ 适合 demo / 测试
- **OnnxReranker 路径（Day 5 新增）**：BGE-Reranker-v2-m3 真实推理 → 适合生产

### 交付清单

| 模块 | 变化 |
|---|---|
| `arkui-rag-embedding` | 新增 2 个文件 ~250 行：`reranker_onnx.rs`（同步 API） + `onnx_reranker.rs`（async wrapper） |
| `arkui-rag-embedding/lib.rs` | 导出 `OnnxReranker` / `RerankerModel` / `SharedReranker`（feature `onnx` gated） |
| `arkui-rag-cli` | `RerankerKind` enum + 4 个新参数 + `build_reranker` 双 cfg + cmd_query 链式调用 |
| `crates/arkui-rag-embedding/README.md` | 增 Day 5 Reranker 节 + 用法示例 + 模型获取（reranker 同 embedding） |
| `crates/README.md` | 速查表标 Day 5 进度 |

### 流水线扩展

```
Query
  → Embedder.encode(query) ─┐
                            ↓
  ┌─ VectorStore.search ────┤
  │  BM25Index.search ──────┼─→ RRF 融合 → Top-50（Hybrid）
  └─────────────────────────┤
                            ↓
                       Reranker.rerank(query, hits) → Top-5
                            ↓
                        最终输出 + Citation
```

Day 5 新增最后一步（Reranker），仅在 `--rerank none` 之外的模式触发。

---

## 输入契约

### CLI 新增参数（Query 子命令）

| 参数 | 取值 | 默认 | 用途 |
|---|---|---|---|
| `--rerank` | `none` / `mock` / `onnx` | `none` | Reranker 类型选择 |
| `--pre-rerank-k <N>` | usize | `50` | Reranker 启用时召回阶段取 Top-K 送精排 |
| `--reranker-model-path <DIR>` | path | - | onnx 必填，模型目录 |
| `--reranker-model-id <NAME>` | string | `bge-reranker-v2-m3` | 模型标识（日志/标识） |

### 编译开关

| feature | 行为 |
|---|---|
| 默认 | OnnxReranker 不编译，`--rerank onnx` 报错 |
| `--features onnx` | OnnxReranker 可用（与 OnnxEmbedder 同 feature） |
| `--features full` | onnx + tantivy 一键全启 |

### 模型文件

| 模型 | 用途 | 大小 | 位置约定 |
|---|---|---|---|
| BGE-M3 ONNX | Embedder | ~2.2GB（int8 后 ~568MB） | `~/.arkui-rag/models/bge-m3-onnx/` |
| **BGE-Reranker-v2-m3 ONNX** | Reranker | ~568MB（int8 后 ~140MB） | `~/.arkui-rag/models/bge-reranker-v2-m3-onnx/` |

每个目录含：`model.onnx` + `tokenizer.json`。

---

## 输出契约

### CLI query 输出

```
✅ Top-5 hits (embedder=bge-m3 · bm25=tantivy · rerank=bge-reranker-v2-m3)
                                                ^^^^^^^^^^^^^^^^^^^^^^^^^^ Day 5 新增

─── [1] score=8.7421 ──────────────────       ← 来自 reranker 的 logit 分数
  source : router.md L9-11
  heading: Router > pushUrl
  preview: 推送新页面到路由栈。可以传递参数和回调。
```

注意：开启 reranker 时 score 是 reranker 的 logit（无界，越高越好）；不开启时 score 是 RRF 分数（≈ 1/(k+rank)）。两种语义不同。

### Hit.source 字段新值

| 源 | 触发场景 |
|---|---|
| `Vector` | 仅向量召回 |
| `Bm25` | 仅 BM25 召回 |
| `Hybrid` | RRF 融合（Day 4 默认） |
| **`Reranked`** | **Day 5 新增** · 经过 reranker 重排 |

序列化为 JSON 时（`Citation` / `IndexSnapshot`）值是 `"reranked"`。

### 内部 trait 实现

`OnnxReranker` 实现 `arkui_rag_core::Reranker`：

```rust
#[async_trait]
impl Reranker for OnnxReranker {
    fn model_id(&self) -> &str;
    async fn rerank(&self, query: &str, hits: Vec<Hit>, top_n: usize) -> Result<Vec<Hit>>;
}
```

---

## 验证手段

### 用户手动

```bash
# 1. 默认编译（reranker 代码 feature-gated 不参编）
make check
make test                                # 24 个测试

# 2. 启用 onnx 编译（含 reranker，首次 5-10 分钟）
make build-onnx
cd crates && cargo test --features arkui-rag-embedding/onnx
# 期望：25 个测试（24 + OnnxReranker::load_missing_model_returns_err）

# 3. 全启编译（onnx + tantivy）
make build-full
# 期望：所有真活 + ~10 分钟 ort + tantivy 编译

# 4. 真实端到端 Hybrid + Rerank（需 ~2.6GB 模型）
cargo run --features full -p arkui-rag-cli -- \
    index --source corpus --embedder onnx --model-path ~/.arkui-rag/models/bge-m3-onnx --bm25 tantivy

cargo run --features full -p arkui-rag-cli -- \
    query --text "ArkUI-X 如何实现下拉刷新" --k 5 \
    --embedder onnx --model-path ~/.arkui-rag/models/bge-m3-onnx \
    --bm25 tantivy \
    --rerank onnx --reranker-model-path ~/.arkui-rag/models/bge-reranker-v2-m3-onnx \
    --pre-rerank-k 50

# 5. 真模型集成测试（需环境变量）
export ARKUI_RAG_RERANKER_DIR=~/.arkui-rag/models/bge-reranker-v2-m3-onnx
cd crates && cargo test --features arkui-rag-embedding/onnx onnx_reranker -- --ignored
# 期望：rerank_with_real_model 测试断言"rel" chunk 排第 1，"noise" 排末位
```

### 自动化

| 手段 | 范围 | 数量 |
|---|---|---|
| 单元测试 × 24 | 默认 features（mock + memory + RRF） | 不变 |
| OnnxReranker 单测 × 1 | feature `onnx` 启用 | +1（24 → 25） |
| OnnxReranker 集成测 × 1 | `#[ignore]` 真模型 | +1 ignored |
| pre-commit `M-STATUS-PER-ROUND` | 本 commit 强制：Round 9 + STATUS-day5-reranker.md 配套 | ✅ |
| HitSource::Reranked 标识 | 让 retrieved vs reranked 在 JSON 序列化时区分 | 编译期保证 |

### 输出动态 shape 兼容

针对 BGE-Reranker 不同 optimum 导出的 ONNX 输出形状，`RerankerModel::score` 同时支持：
- `[batch]`（1 维）
- `[batch, 1]`（2 维 num_labels=1）
- `[batch, 2]`（2 维 binary classification，取索引 1）

形状不符 → 显式 `bail!("Reranker 输出 shape 异常")`。

---

## 与上一阶段（STATUS-bootstrap-status-rule / Day 4）的关联性

### 增量

| 维度 | Day 4 + 规则化 | Day 5 |
|---|---|---|
| 流水线 | 召回（vector + bm25 + RRF） | + **精排（reranker）** |
| CLI 参数总数 | 8（含 bm25） | **12**（+ 4 个 rerank 系列） |
| 编译矩阵 | onnx / tantivy / full | 不变（reranker 复用 onnx feature） |
| 测试数 | 24 + 7 onnx + 7 tantivy = 38 极限 | 25 + 7 onnx + 7 tantivy = 39 极限 |
| 业界基线 | §1.6 第 1 条达成 | + **§8.5 共识 2 达成** |

### 8 种检索质量组合

Day 5 起，用户可在 3 维上自由组合：

| Embedder | BM25 | Rerank | 等级 |
|---|---|---|---|
| Mock | Memory | None | Day 2 demo（无语义无关键词无精排） |
| Mock | Tantivy | None | Day 4 简化 hybrid |
| Onnx | Memory | None | Day 3 纯语义 |
| Onnx | Tantivy | None | Day 4 真 hybrid |
| Onnx | Memory | Mock | Day 5 占位精排（无意义） |
| Onnx | Tantivy | Mock | Day 5 truncate 精排（识别力低） |
| Onnx | Memory | Onnx | Day 5 语义召回 + 精排 |
| **Onnx + Tantivy + Onnx Rerank** | | | **业界产品级标配** |

### 破坏性变更

- 无（所有新参数默认 none/50，老命令完全兼容）
- HitSource::Reranked 是 Day 1 已在 enum 里预留的变体，不破坏 JSON schema

### 与既有规则的协同（规则化效果验证）

本 STATUS 是规则 #17 立起来后**首份按新规则走的产物**。流程：

```
Agent 写 9-*.md feature log
  → 同 commit 必须有 docs/STATUS-day5-reranker.md（本文件）
  → scripts/commit.sh touch .agent-pending
    → pre-commit → M-STATUS-PER-ROUND 校验
    → PASS（feature log slug "day5-reranker" ↔ STATUS-day5-reranker.md 严格一致）
  → 入库
```

规则真活 ✓

---

## 完成度 / 下一阶段

### Day 5 完成度

| 项 | 状态 |
|---|---|
| OnnxReranker async wrapper | ✅ |
| CLI --rerank 接入 + 4 个参数 | ✅ |
| ONNX 输出 shape 兼容（1D / 2D） | ✅ |
| feature gate（onnx / full） | ✅ |
| HitSource::Reranked 区分 | ✅ |
| 单元测试（load_missing 路径） | ✅ |
| 集成测试（真模型 ignored） | ✅ |
| 文档（embedding + crates README） | ✅ |
| 端到端真活验证 | ⏳ 用户需先获取 BGE-Reranker-v2 模型 |

**对照 6 周 MVP**：

| 章节 | 状态 |
|---|---|
| §1.6 第 1 条（Hybrid 是基线） | ✅ |
| §8.5 共识 2（Reranker 是分水岭） | ✅ |
| §6.2 模型 2（Reranker 双阶段） | ✅ |
| §9 图 6 检索时序图 | ✅ 完整对齐 |

### 下一阶段建议

| 候选 | 价值 | 预估 |
|---|---|---|
| 🟢 **Day 6 检索质量评估**（推荐） | Hybrid+Rerank 整套就位，可量化评估；建立 recall@k / MRR / RAGAS 基线 | 2 commit |
| 🟡 HyDE 改写器（小 LLM） | 让用户口语 query 转代码风格再检索 | 1-2 commit |
| 🟡 tantivy-jieba | 中文 BM25 精度（替换 ngram） | 0.5 commit |
| 🟡 LanceDB | chunks > 10k 解锁 | 1-2 commit |
| 🟡 tree-sitter | 代码 corpus 真活 | 1-2 commit |
| ⚪ Week 4 HTTP/MCP/LSP | 协议层 | 大工程 |

**Agent 推荐**：Day 6 检索质量评估。理由：
1. 现在有真 Hybrid + 真 Rerank，但无评估 → 看不见质量演进
2. 评估集是后续调参（pre_rerank_k / rrf_k / 模型版本）的前提
3. 投资低（建 50 query 评估集 + 跑 RAGAS）回报高

### 重要的"非完成"项

- ❌ 中文分词仍是 ngram（精度有限）
- ❌ 检索质量没有量化基线 → Day 6 解决
- ❌ Reranker 真实模型未实测（用户责任：拉 ~568MB 模型）
- ❌ 大规模 corpus（>10k chunks）InMemoryVectorStore 会慢
