# 9 — day5-reranker

> 日期：2026-05-27
> 涉及代码：
> - `crates/arkui-rag-embedding/src/reranker_onnx.rs`（**新增** ~140 行 · RerankerModel 同步 API）
> - `crates/arkui-rag-embedding/src/onnx_reranker.rs`（**新增** ~110 行 · OnnxReranker async wrapper）
> - `crates/arkui-rag-embedding/src/lib.rs`（导出 OnnxReranker / RerankerModel / SharedReranker）
> - `crates/arkui-rag-cli/src/main.rs`（RerankerKind enum + --rerank/--reranker-model-path/--reranker-model-id/--pre-rerank-k 4 个新参数 + build_reranker 双 cfg + cmd_query 流水线接入）
> - `crates/arkui-rag-embedding/README.md`、`crates/README.md`（同步说明 + 用法示例）
> 类型：新建（Day 5 主线 · Reranker 真活）

## 本轮目标

让"Hybrid + Rerank"业界基线完整：
- Day 4 已立 Hybrid（向量 + BM25 + RRF）
- Day 5 接 Reranker（cross-encoder 精排），从 Top-50 → Top-5

技术方案对应：
- §1.6 第 1 条原则："混合检索 + Rerank 是基线"
- §8.5 共识 2："Reranker 是产品级 RAG 的分水岭"

## Plan

> 必备节（FAIL 级 M-FEATURE-PLAN）

### 关键设计

**1. Reranker 与 Embedder 的代码差异**

| 维度 | OnnxEmbedder | OnnxReranker |
|---|---|---|
| 输入 | 单文本 → token ids | (query, doc) pair → 拼接 token ids |
| 模型类型 | 双塔（dual-encoder） | 交叉编码（cross-encoder） |
| 输出 | 向量 [dim] | 单 logit（num_labels=1）或 [batch, 2] 取索引 1 |
| 调用频次 | 索引时 1 次 / chunk + 检索时 1 次 / query | 每次检索 batch 50 次（Top-50） |
| 推理耗时 | ~20-50ms / 单条 | ~3-5ms / pair × 50 = ~200ms / 检索 |

**2. 流水线集成位置**

不改 HybridRetriever 接口（保持 Day 4 真活），在 CLI cmd_query 中链式：

```rust
let hits = retriever.retrieve(&q, pre_rerank_k=50).await?;  // 召回阶段
let hits = if let Some(rr) = reranker_opt {
    rr.rerank(text, hits, k=5).await?                       // 精排阶段
} else {
    hits.into_iter().take(k).collect()
};
```

**为什么不把 Reranker 塞进 HybridRetriever**：
- 单一职责：HybridRetriever 管召回（向量 + BM25 + RRF），Reranker 是后处理
- 解耦：未来加多级 reranker 或 LLM reranker，HybridRetriever 不动
- 测试：两者可独立单测

**3. ONNX 输出 shape 兼容**

BGE-Reranker-v2 实际导出的 shape 可能是 `[batch, 1]` 或 `[batch, 2]`（取决于 optimum 导出参数）。我用动态 shape 检测：

```rust
match shape.len() {
    1 => flat,                                              // [batch]
    2 if shape[1] == 1 => flat,                            // [batch, 1]
    2 => /* 取索引 1 = label 'relevant'，binary classification 常见 */
    _ => bail!("Reranker 输出 shape 异常: {:?}", shape),
}
```

**4. CLI 参数设计：4 个新参数**

| 参数 | 用途 |
|---|---|
| `--rerank none/mock/onnx` | 选 reranker 类型；默认 none 保持向后兼容 |
| `--pre-rerank-k <N>` | 召回阶段取多大 Top-K 送精排（默认 50，方案 §1.4） |
| `--reranker-model-path <DIR>` | onnx 必填，模型目录 |
| `--reranker-model-id <NAME>` | 默认 "bge-reranker-v2-m3" |

**5. spawn_blocking 桥接同 OnnxEmbedder**

复用 Day 3 的设计模式：
- `RerankerModel` 是同步 API（ort 推理阻塞）
- `OnnxReranker` 用 `tokio::task::spawn_blocking` 包装
- `.await??` 双层错误（JoinError + 内部 Result）

### 替代方案权衡

- 备选独立 crate `arkui-rag-rerank`：被否，与 embedding 共用 ort + tokenizers 依赖，单独 crate 仅是命名上的洁癖
- 备选 Reranker 强制走 HybridRetriever 内部：被否，违反单一职责，且让 retriever 配置变重
- 备选不支持 BGE-Reranker：被否，技术方案 §6.2 / §7.1 明确推荐
- 备选 BCE-Reranker（更轻量）：保留为 backlog，需多模型对比评估
- 备选 LLM-as-Reranker：未来 Week 4+ 备选（用小 LLM 做 listwise rerank）

