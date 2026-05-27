# rag4arkui-core

> 状态：in-progress (Day 2 端到端 Mock Demo 可跑)
> 创建：2026-05-27

## 用途

`RAG4ArkUI` 产品的核心代码 —— 7 个 Cargo crate 组成的本地 RAG 引擎。
完整设计依据见 [`docs/RAG4ArkUI-完整技术方案.md`](../../../docs/RAG4ArkUI-完整技术方案.md) 与 [`docs/ADR-002-crate-structure.md`](../../../docs/ADR-002-crate-structure.md)。

## 涉及代码

- Rust workspace：[`crates/`](../../../crates/)（**8 个 crate**，Day 2 新增 indexer）
  - `arkui-rag-core` —— trait + 类型
  - `arkui-rag-embedding` —— ONNX BGE-M3（§7.2）+ Mock
  - `arkui-rag-storage` —— VectorStore/BM25Index/MetadataStore traits + **InMemory 实现 + JSON 持久化**
  - `arkui-rag-chunker` —— Markdown（**含 YAML frontmatter**）+ tree-sitter stub
  - `arkui-rag-retrieval` —— **Hybrid 真活** + RRF + Reranker stub
  - `arkui-rag-indexer` —— **Day 2 新增**：索引流水线编排
  - `arkui-rag-server` —— HTTP + MCP + LSP (stub)
  - `arkui-rag-cli` —— `arkui-rag` binary（**index/query 真活**）
- Corpus 契约：[`corpus/README.md`](../../../corpus/README.md)
- 构建：[`Makefile`](../../../Makefile)、[`rust-toolchain.toml`](../../../rust-toolchain.toml)

## 迭代日志

- [1-2026-05-27-day1-skeleton.md](1-2026-05-27-day1-skeleton.md) — Day 1 骨架（trait + 类型 + ONNX 代码 + CLI stub）
- [2-2026-05-27-day2-mock-demo.md](2-2026-05-27-day2-mock-demo.md) — Day 2 端到端 Mock Demo（indexer + 持久化 + frontmatter + HybridRetriever 真活）
