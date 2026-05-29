# 当前状态

> 实时反映最新一轮 ROADMAP 进度。每个 commit 后 agent 同步更新。

## 完成度

| 阶段 | 进度 |
|---|---|
| Week 1: Rust 骨架 + tree-sitter + LanceDB + Tantivy + BGE-M3 | **7/7** ✅ |
| Week 2: 混合检索 + Reranker + HyDE + 评估集 | **4/4** ✅ |
| Week 3: HTTP + MCP + CLI | **3/3** ✅ |
| Week 4: 协议层（HTTP + MCP + LSP） | **3/3** ✅ ⭐ |
| Week 5: Claude Code 接入 | **1/1** ✅ |
| Week 6: 发布 + 文档站 + 评估报告 | **4/4** ✅ |

**总完成度 ~85%** · 仅剩 mdBook 文档站（本轮）+ 1.0 release page 待做。

## 关键里程碑

- ✅ Day 16: 协议层 3/3 完整（HTTP + MCP + LSP）
- ✅ Day 19: Claude Code MCP 接入指南 + demo
- ✅ Day 20a: 本地 host release artifact
- ✅ Day 20b: 4 平台 CI matrix 自动 release
- ✅ Day 21: `arkui-rag corpus pull` 真活
- ✅ Day 22（本轮）: mdBook 文档站

## 默认 release 能力

- 6 个 Cargo features: `http,mcp,lsp,tantivy,typescript,corpus-pull`
- Release binary: 11 MB (Apple Silicon arm64)
- Release tarball: 4.1 MB (gzip)
- 4 平台 build matrix: aarch64/x86_64 darwin + linux gnu + windows msvc

更多细节见 [完整路线图](roadmap.md)。
