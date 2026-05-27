# STATUS · Day 4 · BM25 / Tantivy 实装

> 日期：2026-05-27
> 对应 commit：`331a912 feat(storage,cli): TantivyBM25Index + --bm25 memory|tantivy (Day 4)`
> 对应 feature log：[`feedback/features/rag4arkui-core/7-2026-05-27-day4-bm25-tantivy.md`](../feedback/features/rag4arkui-core/7-2026-05-27-day4-bm25-tantivy.md)
> 上一阶段：[`STATUS-day2.md`](STATUS-day2.md)（Day 2 端到端 Mock Demo · 唯一一份历史 STATUS）
> 下一阶段：`STATUS-day5-reranker.md`（计划：BGE-Reranker-v2 真活，让"Hybrid + Rerank"基线完整）

> ⚠️ 本文件是 **规则化追溯**：Day 4 commit 时还没立"每轮 STATUS 硬性规则"，本文档是 Commit A 中补建的回溯快照。

---

## 当前状态

Day 4 关键交付（feature log Round 7 详）：

| 模块 | 变化 |
|---|---|
| `arkui-rag-storage` | 新增 `tantivy_bm25.rs` (~340 行) · feature `tantivy` 启用 |
| `TantivyBM25Index` | 实现 `BM25Index` trait · 10 字段 schema · ngram(2,3) 中文分词 · 7 单测 |
| `arkui-rag-cli` | 加 `--bm25 memory\|tantivy` 参数 · feature `tantivy` / `full` 转发 |
| `HybridRetriever` | RRF 终于**真正双路融合**（之前 BM25 路径返回空，退化为纯向量） |
| Makefile | 加 `build-tantivy` / `build-full` / `check-tantivy` |

**业界基线达标**：技术方案 §1.6 第 1 条原则（"混合检索是基线"）+ §8.5 共识 1 落地。

---

## 输入契约

### CLI 新增参数

```bash
# 默认（向后兼容 Day 2/3）：BM25 走 InMemoryBM25Index 空 stub
arkui-rag index --source corpus

# Day 4 启用：真实 BM25 倒排检索
arkui-rag index --source corpus --bm25 tantivy   # 需 --features tantivy

# query 必须用同样的 --bm25 参数
arkui-rag query --text "..." --bm25 tantivy --features tantivy
```

### 编译开关

| feature | 行为 |
|---|---|
| 默认 | Tantivy 不编译，`--bm25 tantivy` 报错 |
| `--features tantivy` | 启用 TantivyBM25Index |
| `--features full` | onnx + tantivy 一键全启 |

### Corpus 文档

无变化（仍是 markdown + YAML frontmatter，见 STATUS-day2 §3）。

---

## 输出契约

### 索引产物

```
corpus/_index/
├── index.json       # 向量索引（Day 2 起，JSON 格式）
└── bm25/            # 【Day 4 新增】Tantivy BM25 索引目录
    ├── meta.json    #   schema + segments 元数据
    └── *.tantivy    #   实际倒排数据
```

BM25 目录自动推导：`<index-path-dir>/bm25/`。无需用户指定。

### index 命令输出

```
✅ 索引完成
   embedder    : mock-384
   dim         : 384
   bm25        : tantivy            ← Day 4 新增行
   files       : 12
   chunks      : 47
   skipped     : 3
   elapsed_ms  : 89
   saved to    : corpus/_index/index.json
   bm25 index  : corpus/_index/bm25  ← 仅 tantivy 时打印
```

### query 命令输出

```
✅ Top-3 hits (embedder=mock-384 · bm25=tantivy)   ← Day 4 新增 bm25 标识
...
```

### Hit 对象的 source 字段

| 来源 | source | 触发场景 |
|---|---|---|
| 向量唯一命中 | `Vector` | BM25 无该 chunk |
| BM25 唯一命中 | `Bm25` | 向量无该 chunk |
| 双路命中 | `Hybrid` | RRF 融合后（最优结果） |

---

## 验证手段

### 用户手动

```bash
# 1. 默认编译（Tantivy 不拉，~3 分钟）
make check
make test                                # 24 个测试

# 2. 启用 Tantivy（首次 ~3-5 分钟）
make check-tantivy
cd crates && cargo test -p arkui-rag-storage --features tantivy
# 期望：12 个测试（5 InMemoryVectorStore + 7 TantivyBM25Index）全过

# 3. 端到端 hybrid demo（需 corpus/ 有文档）
cd ..
cargo run --manifest-path crates/Cargo.toml -p arkui-rag-cli --features tantivy -- \
    index --source corpus --bm25 tantivy
cargo run --manifest-path crates/Cargo.toml -p arkui-rag-cli --features tantivy -- \
    query --text "router pushUrl" --k 5 --bm25 tantivy

# 4. 一键全启（onnx + tantivy）
make build-full
```

