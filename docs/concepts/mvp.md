# MVP（Minimum Viable Product）

> 一句话：业界术语 · **用最少功能验证核心价值**的产品版本 · 在本项目特指完整技术方案规划的 6 周交付物。

## 在本项目的具体含义

| 维度 | 含义 |
|---|---|
| 时间盒 | 完整技术方案 `docs/RAG4ArkUI-完整技术方案.md` 规划的 **6 周** |
| 范围 | Week 1-6 共 22 个 Day 切片（详见 `docs/ROADMAP.md`） |
| 核心价值闭环 | 拿一个本地 corpus + 自然语言 query → 返回带引用溯源的 Top-K 命中 → 三协议（HTTP / MCP / LSP）给消费方调 |
| 排除 | DevEco Plugin / VSCode Extension（IDE 集成层）· XDB 错误飞轮 · Code GraphRAG 等阶段 3-4 长期能力 |

## 当前 MVP 完成度（task #81 + Phase A/B/C 后 ~92%）

- ✅ 引擎：Hybrid + Reranker + HyDE + Parent-Child + 评估闭环
- ✅ 三协议：HTTP + MCP + LSP 全实装
- ✅ 后端：Tantivy BM25 + LanceDB（task #81 解锁）+ in-memory
- ✅ 分发：本地 + 4 平台 CI matrix
- ✅ corpus + model 一键 pull
- ✅ mdBook 文档站 + 用户验证清单
- ❌ 用户首推 master + Settings→Pages（让站真上线）
- ❌ 用户 push tag v1.0.0（出 1.0 release page）
- ❌ ONNX 真语义（task #87 blocker · 仍 mock-384）

「MVP 上线」= 让代码 + 文档 + release 包都对外可见 · 不只在本地 repo 里待着。

## MVP 与正式 release 的边界

| 类别 | MVP 范围内 | MVP 范围外（阶段 2+） |
|---|---|---|
| 检索能力 | Hybrid + Reranker + HyDE + Parent-Child | XDB 错误飞轮 · Code GraphRAG · Self-RAG / CRAG |
| 协议 | HTTP / MCP / LSP | gRPC（如有需要）|
| 后端 | Tantivy + LanceDB + in-memory | Qdrant · Chromadb（如有需要）|
| 切分 | tree-sitter typescript + markdown | Kotlin / Swift（feature 占位 · 实装待） |
| 集成 | CLI + 三协议 server | DevEco Plugin · VSCode Extension · IntelliJ Plugin |
| 评估 | recall@k + MRR + 延迟分位 | RAGAS · LangSmith 等三方平台接入 |

## 类比

| 已知 | MVP 在本项目中相当于 |
|---|---|
| Docker 1.0 | 容器能跑 · 不含 Swarm / Compose |
| Kubernetes 1.0 | Pod 能调度 · 不含 Operator / Service Mesh |
| Rust 1.0 | 借用检查器 + 包管理器到位 · async/await 等之后才加 |
| React 16 | 函数组件 + Hooks 可用 · Concurrent Mode 之后才加 |

业界共识：MVP 不是「丑陋的半成品」· 而是「砍掉无关支线 · 留核心闭环 · 真能交付的最小版本」。
