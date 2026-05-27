# STATUS · Day 10 · tree-sitter 代码切分

> 日期：2026-05-27
> 对应 commit：[本 commit · Day 10 tree-sitter]
> 对应 feature log：[`feedback/features/rag4arkui-core/13-2026-05-27-day10-tree-sitter.md`](../feedback/features/rag4arkui-core/13-2026-05-27-day10-tree-sitter.md)
> 上一阶段：[`STATUS-day7-hyde.md`](STATUS-day7-hyde.md)
> 下一阶段：`STATUS-day9-lancedb.md` 或 `STATUS-day8-jieba.md` 或 `STATUS-day14-http.md`（按用户选择）

> 🎯 **里程碑**：**代码 corpus 真活**。用户投放 `.ets` / `.ts` 文件后 indexer 自动识别 + 切分。
> 方案 §2.3 "代码感知的 Chunking 策略" 落地。

---

## 当前状态

| 模块 | 变化 |
|---|---|
| `arkui-rag-chunker` | **新增** TypeScriptChunker（tree-sitter + ArkTS）· ChunkerDispatcher · Kotlin/Swift stub |
| `arkui-rag-indexer` | **重构** 接 `Arc<ChunkerDispatcher>` 替代单 chunker · 自动按扩展名路由 |
| `arkui-rag-core::chunker::SourceLang` | 加 `derive(Hash)` 用作 HashMap key |
| `arkui-rag-cli` | `build_dispatcher` helper · feature 转发 `typescript/kotlin/swift` · `full` 加入新 feature |
| `Makefile` | 加 `check-treesitter` / `build-treesitter` target |
| Cargo workspace | 加 `tree-sitter = "0.22"` + `tree-sitter-typescript = "0.21"` |

### 测试覆盖

| 测试组 | 数量 |
|---|---|
| ChunkerDispatcher 单测 | 4（detect/dispatch/missing/has） |
| Indexer 集成测（含 `.ets skipped when no ts chunker`） | 2（+1 新） |
| TypeScriptChunker 单测（typescript feature） | 8（class/function/interface/ArkTS Component/line_range/empty/lang reject/fallback） |
| **本轮新增小计（默认 features）** | **+5** |
| **本轮新增小计（typescript feature）** | **+8** |
| **累计（默认）** | 37 → **44**（约） |
| **累计（typescript feature 启用）** | **52** |

---

## 输入契约

### 用户视角

```bash
# 默认编译（不拉 tree-sitter，约 3 分钟）
make check

# 启用 ArkTS 代码切分（首次拉 tree-sitter-typescript ~30 秒 C 编译）
make check-treesitter
make build-treesitter

# 投放 .ets 文件 + 跑 index
cd corpus/samples
cat > MyComponent.ets <<'EOF'
@Component
struct MyComponent {
  @State count: number = 0;
  build() {
    Column() {
      Text(`count: ${this.count}`)
    }
  }
}
EOF

cd ../..
cargo run --features typescript -p arkui-rag-cli -- index --source corpus
# 输出 IndexStats files 含 .ets 文件 + chunks 数包含每个 method
```

### 文件路径 → SourceLang 自动检测

| 扩展名 | SourceLang |
|---|---|
| `.md` / `.markdown` | Markdown |
| `.ets` / `.ts` / `.tsx` | ArkTs（需 `--features typescript`） |
| `.kt` / `.kts` | Kotlin（stub · 启用后 NotImplemented） |
| `.swift` | Swift（stub） |
| `.json` | Json（无 chunker，跳过） |
| 其他 | Auto（跳过） |

未启用对应 feature 时，dispatcher 不注册该 lang → indexer skipped + warn。

### 库 API（trait + 类型）

```rust
// 新 trait（在 treesitter_base.rs 内部，不对外暴露）
trait LangStrategy {
    fn interesting_kinds(&self) -> &'static [&'static str];
    fn extract_name(&self, node: Node, source: &str) -> Option<String>;
    fn is_scope_kind(&self, kind: &str) -> bool;
}

// ChunkerDispatcher API
let d = ChunkerDispatcher::new()
    .register(SourceLang::Markdown, Arc::new(MarkdownChunker::new()))
    .register(SourceLang::ArkTs, Arc::new(TypeScriptChunker::new(SourceLang::ArkTs)));
let chunks = d.chunk_as("a.ets", content, SourceLang::ArkTs).await?;
let lang = ChunkerDispatcher::detect_lang(Path::new("a.ets"));  // SourceLang::ArkTs

// Indexer 新签名
Indexer::new(
    Arc::new(d),                  // ⚠ Day 10 起：ChunkerDispatcher（破坏性变更）
    embedder, vector, bm25,
)
```

