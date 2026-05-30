# 概念解释（Concepts）

> 用途：对**高密度概念**做深度展开 · 区别于 `docs/GLOSSARY.md` 的一句话速查。
>
> 触发：用户问 agent「X 是什么 / 为啥这样设计 / 跟 Y 啥区别」· agent 答完后归档到这里 · 下次新读者能直接 grep 找到。
>
> 边界：
> - `GLOSSARY.md`：一行装得下（如「真活 → 实装」「stub → 占位实现」）
> - `concepts/<term>.md`：值得 3-5 段深度展开（如「tree-sitter 是什么 + 业界用法 + 项目里怎么接 + 类比」）
>
> 写完后：`docs/GLOSSARY.md` 加一行链接到 `concepts/<term>.md`。

## 现有条目

| 概念 | 一句话 |
|---|---|
| [mdBook](mdbook.md) | Rust 官方推的文档生成器 · 给 markdown + 目录配置 · 产出可搜索的静态 HTML 站点 |
| [tree-sitter](tree-sitter.md) | 跨语言的增量式 AST parser 框架 · 用于按语法切代码 chunk |
| [MVP](mvp.md) | Minimum Viable Product · 在本项目特指 6 周完成的最小可用版本 |
| [ONNX 链路](onnx-chain.md) | 把文本经 BGE-M3 ONNX 推理为向量 · RAG 真语义检索的核心 · task #87 决策梳理 |
| [GitHub Pages 部署](github-pages-deploy.md) | book.yml workflow + actions/deploy-pages · 把 mdBook 站自动推到 `<user>.github.io/<repo>/` |

## Agent 自我约束（硬性规则）

当用户问以下任一意图的问题时：

- 「X 是什么」「X 有什么用」「X 有什么功能」
- 「为什么用 X」「X 跟 Y 啥区别」
- 「X 怎么工作的」「X 的原理」

Agent **答完之后必须主动询问**：

> 「这个解答要不要归档到 `docs/concepts/<term>.md`？」

不能默认归 / 默认不归 · 必须问。用户确认后按下文「加新条目的步骤」5 步执行。

理由：

- 概念问答是高复用资产 · 不归档下次新读者还得重问
- 用户最清楚这个问题是否值得沉淀（有时只是临时确认 · 有时是核心知识）
- 强制询问 = 强制 agent 停下来评估「本轮对话是否产生了知识增量」

呼应 `AGENTS.md` 全局规则 #18。

## 加新条目的步骤

1. 用户问 agent「X 是什么」 · agent 答完
2. 用户说「归档」
3. Agent 做 5 件事：
   - 写 `docs/concepts/<term>.md`（4 节模板：一句话 / 业界用法 / 本项目里怎么用 / 类比）
   - 更新本文件「现有条目」表
   - `docs/GLOSSARY.md` 加一行 → 链到 `concepts/<term>.md`
   - `mdbook/src/reference/concepts/<term>.md` 一行 `{{#include ../../../../docs/concepts/<term>.md}}`
   - `mdbook/src/SUMMARY.md` 「概念解释」节加一个子项
4. 双轨归档 + commit
