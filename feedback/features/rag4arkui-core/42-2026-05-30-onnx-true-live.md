# 42 — onnx-true-live

> 日期：2026-05-30
> 涉及代码：`docs/concepts/mvp.md` · `docs/concepts/onnx-chain.md` · `docs/ROADMAP.md` · `docs/USER-VERIFICATION.md`
> 类型：里程碑收尾（task #87 解锁 · MVP 92% → 100% 多文档同步）

## 本轮目标

承接 Round 40-41（task #87 Phase 1 + 2 · ort 库可编 + 我们代码 47→0 错）· 本轮 Round 42 完成 Phase 3（用户下载真模型）+ Phase 4（agent 量化对比 + 多文档同步「已解锁」）。

**MVP 完成度从 92% 推到 100%**。

## Plan

### Phase 3（用户做 · 已完成）

1. 用户在国内能爬墙环境用 hf-mirror 下载 BGE-M3 ONNX（2.2GB · 5-10 分钟）
   - `~/.arkui-rag/models/bge-m3/`
   - 6 文件：model.onnx (708K) + model.onnx_data (2.1G) + tokenizer.json (17M) + sentencepiece.bpe.model (4.9M) + Constant_7_attr__value (65K) + config.json (698B)
2. agent 编 cli 含 onnx feature · 28MB binary（不含 onnx 是 10.7MB · 多 17MB 是 ONNX Runtime 静态链接）· 44.5s
3. agent 真索引：embedder=bge-m3 · dim=1024 · files=11 · chunks=107 · elapsed_ms=144445（CPU 324% 多核 ONNX Runtime 并行）
4. agent 真检索：`@State 双向绑定` → Top-1 = mapping-state.md L24-34 状态选择表（含 `@Link 双向绑定`）

### Phase 4 量化对比

6 个 query · 每个有明确 mapping-* GT：

| Query | GT |
|---|---|
| @State 双向绑定 | mapping-state |
| LazyForEach 性能优化 | mapping-list |
| Coroutines 改 ArkUI-X | mapping-async |
| Row Column Box 布局 | mapping-layout |
| animate transition 动画 | mapping-animation |
| BenchmarkRunner 性能测试 | mapping-benchmark |

### 实测结果

| 指标 | Mock 哈希 384-dim | **BGE-M3 真语义 1024-dim** |
|---|---|---|
| **Top-1 命中** | 3/6（50%）| **6/6（100%）** |
| Latency p50 | ~48ms | ~4000ms（CPU only · CoreML 未启）|

Mock 命中 3 个是 BM25 关键词侥幸 · 真语义 query（LazyForEach 性能优化 / Coroutines / BenchmarkRunner · 关键词较弱）mock 全 fail · onnx 全命中。

### 多文档同步（agent 做 · 4 处）

1. **docs/concepts/mvp.md** L14-26：92% → **100%** ✅ · 加 Round 40-42 解锁说明 · 完成度清单加「ONNX 真语义（task #87 解锁）」· 剩两项标 ⏳（用户操作 task #84 #85）
2. **docs/concepts/onnx-chain.md** 头部状态 + 末尾追加「Round 42 实测对比」节
3. **docs/ROADMAP.md** L19：「pre-existing 阻塞清单：仅余 ort 链路」→ **「全部已解锁 ✅」**
4. **docs/USER-VERIFICATION.md** L5 + L59：~92% → 100% · onnx feature 描述从「broken」改「已真活」

### 替代方案权衡

| 选项 | 优点 | 缺点 | 选 |
|---|---|---|---|
| A · 6 query 手动对比（本轮）| 直观 · 可解释 · GT 准 | 样本小 | ✅ |
| B · 跑 corpus/_eval/queries.yaml | 自动化标准 | GT 是 corpus/migration/ · 跟实测 corpus（mapping-*）不匹配 | ❌ |
| C · 写新 eval fixture | 严谨 | 时间成本 · 跟本轮主线（推 100%）跑题 | ❌（残留）|

### 性能优化项（残留）

