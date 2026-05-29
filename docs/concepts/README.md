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