### 自动化

| 手段 | 范围 | 来源 |
|---|---|---|
| 单元测试 × 32 | 8 个 crate 的 trait 契约 + 算法 | `cargo test --workspace` |
| TantivyBM25Index 单测 × 7 | upsert/search/overwrite/delete/filter/deprecated/empty-query/reopen-persist | `cargo test --features tantivy` |
| 端到端 e2e × 3 | indexer 真实跑通 | `cargo test -p arkui-rag-indexer` |
| pre-commit hook 19 条 | 元数据 + 文档 + 归档一致性 | `scripts/check-consistency.sh` |
| demo smoke 脚本 | CLI 二进制实际行为（mock + memory） | `make smoke` |
| GitHub Actions CI（搁置） | check/test/fmt/clippy/smoke | `.github/workflows/ci.yml`（待用户决定推送目标） |

**未覆盖**：
- 真实 hybrid 双路 e2e（需 ground truth 评估）→ Week 3 评估集
- 中文分词精度（ngram 是粗暴方案，jieba 备选）

---

## 与上一阶段（STATUS-day2 / Day 3）的关联性

### 增量（不破坏向后兼容）

| 维度 | Day 2/3 | Day 4 |
|---|---|---|
| CLI 接口 | `arkui-rag index --source corpus` | + `[--bm25 memory\|tantivy]`（默认 memory，原命令不变） |
| 向量索引格式 | `index.json` | 不变 |
| BM25 索引 | 不存在 / 空 stub | 新增 `bm25/` 目录（与 vector 并列） |
| 编译默认依赖 | 默认 features | 默认仍不拉 tantivy（feature gate） |
| HybridRetriever 接口 | 不变 | 不变（仍 `Retriever` trait） |

### 4 种检索质量组合（用户可自由选）

| Embedder | BM25 | 等级 |
|---|---|---|
| Mock | Memory | Day 2 demo（无语义无关键词） |
| Mock | **Tantivy** | Day 4 新增：关键词匹配可用 |
| OnnxEmbedder | Memory | Day 3：语义可用 |
| OnnxEmbedder + Tantivy | | **业界标配真 Hybrid** |

### 破坏性变更

- `VectorStore` trait 加了 `len()` 方法（Day 2 已加，Day 4 沿用）
- `BM25Index` trait 不变
- 无其他 API break

---

## 完成度 / 下一阶段

### 当前完成度对照技术方案 6 周路线图

| 章节 | 状态 |
|---|---|
| §4.2 决策 4（存储层 LanceDB + Tantivy） | Tantivy ✅ · LanceDB ⏳ |
| §1.6 原则 1（Hybrid 检索基线） | ✅ 名实相符 |
| §8.5 共识 1（混合检索默认） | ✅ |
| §8.5 共识 2（Reranker 是分水岭） | ⏳ Day 5 |

**Day 4 完成度评估**：BM25 真活 100%，但留以下缺口：

| 缺口 | 现状 | 优先级 |
|---|---|---|
| 中文分词精度 | ngram(2,3) 粗暴 | 中（生产前必补） |
| 大规模 corpus 支持 | InMemoryVectorStore 卡在 ~10k chunks | 中 |
| Rerank 缺位 | CrossEncoderReranker 仍 truncate stub | **高（下一阶段）** |
| BM25 与 vector 后端错配检测 | 仅校验 embedder_model_id，不校验 bm25_kind | 低 |

### 下一阶段建议（按优先级）

| 候选 | 价值 | 预估工作量 |
|---|---|---|
| 🟢 **Day 5 Reranker（推荐）** | 技术方案 §8.5 "产品级 RAG 分水岭"；让"Hybrid + Rerank 基线"完整 | 1 commit |
| 🟡 tantivy-jieba | 中文 BM25 精度从 ngram 升级到真分词 | 0.5 commit |
| 🟡 LanceDB 替换 InMemoryVectorStore | 解锁 > 10k chunks 的真实 corpus | 1-2 commit |
| 🟡 HyDE 改写器 + 评估集 | 检索质量评估闭环（RAGAS） | 2-3 commit |
| ⚪ tree-sitter（.ets/.kt/.swift） | corpus 投放代码时关键 | 1-2 commit |

**Agent 推荐**：Day 5 Reranker。理由：
1. BM25 + Vector 双路已通，没有 Rerank 时返回 Top-K 仍是 RRF 粗排
2. Reranker 模型与 §7.2 OnnxEmbedder 同生态（ONNX Runtime），代码复用度高
3. 完成后"Hybrid + Rerank"业界基线整套就位 → 可以认真讨论检索质量
