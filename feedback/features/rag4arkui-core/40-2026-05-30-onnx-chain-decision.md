# 40 — onnx-chain-decision

> 日期：2026-05-30
> 涉及代码：`crates/Cargo.toml`（ort 升级 + 去 load-dynamic）· `docs/concepts/onnx-chain.md`（新）· `docs/concepts/README.md` / `docs/GLOSSARY.md` / `mdbook/src/reference/concepts/onnx-chain.md` / `mdbook/src/SUMMARY.md`（关联更新）
> 类型：决策 + bug 修复 Phase 1（解 ort 库 broken · 概念归档）

## 本轮目标

破解 task #87「ort 2.0 RC 整链路 broken」的根因 · 让 `cargo check -p arkui-rag-embedding --features onnx` 至少**通过 ort 自身的编译**。同时把 ONNX 链路决策梳理归档到 `docs/concepts/onnx-chain.md` · 防止下次新读者重复研究 4 个选项（ort 升级 / ort 1.16 / candle / OpenAI API）。

本轮聚焦 **Phase 1**：
- 找到 broken 真正根因
- 修 Cargo.toml 让 ort 库自身编过
- 决策归档

下一轮 Phase 2 = 修我们 501 行 ONNX 代码适配 rc.12 API。

## Plan

### 决策回顾：A · 升 ort 最新 RC

Round 39 末尾 agent 给了 4 个选项（A 升 ort / B 退 1.16 / C 换 candle / D 接 OpenAI API）。用户选 **A** · 理由：

- 最小改动（不重写 501 行 ort 代码）
- 试 1 小时即可判断 · 不行再转 C

### Phase 1 实施步骤

1. **诊断现状**：
   - Cargo.toml pin `ort = "2.0.0-rc.4"` · 但 Cargo.lock 实际锁到 **rc.12**（caret 解析）· crates.io 最新也是 rc.12 · 没 stable 2.0
   - `cargo check --features onnx` 错：`error[E0609] no field SessionOptionsAppendExecutionProvider_VitisAI`
   - 错位置：**ort 库自身 `src/ep/vitis.rs:47`** · 不是我们代码

2. **找根因**：
   - 看 `vitis.rs:42` —— `#[cfg(any(feature = "load-dynamic", feature = "vitis"))]`
   - 我们 features 含 `load-dynamic` → 触发 vitis.rs 编译 → 撞 ort 自身 bug（ortsys 没暴露 VitisAI 字段）
   - 结论：**不是 ort 整 broken · 是 `load-dynamic` feature 撞了 vitis EP bug**

3. **修法**：
   ```toml
   # before
   ort = { version = "2.0.0-rc.4", default-features = false, features = ["load-dynamic", "ndarray"] }

   # after（Round 40）
   ort = { version = "2.0.0-rc.12", features = ["ndarray"] }
   # 用默认 features（含 download-binaries 静态链接 ONNX Runtime · binary 大几 MB 但稳定）
   ```

4. **验证**：
   - `cargo check -p arkui-rag-embedding --features onnx` → **ort 自身编过 ✓**
   - 但我们 onnx.rs / onnx_embedder.rs / onnx_reranker.rs / reranker_onnx.rs 共 47 个 API 错（rc.4 → rc.12 API 漂移）
   - 默认 features `cargo check --workspace` 不受影响（不拉 ort）· master 仍 buildable ✓

### 概念归档（5 步流程）

1. 写 `docs/concepts/onnx-chain.md` 4 节：一句话 / 业界用法 / 本项目里怎么用 / 类比
2. `docs/concepts/README.md` 「现有条目」表加一行
3. `docs/GLOSSARY.md` 加链接
4. `mdbook/src/reference/concepts/onnx-chain.md` 一行 include
5. `mdbook/src/SUMMARY.md` 「概念解释」节加子项

### 替代方案权衡（已写在 onnx-chain.md）

| 方案 | 工作量 | 风险 | 选 |
|---|---|---|---|
| A 升 ort（本轮选）| 1 天 | ⭐⭐⭐ | ✅ |
| B 退 ort 1.16 | 2-3 天 | ⭐⭐ | ❌ |
| C 换 candle | 4-5 天 | ⭐⭐⭐⭐ | ❌（A 不行再考虑）|
| D 接 OpenAI API | 0.5 天 | ⭐ | ❌（偏离本地优先）|

### 不动的

