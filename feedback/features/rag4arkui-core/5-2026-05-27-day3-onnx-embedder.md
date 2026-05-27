# 5 — day3-onnx-embedder

> 日期：2026-05-27
> 涉及代码：
> - `crates/arkui-rag-embedding/src/lib.rs`（导出 OnnxEmbedder）
> - `crates/arkui-rag-embedding/src/onnx_embedder.rs`（**新增** async wrapper + spawn_blocking 桥接）
> - `crates/arkui-rag-cli/Cargo.toml`（新增 onnx feature 转发）
> - `crates/arkui-rag-cli/src/main.rs`（重写：加 EmbedderKind enum + --embedder/--model-path/--model-id + cfg 双路径 build_onnx + model_id 防错配校验）
> - `crates/arkui-rag-embedding/README.md`、`crates/README.md`、`Makefile`（同步说明 + build-onnx target）
> 类型：新建（Day 3 主线）

## 本轮目标

把 §7.2 verbatim 的 `EmbeddingModel`（同步）包装成 `Embedder` trait 的 async 实现 `OnnxEmbedder`。
让 CLI 通过 `--embedder onnx --model-path <dir>` 跑真实 BGE-M3 推理，**MockEmbedder 退场，真实语义检索上线**。

非目标：
- 真实 HuggingFace / ModelScope 下载（model-pull 仍 stub，但提示完整手动步骤）
- 检索质量评估（recall@k 等，留 Week 3 + RAGAS）
- LanceDB / Tantivy 替换（Day 3+ 备选）

## Plan

> 必备节（FAIL 级 M-FEATURE-PLAN）

### 关键设计

**1. async wrapper 桥接同步 API**

底层 `EmbeddingModel::encode()` 是同步阻塞调用（ort 推理 ~20-50ms）。直接在 `async fn encode()` 里调会阻塞 tokio worker 线程。解法：

```rust
let arr = tokio::task::spawn_blocking(move || {
    let refs: Vec<&str> = owned.iter().map(|s| s.as_str()).collect();
    inner.encode(&refs)
}).await??;
```

注意：
- `&str` 借用不能跨线程移动 → 先 owned 拷贝
- `spawn_blocking` 返回 `JoinHandle`，`.await` 拿到 `JoinError`，再 `?` 拿真实 Result
- ndarray 的 `Array2<f32>` 在线程间安全（仅 owned 数据），可作返回值

**2. CLI feature 转发策略**

```toml
[features]
default = []
onnx = ["arkui-rag-embedding/onnx"]
```

CLI 代码用 `#[cfg(feature = "onnx")] fn build_onnx(...)` + `#[cfg(not(feature = "onnx"))] fn build_onnx(...) { bail!(提示重新构建) }`，避免每个调用点散落 `#[cfg]`。

**3. model_id 防错配（key 设计）**

索引时 `InMemoryVectorStore::new(model_id, dim)` 写入 `model_id`；查询时加载后比对：
```
索引 model_id="bge-m3" vs 查询 model_id="mock-384" → bail!
```

这避免最容易踩的坑：用 Mock 建的索引被 ONNX 查询（或反之），向量空间不通，结果是垃圾。

**4. CLI 参数分层**

| 参数 | mock 模式 | onnx 模式 |
|---|---|---|
| `--embedder` | `mock`（默认） | `onnx` |
| `--dim` | ✅ 配置 mock 维度 | ❌ 忽略（从模型读） |
| `--model-path` | ❌ 不需要 | ✅ 必填 |
| `--model-id` | ❌（mock-{dim} 自动） | ✅ 默认 "bge-m3" |

### 行为对照

| 命令 | 行为 |
|---|---|
| `cargo build -p arkui-rag-cli`（默认） | 不拉 ort，只能用 mock；`--embedder onnx` 报"未启用 onnx feature" |
| `cargo build -p arkui-rag-cli --features onnx` | 拉完整 ort 工具链（~300MB 原生库 + 5-10 分钟首编），两种 embedder 都可用 |
| `arkui-rag index --embedder onnx --model-path ...` | 加载模型 → 索引 → 写 model_id=bge-m3 入 index.json |
| `arkui-rag query --embedder onnx --model-path ...` | 加载 model_id 校验 → encode query → cosine 检索 |

### 替代方案权衡

- 备选：让 OnnxEmbedder 持 `tokio::sync::Mutex<EmbeddingModel>` 而非 `Arc<EmbeddingModel>`
  - 否决：EmbeddingModel 的 encode 是 `&self`（不需要 mut），ONNX session 内部线程安全
- 备选：默认启用 onnx feature
  - 否决：会强制所有用户编译 ort，违反 Day 1 feature gate 策略（详 ADR-002）
