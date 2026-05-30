# STATUS — github-pages-concept

> 配套 feature log：`feedback/features/rag4arkui-core/43-2026-05-30-github-pages-concept.md`
> 日期：2026-05-30

---

## 当前状态

承接 Round 42 末尾用户问「task #84 CI In progress 是正常的吗」· 把「GitHub Pages 自动部署（book.yml 流程）」按 AGENTS.md #18 归档。

本阶段交付：
- `docs/concepts/github-pages-deploy.md` 160 行 · 4 节模板（一句话 / 业界用法 / 项目里怎么用 / 类比）
- 5 步流程跑通：concepts/ + README + GLOSSARY + mdbook include + SUMMARY
- AGENTS.md #18 第 2 次实际归档执行（Round 40 onnx-chain 之后）

意义：下个用户配 Pages 时直接 grep 看 5 步流程 + 5 种 fail 修法 · 不用查 GitHub 官方文档。Round 33 立的「概念问答必询问归档」规则有了第 2 个实证样本。

## 输入契约

### 概念归档触发

用户问类「book.yml / Pages 部署 怎么工作」类问题 → AGENTS.md #18 触发 → agent 答完询问 → 用户「按 #18 归档」 → 5 步流程。

### 不变项

- 没动 `book.yml` workflow 自身
- 没动 `mdbook/` 配置
- 没动 task #84 实际推进路径（用户重新推 master 还是会触发 book.yml）

## 输出契约

### 概念文档结构

```
docs/concepts/github-pages-deploy.md
├── 一句话
├── 业界用法（5 个文档站工具横向对比 · 都用 deploy-pages）
├── 本项目里怎么用
│   ├── book.yml 触发条件
│   ├── 两个 job 顺序（build · deploy）
│   ├── 看状态的 3 种方法
│   ├── 前置一次性配置（Pages Source）
│   ├── 没配的症状
│   ├── 常见 5 种 fail 模式 + 修法
│   └── 与 task #84 / #85 / mdBook / MVP 等的关系
├── 类比（npm publish / Docker Hub / Vercel-Netlify）
└── 相关链接
```

### mdBook 站点新增页

mdbook 重 build 后 · 「参考 → 概念解释」节多一项 `GitHub Pages 部署` · URL 形如 `<site>/reference/concepts/github-pages-deploy.html`。

## 验证手段

### Agent 本轮已做

- 文档 4 节齐 ✓
- 5 步流程齐 ✓
- 路径同既有 onnx-chain.md 模式（一致性）✓

### 用户验证（task #84 完成后）

```bash
# 浏览器打开（task #84 跑完后）：
https://leog1022.github.io/RAG4ArkUI/reference/concepts/github-pages-deploy.html
```

应渲染本轮新加的 4 节内容 · 含表格 + ASCII pipeline + 5 种 fail 模式表。

## 与上一阶段的关联性

| Round | 主题 | 跟本轮关系 |
|---|---|---|
| 33 | AGENTS.md #18「概念问答必询问归档」规则建立 | **规则源头** |
| 40 | 归档 onnx-chain.md（task #87 决策梳理）| #18 第 1 次实际归档 |
| 42 | task #87 完全解锁 + 多文档同步 100% | 上轮 |
| **43（本轮）** | 归档 github-pages-deploy.md（book.yml 流程）| **#18 第 2 次实际归档** |

层次：Round 33 立规则 · Round 40 跑通首次实战 · Round 43 再次验证规则被遵守 · 「概念归档」流程稳定可复用。

兼容性：完全向后兼容 · 只加文档 · 不改 workflow / 代码 / 配置。

破坏性变更：无。

## 完成度 / 下一阶段

### 达成度

| 项 | 状态 |
|---|---|
| github-pages-deploy.md 4 节 | ✅ |
| concepts/README 表格 +1 行 | ✅ |
| GLOSSARY 链接 +1 | ✅ |
| mdbook/src include 文件 | ✅ |
| SUMMARY 节 +1 子项 | ✅ |
| feature log + STATUS（本轮配套）| ✅ |
| 用户在新站点看到该页 | ⏳（等 task #84 book.yml 跑完）|

### 下一阶段建议

立即（用户做）：
- 等当前 In progress 的 book.yml 跑完
- 浏览器打开 `https://leog1022.github.io/RAG4ArkUI/` 确认整站可用
- 跳到「参考 → 概念解释 → GitHub Pages 部署」看本轮新加的页是否渲染对

短期（agent · 1-2 round）：
- 跟踪 AGENTS.md #18 规则遵守的统计 · 现在累计 2 次归档（Round 40 onnx + Round 43 pages）· 看后续是否有「应归档但没问」的漏案
- 考虑加 `docs/concepts/` 索引页的「累计归档统计」段（每次新增 round 自动更 +1）

中期：
- 看 `docs/concepts/` 是否积累到 ≥10 篇 · 考虑分类（如 `concepts/protocol/` `concepts/retrieval/` `concepts/tooling/`）
- 是否做 `/archive-concept <term>` skill 自动化 5 步流程 · 减轻 agent 手动操作

长期：
- 1.0 release 时回顾：`docs/concepts/` 的累计内容是否真有用过（grep 频率 / 解答减少率）· 决定是继续保持自我约束 · 还是放宽
