# rag4arkui-core

> 状态：in-progress (Day 1 骨架)
> 创建：2026-05-27

## 用途

`RAG4ArkUI` 产品的核心代码 —— 7 个 Cargo crate 组成的本地 RAG 引擎。
完整设计依据见 [`docs/RAG4ArkUI-完整技术方案.md`](../../../docs/RAG4ArkUI-完整技术方案.md) 与 [`docs/ADR-002-crate-structure.md`](../../../docs/ADR-002-crate-structure.md)。

## 涉及代码

- Rust workspace：[`crates/`](../../../crates/)
  - `arkui-rag-core` —— trait + 类型
  - `arkui-rag-embedding` —— ONNX BGE-M3（§7.2）+ Mock
  - `arkui-rag-storage` —— VectorStore + BM25Index + MetadataStore traits
  - `arkui-rag-chunker` —— Markdown + tree-sitter (stub)
  - `arkui-rag-retrieval` —— Hybrid + RRF + Rerank
  - `arkui-rag-server` —— HTTP + MCP + LSP (stub)
  - `arkui-rag-cli` —— `arkui-rag` binary
- Corpus 契约：[`corpus/README.md`](../../../corpus/README.md)
- 构建：[`Makefile`](../../../Makefile)、[`rust-toolchain.toml`](../../../rust-toolchain.toml)

## 迭代日志

- [1-2026-05-27-day1-skeleton.md](1-2026-05-27-day1-skeleton.md) — Day 1 骨架（trait + 类型 + ONNX 代码 + CLI stub）
