# 15 — day11-parent-child

> 日期：2026-05-28
> 涉及代码：
> - `crates/arkui-rag-chunker/src/markdown.rs`（heading_stack 提升为 `(level, title, Option<ChunkId>)` · 父子链生成）
> - `crates/arkui-rag-chunker/src/treesitter_base.rs`（scope_stack 提升为 `(String, Option<ChunkId>)` · parent_id 传递）
> - `crates/arkui-rag-retrieval/src/context.rs`（**新增** ContextAssembler + ExpandedHit · 5 单测）
> - `crates/arkui-rag-retrieval/src/lib.rs`（导出）
> - `crates/arkui-rag-retrieval/Cargo.toml`（dev-deps 加 tokio macros）
> - `crates/arkui-rag-cli/src/main.rs`（Query 加 --expand-parent flag · 输出带 ↳ parent 行）
> - `docs/ROADMAP.md`（7 处进度行同步）
> 类型：新建 + 增强（Day 11 主线 · Parent-Child 标准）

## 本轮目标

落地方案 §1.4 标准：「检索粒度小（精准命中），生成粒度大（上下文完整）」。
- Chunker 端：生成时填 `parent_id`，让小 chunk 指向大父级
- Retrieval 端：新 `ContextAssembler` 用 MetadataStore 扩展到父
- CLI 端：`--expand-parent` flag 让 query 输出展示父级 context

## Plan

> 必备节（FAIL 级 M-FEATURE-PLAN）

### 设计：分 3 层落地

| 层 | 改动 | 关键决策 |
|---|---|---|
| **Chunker** | MarkdownChunker + TypeScript（共享 treesitter_base） | scope_stack 升级为 `(name, Option<ChunkId>)` · 生成 chunk 时写回栈顶 id |
| **Retrieval** | 新 `ContextAssembler` + `ExpandedHit` | 注入 `Arc<dyn MetadataStore>` · expand_to_parent 返回 Vec&lt;ExpandedHit&gt; · 不强制全部有父（容错） |
| **CLI** | Query 加 `--expand-parent` flag | InMemoryVectorStore + LanceVectorStore 都同时实现 MetadataStore → 直接复用 |

### Markdown 父子链算法

栈结构：`Vec<(level, title, Option<ChunkId>)>`

```
# Top           → stack: [(1, "Top", None)]              cur_path=[Top]
intro           
## A            → flush "intro" → parent = stack[0].1 = None; gen id_top; stack[0].1 = id_top
                → push (2, "A", None)                    cur_path=[Top, A]
body a
## B            → flush "body a" → parent = stack[0].1 = id_top; gen id_a; stack[1].1 = id_a
                → pop "A" (lv 2 >= 2); push (2, "B", None)  cur_path=[Top, B]
body b
EOF             → flush "body b" → parent = stack[0].1 = id_top
```

→ chunk_top.parent=None · chunk_a.parent=id_top · chunk_b.parent=id_top ✓

### TypeScript 父子链算法（共用 treesitter_base.walk）

scope_stack 从 `Vec<String>` 升级为 `Vec<(String, Option<ChunkId>)>`：
- `walk` 进入 scope 节点 → push (name, None)
- emit chunk 时 → parent = scope_stack[len-2].1（栈顶是"我"，父是 len-2）
- 写回 scope_stack[len-1].1 = chunk_id
- 退出 scope → pop

例：`class C { build() {} }`
- 进入 class C → push ("C", None)
- emit C 的 chunk → parent = None (stack len=1) → 写回 stack[0].1 = id_C
- 递归进入 build() → push ("build", None)
- emit build chunk → parent = stack[0].1 = id_C ✓ → 写回 stack[1].1 = id_build
- pop build / pop C

### ContextAssembler API

```rust
pub struct ExpandedHit {
    pub original: Hit,
    pub parent: Option<Chunk>,
}

impl ContextAssembler {
    pub fn new(store: Arc<dyn MetadataStore>) -> Self;
    pub async fn expand_to_parent(hits) -> Vec<ExpandedHit>;
    pub async fn expand_strict(hits) -> Vec<ExpandedHit>;   // 缺父报错
    pub fn flatten_with_parent_content(expanded) -> Vec<Hit>;  // 保 id 用父 content
}
```

设计要点：
- 自指防护：`parent_id == hit.chunk.id` 时跳过（避免无限循环）
- 失败兜底：parent_id 链到不存在的 chunk → parent = None
- 三种语义：expand_to_parent / expand_strict / flatten 满足不同消费者

### CLI 输出（--expand-parent）

