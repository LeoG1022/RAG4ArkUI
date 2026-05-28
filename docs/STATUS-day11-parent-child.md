# STATUS · Day 11 · Parent-Child 父子索引

> 日期：2026-05-28
> 对应 commit：[本 commit · Day 11 Parent-Child]
> 对应 feature log：[`feedback/features/rag4arkui-core/15-2026-05-28-day11-parent-child.md`](../feedback/features/rag4arkui-core/15-2026-05-28-day11-parent-child.md)
> 上一阶段：[`STATUS-day9-lancedb.md`](STATUS-day9-lancedb.md)
> 下一阶段：`STATUS-day8-jieba.md` 或 `STATUS-day14-http.md`（按用户选择）

> 🎯 **里程碑**：方案 §1.4 "Parent-Child 索引" 标准就位 —— 检索小（精准），返回大（上下文完整）。

---

## 当前状态

| 模块 | 变化 |
|---|---|
| `arkui-rag-chunker/src/markdown.rs` | heading_stack 升级为 `(level, title, Option<ChunkId>)` · 自动填 parent_id |
| `arkui-rag-chunker/src/treesitter_base.rs` | scope_stack 升级为 `(String, Option<ChunkId>)` · TypeScript 自动受益 |
| `arkui-rag-retrieval/src/context.rs` | **新增** ContextAssembler + ExpandedHit + 5 单测 |
| `arkui-rag-retrieval/src/lib.rs` | 导出 ContextAssembler / ExpandedHit |
| `arkui-rag-cli/src/main.rs` | Query 加 `--expand-parent` flag · 输出 `↳ parent` 行 |
| `docs/ROADMAP.md` | 7 处进度行同步（维护约定第 4 次实战） |

### 测试覆盖

| 测试组 | 数量 |
|---|---|
| ContextAssembler 单测（context.rs） | 5（finds_parent / no_parent_id / parent_not_in_store / flatten / strict_fails） |
| MarkdownChunker / TypeScriptChunker 旧测（不破） | 6 + 8（不变） |
| **本轮新增小计（默认 features）** | **+5** |
| **累计默认** | 44 → **49** |

---

## 输入契约

### Chunker 行为（自动）

无需用户操作。所有 Chunker 生成 chunk 时自动填 `metadata.parent_id`：
- **MarkdownChunker**：H2 chunk 的 parent_id = 同 path 的 H1 chunk id；H3 = H2；以此类推
- **TypeScriptChunker**：method chunk 的 parent_id = 所在 class chunk id；class 没父
- 顶层 chunk（H1 / 文件级 class）`parent_id = None`

### CLI 新参数

```bash
arkui-rag query --text "..." --k 5 --expand-parent
```

- 默认 `false`（向后兼容 · 输出与之前完全一致）
- 启用后每个 hit 后跟 `↳ parent` 行（含父级 heading + line_range + 200 字符预览）

### 库 API

```rust
use arkui_rag_retrieval::{ContextAssembler, ExpandedHit};
use arkui_rag_storage::MetadataStore;
use std::sync::Arc;

# tokio_test::block_on(async {
let assembler = ContextAssembler::new(store: Arc<dyn MetadataStore>);

// 不严格：缺父返回 None
let expanded: Vec<ExpandedHit> = assembler.expand_to_parent(hits).await?;

// 严格：缺父报错
let expanded = assembler.expand_strict(hits).await?;

// 压平：保 chunk_id 但 content 用父
let hits_with_parent_content = ContextAssembler::flatten_with_parent_content(expanded);
# Ok::<_, arkui_rag_core::RagError>(())
# });
```

---

## 输出契约

### `ChunkMetadata.parent_id` 字段

```json
{
  "id": "router.md#Router/pushUrl@9",
  "content": "推送新页面到路由栈。",
  "metadata": {
    "parent_id": "router.md#Router@1",   ← Day 11 起自动填充
    "heading_path": ["Router", "pushUrl"],
    ...
  }
}
```

向后兼容：默认 `None`；旧 corpus 无 parent_id 字段时反序列化为 None。

### CLI 输出（--expand-parent）

```
✅ Top-3 hits (embedder=mock-384 · bm25=memory · rerank=none · hyde=none · expand-parent=on)

─── [1] score=0.0163 ──────────────────
  source : router.md L9-11
  heading: Router > pushUrl
  preview: 推送新页面到路由栈。
  ↳ parent (Router L1-15): # Router\n\n## pushUrl\n推送新页面...   ← Day 11 新增行

─── [2] score=0.0156 ──────────────────
  source : list.md L11-15
  heading: List > 下拉刷新
  preview: ArkUI-X 用 Refresh 组件...
  ↳ parent (List L1-30): # List\n\n## 基本组件\n\n## 下拉刷新...
```

### `ExpandedHit` 序列化（JSON）

```json
{
  "original": { "chunk": {...}, "score": 0.85, "source": "Hybrid" },
  "parent": { "id": "...", "content": "...", "metadata": {...} }  // 或 null
}
```

---

## 验证手段

### 用户手动

```bash
# 1. 编译验证
make check
make test                          # 默认 49 个测试（+5 ContextAssembler）

# 2. ContextAssembler 单测
cd crates && cargo test -p arkui-rag-retrieval context

# 3. 端到端：建索引 + 用 --expand-parent 查询
cd ..
cargo run -p arkui-rag-cli -- index --source corpus
cargo run -p arkui-rag-cli -- query --text "下拉刷新" --k 3 --expand-parent
# 应看到每个 hit 后的 ↳ parent 行

# 4. ArkTS 代码 + parent-child（需 typescript feature）
cargo run --features typescript -p arkui-rag-cli -- \
    index --source corpus
cargo run --features typescript -p arkui-rag-cli -- \
    query --text "@Component" --k 5 --expand-parent
# method chunk 的 ↳ parent 应是它所在的 class chunk
```

