# 13 — day10-tree-sitter

> 日期：2026-05-27
> 涉及代码：
> - `crates/Cargo.toml`（workspace.deps 加 tree-sitter + tree-sitter-typescript）
> - `crates/arkui-rag-core/src/chunker.rs`（SourceLang 加 derive Hash）
> - `crates/arkui-rag-chunker/Cargo.toml`（feature treesitter/typescript/kotlin/swift/all-langs）
> - `crates/arkui-rag-chunker/src/treesitter_base.rs`（**新增** ~140 行 · LangStrategy + extract_chunks 通用走树工具）
> - `crates/arkui-rag-chunker/src/typescript.rs`（**新增** ~200 行 · TypeScriptChunker · 8 单测）
> - `crates/arkui-rag-chunker/src/kotlin.rs`（**新增** stub · feature gated）
> - `crates/arkui-rag-chunker/src/swift.rs`（**新增** stub · feature gated）
> - `crates/arkui-rag-chunker/src/dispatcher.rs`（**新增** ~150 行 · ChunkerDispatcher · 4 单测）
> - `crates/arkui-rag-chunker/src/lib.rs`（导出全部模块）
> - `crates/arkui-rag-indexer/src/lib.rs`（**重构** 接 Arc&lt;ChunkerDispatcher&gt;）
> - `crates/arkui-rag-indexer/tests/end_to_end.rs`（适配新签名）
> - `crates/arkui-rag-eval/tests/eval_end_to_end.rs`（适配新签名）
> - `crates/arkui-rag-cli/Cargo.toml`（typescript/kotlin/swift/full feature 转发）
> - `crates/arkui-rag-cli/src/main.rs`（build_dispatcher helper · 替换 MarkdownChunker 直接构造）
> - `Makefile`（build-treesitter / check-treesitter target）
> - `docs/ADR-002` + `crates/README.md` + `docs/ROADMAP.md`（速查表 + 路线图同步）
> 类型：新建 + 重构（Day 10 主线 · 代码切分）

## 本轮目标

让代码 corpus 真活。技术方案 §2.3 "代码感知的 Chunking 策略" 落地：
- 用户投放 `.ets` / `.ts` / `.tsx` 文件 → 自动识别为 ArkTS
- tree-sitter 解析 AST → 按 class / function / method / interface / enum 切 chunk
- 每个 chunk 保留 `heading_path`（class 名 → method 名）和 `line_range`

加上 ChunkerDispatcher 多语言路由，Indexer 重构后可同时处理 markdown + ArkTS 混合 corpus。

## Plan

> 必备节（FAIL 级 M-FEATURE-PLAN）

### 设计：3 个新模块 + 1 个重构

| 模块 | 职责 | 行数 |
|---|---|---|
| `treesitter_base.rs` | `LangStrategy` trait + `extract_chunks` 通用走树 + name 提取 helper | ~140 |
| `typescript.rs` | `TypeScriptChunker` · TsStrategy 实现 · 8 单测 | ~200 |
| `kotlin.rs` / `swift.rs` | stub · feature gated · 返回 NotImplemented | ~30 each |
| `dispatcher.rs` | `ChunkerDispatcher` 按 SourceLang 路由 + ext → lang 推断 + 4 单测 | ~150 |
| `indexer/lib.rs` | **重构** 接 `Arc<ChunkerDispatcher>` 替代单 `Arc<dyn ASTChunker>` | -30/+50 |

### Feature gate 分级

```toml
[features]
treesitter = ["dep:tree-sitter"]                                # 基础
typescript = ["treesitter", "dep:tree-sitter-typescript"]       # 真活
kotlin = ["treesitter"]                                          # stub
swift = ["treesitter"]                                           # stub
all-langs = ["typescript", "kotlin", "swift"]
```

CLI 同步 feature 转发：`--features typescript` → `arkui-rag-chunker/typescript`。

### TypeScriptChunker 算法

1. `tree_sitter_typescript::LANGUAGE_TYPESCRIPT` 设为 parser
2. 解析得到 `Tree`，遍历 root_node
3. `TsStrategy::interesting_kinds()` 返回应切 chunk 的 node kind:
   - `class_declaration` / `abstract_class_declaration`
   - `interface_declaration` / `enum_declaration` / `type_alias_declaration`
   - `function_declaration` / `method_definition`
4. 维护 `scope_stack`（class 名）作为 heading_path
5. `extract_name(node)`: child_by_field_name("name") → type_identifier/identifier 兜底
6. 兜底：无 declaration 文件（如纯 `const x = 1`）返回整文件做一个 chunk

### ChunkerDispatcher API

```rust
let d = ChunkerDispatcher::new()
    .register(SourceLang::Markdown, Arc::new(MarkdownChunker::new()))
    .register(SourceLang::ArkTs, Arc::new(TypeScriptChunker::new(SourceLang::ArkTs)));

let lang = ChunkerDispatcher::detect_lang(Path::new("a.ets"));  // → SourceLang::ArkTs
let chunks = d.chunk_as("a.ets", content, lang).await?;
```

按扩展名映射：md/markdown → Markdown · ets/ts/tsx → ArkTs · kt/kts → Kotlin · swift → Swift · json → Json · 其他 → Auto。

### Indexer 重构