```
─── [1] score=0.85 ──────────────────
  source : list.md L9-11
  heading: List > 下拉刷新
  preview: ArkUI-X 用 Refresh 组件...
  ↳ parent (List L1-20): # List\n\nArkUI-X 列表组件...
```

未启用时输出格式与之前完全一致（向后兼容）。

### 替代方案权衡

- 备选：在 trait `Retriever::retrieve` 内部自动展开
  - 否决：违反单一职责；retriever 不该耦合 metadata_store
- 备选：父链直接序列化到 chunk content（hyde 风格）
  - 否决：违反 Day 1 设计；parent_id 已是 ChunkMetadata 字段
- 备选：Indexer 阶段就生成 expanded chunks 存两份
  - 否决：存储空间翻倍 + 索引时无需要决定展开策略；retrieval 阶段决策更灵活
- 备选：让 ContextAssembler 直接修改 Hit.chunk
  - 否决：丢失原 small-chunk 元数据（heading_path / line_range 等）；ExpandedHit 保留两层

### 自指 / 链断容错

| 场景 | 行为 |
|---|---|
| `chunk.parent_id == None` | `parent = None`（最常见，顶层 chunk） |
| `chunk.parent_id == Some(self.id)` | `parent = None`（防自指 · 显式检查） |
| `chunk.parent_id == Some(other.id)`，但 store 中没有 | `parent = None`（容错 · 不报错） |
| `chunk.parent_id == Some(other.id)`，store 中有 | `parent = Some(...)` ✓ |
| 调用 expand_strict 而有 hit 无父 | 报错 RagError::Retrieval |

## 改动要点

> API 选型 / 算法 / 关键决策

**与 Day 9 的差异**：
- crate 数 9（不变）
- 测试数 默认 44 → 默认 49（+5 ContextAssembler · 含自指 / orphan / flatten / strict）
- API 加：ContextAssembler + ExpandedHit（公开）
- Chunker 行为变更（parent_id 字段被填充） · 现有测试不破坏（旧测试不检 parent_id 默认 None）
- 接口破坏性：**无**（scope_stack 是内部私有类型）

**关键决策**：
- 防自指（`parent_id == self.id` 时返回 None）→ 早 catch 链问题
- 三种 API（expand/strict/flatten）满足不同消费者
- CLI 默认不展开（向后兼容） · 显式 `--expand-parent` 启用

## 对话摘要

> 必备节（FAIL 级 M-FEATURE-PLAN）

按时序：
1. **Day 9 LanceDB commit 后**，agent 推荐 Day 11 Parent-Child（评估集可立即量化）
2. **用户指令**：「继续」
3. **Agent 自主决策 5 项**：
   - parent_id 在 chunker 阶段生成（不延迟到 indexer）
   - scope_stack 升级为 `(name, Option<ChunkId>)`（共用 treesitter_base）
   - ContextAssembler 注入 MetadataStore（不直接接 trait `Retriever`）
   - CLI 默认不展开（`--expand-parent` 显式启用）
   - 5 单测覆盖自指 + 链断 + 严格模式
4. **Agent 不再回问**，5 phase 直接执行至本 commit

## 验证结果

- 编译：⏳ 用户跑 `make check` · 期望 9 crate 全过
- 测试：⏳ `cargo test --workspace`
  - 默认：44 → **49**（+5 ContextAssembler · 不破 MarkdownChunker/TypeScriptChunker 旧测）
  - typescript feature：+0（parent_id 字段填充对老测试断言无影响）
- 端到端：用户跑 `cargo run -p arkui-rag-cli -- query --text "..." --expand-parent` 应看到 ↳ parent 行

## 残留 / 下一轮

- [ ] **关键**：用户校准 corpus/_eval/queries.yaml 的 chunk_id（含父子 id 链）后跑 eval 对比 `with` vs `without` parent 的 recall@k
- [ ] Day 11 续：Evaluator 加 `--expand-parent` flag 量化效果
- [ ] Day 11 续：parent-id 链作为可选指标（cover rate 等）写入报告
- [ ] Week 3 Day 8：tantivy-jieba 中文分词
- [ ] Week 3 Day 12：Query Router + Intent
- [ ] Week 3 Day 13：ContextAssembler 全功能（多层父链 · parent_path 元数据）
- [ ] Week 4：HTTP/MCP/LSP 协议层
- [x] Day 11：Chunker 端 parent_id 生成（Markdown + TypeScript）
- [x] Day 11：ContextAssembler trait 设计 + 实装 + 5 单测
- [x] Day 11：CLI --expand-parent 接入
- [x] Day 11：ROADMAP 维护约定第 4 次实战
