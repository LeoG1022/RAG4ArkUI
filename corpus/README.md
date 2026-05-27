# corpus/ — RAG4ArkUI 知识语料根目录

按 [`docs/ADR-003-corpus-layout.md`](../docs/ADR-003-corpus-layout.md) 与技术方案 §4.5 分五类管理。**仓库默认只保留目录骨架**，实际文档由使用者投放（公共文档可考虑作为 git submodule，私有文档保留在本地）。

## 目录约定

| 子目录 | 内容 | 文档格式建议 |
|---|---|---|
| [`official/`](official/) | ArkUI-X / OpenHarmony 官方文档（API 参考、开发指南、规范） | `.md` 优先；HTML/PDF 需先转 markdown |
| [`samples/`](samples/) | 官方代码示例（`openharmony-sig/arkui-x/sample`）、精筛社区项目 | `.ets` / `.ts` 源文件 + 同目录 `README.md` |
| [`migration/`](migration/) | KMP / Android / iOS → ArkUI-X 迁移规则与样例 | `.md`（规则）+ `.ets` / `.kt` / `.swift`（pair 形式） |
| [`errors/`](errors/) | 错误↔修复 pair 库（XDB 回流） | YAML（结构化）或 markdown |
| [`custom/`](custom/) | 项目私有 / 团队内部规范 | 自由格式 |

## 元数据 schema（每个文档建议带 frontmatter）

```yaml
---
api_name: "router.pushUrl"               # 可选，API 类文档
platforms: ["HarmonyOS", "Android", "iOS"]
api_version: "ArkUI-X 1.2"
deprecated: false
type: "api_doc"                          # api_doc | code_example | migration_rule | error_fix
source_framework: null                   # 仅迁移类填：KMP | Android | iOS
complexity: "intermediate"               # basic | intermediate | advanced
tags: ["routing", "navigation"]
---

# 文档正文……
```

字段语义与 `arkui-rag-core::ChunkMetadata` 一致（详见技术方案 §4.2 决策 6）。

## 现在该做什么

1. **官方文档**：把 https://arkui-x.cn 的 markdown 镜像拖到 `official/`，或挂 git submodule
2. **代码样例**：`git clone https://gitee.com/openharmony-sig/arkui-x sample` 后把 `sample/` 拷到 `samples/`
3. **迁移规则**：参考 `.claude/references/mapping-*.md`，可直接搬到 `migration/` 起步
4. **错误库**：暂留空，等 Week 5-6 接 XDB 流水线后自动填充

## 索引

```bash
# Week 2 起可用
arkui-rag index --source corpus/
```

索引产物在 `corpus/_index/`（已在 `.gitignore`）。