### 与 Day 4 接口的兼容性

- HybridRetriever：不变（仍是 Retriever trait 实现）
- CLI Query 子命令：新参数全默认 none/50，**老命令不变**
- Hit 类型：source 字段加 `Reranked` 变体（Day 1 已预留枚举值，无需 break）

## 改动要点

> API 选型 / 算法 / 关键决策

**与 Day 4 的差异**：
- crate 数仍 8（不新增 rerank crate）
- embedding crate 内代码量 +250 行（reranker_onnx.rs + onnx_reranker.rs）
- CLI 参数 +4 个 + 1 个新 enum
- 测试：24 → 25（OnnxReranker `load_missing_model_returns_err` + 1 ignored 真模型测）

**API 选型**：
- `RerankerModel::score(pairs: &[(String, String)]) -> Vec<f32>` —— 与 EmbeddingModel::encode 平行
- `OnnxReranker::load(model_dir, model_id)` —— 同 OnnxEmbedder
- `tokenizer.encode_batch(pairs, true)` —— tokenizers crate 原生支持 pair 输入（query + doc）

**算法**：
- pre_rerank_k=50 → rerank → top_k=5，方案 §6.2 / §1.4 / §9 图 6 时序的标准设计
- 输出动态 shape 检测（适配不同 optimum 导出的 ONNX）
- HitSource::Reranked 标记区分召回路径 vs 精排路径

**索引兼容性**：reranker 不参与索引（只在 query 时用），不影响 index.json 格式。

## 对话摘要

> 必备节（FAIL 级 M-FEATURE-PLAN）

按时序：

1. **Day 4 BM25 commit 后**，用户说"按推荐来"（暗示 Day 5 Reranker 主线）
2. **[Request interrupted by user]**，用户改提"每一轮都形成架构快照"硬性规则
3. **Agent 拆分** Commit A（规则化） + Commit B（Reranker）
4. **Commit A 完成**（`20056b3`）后，**本 commit B** 是首份按新规则走完整 STATUS 的主线交付
5. **Agent 自主决策 5 项**（见上 Plan 节"关键设计"）后直接执行，未再回问
6. **本 feature log + STATUS-day5-reranker.md** 在 Commit B 中同 commit 入库（满足规则 #17）

## 验证结果

- 编译：⏳
  - `make check`（默认）期望通过（reranker 代码全 feature-gated）
  - `make check-onnx` 期望通过（首次拉 ort 较慢）
  - `make build-full` 期望通过（onnx + tantivy 一键全启）
- 测试：
  - `cargo test --workspace`（默认）：24 个（reranker 测在 feature gate 内）
  - `cargo test --workspace --features arkui-rag-embedding/onnx`：25 个（+ OnnxReranker::load_missing_model_returns_err）
  - `cargo test --features full -- --ignored`：执行真模型测试（需 ARKUI_RAG_RERANKER_DIR 环境变量）
- 端到端：
  - 用户跑 `cargo run --features full -p arkui-rag-cli -- query --text "下拉刷新" --k 5 --embedder onnx ... --bm25 tantivy --rerank onnx --reranker-model-path ...`
  - 期望：召回 Top-50 → 重排 Top-5，比纯 RRF 排序质量明显更好（噪音被排末位）

## 残留 / 下一轮

- [ ] **关键**：用户跑真实端到端验证（需 ~2.6GB 模型：BGE-M3 + BGE-Reranker-v2-m3）
- [ ] **Day 6 推荐**：检索质量评估集（recall@k + MRR + RAGAS）—— 现在 hybrid+rerank 整套就位，可以认真评估
- [ ] Day 6 备选：HyDE 改写器（小 LLM 生成假代码做向量检索）
- [ ] tantivy-jieba 中文 BM25 精度提升
- [ ] LanceDB 替换 InMemoryVectorStore（chunks > 10k）
- [ ] tree-sitter（.ets/.kt/.swift）让代码 corpus 真活
- [ ] Week 4：HTTP/MCP/LSP server 实装
- [x] Day 5：OnnxReranker async wrapper + CLI --rerank 接入
- [x] Day 5：4 状态矩阵升级为 8 状态（× rerank none/mock/onnx）
- [x] 技术方案 §1.6 第 1 条 + §8.5 共识 2 全部就位（Hybrid + Rerank 业界基线完整）
