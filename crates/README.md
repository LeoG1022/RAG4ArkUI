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
| [`arkui-rag-embedding`](arkui-rag-embedding/) | BGE-M3 ONNX 编码器 | ✅ Mock + §7.2 + **OnnxEmbedder async wrapper (Day 3)** | §6 / §7.2 |
| [`arkui-rag-storage`](arkui-rag-storage/) | 存储后端 + InMemory 实现 | ✅ InMemory + JSON 持久化 · TantivyBM25Index (Day 4) · **LanceVectorStore (Day 9 · feature `lancedb` · 解锁 >10k chunks)** | §4.2 决策 4 |
| [`arkui-rag-chunker`](arkui-rag-chunker/) | 切分（含 frontmatter）+ Dispatcher | ✅ MarkdownChunker · **TypeScriptChunker（ArkTS · Day 10 · feature `typescript`）** · Kotlin/Swift stub · ChunkerDispatcher 按扩展名路由 | §2.3 / §4.2 决策 6 |
| [`arkui-rag-retrieval`](arkui-rag-retrieval/) | HybridRetriever + RRF + Rerank | ✅ HybridRetriever 真活；**Day 4 起 RRF 真正双路融合**；**Day 5 起 Reranker 接入（embedding crate 的 OnnxReranker）** | §1.4 / §2.4 |
| [`arkui-rag-indexer`](arkui-rag-indexer/) | 索引流水线编排 | ✅ index_directory 真活 · **Day 10 起接 ChunkerDispatcher 支持多语言路由** · 端到端集成测试 | §9 图 5 / §9 图 2 |
| [`arkui-rag-eval`](arkui-rag-eval/) | **检索质量评估（Day 6 新增）** | ✅ recall@k / MRR / 延迟 + markdown 报告 + 端到端集成测试 | §1.5 / §2.7 / §8.5 共识 4 |
| [`arkui-rag-server`](arkui-rag-server/) | HTTP + MCP + LSP 协议 | ✅ **三协议全部真活**：HTTP（Day 14）· MCP stdio（Day 15 · 4 tools）· **LSP stdio（Day 16 · Content-Length framing · custom commands · hover stub）** ⭐ | §4.2 决策 2 / §9 图 8 |
| [`arkui-rag-cli`](arkui-rag-cli/) | `arkui-rag` 二进制入口 | ✅ index/query/eval/**serve --http（Day 14）** + `--embedder/--bm25/--vector/--rerank/--hyde/--expand-parent` 全套 | §5 / §9 图 8 |

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

# 4. 端到端 demo（Mock 模式，无依赖）
cd ..
cargo run --manifest-path crates/Cargo.toml -p arkui-rag-cli -- index --source corpus
cargo run --manifest-path crates/Cargo.toml -p arkui-rag-cli -- query --text "如何下拉刷新" --k 3

# 5. 真实 ONNX 模式（Day 3，需先获取模型 + onnx feature）
cargo run --manifest-path crates/Cargo.toml -p arkui-rag-cli --features onnx -- \
    index --source corpus --embedder onnx --model-path ~/.arkui-rag/models/bge-m3-onnx --model-id bge-m3
cargo run --manifest-path crates/Cargo.toml -p arkui-rag-cli --features onnx -- \
    query --text "下拉刷新" --k 3 --embedder onnx --model-path ~/.arkui-rag/models/bge-m3-onnx

# 6. 真实双路 hybrid（Day 4，需 tantivy feature；可叠加 onnx）
cargo run --manifest-path crates/Cargo.toml -p arkui-rag-cli --features tantivy -- \
    index --source corpus --bm25 tantivy
cargo run --manifest-path crates/Cargo.toml -p arkui-rag-cli --features tantivy -- \
    query --text "router pushUrl 怎么传参数" --k 3 --bm25 tantivy

# 7. 真实 Reranker（Day 5，需 onnx feature + reranker 模型）
cargo run --manifest-path crates/Cargo.toml -p arkui-rag-cli --features full -- \
    query --text "下拉刷新" --k 5 \
    --embedder onnx --model-path ~/.arkui-rag/models/bge-m3-onnx \
    --bm25 tantivy \
    --rerank onnx --reranker-model-path ~/.arkui-rag/models/bge-reranker-v2-m3-onnx --pre-rerank-k 50

# 8. 一键全启（Day 3 + Day 4 + Day 5）= 业界标配 Hybrid + Rerank
cargo run --manifest-path crates/Cargo.toml -p arkui-rag-cli --features full -- \
    index --source corpus --embedder onnx --model-path ~/.arkui-rag/models/bge-m3-onnx --bm25 tantivy

# 9. 跑检索质量评估（Day 6）
cargo run --manifest-path crates/Cargo.toml -p arkui-rag-cli -- \
    eval --queries corpus/_eval/queries.yaml --k 5
# 输出：reports/eval-<timestamp>-<config>.md，含 recall@k / MRR / latency
```

模型获取见 [`arkui-rag-embedding/README.md`](arkui-rag-embedding/README.md#模型获取day-3-阶段手动--cli-提示)
或跑 `arkui-rag corpus model-pull --name bge-m3` 看完整步骤。

`serve` 仍是 stub（Week 4 实装协议层）。`model-pull` 真实下载是 Week 2-3 backlog。
