# STATUS — onnx-chain-decision

> 配套 feature log：`feedback/features/rag4arkui-core/40-2026-05-30-onnx-chain-decision.md`
> 日期：2026-05-30

---

## 当前状态

**Phase 1 完成**：破解 task #87 「ort 2.0 RC 整链路 broken」根因 · ort 库自身编译过了。Phase 2-4（修我们代码适配 rc.12 API + 真模型端到端 + 质量评估）挂下轮。

本阶段交付：
- `docs/concepts/onnx-chain.md` 130 行 · ONNX 链路决策梳理（4 选项 / 业界用法 / 项目现状 / 类比）
- `crates/Cargo.toml` ort `2.0.0-rc.4` + `load-dynamic` → `2.0.0-rc.12` + 默认 features（静态链接）
- ort 自身编过 ✓ · 我们代码 47 错（Phase 2 修）
- 默认 features `cargo check --workspace` ✓ Finished 6.07s · **master 仍 buildable**
- 概念归档 5 步流程完整跑通 · 这是 Round 33 AGENTS.md #18「概念问答必询问归档」规则的**首次实战**

意义：把 ROADMAP 上挂了多轮的 task #87 推进了关键一步——从「ort 整链路 broken · 不知道怎么走」到「root cause 找到 + ort 编过 + Phase 2 工作量明确（修 47 个 API 错）」。

## 输入契约

### 决策契约

| 选项 | 工作量 | 风险 | 选 |
|---|---|---|---|
| **A** 升 ort 最新 RC | 1 天（实际仅改 1 行 Cargo.toml 解决 ort 自身 bug + 修 47 个 API 错）| ⭐⭐⭐ | ✅（本轮）|
| B 退 ort 1.16 | 2-3 天 | ⭐⭐ | ❌ |
| C 换 candle | 4-5 天 | ⭐⭐⭐⭐ | ❌（A 失败再考虑）|
| D 接 OpenAI API | 0.5 天 | ⭐ | ❌（偏离本地优先）|

### Cargo features 契约变化

| 维度 | Before | After |
|---|---|---|
| ort 版本 | `2.0.0-rc.4`（pinpoint · 实际 lock rc.12）| `2.0.0-rc.12`（明确）|
| ort default-features | `false` | `true`（含 download-binaries · 静态链接 ONNX Runtime）|
| ort features | `["load-dynamic", "ndarray"]` | `["ndarray"]`（去掉 load-dynamic）|

**关键差异**：`load-dynamic` 让 ort 假设运行时 dlopen libonnxruntime + 触发 vitis.rs 编译（ortsys 没暴露 VitisAI 字段 → 编 fail）。**默认走静态链接**（download-binaries 下载 ONNX Runtime binary 编进去）· binary 大几 MB 但稳定。

### 不变项

- onnx feature 仍 opt-in（`cargo build --features onnx` 才拉 ort）
- 默认 features 不含 onnx · 用户 `make release-local` / `cargo build --release` 无影响
- CLI `--embedder onnx --model-path ...` 接口不变
- BGE-M3 模型路径约定 `~/.arkui-rag/models/bge-m3/{model.onnx, tokenizer.json}` 不变
- 索引格式 / MCP 工具签名全部不变

## 输出契约

### 编译验证（Phase 1）

```bash
# 默认 features（master 必须 buildable）
$ cargo check --manifest-path crates/Cargo.toml --workspace
   Finished `dev` profile [unoptimized + debuginfo] target(s) in 6.07s   ✓

# onnx feature（Phase 1 后 · ort 自身过 · 我们代码 47 错待 Phase 2）
$ cargo check -p arkui-rag-embedding --features onnx
   Checking ort v2.0.0-rc.12  ✓
   ...
   error: could not compile `arkui-rag-embedding` due to 47 errors
```

### Phase 2 后预期输出

```
$ cargo check -p arkui-rag-embedding --features onnx
   Checking arkui-rag-embedding v0.0.1
    Finished `dev` profile  ✓
```

### Phase 3 后预期输出（真模型端到端）

```bash
$ arkui-rag corpus model-pull bge-m3   # 真下载 BGE-M3 ONNX
$ arkui-rag index --source corpus --embedder onnx --model-path ~/.arkui-rag/models/bge-m3 --bm25 tantivy
✅ 索引完成
   embedder    : bge-m3 (1024-dim)        ← 不再是 mock-384
   dim         : 1024
   ...
```

### Phase 4 后预期（质量评估）

- mock embedder 下 `@State 双向绑定` Top-1 ≠ mapping-state（score 0.0164 平均分布）
- onnx embedder 下 Top-1 = mapping-state L24-34（score ≥ 0.7 · 真语义命中）

