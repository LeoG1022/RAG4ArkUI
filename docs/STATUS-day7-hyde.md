# STATUS · Day 7 · HyDE 改写器

> 日期：2026-05-27
> 对应 commit：[本 commit · Day 7 HyDE]
> 对应 feature log：[`feedback/features/rag4arkui-core/12-2026-05-27-day7-hyde.md`](../feedback/features/rag4arkui-core/12-2026-05-27-day7-hyde.md)
> 上一阶段：[`STATUS-roadmap-doc.md`](STATUS-roadmap-doc.md) · [`STATUS-day6-eval.md`](STATUS-day6-eval.md)
> 下一阶段：`STATUS-day8-jieba.md` 或 `STATUS-day10-tree-sitter.md`

> 🎯 **里程碑**：**Week 2 全部达成**！Advanced RAG 4 件套（Hybrid + Rerank + Eval + HyDE）业界基线完整。

---

## 当前状态

新增 `QueryEnhancer` trait + 两个实现，让 Advanced RAG 的 Query 改写层就位：

| 模块 | 变化 |
|---|---|
| `arkui-rag-core` | **新增** trait `QueryEnhancer` + `PassthroughEnhancer`（默认）|
| `arkui-rag-retrieval` | **新增** `MockHydeEnhancer`（~180 行 · 6 单测） |
| `arkui-rag-eval::Evaluator` | 加 `with_enhancer` builder · 默认 PassthroughEnhancer |
| `arkui-rag-eval::EvalConfig` | 加 `hyde` 字段 |
| `arkui-rag-eval::report` | 报告头打印 hyde 标识 |
| `arkui-rag-cli` | `query` / `eval` 加 `--hyde none\|mock` · 报告文件名带 hyde 段 |
| `docs/ROADMAP.md` | **维护约定首次实战** · 进度行同步 |

**Week 2 完成度**：4/4 ✅（Hybrid + Rerank + 评估集 + HyDE）

### 测试覆盖

| 测试 | 数量 |
|---|---|
| `PassthroughEnhancer` 单测 | 1 |
| `MockHydeEnhancer` 单测（intent 分类 + ArkTS 模板 + 实体抽取 + 确定性） | 6 |
| **本轮新增小计** | **7** |
| **累计** | 31 → **37**（不含 onnx/tantivy/full feature 扩展） |

---

## 输入契约

### CLI 新增参数

```bash
# query 路径
arkui-rag query --text "如何下拉刷新" --k 5 [--hyde none|mock]

# eval 路径（与 query 完全对齐的 hyde 参数）
arkui-rag eval --queries corpus/_eval/queries.yaml [--hyde mock]
```

| 参数 | 选项 | 默认 |
|---|---|---|
| `--hyde` | `none` / `mock` | `none` |

### `QueryEnhancer` trait（库 API）

```rust
#[async_trait]
pub trait QueryEnhancer: Send + Sync {
    async fn enhance(&self, raw: &str) -> Result<EnhancedQuery>;
    fn name(&self) -> &str;
}
```

两个内置实现：
- `PassthroughEnhancer` — 透传 raw → EnhancedQuery::passthrough(raw)
- `MockHydeEnhancer` — 规则法生成 ArkTS 假代码 + intent 分类 + 实体抽取

### MockHyde 输入 → 输出对应

| 输入 query | intent | hyde_doc 模板 |
|---|---|---|
| "router.pushUrl 怎么传参数" | `ApiLookup` | `import router from '@ohos.router'; ... router.pushUrl({...})` |
| "ArkUI-X 如何下拉刷新" | `NewComponent` | `Refresh({ refreshing: false }) { List() { ForEach(...) } }` |
| "@State 编译错误怎么修复" | `ErrorFix` | `// 检查 @State / @Prop 装饰器位置...` |
| "KMP ViewModel 怎么迁移" | `Migration` | `// 原 ViewModel.launch → async/await` |
| "一多 适配 断点" | `Adaptive` | `GridRow { GridCol({ span: { sm: 12, md: 6 } }) }` |
| 其他 | `Generic` | 通用 `@Component struct Example` 模板 |

---

## 输出契约

### query 输出（控制台）

新增 hyde 标识行：

```
✅ Top-3 hits (embedder=mock-384 · bm25=memory · rerank=none · hyde=mock-hyde-arkts)
```

