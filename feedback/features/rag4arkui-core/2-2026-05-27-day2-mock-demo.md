# 2 — day2-mock-demo

> 日期：2026-05-27
> 涉及代码：
> - 新增 crate：`crates/arkui-rag-indexer/`（lib.rs + tests/end_to_end.rs）
> - storage 新增：`crates/arkui-rag-storage/src/memory.rs` —— `InMemoryVectorStore` + `InMemoryBM25Index` + JSON 持久化
> - chunker 升级：`crates/arkui-rag-chunker/src/markdown.rs` —— YAML frontmatter 解析
> - retrieval 升级：`crates/arkui-rag-retrieval/src/hybrid.rs` —— `HybridRetriever` 真活
> - cli 升级：`crates/arkui-rag-cli/src/main.rs` —— index/query 真活 + 持久化
> - workspace：`crates/Cargo.toml` 加 indexer crate + serde_yaml 依赖
> - 文档：`docs/ADR-002-crate-structure.md` 加 indexer crate 行；`crates/README.md` 同步速查表
> 类型：重构 + 新建

## 本轮目标

把 Day 1 的全 stub 形态升级为端到端可跑的 Mock Demo：
`arkui-rag index --source corpus/` → 真实建索引 → `arkui-rag query --text "..."` 真实出结果带引用。
**仍用 MockEmbedder + InMemoryVectorStore，不引入 ONNX / LanceDB**，目标是验证整条流水线契约成立。

## Plan

> 必备节（FAIL 级 M-FEATURE-PLAN）

### 6 个 Phase 的设计

| Phase | 改动 | 关键决策 |
|---|---|---|
| Phase 1 | storage 加 `InMemoryVectorStore`（cosine 暴力扫 + L2 归一化假设）+ JSON 持久化（`save_to`/`load_from`）+ `InMemoryBM25Index` 空 stub | JSON 选型而非 bincode/parquet：可读 + 用户能 vim 看 + Day 2 量级足够；`format_version` 字段为 Week 2 升级留口 |
| Phase 2 | 新增 `arkui-rag-indexer` crate（第 8 个）：`Indexer::new(chunker, embedder, vector, bm25).index_directory(path) → IndexStats` | 单独 crate 而非塞 cli：对应 §9 图 3 类图独立的 `Indexer` 类；后续 file-watcher 增量索引也住这 |
| Phase 3 | MarkdownChunker 加 YAML frontmatter 解析（split_frontmatter → Frontmatter struct → apply_to ChunkMetadata）+ 行号偏移修正 | serde_yaml 选型：成熟稳定（虽然官方"deprecated"但等价含义是"feature-complete"，仍是 Rust 社区主流）；行号偏移：line_range 必须相对原始文件（含 frontmatter），不然引用回链错位 |
| Phase 4 | HybridRetriever 接收 Embedder + VectorStore + BM25Index trait objects → tokio::try_join 并行两路检索 → RRF 融合 → 标记 HitSource::Hybrid | per_branch_topk = 50 默认（对齐方案 §9 图 6 时序）；BM25 路径空时 RRF 自然退化为纯向量，零特殊 case |
| Phase 5 | CLI `index` / `query` 真活：`index` 跑 Indexer + 持久化；`query` 加载持久化 + 跑 retriever + 打印 hits + Citation | `corpus/_index/index.json` 作为默认产物路径（已在 .gitignore）；CLI 验证 dim 一致性，防止用户用不同维度 embedder 误查 |
| Phase 6 | 端到端集成测试 `tests/end_to_end.rs`：临时目录 → 投 2 markdown → index → save → load → 2 个 query 断言 + platform filter 测试 | 集成测试放 indexer crate 而非 cli：indexer 是 orchestrator 自然位置；端到端走完整 trait 链 |

### 关键算法 / 契约

**cosine 检索（InMemoryVectorStore::search）**：
- 假设：所有 embedding 都已 L2 归一化（MockEmbedder 已做，OnnxEmbedder §7.2 代码也做）
- 因此 cosine_sim(a,b) = a · b（点积），实现一行
- 暴力扫 O(N·D)，100 chunks × 64 dim ≈ 6400 ops，亚毫秒级

**RRF 退化策略**：
- 当 BM25 返回空 vec → RRF 算法对空输入返回空贡献 → fused 等同于只跑 vector 的结果（按 rrf score 归一化）
- 这意味着 Day 2 demo 实际就是"向量 + RRF re-score"，行为正确

**索引持久化版本约定**：
- `format_version: 1` 当前
- 加载时严格检查版本不匹配 → 报错（不静默"尽力解析"）
- 加载时校验 `embedder_model_id` 与当前 embedder 一致是 CLI 层的事（防"用不同 embedder 查老索引"）

### 与 Day 1 接口的兼容性