## 验证手段

### 用户手动

Phase 1 验证（本轮已做）：

```bash
# 1. 默认 features 仍 build
cd crates && cargo check --workspace
# 期望：Finished

# 2. onnx feature ort 自身编过
cargo check -p arkui-rag-embedding --features onnx 2>&1 | grep -E '^error: could not compile'
# 期望：只剩 `could not compile 'arkui-rag-embedding'` · 不再有 `could not compile 'ort'`
```

Phase 2 之后用户验证：

```bash
make build-onnx   # 等 Phase 2 修完
arkui-rag --version
```

Phase 3 用户必做：

```bash
arkui-rag corpus model-pull bge-m3
arkui-rag index --source <你的 corpus> --embedder onnx \
    --model-path ~/.arkui-rag/models/bge-m3 --bm25 tantivy
arkui-rag query --text "@State 双向绑定" --bm25 tantivy --embedder onnx \
    --model-path ~/.arkui-rag/models/bge-m3 -k 3
```

### 自动化

CI 残留：`cargo check --features onnx` step（task #87 完全解锁后）· 防 ort 升级回归。

## 与上一阶段的关联性

| Round | 解决 |
|---|---|
| 33 | AGENTS.md #18「概念问答必询问归档」规则 + concepts/ 基础设施 |
| 34 | cli default features = http,mcp,lsp,tantivy,typescript,corpus-pull（**不含 onnx**）|
| ... |
| 39 | uninstall 反向操作 |
| **40（本轮）**| **task #87 Phase 1**：ort 库 broken 根因 + Cargo.toml fix + 概念归档 |

关键里程碑：
- Round 33 立的「概念归档规则」**首次实战触发** · 用户问「这是什么环节」 → agent 询问归档 → 用户「先归档」 → 5 步走通
- ROADMAP 上挂了 ~15 轮的 task #87 阻塞终于推进 Phase 1（之前一直「ort 2.0 RC 整链路 broken · 不知道怎么修」）

兼容性：
- 默认 features 完全不受影响 · 已实测
- onnx feature 用户：rc.4 → rc.12 升级（Phase 2 修 API 漂移后才可用）

破坏性变更：
- onnx feature 旧调用代码 47 处 API 不兼容 · Phase 2 修

## 完成度 / 下一阶段

### 达成度

| 项 | 状态 |
|---|---|
| ort broken 根因诊断 | ✅ |
| Cargo.toml ort 升级 + 去 load-dynamic | ✅ |
| ort 库自身编过 | ✅ |
| 默认 features `cargo check --workspace` ✓ | ✅ |
| `docs/concepts/onnx-chain.md` 写完 130 行 | ✅ |
| 概念归档 5 步流程跑通 | ✅ |
| 修 47 个我们代码 API 错 | ⏳ Phase 2 |
| BGE-M3 真模型端到端 | ⏳ Phase 3 |
| mock vs onnx 质量评估 | ⏳ Phase 4 |

### 下一阶段建议

立即（agent · Phase 2）：
- 系统性修 47 个 ort API 漂移错
  - 主要：所有 ort 调用的 `?` 加 `.map_err(|e| anyhow::anyhow!("..."))` 转 anyhow
  - `Tensor::from_array` 返回值适配
  - `logits.shape()` 元组解构
- `cargo check -p arkui-rag-embedding --features onnx` → 0 error 为终止条件
- 单元测试：mock embedder 不动 · onnx embedder 跑 build 不跑 test（需真模型）

短期（用户 · Phase 3）：
- 跑 `arkui-rag corpus model-pull bge-m3` 真下载（首次几百 MB）
- 跑 `arkui-rag index --embedder onnx ...` 真索引
- 跑 `arkui-rag query --text "@State 双向绑定" --embedder onnx ...` 看 Top-1 是否真是 mapping-state（语义命中）

中期（agent · Phase 4）：
- 对比 mock vs onnx 在 corpus/_eval/queries.yaml 上的 recall@K · 量化语义检索质量
- 更新 docs/concepts/mvp.md「~92%」→「100%」
- 多处 STATUS / ROADMAP / USER-VERIFICATION 把「task #87 阻塞」描述移除

长期：
- 看 ort 何时出真正 2.0 stable（当前还是 rc 系列 · rc.12 后续可能还 RC）· 升 stable 后再 commit 一次
- 评估 candle 在 BGE-M3 上的性能 / 准确度 vs ort + ONNX Runtime · 作 Round 41+ 的备选迁移
- CI 加 onnx matrix step（macOS + Linux · Windows 看 ONNX Runtime 跨平台支持）
