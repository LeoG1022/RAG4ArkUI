# mdBook

> 一句话：**Rust 官方推的文档生成器** · 给一堆 markdown + 目录配置 · 产出可搜索的静态 HTML 站点。

## 业界对标

| 生态 | 工具 |
|---|---|
| Python | MkDocs / Sphinx |
| JavaScript | docusaurus / vitepress |
| **Rust** | **mdBook** |

业内代表用例：《The Rust Book》《The Cargo Book》《Rustonomicon》《Async Book》全都用 mdBook 出。

## 为啥本项目选它

Day 22 决策（commit `98bf22d`）· 3 个原因：

1. **Rust 生态自带** · 不用引入 Node.js 工具链
2. **`{{#include}}` 内联机制** · 让 `docs/` 是单一信任源 · `mdbook/src` 只做导航与组织
3. **GitHub Actions 部署一行代码** · `actions/deploy-pages@v4` 标准

## 本项目里它产出什么

```
mdbook/
├── book.toml            # 配置（标题/作者/GitHub 编辑链接 等）
├── src/
│   ├── SUMMARY.md       # 目录树（决定左侧导航栏顺序）
│   ├── intro.md         # 首页（include README.md）
│   ├── quickstart.md
│   ├── verify.md        # 端到端验证清单
│   ├── usage/
│   │   ├── cli.md
│   │   ├── http.md
│   │   ├── mcp.md
│   │   ├── lsp.md
│   │   └── corpus.md
│   ├── adrs/             # 3 个架构决策（include docs/ADR-*.md）
│   └── reference/
│       ├── full-plan.md         # 完整技术方案 78 KB include
│       ├── status-timeline.md   # STATUS 链接
│       ├── mcp-integration.md
│       ├── glossary.md          # 术语对照表
│       └── concepts/            # ← 本文所在
└── book/                # mdbook build 产物（gitignored）
    └── ... 2.8 MB 静态 HTML + CSS + JS + 搜索索引
```

## 跑命令做什么

| 命令 | 做什么 | 何时用 |
|---|---|---|
| `make book-build` | mdbook build · 输出到 mdbook/book/ | 写完文档校验 |
| `make book-serve` | 启 web server（http://localhost:3000）+ 自动开浏览器 + 监听改动自动 rebuild | 本地预览 / 调试导航 |
| `make book-clean` | 清 mdbook/book/ | 偶尔 |
| `make install-mdbook` | 提示装 mdbook（brew / cargo） | 首次缺工具 |

## GitHub Actions 部分

`.github/workflows/book.yml`：push master 后自动跑 `mdbook build` 然后推到 GitHub Pages · 站点 URL `https://LeoG1022.github.io/RAG4ArkUI/`。

需要仓库 Settings → Pages → Source 选 "GitHub Actions"。

## 站点结构

```text
┌─ 左侧导航 ─────────┐  ┌─ 主内容 ────────────────────┐
│ 简介                │  │ # RAG4ArkUI                  │
│ 当前状态             │  │                              │
│ # 上手              │  │ > 面向 OpenHarmony...        │
│   快速开始           │  │ ## 项目愿景                  │
│   端到端本地验证     │  │ ...                          │
│   架构总览           │  │                              │
│   完整路线图         │  │                              │
│ # 使用              │  │                              │
│   CLI / HTTP / MCP  │  │                              │
│   LSP / Corpus      │  │                              │
│ # 运维              │  │                              │
│   Release / Cargo   │  │                              │
│ # 架构决策          │  │                              │
│   ADR-001/002/003   │  │                              │
│ # 参考              │  │                              │
│   完整技术方案       │  │                              │
│   STATUS 时间线     │  │                              │
│   MCP 接入指南       │  │                              │
│   术语对照表        │  │                              │
│   概念解释          │  │                              │
└────────────────────┘  └─────────────────────────────┘
顶部还有：🔍 搜索框 / 🌓 主题切换 / ✏️ 编辑此页(跳 GitHub)
```

## 类比

| 已知 | mdBook 相当于 |
|---|---|
| Markdown 文件夹 | mdBook 输入 |
| GitBook（商业版） | mdBook（开源 · 本地优先） |
| Pandoc | Pandoc 是通用转换 · mdBook 专门为静态站做 |
| Hugo / Jekyll | 同类静态站生成 · 但偏 blog · mdBook 偏 book/manual |