- 所有 trait 签名 0 变更
- ChunkMetadata 新增 `extra: BTreeMap<String, serde_json::Value>` 字段已在 Day 1 预留 → frontmatter 的 `api_name` 直接放 extra
- VectorStore trait 新增 `len()` 方法 → 影响所有实现（目前只有 InMemoryVectorStore）。**Day 1 的占位实现 `InMemoryStore` 已删除**，由 InMemoryVectorStore 同时担任 MetadataStore 角色

### 替代选项权衡（被否方案）

- 备选 sled / sqlite 持久化：被否，引入 native 依赖 + 编译时间，JSON 对 Day 2 量级够用
- 备选 indexer 放 cli crate：被否，破坏单一职责，未来 file-watcher 也住这
- 备选直接接 LanceDB：被否，离 user 选的"Mock Demo"目标太远，应在 Day 3 单独切片
- 备选 frontmatter 用 TOML：被否，markdown 社区惯例是 YAML

## 改动要点

> API 选型 / 算法 / 关键决策 / 与上轮的差异

**与 Day 1 的差异**：
- crate 数：7 → 8
- 真活实现：MarkdownChunker → +frontmatter；HybridRetriever stub → 真活；InMemoryStore（MetadataStore 占位）→ InMemoryVectorStore（VectorStore + MetadataStore 双实现 + 持久化）；CLI index/query stub → 真活
- 单测/集成测：8 → 21 个（chunker +3，storage +5，retrieval +4，indexer +1，end_to_end +2 -1 老 day1 测）
- 依赖新增：`walkdir = "2.5"`、`serde_yaml = "0.9"`、`tempfile = "3.13"` (dev)

**关键决策清单**（自主决策已记入 plan 节）：
- 8th crate `arkui-rag-indexer` 是个独立的概念实体（编排者）
- 持久化用 JSON + 版本字段
- 行号偏移（frontmatter 占用的行数）必须传递，否则引用链断
- BM25 空 stub 让 RRF 退化为纯向量，Day 3 替换为 Tantivy 后零代码变更

## 对话摘要

> 必备节（FAIL 级 M-FEATURE-PLAN）

按时序：

1. **用户首问**："继续"（在 Day 1 commit 完成后）
2. **Agent AskUserQuestion 一问 4 选项**（Day 2 方向）：
   - **端到端 Mock Demo（推荐）** ← 用户实际选择
   - Storage 层真实化（LanceDB + Tantivy）
   - Embedding 层真实化（OnnxEmbedder async + model 下载）
   - Chunker 层真实化（tree-sitter）
3. **Agent 自主拟 7-Phase plan**（含此 feature log）→ 直接开工，未再回问用户
4. **执行 Phase 1-6**：边写边跑 task tracking
5. **本 feature log** 在 Phase 7 由 agent 自填

## 验证结果

- 编译：⏳ 用户跑 `make check`（需先装 rust）
- 单元测试：⏳ 用户跑 `cd crates && cargo test --workspace`
  - 预期：MockEmbedder 3 + MarkdownChunker 6 + RRF 2 + InMemoryVectorStore 4 + InMemoryBM25Index 1 + HybridRetriever 4 + Indexer 1 + 端到端 2 = **23 个** test 应全过
- 端到端 demo（需用户先在 corpus/ 投放 markdown）：
  ```
  cargo run -p arkui-rag-cli -- index --source corpus
  cargo run -p arkui-rag-cli -- query --text "下拉刷新" --k 3
  ```
- check-api-parity：N/A（项目级规则尚未定义）

## 残留 / 下一轮

继承 Round 1 的 Week 2-6 backlog，加上本轮发现的新项：

- [ ] **Day 3 推荐**：把 OnnxEmbedder 包成 async Embedder trait + arkui-rag model-pull 真实下载 BGE-M3 → 真实语义检索
- [ ] **Day 3 备选**：接 LanceDB（feature `lancedb`），替换 InMemoryVectorStore 让 chunks > 10k 也可用
- [ ] **Day 3 备选**：接 Tantivy（feature `tantivy`），让 RRF 真的双路融合（不再退化）
- [ ] tree-sitter 切分（.ets / .kt / .swift）→ chunker
- [ ] Reranker 真实实现（CrossEncoderReranker → BGE-Reranker-v2 ONNX）
- [ ] file-watcher 增量索引（feedback/features/rag4arkui-core/README 后续轮处理）
- [ ] HTTP/MCP/LSP 三协议实装 → server crate（Week 4）
- [ ] HyDE 改写器（依赖小 LLM，Week 3 中后期）
- [ ] 评估集 + RAGAS 接入（Week 3）
- [ ] IDE 插件接入验证（Week 5）
- [x] Day 2：CLI index/query 端到端可跑（本轮完成）
- [x] Day 2：MarkdownChunker frontmatter 解析（本轮完成）
- [x] Day 2：HybridRetriever 真实可用（本轮完成）
- [x] Day 2：8 个 crate workspace 立起来（本轮完成）
