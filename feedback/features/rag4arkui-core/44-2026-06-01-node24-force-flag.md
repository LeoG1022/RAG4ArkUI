# 44 — node24-force-flag

> 日期：2026-06-01
> 涉及代码：`.github/workflows/{book,ci,release}.yml`
> 类型：CI 配置 / 防 deprecation

## 本轮目标

用户首次推 v0.0.2-rc.1 后看到 GitHub Actions deprecation warning：「Node 20 actions deprecated · 6/16/2026 强制升 Node 24」· 影响 6 种 v3/v4 action 共 14 处使用。

加 `FORCE_JAVASCRIPT_ACTIONS_TO_NODE24=true` env 到 3 个 workflow · 一行解决 + 提前用 Node 24 跑暴露兼容问题。

## Plan

### 修法（方案 A · 详见 meta/21）

每个 workflow 顶层 env 节加：

```yaml
FORCE_JAVASCRIPT_ACTIONS_TO_NODE24: "true"
```

3 处：
- book.yml line 37（MDBOOK_VERSION 之后）
- ci.yml line 33（注释之后）
- release.yml line 44（FEATURES 之后）

### 不动 action 版本

选 A 不选 B（逐个升 v5）是因为：
- `actions/upload-artifact@v5` / `actions/deploy-pages@v5` 等部分 action 还没出 v5（截至 2026-06）
- 等全部 v5 稳定再升 · 现在用 env 双保险

### 不影响当前 v0.0.2-rc.1

- tag 已存在 · release.yml 在跑（或已完成）· 不重跑
- 下次 release 才生效

### 替代方案权衡

| 选项 | 工作量 | 风险 | 选 |
|---|---|---|---|
| A · env 一行（本轮）| 3 行 | ⭐ 极低 | ✅ |
| B · 逐个升 action 版本 | 30 行 + 多版本检查 | ⭐⭐⭐ 中 | ❌（等 v5 全出）|
| C · 不动等 6/16 | 0 | ⭐⭐⭐⭐ 高 | ❌ |

## 对话摘要

| 轮次 | 用户 | Agent |
|---|---|---|
| 1 | 贴 GitHub Actions Node 20 deprecation warning 原文 | 解释「warning 不阻塞当前 release · 6/16 强制」· 扫 3 workflow · 14 处受影响 · 给方案 A · 实施 |

无方向调整 · 用户隐含「修一下」· agent 自主决定方案 A。

## 改动要点

- `.github/workflows/book.yml` env 节 +3 行（含注释 + env 变量）
- `.github/workflows/ci.yml` env 节 +2 行
- `.github/workflows/release.yml` env 节 +2 行
- 不动 action 版本（v4/v3 仍保留 · env 强制用 Node 24 跑）
- 与 Round 21 ci-github-actions / Round 8 release-ci-matrix 关系：本轮是这两轮 workflow 的维护更新 · 不改触发条件 / 不改 matrix / 不改 features · 只加 env 抑制 deprecation

## 验证结果

- 编译：N/A（YAML 配置）
- yaml 语法：grep / sed 能 parse · 不破触发语义
- 实测：等下次 push master 后看 ci.yml 和 book.yml 的 stderr 没了 Node 20 deprecation warning
- release.yml 验证等下次推 tag

## 残留 / 下一轮

- [x] 3 workflow 加 FORCE_JAVASCRIPT_ACTIONS_TO_NODE24
- [x] 双轨归档（meta/21 + 本 feature log + STATUS）
- [ ] **用户 push 后验证**：看 CI run stderr 没了 deprecation warning
- [ ] **长期 · 各 action 出稳定 v5+ 后**：逐个升 action 版本 + 去掉本 env 变量（约 2026 Q4 / 2027 Q1）
- [ ] **跟踪 GitHub Actions 默认 Node 24 切换**（6/16/2026 起）· 切完后 env 可去掉但不必（无副作用）
- [ ] **release.yml 加 onnx feature matrix 候选**（Round 43 残留延续）· 同时考虑加 lancedb · 但 binary 大 / 编译慢 · 谨慎
