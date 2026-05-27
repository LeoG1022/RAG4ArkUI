# ADR-003 · Corpus 目录布局：5 类知识源 + 元数据 schema

- **Status**：Accepted
- **Date**：2026-05-26
- **Deciders**：项目发起人通过 AskUserQuestion 明确"corpus 目录由用户自己投放文档"
- **Context Doc**：[`RAG4ArkUI-完整技术方案.md`](RAG4ArkUI-完整技术方案.md) §2.2 / §4.2 决策 5、6 / §4.5

## Context

知识库（corpus）的目录结构与元数据 schema 决定：
- 索引器能不能按类型走差异化切分（API 文档 vs 代码示例）
- 检索器能不能精准过滤（按 platform / version / type）
- 用户能不能自己往里投文档（不学新规范、不写脚本）

## Decision

按技术方案 §4.5 分 5 类目录：

```
corpus/
├── official/      # 官方文档
├── samples/       # 官方代码示例
├── migration/     # 跨平台迁移规则
├── errors/        # 错误↔修复 pair（XDB 回流）
├── custom/        # 项目私有
└── README.md      # 各子目录约定
```

**仓库默认只保留 .gitkeep 占位**——实际文档由使用者投放：
- 公共可考虑 git submodule（如官方文档镜像）
- 私有保留在本地（已通过 `.gitignore` 处理索引产物）

## 元数据 schema

每个 markdown 文档建议附 YAML frontmatter：

```yaml
---
api_name: "router.pushUrl"
platforms: ["HarmonyOS", "Android", "iOS"]
api_version: "ArkUI-X 1.2"
deprecated: false
type: "api_doc"                          # api_doc | code_example | migration_rule | error_fix
source_framework: null                   # 仅 migration 类填：KMP / Android / iOS
complexity: "intermediate"
tags: ["routing", "navigation"]
---
```

字段语义与 `arkui-rag-core::ChunkMetadata` 一一对应（详见 `crates/arkui-rag-core/src/chunk.rs`）。**单一事实源是 Rust 类型定义**，frontmatter 是它的序列化形态。

## 索引产物

```
corpus/_index/
├── vectors.lance/      # LanceDB 向量库
├── bm25/               # Tantivy BM25
├── graph.db            # SQLite API 关系图
└── meta.db             # SQLite 元数据 + 原文
```

`corpus/_index/` 已加入 `.gitignore`，不入 git。

## Consequences

**正向**：
- 用户接入流程极简：投文档 → `arkui-rag index --source corpus/`
- 类型过滤靠目录前缀就能粗筛，再叠 frontmatter 精筛
- 切分器可按目录差异化策略（official 用 markdown AST；samples 用 tree-sitter；errors 用 YAML 结构化）

**负向**：
- 没 frontmatter 的文档元数据缺失，召回质量下降——靠 enrich 阶段（Week 2 增加）补救
- 5 类目录硬约定可能不够灵活（如有人想加"教程"分类）——`custom/` 是兜底，长期可考虑扩展

## Compliance

- 子目录新增 / 重命名需更新本 ADR + `corpus/README.md` + 索引器路由表
- 元数据字段新增需先改 `ChunkMetadata`，再更新本文件示例