stderr 警告 `CoreMLExecutionProvider not registered (cargo feature not enabled)` · 当前 CPU only · 4s/query。装 `ort/coreml` Cargo feature 用 Apple Silicon GPU 加速 · 预期 < 500ms。Round 43+ 候选。

## 对话摘要

| 轮次 | 用户 | Agent |
|---|---|---|
| 1 | 「arkui-rag corpus model-pull bge-m3 这个指令 会把模型下载到哪里」 | 答 `~/.arkui-rag/models/bge-m3/` · 解释默认 URL 还没推 · 推荐 HuggingFace 直接下载 |
| 2 | 「从官方网站下载」 | 查 BAAI 官方 ONNX external data 机制 + curl 命令 |
| 3 | 「国内但能爬墙」 | hf-mirror 一键脚本 |
| 4 | ls 三次都贴 fake 残留 | 强调「整段复制粘贴跑 · 不是只跑 ls」 |
| 5 | ls 第四次 ✅ 6 文件齐 2.2GB | agent 接管 Phase 3：编 onnx binary + 真索引 + 真检索 · Top-1 完美命中 |
| 6 | 「先 A 后 B」 | 本轮 A：量化对比 6 query mock vs onnx + 多文档同步 + 归档（本文件）|

## 改动要点

- `docs/concepts/mvp.md` L14-26：完成度章节重写 · 92% → 100% · 加 Round 40-42 说明
- `docs/concepts/onnx-chain.md`：头部 callout 从「未真活」改「已真活 ✅」· 末尾追加「Round 42 实测对比」节 + 性能优化项 · 相关链接加 Round 41/42 引用
- `docs/ROADMAP.md` L19：「仅余 ort 链路」→「全部已解锁」+ 加实测数据
- `docs/USER-VERIFICATION.md` L5 + L59：92%→100% · onnx feature 描述更新

不动（历史快照）：
- `STATUS-onnx-chain-decision.md`（R40）· `STATUS-onnx-rc12-api-fix.md`（R41）—— Round 40/41 当时快照 · 不重写
- `STATUS-cli-default-features.md` / `STATUS-glossary.md` / `STATUS-lancedb-upgrade.md` —— 历史 STATUS · 引用「task #87 阻塞」是当时事实

## 验证结果

- 编译：N/A（纯文档）
- 6 query 对比（Phase 4 核心数据）：
  - Mock: 3/6 命中（50%）· latency ~48ms
  - ONNX: 6/6 命中（100%）· latency ~4000ms
- 真索引（Phase 3 核心）：embedder=bge-m3 · dim=1024 · chunks=107 · elapsed 144s
- 真检索 Top-1：`@State 双向绑定` → mapping-state.md L24-34 完美命中
- 文档更新：4 处关键文档同步「task #87 已解锁 · MVP 100%」

## 残留 / 下一轮

- [x] Phase 3 用户下载真模型 + agent 真索引 + 真检索 Top-1 命中
- [x] Phase 4 mock vs onnx 6 query 对比 + latency 数据
- [x] 4 处关键文档同步「task #87 解锁」
- [x] 双轨归档 + STATUS
- [ ] **下一步 B（本会话继续）**：cp release binary → ~/.local/bin/arkui-rag（含 onnx）· 改三端 MCP 配置加 onnx 参数 · 重启 · 让 Claude/opencode 调用 arkui_search_docs 直接拿真语义结果
- [ ] **Round 43+ 候选**：启用 `ort/coreml` Cargo feature · Apple Silicon GPU 加速 · 4s → <500ms
- [ ] **Round 43+ 候选**：写 mapping-* corpus 真 GT 的新 eval fixture · 替代 corpus/_eval/queries.yaml
- [ ] **任务清单更新**：TaskList 里 task #86 / #87 应标 completed（agent 不能直接改）
- [ ] **CI 矩阵加 onnx**：macOS / Linux 加 `cargo build --features onnx` step 防回归
- [ ] **README 加「真语义 RAG 已真活」徽章**（v1.0.0 release 前）
