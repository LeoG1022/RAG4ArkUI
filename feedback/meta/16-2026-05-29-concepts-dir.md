# 16 — concepts-dir

> 日期：2026-05-29
> 触发：用户「请将解答归入知识库 推荐新创建一个目录归档问答知识库」
> 类型：工程结构调整（新增 docs/concepts/ 子目录 · 不引入顶层新目录）

---

## 用户提出的要求

「将一些问答类内容精简并归档到知识库 · 推荐新创建一个目录归档问答知识库。docs 用于记录架构设计和工程本身的迭代设计等。feedback 中只做迭代的 agent 和用户的原始交互归档。你觉得这样有必要吗，还是有更好的推荐」

## Agent 给出的修改建议

肯定必要性（问答类概念确实没归宿）· 但反对建顶层目录 · 建议 **docs/concepts/ 子目录**：

| 维度 | 顶层新目录 | docs/concepts/ |
|---|---|---|
| 心智负担 | 又多一个一级分类 | 在已有「docs 是各种说明」语义内 |
| mdBook 接入 | 需新建 src/knowledge/ section | 直接放 src/reference/concepts/ |
| 与 GLOSSARY 关系 | 跨目录联动 | docs/ 内部联动更自然 |

业界类比：Rust 标准库 `std::collections::concepts` / Kubernetes 文档 `concepts/` 都是 `<docs>/concepts/`。

三层文档定位（最终架构）：
- 速查 GLOSSARY（一行）
- 深度 concepts/`<term>`.md（4 节）
- 交互归档 feedback/（不变）

加新条目 5 件事流程：写 concepts md + 更 concepts/README + GLOSSARY 链 + mdbook include + SUMMARY 子项。

## 多轮互动

按时序：
1. 用户提议 + 问意见
2. Agent 给细化方案（顶层 vs 子目录权衡 + 三层定位 + GLOSSARY 边界 + 流程）
3. 用户「确认 按推荐进行」
4. Agent 写 4 文件归档

## 实际改动

- **接口变化**：无
- **规则变化**：无（流程是文档约定 · 不入 hook）
- **文件变化**：
  - 新增：`docs/concepts/` 4 文件（README + mdbook + tree-sitter + mvp）
  - 新增：`mdbook/src/reference/concepts/` 4 个 include
  - 修：`mdbook/src/SUMMARY.md`（参考节加「概念解释」子节 + 3 子项）
  - 修：`docs/GLOSSARY.md`（顶部加 concepts 链接区）
- **配置变化**：无

## 执行生效后总结

### 实际产出

| 项 | 状态 |
|---|---|
| docs/concepts/ 4 文件 | ✅ |
| concepts/README.md 边界 + 流程说明 | ✅ |
| GLOSSARY 链入 | ✅ |
| mdbook SUMMARY 加「概念解释」子节 | ✅ |
| mdbook build clean | ✅ |

### 前后对比

| 维度 | 之前 | 之后 |
|---|---|---|
| 概念解释归宿 | ❌ 散落对话历史 | ✅ docs/concepts/`<term>`.md |
| GLOSSARY 定位 | 术语对照 | 术语对照 + 链到 concepts 深度 |
| 文档矩阵 | 4 类（ADR/ROADMAP/STATUS/技术方案）+ feedback | **5 类**（+ concepts）|

### 实测验证

```
$ ls docs/concepts/
README.md  mdbook.md  mvp.md  tree-sitter.md

$ make book-build
✅ HTML book written
```

### 残留 / 下一轮处理

- [ ] 未来加新概念走 5 件事流程
- [ ] 考虑做 skill `/archive-concept <term>` 自动化（暂不 · 朴素流程先跑）
- [x] docs/concepts/ 建好
- [x] 3 个初始条目（mdBook / tree-sitter / MVP）
- [x] 完整定位 + 流程 + GLOSSARY 链接 + mdbook 接入
