# STATUS · task #81 · LanceDB 主版本升级

> 日期：2026-05-29
> 对应 commit：[本 commit · task #81 lancedb 0.10 → 0.30]
> 对应 feature log：[`feedback/features/rag4arkui-core/26-2026-05-29-lancedb-upgrade.md`](../feedback/features/rag4arkui-core/26-2026-05-29-lancedb-upgrade.md)
> 对应 meta：[`feedback/meta/12-2026-05-29-lancedb-major-bump.md`](../feedback/meta/12-2026-05-29-lancedb-major-bump.md)
> 上一阶段：[`STATUS-mdbook-doc.md`](STATUS-mdbook-doc.md)
> 下一阶段：Day 20c onnx（task #87 ort 阻塞）/ Day 17 DevEco / 用户首推 master + v1.0.0

> 🎯 **里程碑**：**lancedb feature 完整真活 · 第 3 层 pre-existing 阻塞清除 · 三层 lancedb 阻塞 final 状态 ✅✅✅** ⭐

---

## 当前状态

| 模块 | 变化 |
|---|---|
| `crates/Cargo.toml` | bump `lancedb 0.10 → 0.30` · `arrow-array/schema 52 → 58` · 移除 `chrono = "=0.4.39"` pin |
| `crates/arkui-rag-storage/Cargo.toml` | lancedb feature 移除 `dep:chrono`（不再需要） |
| `crates/arkui-rag-storage/src/lancedb_store.rs` | `RecordBatchReader` trait cast · `open(dim=0)` 自动从 schema 推导 + `read_vector_dim_from_schema` helper · 3 个新单测 |
| `docs/RELEASE.md` | lancedb 行从 ❌ 改为 ✅ |

### 三层阻塞 final 状态

| Layer | Day 21b 状态 | task #81 状态 |
|---|---|---|
| 1. chrono trait method 歧义 | ✅ 修了（pin 0.4.39） | ✅ pin 移除（arrow-arith 58 已修） |
| 2. lance build 需 protoc | 📄 文档化 | 📄 仍需用户预装 |
| 3. lance 0.17 async 类型递归 | ❌ 致命 | ✅ **修了**（升 lancedb 0.30 内部新 lance） |

---

## 输入契约

### 编译

```bash
# 装 protoc（一次性）
brew install protobuf       # macOS
# 或 apt install protobuf-compiler  # Linux

# 编译 lancedb feature
cargo build --release -p arkui-rag-cli --features corpus-pull,tantivy,lancedb
# binary 95 MB（vs 不含 lancedb 11 MB · lance 引擎约 84 MB）
```

### CLI 用法

```bash
# 建索引（向量后端选 lancedb）
arkui-rag index \
    --source ./corpus \
    --index-path ./corpus/index.json \
    --bm25 tantivy \
    --vector lancedb
# 输出多了 `lance dir : <path>/vectors.lance`

# 查询（自动从 lance schema 反推 dim · 无需 user 传）
arkui-rag query \
    --text "..." \
    --index-path ./corpus/index.json \
    --bm25 tantivy \
    --vector lancedb -k 5
```

---

## 输出契约

### `LanceVectorStore::open(uri, model_id, dim)` 新语义

| `dim` 参数 | 表存在 | 行为 |
|---|---|---|
| `0` | ✅ | 从 Arrow schema 反推 dim |
| `0` | ❌ | 报错（"table 不存在且未指定 dim · 用 dim>0 创建新 table"） |
| `>0` | ✅ 但 schema dim 不匹配 | 报错（"传入 dim={} 与已有 schema dim={} 不一致"） |
| `>0` | ✅ 且 schema dim 匹配 | OK · 用 schema dim |
| `>0` | ❌ | OK · 建空表用此 dim |

### 端到端 query 输出

```
plan_summary="Projection(Take(CoalesceBatches(GlobalLimit(LanceFilter(SortExec(TopK)(KNNVectorDistance(FilteredRead)))))))"
output_rows=22 iops=2 requests=2 bytes_read=37888

✅ Top-3 hits (embedder=mock-384 · bm25=tantivy · rerank=none · hyde=none)

─── [1] score=0.0294 ──────────────────
  source : mapping-state.md L24-34
  heading: Mapping — 状态、Effect 与生命周期 > 状态选择决策（生成时按场景选）
  ...
```

lance 内部跑 `KNNVectorDistance` + `FilteredRead` 真向量索引。

---

## 验证手段

### 用户手动

```bash
# 编译
brew install protobuf
cargo build --release -p arkui-rag-cli --features corpus-pull,tantivy,lancedb

# 测试
cargo test -p arkui-rag-storage --features lancedb
# 13 passed (10 原有 + 3 新 dim auto-detect)

# 端到端
mkdir -p /tmp/lancedb-demo/corpus
cp some.md /tmp/lancedb-demo/corpus/
arkui-rag index --source /tmp/lancedb-demo/corpus --index-path /tmp/lancedb-demo/corpus/index.json --bm25 tantivy --vector lancedb
arkui-rag query --text "..." --index-path /tmp/lancedb-demo/corpus/index.json --bm25 tantivy --vector lancedb -k 3
```

### 自动化

| 手段 | 范围 | 状态 |
|---|---|---|
| `cargo check -p arkui-rag-storage --features lancedb` | 编译 | ✅ |
| `cargo test -p arkui-rag-storage --features lancedb` | 13 测过（含 3 dim auto-detect） | ✅ |
| `cargo build --release -p arkui-rag-cli --features ...,lancedb` | CLI 端编通 | ✅ |
| 端到端 index + query `--vector lancedb` | 业务链路 | ✅ |
| regression memory backend | 不破坏 | ✅ |
| **M-STATUS-PER-ROUND** Round 26 + STATUS-lancedb-upgrade | 元规则 | ✅ |
| **ROADMAP 维护约定（第 14 次实战）** | 当前位置 + 已完成表 | ✅ |

### 暂未自动化（明确缺口）

- ❌ lancedb 进默认 release（暂不 · binary 体积 11 MB → 95 MB 太大）
- ❌ benchmark 1k+ chunks lancedb vs in-memory（验升级价值）
- ❌ CI release.yml 加 protoc 安装步骤（未来加 lancedb 到 default 时需要）
- ❌ mdBook 加 lancedb 真活使用章节

---

## 与上一阶段（STATUS-mdbook-doc）的关联性

### 增量

| 维度 | Day 22 完成时 | 本轮（task #81）后 |
|---|---|---|
| Crate 数 | 9 | 不变 |
| CLI 子命令 | 5 | 不变 |
| 默认 release features | 6 项 | 不变（lancedb 仍不默认） |
| Release binary（默认） | 11 MB | 不变 |
| **lancedb feature 状态** | ❌ 编译阻塞 | ✅ **完整真活** |
| LanceVectorStore tests | 0（不能编） | **13 passed** |
| `--vector lancedb` 端到端 | 不可用 | ✅ 真活 |
| pre-existing 阻塞清单 | 1 项（lancedb） + ort（task #87） | 仅 ort（task #87） |

### 兼容性

- ✅ 无破坏性变更（LanceVectorStore::open 向后兼容 · 新增 dim=0 语义）
- ✅ 默认 release artifact 大小不变（lancedb 不进默认）
- ✅ memory backend 不变
- ✅ chrono pin 移除对其它 crate 无影响（仅 lancedb feature 才用 chrono）

---

## 完成度 / 下一阶段

### task #81 完成度

| 项 | 状态 |
|---|---|
| lancedb 0.10 → 0.30 主版本升级 | ✅ |
| arrow 52 → 58 同步 | ✅ |
| chrono pin 移除 | ✅ |
| `RecordBatchReader` cast 修编译 | ✅ |
| `dim auto-detect from schema` 修运行时 | ✅ |
| 3 个新单测覆盖 dim auto-detect | ✅ |
| CLI 端到端 `--vector lancedb` 验证 | ✅ |
| regression memory backend | ✅ |
| docs/RELEASE.md feature 表更新 | ✅ |
| 加进默认 release features | ⏸️ 决策：暂不（binary 体积过大） |
| benchmark vs in-memory | ⏳ follow-up |
| mdBook 章节 | ⏳ follow-up |

### 6 周路线图达成度

| 章节 | 状态 |
|---|---|
| Week 1: Rust 骨架 + tree-sitter + LanceDB + Tantivy + BGE-M3 | **7/7** ✅ |
| Week 2: 混合检索 + Reranker + HyDE + 评估集 | **4/4** ✅ |
| Week 3: HTTP + MCP + CLI | **3/3** ✅ |
| Week 4: 协议层（HTTP + MCP + LSP） | **3/3** ✅ ⭐ |
| Week 5: Claude Code 接入 | **1/1** ✅ |
| Week 6: 发布 + 文档站 + 评估报告 | **4/4** ✅ |

**总完成度估算：~92%**（Week 1-6 全部达成 · pre-existing 阻塞仅余 ort 链路 · 用户操作 4 件事即可让 1.0 上线）

### 下一阶段建议（按推荐优先级）

| 候选 | 价值 | 工作量 |
|---|---|---|
| 🟢 **用户操作 4 件事让 MVP 真上线**（task #84 + #85） | 文档站 + 1.0 release 真上 | 用户 UI 操作 · 0 commit |
| 🟢 **Day 17 DevEco Plugin MVP** | 关键路径主战场 · IDE 集成 | 5+ commit · 大工程 |
| 🟡 **Day 20c onnx 真活**（task #87 阻塞 · 需先架构决策 ort 1.16 vs candle vs sherpa-onnx） | 解锁真语义 RAG | 4-6h 调研 + 2-3 commit |
| 🟡 ArkTS struct method extraction（custom grammar） | 解锁 ArkTS @Component 方法切分 | 大工程 |
| ⚪️ benchmark lancedb vs in-memory（1k+ chunks） | 验本轮升级价值 | 1 commit |
| ⚪️ mdBook 加 lancedb 章节 | 用户文档 | 0.5 commit |

**Agent 推荐**：**Day 17 DevEco Plugin MVP**（关键路径主战场） 或 **用户操作 task #84/#85**（让 MVP 真上线）。

Day 20c onnx 仍是 ecosystem-level blocker（task #87）· 不建议无架构决策时硬攻。

### 重要的"非完成"项

- ❌ Day 17 DevEco Plugin MVP
- ❌ Day 20c onnx 真活（task #87 ort 阻塞）
- ❌ task #84 用户首推 master 触发 book.yml 部署
- ❌ task #85 用户 push tag v1.0.0
- ❌ ArkTS struct method extraction
- ❌ benchmark + lancedb mdBook 章节（细节优化）
