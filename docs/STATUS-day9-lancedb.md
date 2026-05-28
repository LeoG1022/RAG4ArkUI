# STATUS · Day 9 · LanceDB 嵌入式向量库

> 日期：2026-05-28
> 对应 commit：[本 commit · Day 9 LanceDB]
> 对应 feature log：[`feedback/features/rag4arkui-core/14-2026-05-28-day9-lancedb.md`](../feedback/features/rag4arkui-core/14-2026-05-28-day9-lancedb.md)
> 上一阶段：[`STATUS-day10-tree-sitter.md`](STATUS-day10-tree-sitter.md)
> 下一阶段：`STATUS-day11-parent-child.md` 或 `STATUS-day14-http.md`（按用户选择）

> 🎯 **里程碑**：**Week 1 7/7 全部达成**。规模化能力就位 —— chunks > 10k 不再卡。

---

## 当前状态

| 模块 | 变化 |
|---|---|
| `arkui-rag-storage` | **新增** LanceVectorStore（Arrow schema + delete-by-id upsert + post-filter） |
| `arkui-rag-cli` | **新增** VectorBackend 抽象 + `--vector memory\|lancedb` + lancedb feature 转发 |
| `Makefile` | + `check-lancedb` / `build-lancedb` target |
| Cargo workspace | + lancedb 0.10 / arrow-array 52 / arrow-schema 52 / futures 0.3 |
| crates/README + ADR-002 + ROADMAP | 速查表 + 路线图同步 |

### 测试覆盖

| 测试组 | 数量 |
|---|---|
| `LanceVectorStore` 单测（feature lancedb） | 5 (upsert_search/dim mismatch/upsert overwrite/reopen persist/delete) |
| 默认 features 累计 | **44**（不变 · lancedb 默认关闭） |
| `--features lancedb` 累计 | **49** |
| 全 feature（`--features full`） | 约 64 |

---

## 输入契约

### 用户视角

```bash
# 默认（向后兼容 · LanceDB 不拉）
make check
make test                                  # 默认 44 测试

# 启用 LanceDB（首次 ~5-10 分钟拉 arrow + lancedb）
make check-lancedb
cd crates && cargo test -p arkui-rag-storage --features lancedb
# 期望 5 个新测全过

# 端到端：用 LanceDB 建索引
cd ..
cargo run --features lancedb -p arkui-rag-cli -- \
    index --source corpus --vector lancedb
# 索引产物路径：corpus/_index/vectors.lance/（与 bm25/ 并列）

# 查询（必须用同样 --vector）
cargo run --features lancedb -p arkui-rag-cli -- \
    query --text "下拉刷新" --k 5 --vector lancedb
```

### CLI 参数（新增）

| Subcommand | 新参数 | 默认 |
|---|---|---|
| `index` | `--vector memory\|lancedb` | memory |
| `query` | `--vector memory\|lancedb` | memory |
| `eval` | `--vector memory\|lancedb` | memory |

### 库 API

```rust
#[cfg(feature = "lancedb")]
use arkui_rag_storage::LanceVectorStore;

let store = LanceVectorStore::open(
    "corpus/_index/vectors.lance",
    "bge-m3",
    1024,
).await?;
// 实现 VectorStore + MetadataStore trait，可直接喂给 HybridRetriever
```

---

## 输出契约

### 索引产物布局（启用 LanceDB 后）

```
corpus/_index/
├── index.json          (Memory 走这个；Lancedb 不写)
├── bm25/               (Tantivy；Day 4)
└── vectors.lance/      (LanceDB 数据集 · Day 9)
    ├── data/
    │   └── *.lance
    └── _versions/
```

### index 命令输出（含 vector kind）

```
✅ 索引完成
   embedder    : mock-384
   dim         : 384
   vector      : lancedb              ← Day 9 新增行
   bm25        : memory
   files       : 12
   chunks      : 47
   skipped     : 3
   elapsed_ms  : 145
   lance dir   : corpus/_index/vectors.lance   ← Lancedb 才打印
```

### Hit.score 含义

- Memory backend：cosine 相似度 [0, 1]（已 L2 归一化）
- Lancedb backend：`1 - clamp(distance, 0, 1)` —— 同样越大越好
- 两种 backend 的 score 量纲不同 → 不要跨 backend 直接比较

---

## 验证手段

### 用户手动

```bash
# 1. 默认编译（lancedb 不拉）
make check
make test                                  # 默认 44 测试

# 2. lancedb feature（首次拉 arrow + lancedb · 5-10 分钟）
make check-lancedb
cd crates && cargo test -p arkui-rag-storage --features lancedb

# 3. 端到端：用 LanceDB 建索引 + 查询
cd ..
cargo run --features lancedb -p arkui-rag-cli -- \
    index --source corpus --vector lancedb --embedder mock --dim 384
cargo run --features lancedb -p arkui-rag-cli -- \
    query --text "下拉刷新" --k 5 --vector lancedb

# 4. 配置矩阵 eval：Memory vs LanceDB（同 dim 同 embedder 应得近似分数）
cargo run --features lancedb -p arkui-rag-cli -- \
    eval --queries corpus/_eval/queries.yaml --vector lancedb
```

### 自动化

