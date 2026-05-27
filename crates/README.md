# crates/ — RAG4ArkUI Rust Workspace

本目录是 RAG4ArkUI 的 Rust 主代码（Cargo workspace）。**Day 1 骨架**：trait + 类型契约就位，绝大多数实现是 stub，唯一带真活代码的是 `arkui-rag-embedding` 的 ONNX 模块（feature-gated）。

完整设计依据：
- 目录边界 → [`docs/ADR-002-crate-structure.md`](../docs/ADR-002-crate-structure.md)
- 类图原型 → 完整方案图 3（`docs/RAG4ArkUI-完整技术方案.md` §9 图 3）
- 编译验证 → 见根 `Makefile`

## Crate 速查（Day 2 更新）

| Crate | 定位 | 当前状态 | 对应方案章节 |
|---|---|---|---|
| [`arkui-rag-core`](arkui-rag-core/) | 公共 trait + 类型 + Error | ✅ trait/类型完成 | §4.1 引擎层 / §9 图 3 |
| [`arkui-rag-embedding`](arkui-rag-embedding/) | BGE-M3 ONNX 编码器 | ✅ §7.2 代码 + Mock | §6 / §7.2 |
| [`arkui-rag-storage`](arkui-rag-storage/) | 存储后端 + InMemory 实现 | ✅ trait + InMemoryVectorStore/BM25 + JSON 持久化 | §4.2 决策 4 |
| [`arkui-rag-chunker`](arkui-rag-chunker/) | 切分（含 frontmatter） | ✅ MarkdownChunker（含 YAML frontmatter） | §2.3 / §4.2 决策 6 |
| [`arkui-rag-retrieval`](arkui-rag-retrieval/) | HybridRetriever + RRF + Rerank | ✅ HybridRetriever 真活；Reranker 仍 stub | §1.4 / §2.4 |
| [`arkui-rag-indexer`](arkui-rag-indexer/) | 索引流水线编排（Day 2 新增） | ✅ index_directory 真活 + 单测 + 端到端集成测试 | §9 图 5 / §9 图 2 |
| [`arkui-rag-server`](arkui-rag-server/) | HTTP + MCP + LSP 协议 | ⏳ 路由 stub（Week 4） | §4.2 决策 2 / §9 图 8 |
| [`arkui-rag-cli`](arkui-rag-cli/) | `arkui-rag` 二进制入口 | ✅ index/query/corpus 真活；serve stub | §5 / §9 图 8 |

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

## 当前可用的 CLI 命令（Day 2 端到端）

```bash
cd crates
# 1. 看帮助
cargo run -p arkui-rag-cli -- --help

# 2. 看版本
cargo run -p arkui-rag-cli -- --version

# 3. 看 corpus 子目录状态
cargo run -p arkui-rag-cli -- corpus list

# 4. 端到端 demo（需要 corpus/ 下先放 markdown，参考 corpus/README.md）
cd ..
cargo run --manifest-path crates/Cargo.toml -p arkui-rag-cli -- index --source corpus
cargo run --manifest-path crates/Cargo.toml -p arkui-rag-cli -- query --text "如何下拉刷新" --k 3
```

`serve` 仍是 stub（Week 4 实装协议层）。`model-pull` 仍是 stub（Week 2 接 HuggingFace 下载）。