### eval 输出

```
📊 跑评估：8 个 query · embedder=mock-384 · bm25=memory · rerank=none · hyde=mock-hyde-arkts · k=5
```

eval 报告头新增 hyde 标识：
```markdown
- **配置**: embedder=`mock-384` · bm25=`memory` · rerank=`none` · hyde=`mock-hyde-arkts` · pre_rerank_k=`50`
```

### 报告文件名约定（更新）

旧（Day 6）：`reports/eval-<ts>-<embedder>-<bm25>-<rerank>-<k>.md`
新（Day 7）：`reports/eval-<ts>-<embedder>-<bm25>-<rerank>-<hyde>-<k>.md`

### EnhancedQuery 填充（库视角）

```rust
let enhancer = MockHydeEnhancer::new();
let eq = enhancer.enhance("ArkUI-X 怎么下拉刷新").await?;
// eq.raw       = "ArkUI-X 怎么下拉刷新"
// eq.rewritten = "ArkUI-X 怎么下拉刷新"
// eq.hyde_doc  = Some("@Component\nstruct ListExample { ... Refresh ... }")
// eq.entities  = ["ArkUI-X"]  // 含连字符 / 大写
// eq.intent    = NewComponent
```

### HybridRetriever 自动消费

Day 1 已设计：
```rust
let text_for_vector = query.hyde_doc.as_deref().unwrap_or(query.rewritten.as_str());
```

**所以本轮 HybridRetriever 零改动**，自动用上 hyde_doc。

---

## 验证手段

### 用户手动

```bash
# 1. 编译验证
make check
make test                                       # 37 测试

# 2. HyDE 单 crate 测试
cd crates && cargo test -p arkui-rag-retrieval hyde

# 3. 端到端：build 索引 → query 对比 hyde 开关
cd ..
cargo run --manifest-path crates/Cargo.toml -p arkui-rag-cli -- \
    index --source corpus

# 关闭 HyDE
cargo run --manifest-path crates/Cargo.toml -p arkui-rag-cli -- \
    query --text "如何下拉刷新" --k 5
# 开启 HyDE
cargo run --manifest-path crates/Cargo.toml -p arkui-rag-cli -- \
    query --text "如何下拉刷新" --k 5 --hyde mock

# 4. 评估对比（HyDE on vs off）
cargo run -p arkui-rag-cli -- eval --queries corpus/_eval/queries.yaml --k 5            # baseline
cargo run -p arkui-rag-cli -- eval --queries corpus/_eval/queries.yaml --k 5 --hyde mock # with HyDE
diff reports/eval-*-mock-384-memory-none-none-5.md \
     reports/eval-*-mock-384-memory-none-mock-hyde-arkts-5.md
```

### 自动化

| 手段 | 范围 | 状态 |
|---|---|---|
| `PassthroughEnhancer` 单测 | 透传不改写 | ✅ |
| `MockHydeEnhancer::hyde_generates_arkts_template` | hyde_doc 必含 @Component / build() | ✅ |
| `classifies_*_intent` × 3 | intent 分类正确性 | ✅ |
| `extracts_code_like_entities` | 实体抽取正确性 | ✅ |
| `deterministic_for_same_input` | 同输入同输出（关键 contract） | ✅ |
| Evaluator 集成 | `Evaluator::with_enhancer` 接 MockHyde | ⏳ 用户跑 eval 验证 |
| **M-STATUS-PER-ROUND** | Round 12 + STATUS-day7-hyde 配套 | ✅ |
| **ROADMAP 维护约定** | 同 commit 同步 ROADMAP（**首次实战**） | ✅ |

### 暂未自动化（明确缺口）

- ❌ HyDE 效果量化（需用户跑评估对比）
- ❌ 配置矩阵自动跑（`eval --matrix` 一次跑多个配置）
- ❌ HyDE 真实 LLM 接入（Week 3）

---

## 与上一阶段（STATUS-day6-eval）的关联性

### 增量

