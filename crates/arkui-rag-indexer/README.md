# arkui-rag-indexer

**定位**：索引流水线编排。把 chunker + embedder + storage 三者串联起来，给定一个 corpus 目录，产出可被 retriever 检索的索引。

```text
walk(corpus/) → dispatch by ext → ASTChunker.chunk()
              → batch Embedder.encode()
              → VectorStore.upsert()
              → BM25Index.upsert()
              → save_to(corpus/_index/index.json)
```

技术方案对应：§9 图 5 索引流程图、§4.2 决策 6 chunking 策略。

## Day 2 状态

- ✅ `Indexer` struct + `index_directory(path) -> IndexStats`
- ✅ 按扩展名 dispatch：`.md` → MarkdownChunker；其他 → 跳过 + warn
- ✅ 批量 embed（batch_size 默认 32）
- ✅ 通过 trait object 接收 Embedder / VectorStore / BM25Index → 后端可热插拔
- ⏳ Week 2 续：tree-sitter 切分（`.ets` / `.kt` / `.swift`）
- ⏳ Week 2 续：file-watcher 增量索引

## 用法

```rust,ignore
use std::sync::Arc;
use std::path::Path;
use arkui_rag_chunker::MarkdownChunker;
use arkui_rag_embedding::MockEmbedder;
use arkui_rag_storage::{InMemoryVectorStore, InMemoryBM25Index};
use arkui_rag_indexer::Indexer;

# tokio_test::block_on(async {
let embedder = Arc::new(MockEmbedder::new(1024));
let vector = Arc::new(InMemoryVectorStore::new("mock-1024", 1024));
let bm25 = Arc::new(InMemoryBM25Index);
let chunker = Arc::new(MarkdownChunker::new());

let indexer = Indexer::new(chunker, embedder, vector.clone(), bm25);
let stats = indexer.index_directory(Path::new("corpus")).await.unwrap();
println!("indexed {} files / {} chunks in {} ms", stats.files, stats.chunks, stats.elapsed_ms);
# });
```
