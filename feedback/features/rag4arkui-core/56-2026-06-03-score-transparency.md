# 56 — score-transparency（Round 52）

> 日期：2026-06-03
> 涉及代码：`crates/arkui-rag-core/src/hit.rs` · `crates/arkui-rag-retrieval/src/{hybrid,rrf}.rs` · `crates/arkui-rag-cli/src/main.rs`（+ 8 个 Hit literal 补字段）
> 类型：可观测性 + 阈值过滤（暴露真实 cosine 相似度 · 解决 RRF score 无信息量）

## 本轮目标

Phase A smoke 报告暴露：**所有 query top-3 score 恒定 = 0.0164/0.0161/0.0159**（= 1/61, 1/62, 1/63）· 这是 RRF rank-based fusion 的特征。用户无法从 score 判断"命中是否相关"：

- 入门问题命中合理 · score 0.0164
- 负样本（"今天天气怎么样" / "怎么炒鸡蛋"）也命中 wrong chunk · score 也 0.0164

本轮把真实 cosine / BM25 score 暴露出来：用户看到 cosine=0.85 vs cosine=0.15 一眼分辨"靠谱不靠谱"。同时加 `--min-vector-score` 阈值参数过滤负样本。

## Plan

### 决策 A · Hit struct 加 Optional<f32> 字段（最小侵入）

```rust
pub struct Hit {
    pub chunk: Chunk,
    pub score: f32,                  // 仍是 RRF（或单路 raw · 看上下文）
    pub source: HitSource,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vector_score: Option<f32>,   // 真实 cosine（仅 vector 路径 / hybrid 保留）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bm25_score: Option<f32>,     // 真实 BM25 raw（仅 bm25 路径 / hybrid 保留）
}
```

`#[serde(default)]` 保证既有 index.json 反序列化无破坏。

### 决策 B · HybridRetriever 在 fuse 前保留 raw

```rust
for h in vec_hits.iter_mut() { h.vector_score = Some(h.score); }
for h in bm_hits.iter_mut() { h.bm25_score = Some(h.score); }
let mut fused = rrf_fuse(vec![vec_hits, bm_hits], self.rrf_k);
```

`rrf_fuse` 合并时取 Copy 字段提前 · 避免 closure 借不到。

### 决策 C · CLI 加 `--min-vector-score <f32>`

```bash
arkui-rag query --text "..." --min-vector-score 0.3
```

BGE-M3 经验值：
- 0.3 弱相关
- 0.5 中等相关
- 0.7+ 强相关

在 RRF 之前过滤 · 排除负样本。

### 决策 D · CLI 输出三 score

```
─── [1] rrf=0.0164  vector=0.7234  bm25=12.50 ──
  source : start-overview.md L19-21
  heading: 开发准备 > 开发工具
```

让用户清晰看到三个不同维度的相似度。

### 不动

- 默认行为：不传 `--min-vector-score` 不过滤（兼容既有用户）
- index.json 格式（serde default 兼容老 index）
- HybridRetriever 默认参数（per_branch_topk=50 / k=60）

## 对话摘要

| 轮次 | 用户 | Agent |
|---|---|---|
| 1 | Phase A smoke 报告 | 发现 score 全 0.0164/0.0161/0.0159（= RRF 1/(k+rank) 特征）· 命中 60% 准但 score 无信息量 |
| 2 | 「执行A」(暴露真 cosine + 加阈值) | 加 Hit.vector_score / bm25_score · cli --min-vector-score · 输出三 score |

## 改动要点

### 新增字段
- `Hit::vector_score: Option<f32>`（serde default · None 兼容老 index）
- `Hit::bm25_score: Option<f32>`
- CLI Query `--min-vector-score <f32>` 阈值参数

### 修改
- `crates/arkui-rag-retrieval/src/hybrid.rs` fuse 前赋值
- `crates/arkui-rag-retrieval/src/rrf.rs` fuse 合并保留 Copy 字段
- `crates/arkui-rag-cli/src/main.rs` cmd_query 加阈值 + 三 score 输出
- 8 个 Hit literal 补 `vector_score: None, bm25_score: None`：
  - arkui-rag-storage/src/memory.rs
  - arkui-rag-storage/src/tantivy_bm25.rs
  - arkui-rag-storage/src/lancedb_store.rs
  - arkui-rag-eval/src/evaluator.rs
  - arkui-rag-embedding/src/onnx_reranker.rs
  - arkui-rag-server/src/mcp.rs
  - arkui-rag-server/src/lsp.rs
  - arkui-rag-retrieval/src/{rrf,context}.rs

## 验证结果

```bash
cargo build --release ...,onnx   # ✓ 待完成
~/.local/bin/arkui-rag query --text "ArkUI-X 怎么创建第一个应用" \
    --min-vector-score 0.3 \
    --embedder onnx --model-path ~/.arkui-rag/models/bge-m3 \
    --index-path /Users/leo/tmp-index-pull2/index.json --bm25 tantivy
# 期望：rrf / vector / bm25 三列分开 · 负样本被 0.3 阈值过滤
```

## 残留 / 下一轮

- [x] Hit 加 vector_score / bm25_score
- [x] HybridRetriever fuse 前保留 raw
- [x] CLI 三 score 输出 + --min-vector-score
- [x] 8 个 Hit literal 补 None
- [ ] **cli build 完成 + 跑 smoke 重新报告**
- [ ] **Phase B 全量 build 完毕后跑同一 smoke · 对比覆盖度**
- [ ] **reranker 启用对 score 影响**（当前阈值在 rerank 之前 · 顺序是否合理？）
- [ ] **eval 命令也用 vector_score 计算指标**（recall@K 时按真实相似度排）
- [ ] **HTTP / MCP / LSP server 输出 vector_score / bm25_score**（当前 mock 返回 None · 真活后透传）
