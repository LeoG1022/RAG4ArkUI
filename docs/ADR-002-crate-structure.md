# ADR-002 · Cargo Workspace 拆分：7 个 Crate

- **Status**：Accepted
- **Date**：2026-05-26
- **Deciders**：Agent 自主决策（依据完整技术方案 §9 图 3 类图）
- **Context Doc**：[`RAG4ArkUI-完整技术方案.md`](RAG4ArkUI-完整技术方案.md) §4.1 / §9 图 3

## Context

需要把 RAG 引擎拆成多个 crate，平衡：
- **可热插拔**：trait 与实现分离（Retriever / Embedder / Reranker / Chunker 都可换后端）
- **可独立编译**：开发期改一个 crate 不需要 rebuild 全世界
- **可独立分发**：未来可能把 core 独立发到 crates.io，server 留在私仓
- **编译时长可控**：重 ML 依赖（ort ~300MB）必须能 opt-out

## Decision

7 个 crate，照搬完整方案图 3 类图边界：

| Crate | 职责 | 关键依赖 |
|---|---|---|
| `arkui-rag-core` | trait + 类型 + Error（无任何后端） | thiserror、serde、async-trait |
| `arkui-rag-embedding` | Embedder 实现（Mock + ONNX BGE-M3） | ort (feature `onnx`)、tokenizers、ndarray |
| `arkui-rag-storage` | VectorStore + BM25Index + MetadataStore traits | (Week 2: lancedb + tantivy) |
| `arkui-rag-chunker` | ASTChunker 实现（Markdown + tree-sitter） | (Week 2: tree-sitter-typescript / kotlin / swift) |
| `arkui-rag-retrieval` | HybridRetriever + RRF + CrossEncoderReranker | core + storage + embedding |
| `arkui-rag-server` | HTTP + MCP + LSP 适配 | axum (feature `http`) / (mcp / lsp 待定) |
| `arkui-rag-cli` | 二进制入口 `arkui-rag` | server + retrieval + chunker + embedding |

依赖图（单向）：
```
cli → server → retrieval → storage → core
                                ↘ embedding → core
                          chunker → core
```

## Feature gate 策略

**核心问题**：ONNX Runtime 编译耗时长（首次 5-10 分钟、~300MB 原生库下载），不能挡在 `cargo check` 默认路径上。

**解法**：把 ONNX 相关依赖在 `arkui-rag-embedding` 中标 `optional = true`，靠 feature `onnx` 触发：

```toml
[dependencies]
ort = { workspace = true, optional = true }
tokenizers = { workspace = true, optional = true }
ndarray = { workspace = true, optional = true }

[features]
default = []
onnx = ["dep:ort", "dep:tokenizers", "dep:ndarray"]
```

`server` crate 同样把 `axum` 放 `http` feature。

**验证矩阵**：
- `cargo check --workspace`（默认）→ 不拉 ORT，~3 分钟
- `cargo check -p arkui-rag-embedding --features onnx` → 完整 ORT，首次 5-10 分钟
- `cargo check -p arkui-rag-server --features http,mcp,lsp` → 完整协议层

## Consequences

**正向**：
- 改 trait 触发 5 个 crate 重编译，但每个 crate 都很薄，单次 < 10 秒
- 后端切换（如 LanceDB → Qdrant）只动 storage crate，core 和 retrieval 不动
- 贡献者可只 clone + 关注单个 crate

**负向**：
- 跨 crate import 路径较长（`arkui_rag_core::Retriever`）
- workspace.dependencies 集中管理意味着升级一个版本要测所有 crate
- 7 个 Cargo.toml + 7 个 README，维护成本增加

## Anti-Patterns（避免犯的错）

❌ 把所有代码放一个 `arkui-rag` crate：编译时长爆炸、不可独立分发
❌ 按层切分（如 `traits` crate + `impl` crate）：违反 Rust 习惯（trait 通常和首要实现同 crate）
❌ 把 `chunker` 合到 `core`：会把 tree-sitter 这种重依赖拖进 core，违反 core 必须 light 的原则