| 维度 | Day 6 完成时 | 本轮（Day 7）后 |
|---|---|---|
| Crate 数 | 9 | 不变 |
| trait 数 | Retriever / Embedder / Reranker / ASTChunker / VectorStore / BM25Index / MetadataStore | + **QueryEnhancer** = 8 |
| CLI subcommand 数 | 5 | 不变 |
| CLI 可调参数 | 14 | + 2（query/eval 各加 --hyde） |
| 测试数 | 31 | **37**（+7） |
| Week 2 完成度 | 3/4 | **4/4** ✅ |
| 业界共识达成 | 4/5 | 不变（HyDE 不在 §8.5 共识 5 之列；是 §1.2 Advanced 范畴） |
| Advanced RAG 4 件套 | 3/4 | **4/4** ✅ |

### 兼容性

- ✅ 无破坏性变更：`--hyde` 默认 `none`，原命令不变
- ✅ Evaluator 默认 PassthroughEnhancer，Day 6 行为不变
- ✅ HybridRetriever 零改动（早在 Day 1 就支持 hyde_doc 字段）
- ✅ ROADMAP 维护规则按"维护约定"piggyback（不单独 commit）

### 与 ROADMAP 维护约定的协同（首次实战）

按 Round 11 ROADMAP 立的"维护约定"，本 commit 内同步更新 ROADMAP：
- **当前位置** 段：Day 6 → Day 7
- **mermaid gantt**：Day 7 从 ⏳ 改 ✅ + 标 active
- **已完成表**：追加 Round 12 行
- **剩余切片表**：Day 7 划掉 + 标"完成"
- **达成度**：Week 2 从 3/4 → 4/4
- **里程碑预测**：累计 round 11 → 12
- **关键路径**：Day 7 ✓

→ **维护约定可执行性验证通过**，惯例成立。3-5 round 后评估是否升级为强制规则。

---

## 完成度 / 下一阶段

### Day 7 完成度

| 项 | 状态 |
|---|---|
| QueryEnhancer trait + PassthroughEnhancer | ✅ |
| MockHydeEnhancer 规则法生成 ArkTS 假代码 | ✅ |
| 5 类 intent 分类 + 模板覆盖 | ✅ |
| Evaluator with_enhancer + EvalConfig hyde 字段 | ✅ |
| CLI query/eval 加 --hyde | ✅ |
| 报告文件名带 hyde 段 | ✅ |
| ROADMAP 维护约定首次实战 | ✅ |
| 用户跑 HyDE 效果对比评估 | ⏳ 用户责任 |
| 远程 LLM 真活接入（Week 3） | ⏳ |

### Week 2 完成度（全部达成）

| 章节 | 状态 |
|---|---|
| 混合检索 + RRF（Day 4） | ✅ |
| Reranker 接入（Day 5） | ✅ |
| 基础评估集（Day 6） | ✅ |
| **HyDE 改写**（本轮 Day 7） | ✅ |

### 下一阶段建议（按推荐顺序）

| 候选 | 价值 | 工作量 |
|---|---|---|
| 🟢 Day 10 tree-sitter 代码切分 | 代码 corpus 真活（解锁 ArkTS/Kotlin/Swift） | 2 commit |
| 🟢 Day 8 tantivy-jieba 中文分词 | 中文 BM25 精度从 ngram 升级 | 0.5 commit |
| 🟡 Day 9 LanceDB | 大规模 corpus（>10k chunks） | 1-2 commit |
| 🟡 Day 11 Parent-Child 父子索引 | 检索小返回大（方案 §1.4 标准） | 1 commit |
| 🟡 Week 3 Day 14 HTTP Server | 协议层入门，IDE 接入前置 | 2-3 commit |
| ⚪ Week 3 RemoteHydeEnhancer | LLM 真活生成假代码（HyDE 完整版） | 2 commit |

**Agent 推荐**：**Day 10 tree-sitter**。理由：
1. Week 2 完成度 100%，下一关键缺口是"代码 corpus"
2. 方案 §2.3"代码感知的 Chunking 策略"是核心差异化
3. 代码 corpus 解锁后，HyDE 的"假代码 → 真代码"才能真正发挥
4. 工作量约 2 commit，与 Day 4-5 同级
5. 之后才是 LanceDB（规模化）或协议层（Week 3）

### 重要的"非完成"项

- ❌ HyDE 效果量化（agent 没真实 corpus，无法跑真评估）
- ❌ 远程 LLM 真活接入（Week 3 单独切片）
- ❌ HyDE 模板与 ChunkType 自动适配（如 type=api_doc 走 ApiLookup 模板）