- 我们 501 行 ONNX 代码（onnx.rs / onnx_embedder.rs / onnx_reranker.rs / reranker_onnx.rs）—— Phase 2 修
- onnx feature default = opt-in 不变
- CLI 参数 `--embedder onnx --model-path ...` 不变
- 索引格式 / MCP 工具签名不变

## 对话摘要

| 轮次 | 用户 | Agent |
|---|---|---|
| 1 | 「先梳理 ONNX 链路怎么走？详细的说明这是什么环节」 | 给 4 个决策选项 + 详细权衡 + 推荐 A 试 |
| 2 | 「先归档，然后按 A 执行」 | 本轮 Phase 1：归档 onnx-chain.md（5 步流程）+ 诊断 broken 根因（不是 ort 整 broken · 是 load-dynamic feature 撞 vitis bug）+ Cargo.toml 改 |

无方向调整 · 用户明确选 A + 归档优先。

## 改动要点

- **crates/Cargo.toml** L52-55：
  - `ort = "2.0.0-rc.4", default-features = false, features = ["load-dynamic", "ndarray"]`
  - → `ort = "2.0.0-rc.12", features = ["ndarray"]`（去 default-features=false · 去 load-dynamic · 加版本注释）
- **docs/concepts/onnx-chain.md** 新建 130 行 · 4 节模板 + 决策选项表 + 配套基础设施列表 + 2 套类比
- **docs/concepts/README.md** 「现有条目」表 +1 行 onnx-chain
- **docs/GLOSSARY.md** 链接区 +「ONNX 链路」
- **mdbook/src/reference/concepts/onnx-chain.md** 一行 include
- **mdbook/src/SUMMARY.md** 「概念解释」节 +1 子项

与 Round 33 关系：33 建 `docs/concepts/` 基础设施 + AGENTS.md #18 必询问归档规则。本轮 = 该规则**首次实战触发**：用户问「这是什么环节」概念问题 · agent 答完询问归档 · 用户「先归档，然后按 A」明确同意 · 完整 5 步走通。

## 验证结果

- 编译（关键）：
  - `cargo check -p arkui-rag-embedding --features onnx` → ort 自身编过 ✓ · 我们代码 47 错（Phase 2 修）
  - `cargo check --manifest-path crates/Cargo.toml --workspace`（默认 features）→ ✓ Finished 6.07s · master 仍 buildable
- check-api-parity：N/A（决策 + Cargo.toml 改 · 不动业务代码）
- 概念归档 5 步完整 · `bash scripts/preflight.sh` 应正常

47 错分类（Phase 2 准备）：
- ~30 错 · `?` 自动转 anyhow Error 不满足 Send + Sync（rc.12 的 OrtSessionOptions / OrtMemoryInfo 等含 NonNull）
- ~4 错 · `Tensor::from_array` 返回值变化（不再是 Result）
- ~2 错 · `logits.shape()` 返回 `(&Shape, &[f32])` 元组（不再是 `&[usize]`）
- ~11 错 · 其它细节

## 残留 / 下一轮

<!-- 用 - [ ] 标记未解决项，- [x] 标记已解决项；若无残留写"无" -->
- [x] 找到 broken 根因（load-dynamic feature 撞 vitis bug · 不是 ort 整 broken）
- [x] Cargo.toml 改 ort 2.0.0-rc.12 + 去 load-dynamic + 用默认 features（静态链接）
- [x] 概念归档 5 步（onnx-chain.md + README + GLOSSARY + mdbook include + SUMMARY）
- [x] 默认 features cargo check 验证 master 仍 buildable
- [ ] **Phase 2（下一 round）**：修 501 行代码适配 rc.12 API · 47 个错
  - `?` Send/Sync 集中改：所有 ort 调用加 `.map_err(|e| anyhow::anyhow!("..."))` 转 anyhow
  - `Tensor::from_array` 返回值适配
  - `logits.shape()` 元组解构
  - 验证：`cargo check -p arkui-rag-embedding --features onnx` → 0 error
- [ ] **Phase 3**：用户下载 BGE-M3 ONNX 模型 + 跑端到端 RAG 验证
- [ ] **Phase 4**：mock vs onnx embedder 对比测试 · 评估真语义检索质量提升
- [ ] **配套**：CI 加 `cargo check --features onnx` step 防回归（task #87 解锁后）
- [ ] **配套**：USER-VERIFICATION 加 onnx feature 验证段（当前明确说「不试 onnx · task #87 阻塞」· 解锁后改）
- [ ] **配套**：ROADMAP / STATUS-* 多处「task #87 阻塞」描述更新（解锁后）
