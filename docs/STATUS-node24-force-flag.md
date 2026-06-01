# STATUS — node24-force-flag

> 配套 feature log：`feedback/features/rag4arkui-core/44-2026-06-01-node24-force-flag.md`
> 配套 meta：`feedback/meta/21-2026-06-01-node24-force-flag.md`
> 日期：2026-06-01

---

## 当前状态

承接用户首推 v0.0.2-rc.1 后看到 GitHub Actions deprecation warning · 3 个 workflow 各加 `FORCE_JAVASCRIPT_ACTIONS_TO_NODE24=true` env · 一行解决 + 提前用 Node 24 跑暴露兼容问题。

本阶段交付：
- 3 个 workflow 各 +2-3 行 env 注释 + 变量
- 双轨归档（meta/21 + feature/44 + STATUS · 本文件）
- 不动 action 版本（v4/v3 保留 · env 强制 Node 24）

意义：提前 15 天（距 6/16/2026 强制升级）处理 deprecation · 不影响当前 v0.0.2-rc.1 release（已经在跑）· 下次 push master / 推 tag 时立刻验证。

## 输入契约

### Workflow env 节新增

每个 workflow 顶层 env 节 +1 行：

```yaml
env:
  ...其它 env...
  FORCE_JAVASCRIPT_ACTIONS_TO_NODE24: "true"
```

### 不变项

- workflow 触发条件全不变（book.yml docs/** · ci.yml master · release.yml v*）
- workflow matrix 全不变
- workflow features 全不变（仍 `http,mcp,lsp,tantivy,typescript,corpus-pull`）
- action 版本全不变（仍 v4 / v3）
- CLI / API / cargo / 文档全不变

## 输出契约

### 跑 CI 时 stderr 变化

| Before | After |
|---|---|
| 每次输出 warning「Node.js 20 actions are deprecated ...」 | 抑制 ✓ |
| Node runtime 默认 Node 20 | 强制 Node 24 |

### 6/16/2026 之后

| Before | After |
|---|---|
| 自动切 Node 24 · 可能踩兼容 bug 影响正在跑 release | 已用 Node 24 跑过 2 周 · 提前发现问题 |
| 配置：无 env 变量 | env 变量仍在（无副作用 · 自然成为冗余）|

## 验证手段

### 自动化（用户 push 后自然触发）

```bash
# push 本 commit
git push origin master
```

之后看：

| Action | URL |
|---|---|
| ci.yml | `github.com/LeoG1022/RAG4ArkUI/actions/workflows/ci.yml` |
| book.yml | 同上 + `/book.yml` |

期望：跑 run 详情 stderr 没了「Node.js 20 actions are deprecated ...」warning。

### release.yml 验证

等下次 push tag（不一定要立刻 · 等需要新 release 时）· 看 release.yml 同样没了 warning。

## 与上一阶段的关联性

| Round | 主题 | 跟本轮关系 |
|---|---|---|
| 21 | CI: 写 ci.yml（Day 22）| **本轮维护它** |
| 75 (Day 20b) | release.yml CI matrix | **本轮维护它** |
| 83 (Day 22) | book.yml mdBook 部署 | **本轮维护它** |
| 76 | 用户首推 v0.0.2-rc.1 | **本轮触发源** |
| 43 | github-pages-deploy concept | 上轮 |
| **44（本轮）** | node24-force-flag | 防 deprecation |

层次：本轮纯 CI 维护 · 不改既有逻辑 · 只加 env 防风险。

兼容性：完全向后兼容 · GitHub 默认 Node 24 切换前后表现一致（都用 Node 24 跑）。

破坏性变更：无。

## 完成度 / 下一阶段

### 达成度

| 项 | 状态 |
|---|---|
| 3 workflow 加 FORCE env | ✅ |
| 双轨归档 + STATUS | ✅ |
| 用户 push 后验证（CI 跑 stderr） | ⏳ |
| 6/16 后跟踪是否平滑切换 | ⏳ |

### 下一阶段建议

立即（用户做）：
- 看当前 v0.0.2-rc.1 的 release.yml 跑结果（本轮 commit 不影响这个 · 但用户可能想确认完成）

短期：
- agent 跟踪 6/16/2026 GitHub Actions 强制 Node 24 切换 · 看 workflow 是否平稳
- 任何 action 出稳定 v5+ → 升级 action 版本（独立 commit · 不急）
- 用户决策 task #85 推 v1.0.0 时机 · 推之前最好 release.yml 在 Node 24 下跑过一次（本轮已提前做好）

中期：
- 等所有 v4/v3 actions 出 v5+ 后 · 统一升级一轮 · 然后去掉 FORCE env 变量
- release.yml 考虑加 onnx feature matrix（Round 43+ 残留）

长期：
- GitHub Actions 大概每 2 年 deprecate 一次 Node 版本（10 → 12 → 16 → 20 → 24 → 26...）· 本轮 env 变量模式可重用 · 改 number 即可