---

## 输出契约

### TypeScriptChunker 输出

输入：
```typescript
@Component
struct ProductCard {
  @State count: number = 0;
  build() { Column() { Text(`count: ${this.count}`) } }
  private increment(): void { this.count += 1; }
}
```

输出（约 3 chunks · 取决于 tree-sitter-typescript 解析）：
- `ProductCard` 类整体（class_declaration）— heading_path = ["ProductCard"]
- `build()` method — heading_path = ["ProductCard", "build"]
- `increment()` method — heading_path = ["ProductCard", "increment"]

每 chunk 含 `line_range` + `metadata.r#type = CodeExample`。

### Indexer IndexStats（不变 schema）

```
✅ 索引完成
   embedder    : mock-384
   dim         : 384
   bm25        : memory
   files       : 12              ← 含 .md + .ets
   chunks      : 67               ← markdown 节 + ArkTS class/method
   skipped     : 3                ← .kt / .swift / 未知扩展
   elapsed_ms  : 145
   saved to    : corpus/_index/index.json
```

### Hit / Citation 字段（不变）

Chunk metadata 内 `r#type` 自动设为 `CodeExample`，下游 reranker / context assembler 可据此差异化处理。

---

## 验证手段

### 用户手动

```bash
# 1. 默认编译（不拉 tree-sitter）
make check
make test                                  # 默认 44 个测试

# 2. typescript feature
make check-treesitter                       # 首次 30-60 秒（C 编译）
cd crates && cargo test -p arkui-rag-chunker --features typescript
# 期望 typescript feature 测试组（8 个）全过

# 3. 端到端：建索引含 .ets
cd ..
cargo run --features typescript -p arkui-rag-cli -- index --source corpus
# stats.files 应包含 .ets 文件计数

# 4. 端到端：跑 query 命中 .ets 内的 method
cargo run --features typescript -p arkui-rag-cli -- \
    query --text "Refresh 组件" --k 5
# 期望 Top-K 含 .ets 文件中含 Refresh 的 chunk
```

### 自动化

| 手段 | 范围 | 状态 |
|---|---|---|
| `ChunkerDispatcher` 单测 | detect_lang / dispatch / missing-lang-err / has / supported_langs | ✅ 4 个 |
| `TypeScriptChunker` 单测 | class/function/interface/enum/ArkTS Component/line_range/empty/lang reject/fallback | ✅ 8 个（feature gated） |
| `Indexer` 集成测 `ets_files_skipped_when_no_ts_chunker_registered` | dispatcher 无注册 .ets 时跳过逻辑 | ✅ |
| **M-STATUS-PER-ROUND** | Round 13 + STATUS-day10 配套 | ✅ |
| **ROADMAP 维护约定（第 2 次实战）** | 同 commit 同步 6 处进度行 | ✅ |

### 暂未自动化（明确缺口）

- ❌ tree-sitter-kotlin / tree-sitter-swift 真活（feature stub）
- ❌ .ets 真实 corpus + HyDE 效果对比（需用户投放官方 samples）
- ❌ TypeScript 全部 node kind 覆盖（如 const_declaration / module_declaration 当前会被 fallback 兜底）
- ❌ 代码 chunk 的 `extra` 字段填 AST 元数据（imports / decorators / 等）

---

## 与上一阶段（STATUS-day7-hyde）的关联性

### 增量

| 维度 | Day 7 (HyDE) 完成时 | 本轮（Day 10）后 |
|---|---|---|
| Crate 数 | 9 | 不变 |
| 关键 trait | + QueryEnhancer | + 内部 LangStrategy + 公开 ChunkerDispatcher API |
| corpus 文档类型支持 | 仅 markdown | + **ArkTS / TypeScript** |
| 测试数（默认） | 37 | **44** |
| Indexer API | `new(chunker, ...)` | **`new(dispatcher, ...)`**（破坏性变更） |
| CLI feature 数 | 3（onnx/tantivy/full） | **6**（+ typescript/kotlin/swift） |

### 破坏性变更（API）

