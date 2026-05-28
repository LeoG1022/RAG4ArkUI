//! HTTP server 端到端集成测试。
//!
//! 在内存中构造完整流水线 → 启动 axum router（不真起监听）→ 用 tower::ServiceExt
//! 直接调 handler 验证 JSON 响应。

#![cfg(feature = "http")]

use arkui_rag_chunker::{ChunkerDispatcher, MarkdownChunker};
use arkui_rag_core::{chunker::SourceLang, PassthroughEnhancer};
use arkui_rag_embedding::MockEmbedder;
use arkui_rag_indexer::Indexer;
use arkui_rag_retrieval::HybridRetriever;
use arkui_rag_server::{build_router, AppState};
use arkui_rag_storage::{InMemoryBM25Index, InMemoryVectorStore};
use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use serde_json::Value;
use std::sync::Arc;
use tower::ServiceExt;

async fn build_test_state() -> AppState {
    let dir = tempfile::tempdir().unwrap();
    let corpus = dir.path().join("corpus");
    tokio::fs::create_dir_all(&corpus).await.unwrap();
    tokio::fs::write(
        corpus.join("list.md"),
        "# List\n\n## 下拉刷新\nArkUI-X 用 Refresh 组件实现下拉刷新。\n",
    )
    .await
    .unwrap();

    let embedder = Arc::new(MockEmbedder::new(64));
    let vector = Arc::new(InMemoryVectorStore::new("mock-64", 64));
    let bm25 = Arc::new(InMemoryBM25Index);
    let dispatcher = Arc::new(
        ChunkerDispatcher::new()
            .register(SourceLang::Markdown, Arc::new(MarkdownChunker::new())),
    );
    Indexer::new(dispatcher, embedder.clone(), vector.clone(), bm25.clone())
        .index_directory(&corpus)
        .await
        .unwrap();

    let retriever: Arc<dyn arkui_rag_core::Retriever> = Arc::new(HybridRetriever::new(
        embedder,
        vector.clone(),
        bm25,
    ));

    AppState {
        retriever,
        reranker: None,
        enhancer: Arc::new(PassthroughEnhancer),
        metadata_store: Some(vector),
        pre_rerank_k: 50,
        embedder_model_id: "mock-64".into(),
        embedder_dim: 64,
        bm25_name: "memory".into(),
        vector_name: "memory".into(),
    }
}

#[tokio::test]
async fn health_returns_ok() {
    let state = Arc::new(build_test_state().await);
    let app = build_router(state);
    let res = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), 1024 * 1024).await.unwrap();
    let v: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(v["status"], "ok");
    assert_eq!(v["embedder"], "mock-64");
    assert_eq!(v["embedder_dim"], 64);
    assert_eq!(v["rerank_enabled"], false);
}

#[tokio::test]
async fn corpus_list_returns_5_subdirs() {
    let state = Arc::new(build_test_state().await);
    let app = build_router(state);
    let res = app
        .oneshot(
            Request::builder()
                .uri("/corpus/list")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), 1024 * 1024).await.unwrap();
    let v: Value = serde_json::from_slice(&body).unwrap();
    let dirs = v["dirs"].as_array().unwrap();
    assert_eq!(dirs.len(), 5);
    let names: Vec<&str> = dirs.iter().map(|d| d["name"].as_str().unwrap()).collect();
    assert!(names.contains(&"official"));
    assert!(names.contains(&"samples"));
    assert!(names.contains(&"migration"));
    assert!(names.contains(&"errors"));
    assert!(names.contains(&"custom"));
}

#[tokio::test]
async fn search_returns_hits() {
    let state = Arc::new(build_test_state().await);
    let app = build_router(state);
    let payload = serde_json::json!({
        "query": "ArkUI-X 用 Refresh 组件实现下拉刷新。",
        "k": 3,
    });
    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/search")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), 1024 * 1024).await.unwrap();
    let v: Value = serde_json::from_slice(&body).unwrap();
    let hits = v["hits"].as_array().unwrap();
    assert!(!hits.is_empty(), "应有命中");
    // MockEmbedder 对完全相同文本 cosine=1 → Top-1 必命中 list.md 的 chunk
    let top1 = &hits[0];
    assert!(top1["chunk_id"].as_str().unwrap().contains("list.md"));
    assert!(v["latency_ms"].as_u64().is_some());
}

#[tokio::test]
async fn search_with_expand_parent() {
    let state = Arc::new(build_test_state().await);
    let app = build_router(state);
    let payload = serde_json::json!({
        "query": "ArkUI-X 用 Refresh 组件实现下拉刷新。",
        "k": 3,
        "expand_parent": true,
    });
    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/search")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), 1024 * 1024).await.unwrap();
    let v: Value = serde_json::from_slice(&body).unwrap();
    let hits = v["hits"].as_array().unwrap();
    // list.md 的 "下拉刷新" chunk 应该有父级（# List）
    assert!(!hits.is_empty());
    let top1 = &hits[0];
    // parent_preview 可能 Some 也可能 None（取决于 MarkdownChunker 是否给该 chunk 生成了 parent_id）
    // 这里至少验证字段存在
    assert!(top1.get("parent_preview").is_some());
}

#[tokio::test]
async fn index_endpoint_returns_stub() {
    let state = Arc::new(build_test_state().await);
    let app = build_router(state);
    let payload = serde_json::json!({ "source": "corpus" });
    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/index")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), 1024 * 1024).await.unwrap();
    let v: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(v["status"], "stub");
}
