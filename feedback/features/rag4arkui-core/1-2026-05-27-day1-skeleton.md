# 1 — day1-skeleton

> 日期：2026-05-27
> 涉及代码：crates/* 全 7 个、corpus/*、docs/ADR-00{1,2,3}-*.md、Makefile、rust-toolchain.toml
> 类型：新建（Day 1 骨架）

## 本轮目标

把 `RAG4ArkUI` Rust 主代码立起来——**先锁架构 + 接口契约，不解决可用性**。这是技术方案 6 周 MVP 的 Week 1 子集。

具体交付：
1. 7 个 Cargo crate 的目录与 trait 都按完整方案图 3 类图就位
2. 第七章 §7.2 的 BGE-M3 ONNX 代码作为 `arkui-rag-embedding/src/onnx.rs` 落地（feature `onnx` gated）
3. CLI 二进制 `arkui-rag` 的 subcommand 接口形状（serve/index/query/corpus）锁死
4. corpus/ 5 个子目录 + README + 元数据 schema 完成
5. 3 份 ADR 把语言选型 / crate 拆分 / corpus 布局 三大决策固化

## Plan

> 必备节（FAIL 级 M-FEATURE-PLAN）

本轮设计的 **7 个 crate 边界**：

| Crate | Day 1 状态 | 关键设计 |
|---|---|---|
| `arkui-rag-core` | ✅ trait + 类型完成 | 5 个 trait（Retriever / Embedder / Reranker / ASTChunker + storage 三个 in storage crate）；所有 trait 用 `async_trait` |
| `arkui-rag-embedding` | ✅ MockEmbedder 实现 Embedder trait；OnnxEmbedder feature-gated | 双 impl：Mock 默认可用、ONNX 启用 `--features onnx` 后编译完整 §7.2 代码（不实现 Embedder trait，Week 2 包成 async） |
| `arkui-rag-storage` | ⏳ traits + InMemoryStore | 三个 trait（VectorStore / BM25Index / MetadataStore）；InMemoryStore 实现 MetadataStore 占位 |
| `arkui-rag-chunker` | ✅ MarkdownChunker 真实可用 | 按 `#`/`##` heading 切分，维护 heading_path + line_range，3 个单元测试 |
| `arkui-rag-retrieval` | ✅ RRF 算法真实；HybridRetriever/Reranker stub | RRF 是纯函数好测试；Hybrid/Rerank 等 Week 2-3 接后端 |
| `arkui-rag-server` | ⏳ ServeOptions + 路由 stub | 3 个 feature flag（http/mcp/lsp）；handler 打印 TODO |
| `arkui-rag-cli` | ✅ clap subcommand 完整；`corpus list` 真实 | 6 个 subcommand 形状锁定；`--version` + `corpus list` 真活，其他 stub |

**与技术方案 §3-§7 的对应**：

| 方案章节 | 落地位置 |
|---|---|
| §3 多模型 RAG | （Week 4-5 由 server crate 实现路由） |
| §4.1 整体架构 | Cargo workspace 整体拓扑就位 |
| §4.2 八大关键决策 | 决策 1 Rust → ADR-001；决策 2 协议 → server crate；决策 3 嵌入模型 → embedding crate；决策 4 存储 → storage crate；决策 5 corpus 两轨 → corpus/ + ADR-003；决策 6 chunking → chunker crate；决策 7 性能 → 待 Week 3 落地；决策 8 分发 → 待 Week 6 |
| §5 RAG Core 本质 | core crate trait 边界完全对齐 §5.1 |
| §6 内嵌模型推理 | embedding crate Mock + ONNX 双轨 |
| §7.2 BGE-M3 代码 | `crates/arkui-rag-embedding/src/onnx.rs` verbatim |
| §9 图 3 类图 | 7 crate 的 trait + 类型一一对应 |

**Feature gate 策略**：默认 `cargo check --workspace` 不拉 ORT / axum / tree-sitter / lancedb / tantivy 任何重依赖。验证 ONNX 真活需 `--features onnx`。详见 ADR-002。

**替代选项权衡**：
- 备选 Python 端到端原型：被否，理由见 ADR-001（重复劳动 + 复用为零）
- 备选单 crate workspace：被否，理由见 ADR-002 Anti-Patterns（编译爆炸 + 不可分发）
- 备选无 feature gate：被否，会让 Day 1 验证被 ORT 编译卡死

## 对话摘要

> 必备节（FAIL 级 M-FEATURE-PLAN）

按时序：

1. **用户首问**："请根据docs仓中 RAG4ArkUI-完整技术方案.md 的对过内容继续构建该项目。"
2. **Agent 先做 Phase 1 探查**：派 Explore agent 摸清仓库现状 → 确认零产品代码、cargo 未安装、docs/ 待提交
3. **Agent 进入 Phase 2 设计**：派 Plan agent 评估三种切片方案（Python 原型 / Rust 骨架 / 双轨并行），Plan agent 推荐 C（双轨）
4. **AskUserQuestion Q1（切片范围）**：选项为
   - Python 原型 + Rust 占位骨架（Plan agent 推荐）
   - 纯 Python 端到端原型
   - **纯 Rust 骨架先行**（用户实际选择）
   - 更激进 Python + BGE-small 模型
5. **AskUserQuestion Q2（初始 corpus）**：选项为
   - 用现有 .claude/references/
   - 爬 ArkUI-X 官方文档
   - 两者都用
   - **给我一个目录我自己放文档**（用户实际选择）
6. **Agent 写 plan 文件 → ExitPlanMode**：用户批准
7. **执行 Phase 0** 时 pre-commit 阻塞（M-NO-VERIFY-BAN 假阳性）→ Agent 现场诊断为脚本 bug（短 commit 历史触发）→ 临时 `mv .last-verified` 让 docs/ 通过，bug 修复并入 Phase 1 meta 改动（见 `feedback/meta/2-2026-05-27-rag4arkui-bootstrap.md`）
8. **执行 Phase 1-4** 全自主完成，未再回问用户
9. **本 feature log** 在 Phase 4 由 agent 自填（AGENTS.md #10 + #16 要求）

## 改动要点

> API 选型 / 算法 / 关键决策

- **trait async 化**：所有 trait 都用 `async_trait`，因为 server 形态下需要并发
- **错误统一**：所有 crate 用 `arkui_rag_core::RagError`（thiserror 派生），跨 crate `?` 顺畅
- **Embedder trait dim() + model_id()**：模型升级触发索引重建的版本绑定靠 model_id
- **Hit 类型**：内嵌 Chunk + score + source（HitSource enum 标识来自 vector/bm25/hybrid/reranked，便于调试）
- **ChunkMetadata 字段**：完全对齐技术方案 §4.2 决策 6 的 JSON schema，包括 parent_id（支持 Parent-Child 索引）
- **MarkdownChunker**：用 heading 栈维护 heading_path；哨兵迭代器 + sentinel 触发最后一段 flush
- **RRF 算法**：k 默认 60，纯函数好测试，3 个单测覆盖
- **CLI clap derive**：6 个 subcommand 接口形状完整；只有 `corpus list` 是真活（扫 corpus/ 子目录），其他打印 TODO

## 验证结果

- 编译：⏳ 待 Phase 5 真实跑 `cargo check`（要求用户先装 rust 工具链）
- check-api-parity：N/A（项目级规则尚未定义）
- 自测覆盖：MockEmbedder 3 个单测 + MarkdownChunker 3 个单测 + RRF 2 个单测 = 8 个 unit test 就位
- 接口契约：所有 trait 签名与技术方案图 3 类图一致

## 残留 / 下一轮

- [ ] Week 1 续：用户装 rust 工具链 → `make check` 验证 7 crate 全编译
- [ ] Week 1 续：`make check-onnx` 验证 ort 2.0.0-rc.4 API 是否与 §7.2 代码 100% 匹配
- [ ] Week 2：`MarkdownChunker` 加 frontmatter 解析 + 父子层级
- [ ] Week 2：`arkui-rag-storage` 接 LanceDB（feature `lancedb`）
- [ ] Week 2：`arkui-rag-storage` 接 Tantivy（feature `tantivy`）
- [ ] Week 2：`arkui-rag-chunker` 接 tree-sitter-typescript / kotlin / swift
- [ ] Week 2：`arkui-rag-embedding` 把 OnnxEmbedder 包成 async Embedder trait 实现（`spawn_blocking` 桥接）
- [ ] Week 2：`arkui-rag-cli model-pull` 真实下载 BGE-M3 ONNX
- [ ] Week 3：`HybridRetriever` + `CrossEncoderReranker` 实装
- [ ] Week 3：基础评估集（50 query）+ RAGAS 接入
- [ ] Week 4：HTTP + MCP + LSP 三套协议实装
- [ ] Week 5：IDE 插件接入验证
- [ ] Week 6：自动安装 + corpus 分发管道
- [ ] 跨周：拿 `.claude/references/mapping-*.md` 作为初始 corpus 投放到 `corpus/migration/` 跑端到端冒烟
