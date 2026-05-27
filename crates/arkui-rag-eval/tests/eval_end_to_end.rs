//! 端到端集成测试：完整的 index → eval 闭环。

use arkui_rag_chunker::MarkdownChunker;
use arkui_rag_embedding::MockEmbedder;
use arkui_rag_eval::{load_queries, render_markdown, EvalConfig, Evaluator};
use arkui_rag_indexer::Indexer;
use arkui_rag_retrieval::HybridRetriever;
use arkui_rag_storage::{InMemoryBM25Index, InMemoryVectorStore};
use std::sync::Arc;

#[tokio::test]
async fn full_index_then_eval() {
    let dir = tempfile::tempdir().unwrap();
    let corpus = dir.path().join("corpus");
    tokio::fs::create_dir_all(&corpus).await.unwrap();

    // 投放 fixture corpus
    tokio::fs::write(
        corpus.join("list.md"),
        "---\nplatforms: [HarmonyOS]\ntype: code_example\ntags: [list, refresh]\n---\n\n# List\n\n## 下拉刷新\nArkUI-X 用 Refresh 组件实现下拉刷新。\n",
    )
    .await
    .unwrap();
    tokio::fs::write(
        corpus.join("router.md"),
        "---\nplatforms: [HarmonyOS]\ntype: api_doc\ntags: [routing]\n---\n\n# Router\n\n## pushUrl\n推送新页面到路由栈。\n",
    )
    .await
    .unwrap();

    // 建索引
    let embedder = Arc::new(MockEmbedder::new(64));
    let vector = Arc::new(InMemoryVectorStore::new("mock-64", 64));
    let bm25 = Arc::new(InMemoryBM25Index);
    let chunker = Arc::new(MarkdownChunker::new());
    Indexer::new(chunker, embedder.clone(), vector.clone(), bm25.clone())
        .index_directory(&corpus)
        .await
        .unwrap();

    // 写评估集 YAML（MockEmbedder 对同样文本 cosine=1，所以 GT 用文档原文）
    let queries_path = dir.path().join("queries.yaml");
    let queries_yaml = r#"
- id: q1
  query: "ArkUI-X 用 Refresh 组件实现下拉刷新。"
  relevant:
    - "list.md#List/下拉刷新@9"
- id: q2
  query: "推送新页面到路由栈。"
  relevant:
    - "router.md#Router/pushUrl@9"
- id: q_miss
  query: "完全无关的天气预报"
  relevant:
    - "nonexistent.md#X@1"
"#;
    tokio::fs::write(&queries_path, queries_yaml).await.unwrap();

    // 跑评估
    let queries = load_queries(&queries_path).unwrap();
    assert_eq!(queries.len(), 3);

    let retriever: Arc<dyn arkui_rag_core::Retriever> =
        Arc::new(HybridRetriever::new(embedder, vector, bm25));
    let config = EvalConfig {
        embedder: "mock-64".into(),
        bm25: "memory".into(),
        rerank: "none".into(),
        pre_rerank_k: 50,
        index_path: "in-memory".into(),
        queries_path: queries_path.to_string_lossy().to_string(),
    };
    let summary = Evaluator::new(retriever)
        .with_k(5)
        .with_config(config)
        .run(&queries)
        .await
        .unwrap();

    assert_eq!(summary.total_queries, 3);
    // q1 / q2 应该命中（MockEmbedder 对原文 cosine=1）；q_miss 应漏
    assert!(
        summary.avg_recall_at_k > 0.5,
        "期望 avg recall > 0.5（q1 + q2 命中），实际 {:.3}",
        summary.avg_recall_at_k
    );
    assert!(summary.per_query[0].recall_at_k > 0.5);
    assert!(summary.per_query[1].recall_at_k > 0.5);
    assert_eq!(summary.per_query[2].recall_at_k, 0.0);

    // 渲染报告
    let md = render_markdown(&summary, "2026-05-27 test");
    assert!(md.contains("整体指标"));
    assert!(md.contains("q_miss"));
    assert!(md.contains("失败 query 详情"));
}
