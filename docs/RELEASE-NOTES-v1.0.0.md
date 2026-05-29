# RAG4ArkUI v1.0.0 · Release Notes（草稿）

> 本文件是 v1.0.0 release 的 release notes 草稿。用户决定推 tag `v1.0.0` 时：
> 1. 检查本文内容，按需修订
> 2. 把所有 Cargo.toml `version` 从 `0.0.1` → `1.0.0`（workspace 顶层 + 9 个 crate 自身的 version.workspace = true 自动跟随）
> 3. 把 `INSTALL.txt` 模板里的版本号占位检查一遍
> 4. `git tag v1.0.0 && git push --tags`
> 5. CI matrix 4 平台 build + softprops/action-gh-release 自动上传
> 6. （可选）把本文 `gh release edit v1.0.0 --notes-file docs/RELEASE-NOTES-v1.0.0.md` 同步到 Release page

---

## 总览

RAG4ArkUI 1.0 是面向 OpenHarmony / ArkUI-X 的**本地化** RAG 代码生成与迁移系统的第一个稳定 release。

- 🦀 单文件 Rust 二进制 · 11 MB · 仅依赖 libSystem（macOS）
- 📦 6 个默认 features：HTTP + MCP + LSP + Tantivy + TypeScript + Corpus pull
- 🌐 4 平台 GitHub Releases 自动 build：aarch64/x86_64 darwin · linux gnu · windows msvc
- 📖 mdBook 文档站：https://keerecles.github.io/RAG4ArkUI/

## 主要能力

### 检索引擎

- Hybrid retrieval：向量 + BM25 + RRF 融合（Day 4）
- Cross-encoder reranker（Day 5 · 可选 `--rerank cross-encoder`）
- HyDE 查询改写（Day 7 · 可选 `--hyde mock`）
- Parent-Child 索引扩展（Day 11 · `--expand-parent`）
- 评估闭环（Day 6 · `arkui-rag eval` · recall@k / MRR / 延迟）

### 协议层 3/3 完整

| 协议 | 用途 | 文档 |
|---|---|---|
| HTTP/REST | IDE 插件 / curl | [usage/http](usage/http.md) |
| MCP stdio | Claude Code / Cursor | [usage/mcp](usage/mcp.md) |
| LSP stdio | DevEco / IntelliJ inline | [usage/lsp](usage/lsp.md) |

### Corpus 与分发

- `arkui-rag corpus pull --url|--from-file`（Day 21 · 用户一键拉取默认语料）
- 本地 `make release-local-verify`（Day 20a · 端到端打包+解压自验证）
- 4 平台 CI matrix `release.yml`（Day 20b · push tag `v*` 触发）

### 索引与切分

- tree-sitter 代码切分（Day 10 · ArkTS / TypeScript · `.ets/.ts/.tsx`）
- Markdown frontmatter + heading 切分
- Tantivy 真 BM25 倒排（Day 4）

## 已知 limitation

- LanceDB 向量库 task #81 阻塞（lance 0.17 async 类型递归超 rustc 默认深度 · 需升 lancedb 主版本）
- ArkTS `@Component struct` 方法提取需 custom tree-sitter-arkts grammar（当前用 fallback_full_file 兜底）
- ONNX 真语义 embedding 单独分发（需用户预装 ONNX Runtime · 1.0 不内置）

## 升级到 1.0

无升级路径 —— 1.0 是首个 release。新用户直接见 [Quickstart](quickstart.md)。

## 接下来

短期：
- Day 21b：`corpus model-pull` 真活（BGE-M3 ONNX 自动下载）
- Day 20c：onnx feature 端到端真活 + 单独 release

中期：
- DevEco Plugin MVP（方案 §4.3 主战场）
- task #81：升 lancedb 主版本

长期（阶段 3-4）：
- XDB 错误飞轮（方案 §1.2 护城河）
- Code GraphRAG（SCIP 代码图谱 · 跨文件多跳）
- Self-RAG / CRAG 反思机制

完整路线图：[ROADMAP](roadmap.md)。

## 致谢

- 完整技术方案：[`docs/RAG4ArkUI-完整技术方案.md`](reference/full-plan.md)（78 KB · 2258 行 · 一次性写就 · 22 commit 落地）
- 业界基线：参照 BAAI BGE-M3 / Tantivy / LanceDB / ort / tree-sitter / mdBook
- 协议基线：MCP 2024-11-05 · LSP 3.17 · JSON-RPC 2.0
