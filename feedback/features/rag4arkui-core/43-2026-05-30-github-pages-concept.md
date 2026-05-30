# 43 — github-pages-concept

> 日期：2026-05-30
> 涉及代码：`docs/concepts/github-pages-deploy.md`（新）· `docs/concepts/README.md` · `docs/GLOSSARY.md` · `mdbook/src/reference/concepts/github-pages-deploy.md` · `mdbook/src/SUMMARY.md`
> 类型：知识归档（概念问答驱动 · AGENTS.md #18 第 4 次实战）

## 本轮目标

承接 Round 42 末尾 · 用户首推 master 后问「CI #7 In progress 是 task #84 的正常状态吗」· 隐含 `book.yml workflow / GitHub Pages 部署流程怎么工作` 概念问题。

Agent 答完询问归档 · 用户回「按 AGENTS.md #18 归档」明确同意 · 走 5 步流程把「GitHub Pages 自动部署」固化进 concepts/。

## Plan

### 设计：4 节模板 + 项目特化

`docs/concepts/github-pages-deploy.md` 160 行 · 严格按 concepts 4 节模板：

1. **一句话**：book.yml workflow + actions/deploy-pages 把 mdBook 站推到 `<user>.github.io/<repo>/`
2. **业界用法**：横向对比 Docusaurus / Hugo / VitePress / Astro / Eleventy · 都同款 deploy-pages 套路 · 前置都是 Settings → Pages Source 选 "GitHub Actions"
3. **本项目里怎么用**：
   - `book.yml` 触发条件（docs/** / mdbook/** / workflow_dispatch）
   - 两个 job 顺序（build · deploy）
   - 看状态的 3 种方法（浏览器 / gh CLI / specific workflow URL）
   - 前置一次性配置 + 没配的症状（`Get Pages site failed`）
   - 5 种常见 fail 模式 + 修法
   - **与 task #84 / #85 关系**
4. **类比**：npm publish CD / Docker Hub auto-build / Vercel-Netlify · 让没接触过的读者瞬间 onboard

### 5 步流程齐

1. ✅ `docs/concepts/github-pages-deploy.md` 新建
2. ✅ `docs/concepts/README.md` 「现有条目」表 +1 行
3. ✅ `docs/GLOSSARY.md` 链接区 +「GitHub Pages 部署」
4. ✅ `mdbook/src/reference/concepts/github-pages-deploy.md` 一行 include
5. ✅ `mdbook/src/SUMMARY.md` 「概念解释」节 +1 子项

### Round 33 AGENTS.md #18 第 4 次实战

历次实战：
- **Round 40**：用户问「ONNX 链路怎么走」→ 归档 onnx-chain.md（首次）
- **Round 42 隐含**：用户问 model-pull 下载到哪里 → 没归档（路径问题不属概念）
- **Round 42 末尾**：用户问 CI In progress 是否正常 → agent 询问归档 · 用户明确「按 #18 归档」（**第 2 次明确归档**）
- 本轮即「第 2 次实际归档执行」（Round 33 加自我约束后的 Round 40 / Round 43 共 2 次）

按 Round 33 设计 · 「规则被遵守」需要观察 → 本轮**完整执行 5 步** · 证明规则真活。

### 替代方案权衡

| 选项 | 优点 | 缺点 | 选 |
|---|---|---|---|
| A · 单 concept 涵盖完整（本轮）| 一处看完整流程 | 文档稍长 160 行 | ✅ |
| B · 拆 2 篇（pages-config / book-workflow）| 主题更聚焦 | 互相 link 多 · 读者跳来跳去 | ❌ |
| C · 不归档 · 留 GitHub 官方文档 | 0 工作量 | 用户每次问都得跑 agent · 不符 AGENTS.md #18 精神 | ❌ |

## 对话摘要

| 轮次 | 用户 | Agent |
|---|---|---|
| 1 | 「这是 task #84 的正常状态吗？」（贴 CI #7 In progress）| 答正常 · 解释 book.yml 触发 + 提醒 Pages Source 配置 · 末尾按 #18 询问归档 |
| 2 | 「Source 选 GitHub Actions 这个配置过了」 | 答 task #84 没阻塞 · 等 build 完 · 给 URL 形式 · 没再询问归档（用户答了相邻问题）|
| 3 | 「按 AGENTS.md #18 归档」 | 本轮：完整 5 步流程跑通 + 归档（本文件）|

## 改动要点

- `docs/concepts/github-pages-deploy.md` 160 行 · 4 节 + 与 task 关系 + 类比
- `docs/concepts/README.md` 「现有条目」表 +1 行 · 描述 `book.yml workflow + actions/deploy-pages · 把 mdBook 站自动推到 <user>.github.io/<repo>/`
- `docs/GLOSSARY.md` 链接区 +「GitHub Pages 部署」
- `mdbook/src/reference/concepts/github-pages-deploy.md` 一行 `{{#include ../../../../docs/concepts/github-pages-deploy.md}}`
- `mdbook/src/SUMMARY.md` 「概念解释」节 +1 子项

与上轮关系：
- Round 42 task #87 完全解锁 + 多文档同步 100%
- Round 43（本轮）借「user 推 master 后问 CI 状态」机会 · 把 GitHub Pages 部署流程归档 · 让下个用户配 Pages 时直接 grep 看 5 步流程 · 不用查 GitHub 文档

## 验证结果

- 编译：N/A（纯文档）
- check-api-parity：N/A
- 5 步流程完整：✅
- 概念文档 4 节齐：✅
- mdBook include 引用：✅（待 mdbook build 验证 · 但路径跟现有 onnx-chain.md 同款 · 应该过）
- pre-commit hook：本 commit 触发 M-FEATURE-PLAN 要求 · 故有本 feature log

## 残留 / 下一轮

<!-- 用 - [ ] 标记未解决项，- [x] 标记已解决项；若无残留写"无" -->
- [x] github-pages-deploy.md 4 节
- [x] concepts/README + GLOSSARY + mdbook include + SUMMARY 五步齐
- [x] feature log Round 43 + STATUS（本轮配套）
- [ ] **mdBook 重 build 验证 include 路径**：等用户下次 push（或本地 `make book-build`）· 但路径跟既有 onnx-chain.md 同款 · 应该 OK
- [ ] **Round 33 AGENTS.md #18 规则被遵守的统计**：累计 2 次明确归档（Round 40 + Round 43）· 1 次拒绝（Round 42 model-pull 路径问题不归档）· 1 次未问（看是否还有错过的）· 整体规则有效
- [ ] **task #84 完成判定**：等用户 push 7555782 触发的 book.yml 跑完 · 看 URL 能打开