| 手段 | 范围 | 状态 |
|---|---|---|
| `LanceVectorStore` 单测 × 5 | upsert/search/dim mismatch/overwrite/reopen-persist/delete | ✅ feature gated |
| `M-STATUS-PER-ROUND` | Round 14 + STATUS-day9 配套 | ✅ |
| **ROADMAP 维护约定（第 3 次实战）** | 7 处进度行同步更新 | ✅ |

### 暂未自动化（明确缺口）

- ❌ Filter SQL 下沉（platforms / tags / api_version 全部 post-filter，性能在 > 100k chunks 时退化）
- ❌ LanceDB 内置 manifest 表存 model_id（CLI 跳过严格校验）
- ❌ Memory vs LanceDB 性能基准（criterion）
- ❌ HNSW 向量索引调优（当前用默认 brute force）

---

## 与上一阶段（STATUS-day10-tree-sitter）的关联性

### 增量

| 维度 | Day 10 完成时 | 本轮（Day 9）后 |
|---|---|---|
| Crate 数 | 9 | 不变 |
| 向量后端 | InMemoryVectorStore（Vec + cosine 暴力） | + **LanceVectorStore（嵌入式向量库）** |
| corpus 规模上限 | ~10k chunks（cosine 暴力性能崩） | **>100k chunks**（LanceDB IVF/HNSW 索引） |
| CLI feature 数 | 6（onnx/tantivy/typescript/kotlin/swift/full） | **7**（+ lancedb） |
| 测试数（默认） | 44 | 不变（lancedb 默认关闭） |
| 测试数（lancedb feature） | — | **49** |

### 兼容性

- ✅ `VectorStore` trait 完全不变
- ✅ CLI `--vector memory`（默认）保持 Day 2 起的行为
- ✅ LanceVectorStore 同时实现 `MetadataStore`（双 trait，与 InMemoryVectorStore 对称）
- ⚠ Day 9 简化：Lance 路径不严格校验 model_id（用户责任）

### Week 1 全部达成（7/7） ⭐

| 章节 | 状态 |
|---|---|
| Rust workspace 骨架 | ✅ Day 1 |
| tree-sitter（ArkTS） | ✅ Day 10（Kotlin/Swift stub） |
| LanceDB | ✅ **Day 9（本轮）** |
| Tantivy | ✅ Day 4 |
| BGE-M3 ONNX 推理 | ✅ Day 3 |
| 索引管道（Indexer） | ✅ Day 2 |
| CLI 二进制 | ✅ Day 2 + 持续演进 |

---

## 完成度 / 下一阶段

### Day 9 完成度

| 项 | 状态 |
|---|---|
| `LanceVectorStore` 实现 VectorStore + MetadataStore | ✅ |
| CLI `--vector` 选项 + VectorBackend 抽象 | ✅ |
| 5 单测覆盖 | ✅ |
| Makefile + 文档同步 | ✅ |
| ROADMAP 维护约定第 3 次实战 | ✅ |
| Filter SQL 下沉 | ⏳ Day 9 续 |
| manifest 表存 model_id | ⏳ Day 9 续 |

### 6 周路线图达成度更新

| 章节 | 状态 |
|---|---|
| Week 1: Rust 骨架 + tree-sitter + LanceDB + Tantivy + BGE-M3 | **7/7** ✅ |
| Week 2: 混合检索 + Reranker + HyDE + 评估集 | **4/4** ✅ |
| Week 3: HTTP + MCP + CLI | **1/3** ✅ (CLI 完整 · HTTP/MCP ⏳) |
| Week 4: IDE 插件 | **0/2** ⏳ |
| Week 5: Claude Code 接入 | **0/1** ⏳ |
| Week 6: 发布 + 文档站 + 评估报告 | **1/4** ✅ |

**总完成度估算：~50%**

### 下一阶段建议（按推荐优先级）

| 候选 | 价值 | 工作量 |
|---|---|---|
| 🟢 **Day 11 Parent-Child 父子索引**（推荐） | 检索小 chunk 返回父级 chunk（方案 §1.4 标准）；评估集会立即看到 recall 提升 | 1 commit |
| 🟢 Day 14 HTTP/REST Server | 协议层入门 · IDE 接入前置 · 关键路径 | 2-3 commit |
| 🟢 Day 12 Query Router + Intent | 不同 query 走不同流水线（方案 §1.2） | 1 commit |
| 🟡 Day 8 tantivy-jieba | 中文 BM25 精度 | 0.5 commit |
| 🟡 Day 9 续 | filter SQL 下沉 + manifest + 性能基准 | 1 commit |

**Agent 推荐**：**Day 11 Parent-Child 父子索引**。理由：
1. 方案 §1.4 标准做法："检索小（精准）→ 返回父（上下文完整）"
2. 现有 ChunkMetadata 已预留 `parent_id` 字段（Day 1 设计），只需在 chunker 端生成 + retriever 端扩展
3. 评估集（Day 6）能立即量化效果（recall@5 应有可见提升）
4. 工作量 1 commit，快速迭代
5. 完成后 Week 3 检索能力进入第 2 阶段（Parent-Child + Query Router + ContextAssembler 三件套）

### 重要的"非完成"项

- ❌ Filter SQL 下沉（platforms / tags / api_version post-filter 性能在 > 100k chunks 时退化）
- ❌ Manifest 表（model_id / dim 不在 Lance schema 内存储 → CLI 跳过严格校验）
- ❌ 性能基准（criterion）：缺少 Memory vs LanceDB 的实测对比数据
- ❌ HNSW / IVF 向量索引调优（用 LanceDB 默认配置）
