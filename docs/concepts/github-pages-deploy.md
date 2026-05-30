# GitHub Pages 自动部署（book.yml）

> 上下文：用户首推 master 后问「CI #7 In progress 是 task #84 的正常状态吗」· 暴露这个流程值得归档（Round 42 触发）。

## 一句话

用一个 GitHub Actions workflow（`book.yml`）监听 docs / mdbook 改动 · 自动 `mdbook build` 出静态站 · 然后用 `actions/deploy-pages` 推到 GitHub Pages · 用户浏览器打开 `https://<user>.github.io/<repo>/` 看新文档。

## 业界用法

GitHub Pages 是 GitHub 内建的免费静态站托管 · 主流文档站工具几乎都有官方 GitHub Actions workflow 模板：

| 文档工具 | 业界做法 | 典型耗时 |
|---|---|---|
| **mdBook**（本项目）| `mdbook build` + `actions/deploy-pages@v4` | 2-3 分钟 |
| Docusaurus（React 体系）| `npm run build` + 同款 deploy-pages | 3-5 分钟 |
| Hugo / Jekyll | 官方有专属 action | 1-2 分钟 |
| VitePress（Vue 体系）| `vitepress build` + deploy-pages | 2-4 分钟 |
| Astro / Eleventy | 同上 | 2-3 分钟 |

业界**前置规则**几乎一样：

1. 仓库 Settings → Pages → Source 必须选 **"GitHub Actions"**（不是默认的 "Deploy from a branch"）
2. workflow 用 `environment: github-pages` + `actions/deploy-pages@v4`
3. workflow 需要 `permissions: pages: write, id-token: write`

任何文档站接 GitHub Pages CD 都按这套来 · 跟用哪个工具无关。

## 本项目里怎么用

### `book.yml` 触发条件

```yaml
on:
  push:
    branches: [master, main]
    paths:
      - "mdbook/**"          # mdBook 源文件改了
      - "docs/**"            # docs/ 改了（mdbook 通过 {{#include}} 引）
      - ".github/workflows/book.yml"   # workflow 自身改了
  workflow_dispatch:         # 也能在 GitHub UI 手动触发
```

任何动 `docs/**` 或 `mdbook/**` 的 commit push 到 master → 自动跑 build + deploy。

### 两个 job · 顺序执行

```
build  (ubuntu-latest)
  ├─ checkout
  ├─ install mdbook
  ├─ mdbook build           ← 出 mdbook/book/ 静态 HTML
  └─ upload-pages-artifact  ← 上传到 GitHub 的临时存储

deploy (ubuntu-latest · needs: build)
  ├─ environment: github-pages
  └─ actions/deploy-pages@v4   ← 拉 artifact · 推到 Pages
```

### 触发后看什么

```
浏览器:
https://github.com/LeoG1022/RAG4ArkUI/actions
```

每条 workflow run 三种状态：

| 颜色 | 含义 |
|---|---|
| 🟡 黄色转圈 | 正在跑（build 或 deploy）|
| 🟢 绿勾 | 成功 · deploy job 输出 URL |
| 🔴 红叉 | 失败（看 job 详情）|

成功后站点 URL（form）：

```
https://<user>.github.io/<repo>/
# 本项目 = https://leog1022.github.io/RAG4ArkUI/
```

### 前置一次性配置（task #84 关键 step）

仓库 Settings → Pages → Source → **选 "GitHub Actions"**（不是默认的 "Deploy from a branch"）。

**没配的症状**：
- build job 成功 ✅
- deploy job 报 `Get Pages site failed`（最常见）或者一直等 environment approval

**配好之后**：永久生效 · 每次 push 自动 build + deploy · 不用再动。

### 常见 fail 模式

| 错误 | 原因 | 修法 |
|---|---|---|
| `Get Pages site failed` | Settings → Pages Source 未配 | 选 "GitHub Actions" |
| `book.toml not found` | mdbook 配置出错 | 看 `mdbook/book.toml` |
| `mdbook: command not found` | install mdbook step 失败 | 看 cargo install 或 brew 是否过 |
| `MDBook build failed: <preprocess>` | `{{#include ../../path}}` 路径错 | 看 SUMMARY.md 引用 |
| `Pages: 404 after deploy success` | 第一次 deploy 后 CDN 还没刷新 | 等 1-5 分钟 |

## 类比

| 已知 | GitHub Pages CD 对应 |
|---|---|
| `npm publish` 自动化版本 | push tag → CI build + 推 npm registry |
| Docker Hub auto-build | push 到 master → 自动 build image + push |
| 静态站 CD（Vercel / Netlify）| push → 自动 build + serve（同款思路 · 但 Pages 免费 + GitHub 内建）|
| Wiki 自动同步 | docs/ 改 → 网站自动跟着改 · 编辑流程 = git workflow |

或者：

```
你写文档              GitHub 编辑器
   ↓
git push master       触发 workflow
   ↓
build job             mdbook build 出 HTML
   ↓
deploy job            推到 gh-pages branch / Pages CDN
   ↓
浏览器 URL            读者看到新文档
```

跟「写代码 → push → CI 跑测试 → 上线」一样 · 只是产物从「服务」变成「静态站」。

## 与本项目其它概念的关系

| 相关 | 关系 |
|---|---|
| [mdBook](mdbook.md) | book.yml 用的工具 · 把 markdown 变 HTML |
| [MVP](mvp.md) | 文档上线是 MVP 100% 的「让代码 + 文档 + release 对外可见」中的「文档」环节 |
| task #84 | 「用户首次推 master 触发 book.yml + 配 Settings→Pages」 = 跑通本流程的实操项 |
| task #85 | 推 v1.0.0 tag 跟本流程独立 · v1.0.0 是 release.yml 的事 · 不是 book.yml |

## 相关链接

- workflow 文件：[`.github/workflows/book.yml`](../../.github/workflows/book.yml)
- mdBook 源：[`mdbook/`](../../mdbook/) · 入口 [`mdbook/src/SUMMARY.md`](../../mdbook/src/SUMMARY.md)
- 上游：[mdBook docs](https://rust-lang.github.io/mdBook/) · [GitHub Pages docs](https://docs.github.com/pages) · [actions/deploy-pages](https://github.com/actions/deploy-pages)
- 决策上下文：`feedback/meta/11-2026-05-29-mdbook-deploy-workflow.md`（最初引入 book.yml 的 round）
