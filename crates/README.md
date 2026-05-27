# crates/ — RAG4ArkUI Rust Workspace

本目录是 RAG4ArkUI 的 Rust 主代码（Cargo workspace）。**Day 1 骨架**：trait + 类型契约就位，绝大多数实现是 stub，唯一带真活代码的是 `arkui-rag-embedding` 的 ONNX 模块（feature-gated）。

完整设计依据：
- 目录边界 → [`docs/ADR-002-crate-structure.md`](../docs/ADR-002-crate-structure.md)
- 类图原型 → 完整方案图 3（`docs/RAG4ArkUI-完整技术方案.md` §9 图 3）
- 编译验证 → 见根 `Makefile`

## Crate 速查

| Crate | 定位 | Day 1 状态 | 对应方案章节 |
|---|---|---|---|
| [`arkui-rag-core`](arkui-rag-core/) | 公共 trait + 类型 + Error | ✅ trait/类型完成 | §4.1 引擎层 / §9 图 3 |
| [`arkui-rag-embedding`](arkui-rag-embedding/) | BGE-M3 ONNX 编码器 | ✅ §7.2 代码 + Mock | §6 / §7.2 |
| [`arkui-rag-storage`](arkui-rag-storage/) | LanceDB + Tantivy 后端 | ⏳ trait + stub | §4.2 决策 4 |
| [`arkui-rag-chunker`](arkui-rag-chunker/) | tree-sitter / markdown 切分 | ⏳ Markdown stub | §2.3 / §4.2 决策 6 |
| [`arkui-rag-retrieval`](arkui-rag-retrieval/) | HybridRetriever + RRF + Rerank | ⏳ 全 stub | §1.4 / §2.4 |
| [`arkui-rag-server`](arkui-rag-server/) | HTTP + MCP + LSP 协议 | ⏳ 路由 stub | §4.2 决策 2 / §9 图 8 |
| [`arkui-rag-cli`](arkui-rag-cli/) | `arkui-rag` 二进制入口 | ✅ clap subcommand stub | §5 / §9 图 8 |

## 构建

```bash
# 默认（不拉 ONNX 原生库，~3 分钟）：
make check
# 或：
cd crates && cargo check --workspace

# 启用 ONNX 完整编译（首次 5-10 分钟）：
make check-onnx
# 或：
cd crates && cargo check -p arkui-rag-embedding --features onnx
```

## Feature gate 策略

为了让 Day 1 验证不被 ONNX Runtime 编译卡死，`arkui-rag-embedding` 的 `ort` / `tokenizers` / `ndarray` 三个重依赖全部声明为 `optional = true`，通过 feature `onnx` 启用：

- **默认**：`cargo check --workspace` 只编译 stub 与类型签名，无原生库下载
- **`--features onnx`**：编译完整 BGE-M3 推理代码（§7.2 verbatim）

详细决策 → [`docs/ADR-002-crate-structure.md`](../docs/ADR-002-crate-structure.md#feature-gate-策略)。

## 当前可用的 CLI 命令

```bash
cd crates && cargo run -p arkui-rag-cli -- --help
cd crates && cargo run -p arkui-rag-cli -- --version
cd crates && cargo run -p arkui-rag-cli -- corpus list
```

绝大多数 subcommand 当前打印 `TODO: implement in Week X` 然后退出——这是有意为之，Day 1 只锁定 CLI 接口形状。
