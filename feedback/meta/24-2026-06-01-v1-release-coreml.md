# 24 — v1-release-coreml

> 日期：2026-06-01
> 触发：Round 47 业务 commit（v1.0.0 + CoreML）含 README.md 版本号同步 · classify-change 把 README.md 判 meta · pre-commit 强制 meta feedback
> 类型：版本号 / 文档同步（mixed commit 配套元归档）

---

## 用户提出的要求

无独立要求 · 配套 Round 47 业务 commit。完整业务上下文见 `feedback/features/rag4arkui-core/47-2026-06-01-v1-release-coreml.md`。

## Agent 给出的修改建议

Round 47 commit 含 6 个业务文件 + 1 个 meta 文件（README.md）：

| 类 | 文件 |
|---|---|
| business | crates/Cargo.toml · docs/RELEASE.md · docs/USER-VERIFICATION.md · feedback/features/.../47 · docs/STATUS-v1-release-coreml.md |
| business（unknown）| Makefile · mdbook/src/quickstart.md |
| **meta** | **README.md** |

README.md 被 classify-change 判 meta（基础设施根文档）· 必须配套 meta feedback。本归档即占位。

## 多轮互动

无 —— 跟 Round 47 同一 commit · agent 自主补本归档。

## 实际改动

- 接口变化：无
- 规则变化：README.md 顶部下载链接 v0.0.1 → v1.0.0（5 处替换）
- 文件变化：README.md（sed 替换）
- 配置变化：无

## 执行生效后总结

### 实际产出

| 文件 | 改动 |
|---|---|
| `README.md` | sed v0.0.1 → v1.0.0（5 处）|
| `feedback/meta/24-...` | 本归档（占位 · 真实内容在 feature/47）|

### 前后对比

| 维度 | Before | After |
|---|---|---|
| README 顶部下载链接 | v0.0.1 占位 | v1.0.0（task #85 推 tag 后真活）|

### 实测验证

- `cargo check --workspace` ✓（Round 47 已验证）
- pre-commit hook：classify mixed(meta=1, business=5) · 需此 meta 归档

### 残留 / 下一轮处理

<!-- 用 - [ ] 标记未解决项，- [x] 标记已解决项；若无残留写"无" -->
- [x] README.md v0.0.1 → v1.0.0
- [x] 占位 meta 归档
- [ ] 用户 push v1.0.0 后 README 下载链接才真活（task #85 等用户操作）