- 备选：把 model-pull 真实做了
  - 否决：用户没指定要做；脚本提示完整手动步骤就够了；真实下载（reqwest + indicatif）是 Week 2-3 任务

## 改动要点

> API 选型 / 算法 / 关键决策 / 与上轮的差异

**与 Day 2 的差异**：
- 新增 `OnnxEmbedder` 类型（feature gated），CLI 可选 mock / onnx
- CLI 参数：新增 `--embedder` `--model-path` `--model-id`
- CLI 错误处理：未启用 onnx feature 时用 onnx → 友好报错带重建命令
- index.json `embedder_model_id` 字段从 query 时被严格校验

**API 选型**：
- `OnnxEmbedder::load(model_dir, model_id)` 而非 `from_file(path)` —— 与 §7.2 一致 + 显式 model_id
- 加载失败转 `RagError::Embedding`（而非 `anyhow::Error` 透出）→ 错误类型统一

**算法**：
- spawn_blocking 后 `.await??` 双层错误展开（JoinError + 内部 anyhow Error）
- Array2 → Vec<Vec<f32>> 按行 to_vec 转换；ndarray 在 tokio 任务间用 owned 即可

## 对话摘要

> 必备节（FAIL 级 M-FEATURE-PLAN）

按时序：

1. **Day 2.5 smoke commit 后**（`1b0e04f`），用户授权"做完自动化后继续 Day3"
2. **Agent 拆分** Day 3 为 3 个 phase：A (OnnxEmbedder wrapper) → B (CLI 参数 + feature) → C (文档 + 归档)
3. **Agent 自主决策**（已记入本 plan 节 + Agent 决策分析风格内嵌）：
   - spawn_blocking 桥接（不引 Mutex）
   - CLI feature 转发而非默认启用
   - model_id 严格校验
   - model-pull 仅打印手动步骤（不做真实下载）
4. **Agent 不再回问用户**，直接执行 3 phase 至本 commit

## 验证结果

- **默认编译**：⏳ 用户跑 `make check` —— 期望 8 个 crate 全过，OnnxEmbedder 因 feature gate 不参编
- **onnx 编译**：⏳ 用户跑 `make build-onnx`（首次 5-10 分钟拉 ort + 编译）
- **测试**：
  - `cargo test --workspace`：23 个原测试 + `OnnxEmbedder::load missing path returns err` 1 个新单测 = **24 个** (其中 1 个 ignored 等真模型)
  - `cargo test --workspace --features arkui-rag-embedding/onnx`：加 OnnxEmbedder 的 load_missing_model_returns_err
- **手动端到端**（需用户先获取 BGE-M3 ONNX 模型）：
  ```
  cargo run --features onnx -p arkui-rag-cli -- \
      index --source corpus --embedder onnx --model-path ~/.arkui-rag/models/bge-m3-onnx
  cargo run --features onnx -p arkui-rag-cli -- \
      query --text "如何下拉刷新" --k 3 --embedder onnx --model-path ~/.arkui-rag/models/bge-m3-onnx
  ```
  期望：真实语义检索 —— "如何下拉刷新"（query）应能命中 "ArkUI-X 用 Refresh 组件实现下拉刷新"（doc），即使不是逐字匹配
- check-api-parity：N/A

## 残留 / 下一轮

- [ ] **关键**：用户获取 BGE-M3 ONNX 模型 + 跑真实端到端验证（参考 `arkui-rag corpus model-pull` 输出的步骤）
- [ ] **关键**：用户跑 `make build-onnx` 验证 ort 2.0.0-rc.4 API 与 §7.2 代码 100% 匹配（**首次可能因 API 漂移有小修**）
- [ ] **Day 3 续**：写 `scripts/demo-smoke-onnx.sh`（依赖真实模型，需用户授权）
- [ ] **Day 3 续**：`arkui-rag corpus model-pull` 真实下载（reqwest + indicatif 进度条）
- [ ] Week 2 续：接 LanceDB（feature `lancedb`）让 chunks > 10k 也可用
- [ ] Week 3：接 Tantivy（feature `tantivy`）让 BM25 路径真活；CrossEncoderReranker 真实化
- [ ] Week 3：HyDE 改写器（小 LLM）+ 检索质量评估（recall@k + RAGAS）
- [ ] Week 4：HTTP/MCP/LSP 三协议实装
- [x] Day 3：OnnxEmbedder async Embedder trait 实装
- [x] Day 3：CLI 加 --embedder/--model-path 选项
- [x] Day 3：CLI onnx feature 转发 + 友好报错
- [x] Day 3：index/query model_id 防错配校验
