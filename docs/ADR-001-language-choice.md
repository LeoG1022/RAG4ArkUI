# ADR-001 · 主语言选型：Rust（终态）

- **Status**：Accepted
- **Date**：2026-05-26
- **Deciders**：项目发起人；通过 AskUserQuestion 在 Day 1 切片选项中明确选择 "纯 Rust 骨架先行"
- **Context Doc**：[`RAG4ArkUI-完整技术方案.md`](RAG4ArkUI-完整技术方案.md) §4.2 决策 1 / §5.3

## Context

RAG4ArkUI 需要作为本地化二进制分发给 IDE 插件、MCP agent、CLI 用户。语言选择决定后续 5 周的所有工程实施路径，影响：性能、二进制体积、依赖管理、跨平台分发、生态可用 crate。

候选语言：

| 选项 | 优势 | 劣势 |
|---|---|---|
| **Rust** | 单二进制、性能极佳、ort/tantivy/lancedb 全是 Rust 原生 | 开发周期长、ML 生态弱 |
| Python | ML 生态最全、原型快 | 分发地狱（依赖、CUDA、虚拟环境）、性能弱 |
| Go | 分发简单 | ML 库少、CGO 调 ONNX 麻烦 |
| Node/TS | VSCode 契合 | 内存占用高、性能差 |

## Decision

**主语言选 Rust，作为终态决策**。Day 1 不做 Python 原型，原因：

1. **代码复用为零**：Python 原型与 Rust 终态零代码复用，纯重复劳动
2. **风险最高的依赖在 Python**：`torch` / `sentence-transformers` 在 Python 3.14 上轮子还不稳定，调通成本高
3. **业界先例一致**：Qdrant / Meilisearch / Tantivy / LanceDB / ripgrep / ollama 全是单 Rust（或 Go）二进制，无成功的"Python 服务端 + IDE 直连"范式
4. **完整方案 §7.2 已经给了可直接迁移的 Rust 代码**（BGE-M3 ONNX 推理），起步成本远低于 Python 重写

## Consequences

**正向**：
- 单二进制分发，IDE 插件 / MCP agent 都能直接 fork 进程
- 启动 < 100ms、检索 P99 < 200ms 的性能目标可达
- 编译期类型检查捕获绝大部分 RAG 流水线类型错误（hit / chunk / embedding 维度）

**负向**：
- 开发节奏比 Python 慢 ~2x（特别是字符串 / 集合操作）
- 部分 RAG 实验性论文（CRAG / Self-RAG）的实现还以 Python 为主，需自行移植
- 团队/贡献者需要 Rust 经验门槛

## Compliance

- 工具链版本由 `rust-toolchain.toml` 锁定（`channel = "stable"`，components: `rustfmt`, `clippy`）
- 所有重 ML 依赖（ort / tokenizers / ndarray）通过 Cargo feature 隔离，让默认 `cargo check --workspace` 不被原生库编译卡死
- 改语言需新增 ADR-00X 推翻本决策；改 Cargo.toml `rust-version` 需同步更新本文件

## Review Triggers

如果出现以下任一情况，应重新评估：
- Rust crate 生态出现关键缺口（如 BGE-M3 必须用 Python-only 的预处理）
- 团队 Rust 经验断档
- ArkUI-X 官方提供 Rust 不友好的 SDK 强依赖
