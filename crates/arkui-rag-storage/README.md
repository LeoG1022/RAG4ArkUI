# arkui-rag-storage

**定位**：存储层接口 + 后端适配器。

| Trait | 默认实现 | feature-gated 实现 |
|---|---|---|
| `VectorStore` | `InMemoryVectorStore`（cosine 暴力扫 + JSON 持久化）| LanceDB ⏳ Week 2 续 |
| `BM25Index` | `InMemoryBM25Index`（空 stub）| **`TantivyBM25Index`（Day 4 新增，feature `tantivy`）** |
| `MetadataStore` | `InMemoryVectorStore`（双实现）| SQLite ⏳ Week 2 续 |

技术方案对应：§4.2 决策 4、§4.5 双轨知识库。

## Day 4 新增：TantivyBM25Index

`feature = "tantivy"` 启用后，`TantivyBM25Index` 提供真实的 BM25 倒排检索：
- 字段：id/content/heading_path/source/platforms/api_version/chunk_type/tags/deprecated
- 中文友好：注册 `ngram(2,3)` tokenizer（生产建议接 `tantivy-jieba`）
- 完整 ChunkMetadata 序列化到 `meta_json` stored 字段 → search 返回完整 Hit
- 跨进程持久化：commit 即落盘，新进程 `open()` 即可
- 元数据过滤：platforms / api_version / tags / deprecated 全支持
- 7 单测覆盖：upsert/search/delete/filter/deprecated/empty-query/reopen-persist

### 启用方式

```bash
# 单独编译 storage crate
cargo check -p arkui-rag-storage --features tantivy

# 通过 CLI 启用（Day 4 后）
cargo build -p arkui-rag-cli --features tantivy
```

### 用法

```rust
# #[cfg(feature = "tantivy")]
# tokio_test::block_on(async {
use arkui_rag_core::{Chunk, ChunkId, ChunkMetadata, QueryFilters};
use arkui_rag_storage::{TantivyBM25Index, BM25Index};
use std::path::Path;

let bm = TantivyBM25Index::open(Path::new("/tmp/my-bm25"))?;
let chunk = Chunk {
    id: ChunkId::new("a"),
    content: "ArkUI-X 用 Refresh 组件实现下拉刷新".into(),
    metadata: ChunkMetadata::default(),
};
bm.upsert(&[chunk]).await?;
let hits = bm.search("下拉刷新", 5, &QueryFilters::default()).await?;
# Ok::<_, arkui_rag_core::RagError>(())
# });
```

## 设计要点

- **trait 不变**：`BM25Index` 接口 Day 1 就锁定，Tantivy 实现是后端替换不破坏上游
- **HybridRetriever 自动受益**：RRF 真正双路融合（之前 BM25 路径返回空，名实不符）
- **持久化路径约定**：CLI 把 BM25 索引存到 `<index-path-dir>/bm25/`，与 vector 索引并列
