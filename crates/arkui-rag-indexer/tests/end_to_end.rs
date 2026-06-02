//! 端到端集成测试：模拟 CLI 的 index → save → load → query 闭环。

use arkui_rag_chunker::{ChunkerDispatcher, MarkdownChunker};
use arkui_rag_core::{chunker::SourceLang, EnhancedQuery, Retriever};
use arkui_rag_embedding::MockEmbedder;
use arkui_rag_indexer::Indexer;
use arkui_rag_retrieval::HybridRetriever;
use arkui_rag_storage::{InMemoryBM25Index, InMemoryVectorStore, VectorStore};
use std::sync::Arc;

fn dispatcher_markdown() -> Arc<ChunkerDispatcher> {
    Arc::new(
        ChunkerDispatcher::new().register(SourceLang::Markdown, Arc::new(MarkdownChunker::new())),
    )
}

#[tokio::test]
async fn index_save_load_query_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let corpus = dir.path().join("corpus");
    tokio::fs::create_dir_all(&corpus).await.unwrap();

    // 投放两份带 frontmatter 的 markdown
    tokio::fs::write(
        corpus.join("router.md"),
        "---\nplatforms: [HarmonyOS]\napi_version: \"ArkUI-X 1.2\"\ntype: api_doc\ntags: [routing]\n---\n\n# Router\n\n## pushUrl\n推送新页面到路由栈。\n\n## back\n返回上一页。\n",
    )
    .await
    .unwrap();
    tokio::fs::write(
        corpus.join("list.md"),
        "---\nplatforms: [HarmonyOS, Android]\ntype: code_example\ntags: [list, refresh]\n---\n\n# List\n\n## 下拉刷新\nArkUI-X 用 Refresh 组件实现下拉刷新。\n",
    )
    .await
    .unwrap();

    // 建索引（用 MockEmbedder 维持确定性）
    let dim = 64;
    let embedder = Arc::new(MockEmbedder::new(dim));
    let vector = Arc::new(InMemoryVectorStore::new("mock-64", dim));
    let bm25 = Arc::new(InMemoryBM25Index);

    let indexer = Indexer::new(
        dispatcher_markdown(),
        embedder.clone(),
        vector.clone(),
        bm25.clone(),
    );
    let stats = indexer.index_directory(&corpus).await.unwrap();
    assert_eq!(stats.files, 2);
    assert!(
        stats.chunks >= 3,
        "expected ≥3 chunks, got {}",
        stats.chunks
    );

    // 持久化 + 重载（模拟 CLI 跨进程）
    let index_path = dir.path().join("idx.json");
    vector.save_to(&index_path).await.unwrap();

    let reloaded_vec = Arc::new(InMemoryVectorStore::load_from(&index_path).await.unwrap());
    assert_eq!(reloaded_vec.len().await.unwrap(), stats.chunks);

    // 用同一 MockEmbedder 跑检索（编码确定性 → 同样文本同样向量 → cosine=1）
    let retriever = HybridRetriever::new(embedder, reloaded_vec, bm25);

    // Q1：查"下拉刷新" → 应命中 list.md#下拉刷新
    let q = EnhancedQuery::passthrough("ArkUI-X 用 Refresh 组件实现下拉刷新。");
    let hits = retriever.retrieve(&q, 3).await.unwrap();
    assert!(!hits.is_empty(), "应有 hits");
    assert_eq!(
        hits[0].chunk.metadata.source,
        "list.md",
        "Top-1 应来自 list.md，实际：{} hits={:?}",
        hits[0].chunk.metadata.source,
        hits.iter().map(|h| h.chunk.id.as_str()).collect::<Vec<_>>()
    );
    // frontmatter 元数据正确继承
    assert!(
        !hits[0].chunk.metadata.tags.is_empty(),
        "list.md chunk 应该继承 frontmatter tags"
    );

    // Q2：查"pushUrl" → 应命中 router.md
    let q2 = EnhancedQuery::passthrough("推送新页面到路由栈。");
    let hits2 = retriever.retrieve(&q2, 3).await.unwrap();
    assert_eq!(hits2[0].chunk.metadata.source, "router.md");
}

#[tokio::test]
async fn platform_filter_works_end_to_end() {
    use arkui_rag_core::{chunk::Platform, QueryFilters};

    let dir = tempfile::tempdir().unwrap();
    let corpus = dir.path().join("corpus");
    tokio::fs::create_dir_all(&corpus).await.unwrap();
    tokio::fs::write(
        corpus.join("harmony.md"),
        "---\nplatforms: [HarmonyOS]\n---\n\n# X\nharmony body\n",
    )
    .await
    .unwrap();
    tokio::fs::write(
        corpus.join("android.md"),
        "---\nplatforms: [Android]\n---\n\n# X\nandroid body\n",
    )
    .await
    .unwrap();

    let embedder = Arc::new(MockEmbedder::new(32));
    let vector = Arc::new(InMemoryVectorStore::new("mock-32", 32));
    let bm25 = Arc::new(InMemoryBM25Index);

    Indexer::new(
        dispatcher_markdown(),
        embedder.clone(),
        vector.clone(),
        bm25.clone(),
    )
    .index_directory(&corpus)
    .await
    .unwrap();

    let retriever = HybridRetriever::new(embedder, vector, bm25);
    let q = EnhancedQuery {
        raw: "body".into(),
        rewritten: "body".into(),
        hyde_doc: None,
        entities: vec![],
        intent: Default::default(),
        filters: QueryFilters {
            platforms: vec![Platform::HarmonyOs],
            ..Default::default()
        },
    };
    let hits = retriever.retrieve(&q, 10).await.unwrap();
    assert_eq!(hits.len(), 1, "platform filter 应只保留 1 个");
    assert_eq!(hits[0].chunk.metadata.source, "harmony.md");
}
