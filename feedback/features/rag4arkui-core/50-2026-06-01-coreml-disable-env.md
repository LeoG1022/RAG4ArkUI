# 50 — coreml-disable-env

> 日期：2026-06-01
> 涉及代码：`crates/arkui-rag-embedding/src/onnx.rs` · `crates/arkui-rag-embedding/src/reranker_onnx.rs` · `crates/arkui-rag-cli/src/main.rs`
> 类型：bug 修复（Round 49 PoC 暴露 CoreML + BGE-M3 external data 加载 bug · 用 env 绕过）

## 本轮目标

Round 49 PoC build index 时触发 ort rc.12 + CoreML EP + BGE-M3 external data 加载 bug · 无法重 build index。本轮用最便携方案修：env 检测 · `cmd_index` 进程禁用 CoreML EP · `cmd_query` 保留（CoreML 21× 加速不变）· 同 binary 两端共用。

## Plan

### 选 A' (env 绕过) 而非 B (single-file)

| 方案 | 工作量 | 选 |
|---|---|---|
| A · CPU-only binary 单独 build（双 binary）| 30 分钟 + 部署复杂 | ❌ |
| **A' env 检测 · index 时跳 CoreML（本轮）** | 15 分钟 · 1 binary 两路共用 | ✅ |
| B · BGE-M3 single-file 化（Python onnx merge）| 1 round + Python 装 onnx 包 100MB+ | ❌（Python 3.14 没装 onnx · 装慢）|
| C · 等 ort 上游修 | 不可控 | ❌ |

选 A' 因为：
- 不依赖 Python 环境（Python 3.14 没 onnx 包 · 装要几分钟）
- 不增加 binary 数量
- 不动模型文件（保留 BGE-M3 原 external data 结构 · 跟 model-pull 一致）
- 实测可行（15 分钟改完 + 验证 OK）

### 实施

**onnx.rs / reranker_onnx.rs**：

```rust
let disable_coreml = std::env::var("ARKUI_RAG_DISABLE_COREML").is_ok();
let mut providers = Vec::new();
if !disable_coreml {
    providers.push(CoreMLExecutionProvider::default().build());
}
providers.push(CUDAExecutionProvider::default().build());
providers.push(CPUExecutionProvider::default().with_arena_allocator(true).build());

ort::init()
    .with_name("arkui-rag")
    .with_execution_providers(providers)
    .commit();
```

**cli main.rs cmd_index 入口**：

```rust
#[allow(unsafe_code)]
unsafe {
    std::env::set_var("ARKUI_RAG_DISABLE_COREML", "1");
}
```

Rust 2024 起 `std::env::set_var` 标 unsafe（multi-thread 不安全）· 这里在 tokio runtime 启动后单线程阶段 set · OK。`ort::init()` 是 lazy + 全局 singleton · 第一次 OnnxEmbedder::load 时读 env · 决定 EP 列表。

### 端到端验证

```bash
cargo build --release --features ...,onnx   # 26.46s ✓
cp target/release/arkui-rag ~/.local/bin/
codesign --force --sign - ~/.local/bin/arkui-rag

# index 跑（CoreML 禁用 · CPU 兜底）
arkui-rag index --source corpus/official --embedder onnx ...
# ✅ 索引完成 · files=8 · chunks=68 · elapsed=88.6s
# 比 Round 42 .claude/references/ 11 文件 107 chunks 144s 慢 · 因为 CoreML 跳过 · CPU only

# query 跑（CoreML 启用 · 保持 21× 加速）
arkui-rag query --text "@State 双向绑定" --embedder onnx ...
# ~200ms · 跟 Round 47 实测一致
```

### 不动

- BGE-M3 模型文件不动（保留 external data 结构 · 跟 model-pull 一致）
- CLI 接口不动（用户不感知 env · cmd_index 内部自动 set）
- query / serve 路径不动（仍 CoreML）

## 对话摘要

| 轮次 | 用户 | Agent |
|---|---|---|
| 1 | 「按你推荐的做 · 然后挑合适的时机压缩 context」 | 评估方案 B（Python onnx）· Python 3.14 没装 onnx · 装慢 100MB · 改 A' env 绕过 · 实测 build index 真活 |

无方向调整 · agent 根据环境制约自主切换方案 B → A'（meta/8 在 task #87 历史也用过 env trick · 同款思路）。

## 改动要点

- `crates/arkui-rag-embedding/src/onnx.rs` `EmbeddingModel::load()` 加 env 检测 + 条件 EP 列表（+13 行 · 替换原 4 行）
- `crates/arkui-rag-embedding/src/reranker_onnx.rs` `RerankerModel::load()` 同款
- `crates/arkui-rag-cli/src/main.rs` `cmd_index` 入口 set env（+5 行）
- 与 Round 47 关系：47 加 ort/coreml feature 实现 21× 加速 · 50 修 index 阶段兼容 bug · 配套 · 同 binary 两端共用

## 验证结果

- 编译：`cargo build --release ...,onnx` ✓ Finished 26.46s
- index 重 build：✓ 8 文件 / 68 chunks / 89s（CPU only · 不再 fail）
- 打包重做：
  - corpus tarball 14KB（同前）
  - index tarball **775KB**（用新 build · 比 Round 49 PoC 988KB 更精简）
- query 路径不受影响（不 set env · 仍 CoreML 加速 · Round 47 测的 ~200ms）

## 残留 / 下一轮

- [x] CoreML bug 修：env 绕过 · 不需要 Python / single-file 模型
- [x] index 真重 build 验证 ✓
- [x] tarball 重打包用新 index
- [x] 双轨归档（仅 feature log · 业务变更）
- [ ] **Round 49.5 第 2 半**：等用户给 ArkUI-X / OpenHarmony 真仓库 URL · 收集 + build 真 corpus + index
- [ ] **Round 49.6**：推 GitHub Release `corpus-v1.0.0` · 让默认 corpus pull URL 真活
- [ ] **Round 50**（编号占位 · 本轮实际是 Round 49.5）：加 `arkui-rag index-pull` 命令
- [ ] **Round 51**：maintainer CI 自动 re-build + 推 release
- [ ] **Round 52**：加 `arkui-rag init` wizard
- [ ] **Round 53**：终端用户视角文档
- [ ] **长期**：跟踪 ort 上游修 CoreML + external data bug · 修了之后可去掉 ARKUI_RAG_DISABLE_COREML env 检测