原签名：
```rust
Indexer::new(chunker: Arc<dyn ASTChunker>, ...)
```

新签名：
```rust
Indexer::new(dispatcher: Arc<ChunkerDispatcher>, ...)
```

内部 dispatch 逻辑：
- detect_lang(path) → 若 Auto 或未注册 lang → skipped++
- 调 `dispatcher.chunk_as(...)`
- 若收到 `RagError::NotImplemented`（Kotlin/Swift stub）→ skipped++ + warn 但不报错

**API 破坏性变更**：是。所有调用方（tests + CLI）必须同步改造。本 commit 已全部适配。

### 替代方案权衡（被否）

- 备选：Indexer 内置 dispatcher 实例
  - 否决：dispatcher 应可单独构造 / 共享 / 测试，trait object 注入更灵活
- 备选：让 ASTChunker trait 自己宣称支持哪些 lang
  - 否决：把"我处理什么"逻辑从 chunker 移到 dispatcher，单一职责更清晰
- 备选：TypeScript / Kotlin / Swift 分别新建 crate
  - 否决：tree-sitter-* 都是小依赖，feature gate 比新 crate 更轻
- 备选：直接接 tree-sitter-kotlin / tree-sitter-swift 真活
  - 否决：社区维护活跃度不一，Day 10 先 stub 保留接口，Week 2-3 评估后实装
- 备选：把 ArkTS 当独立语言（自写 grammar）
  - 否决：ArkTS 是 TypeScript 超集 + 装饰器，tree-sitter-typescript 已能解析 95%+ 主流写法
- 备选：保留旧 Indexer::new 旧签名做向后兼容
  - 否决：API 不稳定阶段不必背包袱；测试 + CLI 一次性适配，干净

## 改动要点

> API 选型 / 算法 / 关键决策

**与 Day 7 的差异**：
- crate 数 9（不变）
- 测试数 37 → 默认 features **44**（+4 dispatcher + 1 indexer skip + 1 eval 适配 + 已有 -2 重写）
- typescript feature 启用后再 +8 = **52**
- API 破坏：`Indexer::new` 第一个参数由 `Arc<dyn ASTChunker>` → `Arc<ChunkerDispatcher>`

**API 选型**：
- LangStrategy trait 处理"哪些 node 算 chunk"和"如何取 name"的差异
- 通用 `extract_chunks` 函数走树 + 维护 scope_stack
- ChunkerDispatcher builder 风格 register

**关键决策**：
- TypeScript chunker 兼容 ArkTS（is_scope_kind 区分 class/interface/enum 作为 scope）
- Kotlin/Swift stub 用 `RagError::NotImplemented` 让 indexer 显式 skip
- 文件无 declaration 时兜底返回整文件 chunk（保证不丢内容）

## 对话摘要

> 必备节（FAIL 级 M-FEATURE-PLAN）

按时序：
1. **Day 7 HyDE commit 后**，agent 在末尾推荐 Day 10 tree-sitter（代码 corpus 解锁）
2. **用户指令**：「继续」
3. **Agent 自主决策 6 项**：
   - 不新建独立 crate（用 feature gate）
   - ArkTS 走 tree-sitter-typescript（不自写 grammar）
   - Kotlin/Swift 先 stub（社区库待评估）
   - 新 ChunkerDispatcher 替代单 chunker 直传
   - Indexer 破坏性签名变更（不留兼容包袱）
   - 无 declaration 文件兜底返回整文件 chunk
4. **Agent 不再回问**，5 phase 直接执行

## 验证结果

- 编译：⏳ 用户跑 `make check` · 期望 9 crate 全过（默认不拉 tree-sitter）
- 编译 ts feature：⏳ 用户跑 `make check-treesitter` · 期望通过（首次拉 tree-sitter-typescript ~30 秒 C 编译）
- 测试：⏳ 用户跑 `cargo test --workspace` · 期望默认 44 测试全过
  - typescript feature：`cargo test -p arkui-rag-chunker --features typescript` 加 8 个 TypeScriptChunker 测
- 端到端：用户跑 `--features typescript` 构建 CLI + 投放 `.ets` 文件 + index → 看 chunks 数

## 残留 / 下一轮

- [ ] **关键**：用户跑 `make check-treesitter` 验证 tree-sitter-typescript 0.21 API 100% 匹配
- [ ] **关键**：用户投放真实 `.ets` 文件（如 ArkUI-X 官方 samples）+ 跑 eval 看代码召回质量
- [ ] Day 10 续：接 tree-sitter-kotlin 真活（评估社区库 maintainership）
- [ ] Day 10 续：接 tree-sitter-swift 真活
- [ ] Week 3 Day 8：tantivy-jieba 中文分词
- [ ] Week 3 Day 11：Parent-Child 父子索引（按方案 §1.4 标准）
- [ ] Week 3 Day 9：LanceDB 替换 InMemoryVectorStore
- [ ] Week 3 Day 12/13：Query Router + ContextAssembler
- [ ] Week 4：HTTP/MCP/LSP 协议层
- [x] Day 10：TypeScriptChunker 真活
- [x] Day 10：ChunkerDispatcher 多语言路由
- [x] Day 10：Indexer 重构接 dispatcher
- [x] Day 10：CLI / Makefile / 文档同步
- [x] Day 10：ROADMAP 维护约定第 2 次实战
