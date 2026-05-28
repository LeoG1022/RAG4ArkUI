# rag4arkui-core

> 状态：in-progress (Day 19 接入指南就绪 · **Claude Code 接入完整闭环** · 完成度 ~70%)
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
- [3-2026-05-27-day2-status-doc.md](3-2026-05-27-day2-status-doc.md) — Day 2 阶段快照文档（架构图 + 输入/输出 + 验证手段）
- [4-2026-05-27-day2-smoke.md](4-2026-05-27-day2-smoke.md) — Day 2.5 demo smoke 脚本（端到端 CLI 二进制行为验证）
- [5-2026-05-27-day3-onnx-embedder.md](5-2026-05-27-day3-onnx-embedder.md) — Day 3 OnnxEmbedder async wrapper + CLI --embedder onnx 真实语义检索上线
- [6-2026-05-27-day3-ci.md](6-2026-05-27-day3-ci.md) — Day 3.5 GitHub Actions CI（check / test / clippy / fmt / smoke + onnx 手动）
- [7-2026-05-27-day4-bm25-tantivy.md](7-2026-05-27-day4-bm25-tantivy.md) — Day 4 BM25 / Tantivy 实装（HybridRetriever 真正双路 RRF 融合）· STATUS：`docs/STATUS-day4-bm25-tantivy.md`（追溯）
- [8-2026-05-27-bootstrap-status-rule.md](8-2026-05-27-bootstrap-status-rule.md) — Bootstrap：立 AGENTS.md 规则 #17（每轮 STATUS 硬性）· STATUS：`docs/STATUS-bootstrap-status-rule.md`
- [9-2026-05-27-day5-reranker.md](9-2026-05-27-day5-reranker.md) — Day 5 Reranker 真活（BGE-Reranker-v2 ONNX）· STATUS：`docs/STATUS-day5-reranker.md`
- [10-2026-05-27-day6-eval.md](10-2026-05-27-day6-eval.md) — Day 6 检索质量评估（arkui-rag-eval crate · recall@k + MRR + 延迟）· STATUS：`docs/STATUS-day6-eval.md`
- [11-2026-05-27-roadmap-doc.md](11-2026-05-27-roadmap-doc.md) — ROADMAP 全景图归档到 docs/ · STATUS：`docs/STATUS-roadmap-doc.md`
- [12-2026-05-27-day7-hyde.md](12-2026-05-27-day7-hyde.md) — Day 7 HyDE 改写器（QueryEnhancer + MockHyde · CLI --hyde · Week 2 全部达成 ⭐）· STATUS：`docs/STATUS-day7-hyde.md`
- [13-2026-05-27-day10-tree-sitter.md](13-2026-05-27-day10-tree-sitter.md) — Day 10 tree-sitter 代码切分（TypeScriptChunker · ChunkerDispatcher · Indexer 重构 · 代码 corpus 真活）· STATUS：`docs/STATUS-day10-tree-sitter.md`
- [14-2026-05-28-day9-lancedb.md](14-2026-05-28-day9-lancedb.md) — Day 9 LanceDB 嵌入式向量库（VectorBackend 抽象 · CLI --vector · **Week 1 7/7 全部达成** ⭐）· STATUS：`docs/STATUS-day9-lancedb.md`
- [15-2026-05-28-day11-parent-child.md](15-2026-05-28-day11-parent-child.md) — Day 11 Parent-Child 父子索引（Chunker 自动填 parent_id + ContextAssembler + CLI --expand-parent · 方案 §1.4 标准）· STATUS：`docs/STATUS-day11-parent-child.md`
- [16-2026-05-28-day14-http.md](16-2026-05-28-day14-http.md) — Day 14 HTTP/REST Server（axum 真活 · /search /health /corpus/list + AppState 抽象 · 关键路径起点突破 ⭐）· STATUS：`docs/STATUS-day14-http.md`
- [17-2026-05-28-day15-mcp.md](17-2026-05-28-day15-mcp.md) — Day 15 MCP Server（手撸 JSON-RPC stdio · 4 tools · **Claude Code 接入就绪 · Week 3 协议层 3/3 ⭐**）· STATUS：`docs/STATUS-day15-mcp.md`
- [18-2026-05-28-day19-claude-code.md](18-2026-05-28-day19-claude-code.md) — Day 19 Claude Code 接入验证（接入指南 10 节 + bash 端到端 demo · **Week 5 1/1 ⭐ · ~70%**）· STATUS：`docs/STATUS-day19-claude-code.md`
