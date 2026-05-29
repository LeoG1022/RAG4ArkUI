# 术语对照表

> 用途：澄清 RAG4ArkUI 文档里 agent 自造词 / 项目惯用词 / 业界标准术语的对应关系。
> 起因：commit 历史里有不少 agent 自造表达（最典型「真活」），新读者容易困惑。
>
> 自 commit `<本 commit>` 起，**新文档不再使用「真活」**，统一改成「实装 / 真实可用 / 端到端跑通」。
> 历史归档（22 commit · 19 STATUS · 12 meta）保持原样不重写——同义替换不创造新价值。

---

## Agent 自造词 → 标准中文

| 自造词 | 实际含义 | 标准表达 | 用例 |
|---|---|---|---|
| **真活** | 把占位实现替换为可工作的完整实现 | **实装 / 真实可用 / 端到端跑通 / 真启动** | "LSP 真活" → "LSP 实装" |
| **三协议互斥** | HTTP/MCP/LSP 同进程只能选一个跑 | （保留 · 已是描述性） | — |
| **真启动** | 不只是 CLI 参数解析 · 进程真起来 + 端口/stdio 真监听 | （保留 · 已较清楚） | — |
| **真上线** | 代码 + 文档 + release 包都对外可见 | **公开上线 / 发布上线** | "MVP 真上线" → "MVP 发布上线" |

> 深度展开见 [`docs/concepts/`](concepts/README.md)（每个概念一篇）：
> - [mdBook](concepts/mdbook.md) · [tree-sitter](concepts/tree-sitter.md) · [MVP](concepts/mvp.md)

## 项目惯用词

| 词 | 含义 | 与业界关系 |
|---|---|---|
| **meta 变更** | 改 `scripts/` / `.claude/skills/` / `AGENTS.md` / hooks / 规则 / Cargo manifest 等基础设施 | 项目自创二分（meta vs business） · 由 `scripts/classify-change.sh` 自动判定 |
| **business 变更** | 改 `crates/` / `corpus/` / `docs/` 等业务代码或文档 | 同上 |
| **双轨归档** | meta 改动写 `feedback/meta/<N>-*.md` · business 改动写 `feedback/features/<name>/<N>-*.md` · 一次 commit 含两者时**双轨各一份** | 项目自创 · 由 pre-commit hook 强制 |
| **STATUS-PER-ROUND** | Agent 每个 feature log 必须配套 `docs/STATUS-<slug>.md` 单轮快照 | 项目自创规则 #17 · AGENTS.md 定义 · FAIL 级 |
| **M-XXX** | check-consistency.sh 里的规则 ID 前缀（如 M-FB-01 / M-STATUS-PER-ROUND） | 项目自创命名 |
| **第 N 次实战** | ROADMAP「维护约定」每次执行的计数 · 提醒 agent 同步当前位置 + 已完成表 | 项目自创 |

## RAG/检索领域标准术语

| 词 | 含义 |
|---|---|
| **Hybrid retrieval** | 向量检索 + 关键词检索（BM25）+ 融合（RRF） |
| **RRF** | Reciprocal Rank Fusion · 把多路召回的排名倒数加权融合 |
| **Reranker** | Cross-encoder 二阶段重排 · 把 Top-K 召回交给更精的模型重排 |
| **HyDE** | Hypothetical Document Embeddings · 让 LLM 先生成一段假设文档再检索 |
| **Parent-Child** | 子粒度切分召回 + 父粒度扩展返回 · 兼顾召回与上下文 |
| **Citation / 引用溯源** | 每条检索结果带 source / heading_path / line_range · 让用户可校验 |

## 协议层标准术语

| 词 | 含义 |
|---|---|
| **MCP** | Model Context Protocol · Anthropic 2024 推 · Claude Code / Cursor 等 agent 用 |
| **LSP** | Language Server Protocol · Microsoft 2016 推 · IDE 内联用 |
| **JSON-RPC 2.0** | MCP 和 LSP 都基于的 RPC 协议 |
| **Content-Length framing** | LSP 的消息分隔约定（区别于 MCP 的行分隔） |
| **stdio server** | 用 stdin/stdout 做 transport 的 server（与 HTTP server 对比） |

## Pre-existing / 状态形容词

| 词 | 含义 |
|---|---|
| **pre-existing 阻塞 / 缺陷** | 不是本轮引入的 · 历史代码里就有 · 浮出后单独修 |
| **stub** | 占位实现 · 函数签名就位但内部 `todo!()` 或打印 TODO |
| **feature gated** | 用 Cargo `[features]` 包起来 · 默认不启用 · `--features X` 才编进去 |
| **shadow / 占位** | 临时返回值（如 `dim=0` 占位让 CLI 编过）· 后续被真值替换 |

---

## 未来防漂移

新归档（feature log / STATUS / commit message）写作约定：

- ❌ 不用：「真活」（除引用历史 commit）
- ✅ 用：「实装」「真实可用」「端到端跑通」「真启动」「实测通过」「业务闭环打通」

如果新增 agent 自造词，先在本文件加一行对照，再用。