### 自动化

| 手段 | 范围 | 状态 |
|---|---|---|
| ContextAssembler 单测 × 5 | 找父 / 无 parent_id / 父不在 store / flatten / strict-fail | ✅ |
| MarkdownChunker 旧测 × 6 | 不破（parent_id 字段填充对老断言无影响） | ✅ |
| TypeScriptChunker 旧测 × 8 | 同上（typescript feature gated） | ✅ |
| **M-STATUS-PER-ROUND** | Round 15 + STATUS-day11 配套 | ✅ |
| **ROADMAP 维护约定（第 4 次实战）** | 7 处进度行同步 | ✅ |

### 暂未自动化（明确缺口）

- ❌ 评估器 `--expand-parent` flag（Day 11 续）
- ❌ Parent cover rate 指标（多少 hit 实际有父）
- ❌ Eval 报告内嵌"with vs without parent"对比
- ❌ MarkdownChunker H1 无 body case 的父级 placeholder（极端 case）
- ❌ Multi-level parent 链（A 的父是 B，B 的父是 C —— 当前只展开 1 层）

---

## 与上一阶段（STATUS-day9-lancedb）的关联性

### 增量

| 维度 | Day 9 (LanceDB) 完成时 | 本轮（Day 11）后 |
|---|---|---|
| Crate 数 | 9 | 不变 |
| Chunker 行为 | 不填 parent_id | **自动填 parent_id** |
| Retrieval 模块 | hybrid / hyde / rerank / rrf | + **context (ContextAssembler)** |
| CLI subcommand 参数 | `--vector / --bm25 / --rerank / --hyde` | + **`--expand-parent`** |
| 测试数（默认） | 44 | **49** |

### 兼容性

- ✅ 无破坏性变更
- ✅ ChunkMetadata.parent_id 字段已 Day 1 预留
- ✅ 旧 corpus / 旧 index.json 无 parent_id 字段 → 反序列化为 None → 行为与 Day 10 完全相同
- ✅ CLI `--expand-parent` 默认 false → 老命令完全不变
- ✅ ContextAssembler 是新增公开 API，不影响 trait `Retriever`

### 业界基线对照（再次更新）

| 业界共识 | 状态 |
|---|---|
| §1.6 第 1 条 Hybrid 检索 | ✅ Day 4 |
| §8.5 共识 2 Reranker 分水岭 | ✅ Day 5 |
| §8.5 共识 3 引用溯源 | ✅ Day 2 起 |
| §8.5 共识 4 Eval-Driven | ✅ Day 6 |
| **§1.4 Parent-Child 索引（生产标配）** | **✅ Day 11（本轮）** |

---

## 完成度 / 下一阶段

### Day 11 完成度

| 项 | 状态 |
|---|---|
| MarkdownChunker parent_id 生成 | ✅ |
| TypeScriptChunker parent_id 生成（共用 treesitter_base） | ✅ |
| ContextAssembler + ExpandedHit + 3 公开方法 | ✅ |
| 5 单测覆盖（含自指 / 链断 / 严格模式） | ✅ |
| CLI --expand-parent flag + 输出 ↳ parent 行 | ✅ |
| ROADMAP 维护约定第 4 次实战 | ✅ |
| Evaluator 集成 | ⏳ Day 11 续 |
| Multi-level parent 链 | ⏳ Day 11 续 |

### 6 周路线图达成度更新

| 章节 | 状态 |
|---|---|
| Week 1: Rust 骨架 + tree-sitter + LanceDB + Tantivy + BGE-M3 | **7/7** ✅ |
| Week 2: 混合检索 + Reranker + HyDE + 评估集 | **4/4** ✅ |
| **Week 2 末新增能力**：Parent-Child 标准 | ✅ Day 11 |
| Week 3: HTTP + MCP + CLI | **1/3** ✅ |

**总完成度估算：~55%**

### 下一阶段建议（按推荐优先级）

| 候选 | 价值 | 工作量 |
|---|---|---|
| 🟢 **Day 14 HTTP/REST Server**（推荐） | 关键路径 · IDE 接入前置 · 让外部能接入当前所有能力 | 2-3 commit |
| 🟢 Day 12 Query Router + Intent | 不同 query 走不同流水线（方案 §1.2） | 1 commit |
| 🟢 Day 13 ContextAssembler 全功能 | Multi-level parent 链 + Citation 元数据 | 1 commit |
| 🟡 Day 8 tantivy-jieba | 中文 BM25 精度 | 0.5 commit |
| 🟡 Day 11 续 | Evaluator --expand-parent + cover rate 指标 | 0.5 commit |

**Agent 推荐**：**Day 14 HTTP/REST Server**。理由：
1. 关键路径起点 —— 让 9 crate 的所有能力都能被外部消费（IDE 插件、Agent、curl）
2. Week 2 末 + Week 1 全部完成后，检索能力已成熟稳定，是开放协议层的合适时机
3. 工作量 2-3 commit，但解锁后续 IDE 插件 + Claude Code 接入
4. 业界标准 axum 框架，Rust 生态首选

### 重要的"非完成"项

- ❌ MarkdownChunker H1 无 body 的极端 case（H2 chunk 找不到 H1 父级）
- ❌ Multi-level parent 链（当前 expand 只展 1 层）
- ❌ Evaluator 内置 expand_parent 对比
- ❌ Citation 加 `parent_path` 字段（heading_path 链）
- ❌ HyDE 改写时考虑 parent context（让假代码更贴近父级风格）
