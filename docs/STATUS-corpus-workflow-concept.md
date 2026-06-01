# STATUS — corpus-workflow-concept

> 配套 feature log：`feedback/features/rag4arkui-core/48-2026-06-01-corpus-workflow-concept.md`
> 日期：2026-06-01

---

## 当前状态

Round 47 v1.0.0 上线后用户两连问触发 #18 第 3 次实战 + 提出新需求「整套分发给最终用户」。

本阶段交付：
- `docs/concepts/corpus-workflow.md` 200 行 · 4 节模板
- 5 步流程齐
- 「场景 C · maintainer 分发整套」节作为 Round 48+ roadmap 锚点

意义：把「release 包 vs corpus vs index」三者边界永久文档化 · 下个用户问「文档放哪 / index 怎么分发」不必重新解释。同时为「整套分发」新需求提供初步设计。

## 输入契约

无（纯概念归档）。

## 输出契约

### 概念文档「场景 C」描绘新功能

```
用户体验目标:
  arkui-rag init    # 一键完成
    → corpus pull           (~5MB)
    → model-pull bge-m3     (~2GB)
    → index-pull            (~50MB · 新加 · 未实装)
    → install-binary 配三端 MCP
    → 完成 · 直接 chat 调用
```

需要实装的 4 件事（Round 49-52）：
- 收集 ArkUI-X / OpenHarmony 官方文档放 `corpus/official/`
- 加 `arkui-rag index-pull` 命令
- maintainer CI 自动 re-build + 推 release
- 加 `arkui-rag init` wizard

详见 feature log Round 48 「新需求设计」节。

### 不变项

- 当前 v1.0.0 功能不动 · 所有现有 CLI 命令保留
- 本地优先模式不变（场景 A 私有文档全本地）

## 验证手段

### 用户手动

- 浏览器打开 mdBook 站「概念解释 → Corpus 工作流」看新加页是否渲染对（等 book.yml 跑完 2 分钟）
- 或本地 `make book-build && make book-serve`

### 自动化

无（纯文档归档 · 无代码改动）。

## 与上一阶段的关联性

| Round | 主题 | 关系 |
|---|---|---|
| 33 | AGENTS.md #18 规则建立 | 规则源头 |
| 40 | 归档 onnx-chain | #18 第 1 次 |
| 43 | 归档 github-pages-deploy | #18 第 2 次 |
| 47 | v1.0.0 release · CoreML 21× | 上轮 · 本轮触发问题源 |
| **48（本轮）** | 归档 corpus-workflow + 新需求设计 | **#18 第 3 次 + Round 49+ roadmap 启动** |

#18 规则 3 次实战 · 流程稳定。

兼容性：完全向后兼容（纯文档 · 无代码 / 配置改动）。

破坏性变更：无。

## 完成度 / 下一阶段

### 达成度

| 项 | 状态 |
|---|---|
| corpus-workflow.md 4 节模板 | ✅ |
| 5 步流程齐 | ✅ |
| 「场景 C」Round 48+ roadmap 锚点 | ✅ |
| feature log + STATUS（本轮）| ✅ |
| 用户浏览器看到新页 | ⏳（book.yml 跑完 2 分钟）|

### 下一阶段建议（用户决策启动哪个）

| 选项 | 工作量 | agent / 用户 | 时长 |
|---|---|---|---|
| **49 · 收集 ArkUI-X / OpenHarmony 官方文档** | 1-2 round · 含法务确认 | agent 推荐方案 · 用户拍板 | 1-3 天 |
| 50 · 加 `arkui-rag index-pull` 命令 | 1 round · 共用 corpus pull 代码 | agent 做 | 30 分钟 |
| 51 · maintainer CI 自动 re-build + 推 release | 1-2 round · 写 workflow | agent 做 · 用户 review | 1 round |
| 52 · 加 `arkui-rag init` wizard | 1 round · 串联现有命令 | agent 做 | 1 round |
| 53 · 终端用户视角文档 | 1 round | agent 做 | 1 round |

完整路径估算：**Round 49 → 50 → 51 → 52 → 53 共 5-7 round · ~1 周** · 完成「整套分发」目标。

中期：
- Round 50 之前可以临时方案：用户自己 build 本地 corpus + index · scp / rsync 给同事用（手动）· 验证分发价值
- Round 49 ArkUI-X 文档收集卡在法务 · 可先收集**自己的 demo corpus**（如 docs/concepts/* + .claude/references/mapping-*.md）验证流水线

长期：
- multi-corpus 支持（用户可同时索引多个版本 · query 时指定）
- 增量 update（changed 文档自动 reindex · 不必全 build）
- 版本绑定语义（index-v1.x 跟 binary v1.x 兼容性矩阵）
