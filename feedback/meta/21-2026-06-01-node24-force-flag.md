# 21 — node24-force-flag

> 日期：2026-06-01
> 触发：用户首次推 v0.0.2-rc.1 后 GitHub Actions 输出「Node.js 20 actions are deprecated · 6/16/2026 强制升 Node 24」warning · 用户贴给 agent
> 类型：CI / workflow 配置（防 deprecation 风险）

---

## 用户提出的要求

用户贴 release.yml 跑时 GitHub Actions 输出原文 warning：

> Node.js 20 actions are deprecated. The following actions are running on Node.js 20 and may not work as expected: actions/checkout@v4, actions/upload-artifact@v4. Actions will be forced to run with Node.js 24 by default starting June 16th, 2026.

隐含「这玩意要紧不 / 怎么修」。

## Agent 给出的修改建议

### 受影响 actions（3 workflow · 6 种 · 14 处）

- `actions/checkout@v4` × 9 处（全 3 workflow）
- `actions/cache@v4` × 1 处（book.yml）
- `actions/upload-artifact@v4` × 1 处（release.yml）
- `actions/download-artifact@v4` × 1 处（release.yml）
- `actions/deploy-pages@v4` × 1 处（book.yml）
- `actions/upload-pages-artifact@v3` × 1 处（book.yml）

未受影响（已 v5+ 或第三方）：
- `actions/configure-pages@v5`
- `dtolnay/rust-toolchain@stable`
- `Swatinem/rust-cache@v2`
- `softprops/action-gh-release@v2`

### 修法方案对比

| 选项 | 工作量 | 风险 | 长期 |
|---|---|---|---|
| **A · 加 `FORCE_JAVASCRIPT_ACTIONS_TO_NODE24=true` env**（本轮）| 3 行 | ⭐ 极低 | 等 GitHub 完全切到 Node 24 默认就可去掉 |
| B · 逐个升 action 版本 | 30 行 + 多版本检查 | ⭐⭐⭐ 中（v5 不全 · 部分 action 还没 v5）| 长期更彻底 · 但等 v5 全出后才能做 |
| C · 不动 · 等 6/16 后处理 | 0 | ⭐⭐⭐⭐ 高 · 那时如果出问题影响正在跑的 release | 等于赌 GitHub 平滑迁移 |

选 A · GitHub 官方推荐做法 · 5 分钟修完 · 双保险（warning 抑制 + 强制 Node 24 跑 = 提前 2 周暴露兼容问题）。

### 实施位置

每个 workflow 顶层 env 节加：

```yaml
env:
  ...其它现有 env...
  # Round 44 · 抑制「Node 20 deprecated」warning（6/16/2026 后强制 Node 24）
  FORCE_JAVASCRIPT_ACTIONS_TO_NODE24: "true"
```

具体：
- `.github/workflows/book.yml` line 37（MDBOOK_VERSION 之后）
- `.github/workflows/ci.yml` line 33（注释之后）
- `.github/workflows/release.yml` line 44（FEATURES 之后）

### 对当前 v0.0.2-rc.1 release 不影响

- v0.0.2-rc.1 tag 已存在 · release.yml 已经在跑（或已完成）
- 本轮 env 在新 commit 里 · 不会重跑 v0.0.2-rc.1（tag 不变）
- 下次 release 才生效

## 多轮互动

| 轮次 | 用户 | Agent |
|---|---|---|
| 1 | 贴 Node 20 deprecation warning 原文 | 解释「warning 不阻塞当前 release · 但 6/16 强制」· 扫 3 workflow · 给方案 A · 实施修改 |

无方向调整。

## 实际改动

- 接口变化：无
- 规则变化：CI / release / book 工作流默认 Node 24 跑（之前 Node 20）· 所有 v4 / v3 actions 强制 Node 24
- 文件变化：
  - `.github/workflows/book.yml` env 节 +3 行（注释 + env 变量）
  - `.github/workflows/ci.yml` env 节 +2 行
  - `.github/workflows/release.yml` env 节 +2 行
- 配置变化：无（不动 action 版本）

## 执行生效后总结

### 实际产出

| 文件 | 改动 |
|---|---|
| `.github/workflows/book.yml` | +3 行 |
| `.github/workflows/ci.yml` | +2 行 |
| `.github/workflows/release.yml` | +2 行 |
| `feedback/meta/21-2026-06-01-node24-force-flag.md` | 本归档 |
| `feedback/features/.../44-...` | feature log（pre-commit 强制）|
| `docs/STATUS-node24-force-flag.md` | STATUS |

### 前后对比

| 维度 | Before | After |
|---|---|---|
| Actions warning「Node 20 deprecated」 | 每次 run 输出 | 抑制 ✓ |
| Node runtime | 默认 Node 20（v4 actions 用）| 强制 Node 24 |
| 6/16/2026 后行为 | 自动切 Node 24 + 可能踩兼容 bug | 已用 Node 24 跑过 2 周 · 提前发现问题 |

### 实测验证

- 本地 yaml grep / sed 可 parse · 不破 workflow trigger 语义
- 下次 push master → ci.yml + book.yml 用新 env 跑 · 看 stderr 没了 deprecation
- 下次 push tag v0.0.2-rc.2（或重推）→ release.yml 同上

注：本 commit push master 后 · ci.yml + book.yml 立刻验证。release.yml 等下次推 tag。

### 残留 / 下一轮处理

- [x] 3 workflow 加 FORCE_JAVASCRIPT_ACTIONS_TO_NODE24
- [x] 双轨归档 + STATUS
- [ ] **用户 push 后验证**：看下次 CI run stderr 没了 Node 20 deprecation warning
- [ ] **长期 · 各 action 出稳定 v5+ 后**：逐个升 action 版本 · 然后去掉本 env 变量（约 2026 Q4 / 2027 Q1）
- [ ] **跟踪 GitHub Actions 默认 Node 24 切换**（6/16/2026 起）· 切完后 env 变量可去掉但不必（无副作用）