⚠ **Indexer::new 第一个参数变了**：
```rust
// 旧（Day 2~7）
Indexer::new(chunker: Arc<dyn ASTChunker>, ...)

// 新（Day 10+）
Indexer::new(dispatcher: Arc<ChunkerDispatcher>, ...)
```

所有调用方（CLI / 内部 tests / 外部用户）必须同步改造。本 commit 已适配：
- `crates/arkui-rag-cli/src/main.rs::cmd_index`
- `crates/arkui-rag-indexer/src/lib.rs` 内 2 个 unit test
- `crates/arkui-rag-indexer/tests/end_to_end.rs` 2 处
- `crates/arkui-rag-eval/tests/eval_end_to_end.rs` 1 处

### 兼容性

- ✅ MarkdownChunker / HybridRetriever / Evaluator / HyDE 全部不动
- ✅ `--features typescript` 默认关闭 → 不影响已编译的二进制
- ✅ ROADMAP 维护约定按惯例 piggyback

### 与 ROADMAP 维护约定的协同（第 2 次实战）

本 commit 同步更新 ROADMAP 7 处：
- 当前位置 段：Day 7 → Day 10
- mermaid gantt：d10 active + d7 done
- 已完成表追加 Round 13 行
- 剩余切片表 Day 10 划掉
- 6 周达成度：Week 1 6/7 → 6.5/7
- 完成度：~40% → ~45%
- 里程碑预测 + 关键路径同步

→ **维护约定第 2 次实战通过**，惯例稳固。3-5 次后评估是否升级为强制规则。

---

## 完成度 / 下一阶段

### Day 10 完成度

| 项 | 状态 |
|---|---|
| TypeScriptChunker（ArkTS + TS） | ✅ |
| ChunkerDispatcher 路由 | ✅ |
| Indexer 重构 | ✅ |
| Kotlin / Swift stub（feature gated） | ✅ |
| CLI feature 转发 + Makefile | ✅ |
| 文档（ADR-002 / crates README / ROADMAP / STATUS） | ✅ |
| Kotlin / Swift 真活 | ⏳ Day 10 续 |

### 6 周路线图达成度更新

| 章节 | 状态 |
|---|---|
| Week 1: Rust 骨架 + tree-sitter + LanceDB + Tantivy + BGE-M3 | **6.5/7** ✅ (tree-sitter ArkTS ✓ · Kotlin/Swift stub · LanceDB ⏳) |
| Week 2: 混合检索 + Reranker + HyDE + 评估集 | **4/4** ✅ |
| Week 3: HTTP + MCP + CLI | **1/3** ✅ |

### 下一阶段建议（按推荐优先级）

| 候选 | 价值 | 工作量 |
|---|---|---|
| 🟢 **Day 9 LanceDB**（推荐） | 解锁 chunks > 10k；Week 1 最后一个缺口 | 1-2 commit |
| 🟢 Day 11 Parent-Child | 检索小返回大（方案 §1.4 标准）| 1 commit |
| 🟡 Day 8 tantivy-jieba | 中文 BM25 精度 | 0.5 commit |
| 🟡 Day 10 续 · tree-sitter-kotlin/swift | 跨平台迁移 corpus 真活 | 1-2 commit |
| 🟢 Day 14 HTTP Server | 协议层入门 · IDE 接入前置 | 2-3 commit |

**Agent 推荐**：**Day 9 LanceDB**。理由：
1. Week 1 最后一个缺口（除 Kotlin/Swift stub）
2. 当前 InMemoryVectorStore 卡在 ~10k chunks；真实代码 corpus 投放后立即触顶
3. 方案 §4.2 决策 4 明确指定 LanceDB
4. 工作量约 1-2 commit，与 Day 4 类似
5. 完成后可处理真实规模 corpus，HyDE / 评估 / 代码切分 全功能

### 重要的"非完成"项

- ❌ tree-sitter-kotlin / tree-sitter-swift 实装（社区库 maintenance 待评估）
- ❌ .ets 真实评估（需用户投放官方 samples + 校准 corpus/_eval/queries.yaml）
- ❌ ArkTS 装饰器作为独立 chunk metadata（如 `@Component` 标记到 extra）
- ❌ 跨文件依赖图谱（方案 §4.2 决策 4 "Knowledge Graph (SQLite + JSON)"，Day 25+ 长期）
