# STATUS · 问答知识库（docs/concepts/）

> 日期：2026-05-29
> 对应 commit：[本 commit · concepts-kb]
> 对应 feature log：feedback/features/rag4arkui-core/32-2026-05-29-concepts-kb.md
> 对应 meta：feedback/meta/16-2026-05-29-concepts-dir.md
> 上一阶段：STATUS-readme-trim.md

> 🎯 新增 docs/concepts/ 子目录作为问答知识库 · 三层文档定位完整

## 当前状态

新增：
- `docs/concepts/` 4 文件（README + mdbook + tree-sitter + mvp）
- `mdbook/src/reference/concepts/` 4 include 文件
- mdBook SUMMARY「参考 > 概念解释」子节

修：
- `docs/GLOSSARY.md` 顶部加链接区引导用户去 concepts 看深度

## 三层定位

| 层 | 文件 | 内容长度 |
|---|---|---|
| 速查 | `docs/GLOSSARY.md` | 1-2 行 |
| 深度 | `docs/concepts/<term>.md` | 4 节模板 |
| 交互 | `feedback/{meta,features}/*` | 时序对话摘要 |

## 加新条目流程（5 件事）

1. 写 `docs/concepts/<term>.md`
2. 更新 `docs/concepts/README.md` 现有条目表
3. `docs/GLOSSARY.md` 加链接到 concepts
4. `mdbook/src/reference/concepts/<term>.md` 一行 `{{#include}}`
5. `mdbook/src/SUMMARY.md` 「概念解释」节加子项

## 验证

- ✅ docs/concepts/ 4 文件
- ✅ mdbook build clean
- ✅ GLOSSARY 顶部链接
- ✅ SUMMARY 加子节
