//! HTTP/REST adapter（Day 14 真活）。
//!
//! 暴露当前所有检索能力供外部消费（IDE 插件 / curl / 外部 agent）。
//!
//! 路由：
//! - `GET  /health`        健康检查 + 配置摘要
//! - `GET  /corpus/list`   列 corpus/ 子目录
//! - `POST /search`        端到端检索（含可选 reranker / hyde / expand_parent）
//! - `POST /index`         触发索引（Day 14 stub · Week 4 接异步任务）
//!
//! 设计：所有依赖（Embedder / Retriever / Reranker / Enhancer / MetadataStore）
//! 由调用方（CLI）构造好后传入 `AppState`，server crate 不直接依赖具体后端。

#![cfg(feature = "http")]

use arkui_rag_core::{Citation, EnhancedQuery, QueryEnhancer, RagError, Reranker, Result, Retriever};
use arkui_rag_retrieval::{ContextAssembler, ExpandedHit};
use arkui_rag_storage::MetadataStore;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json as JsonResp},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;

/// 注入到 axum router 的应用状态。
///
/// CLI 在 `serve` 命令里构造后传给 `serve()`。
pub struct AppState {
    pub retriever: Arc<dyn Retriever>,
    pub reranker: Option<Arc<dyn Reranker>>,
    pub enhancer: Arc<dyn QueryEnhancer>,
    /// 可选 MetadataStore，用于 expand_parent；通常和 VectorStore 同一对象（InMemory/Lance）
    pub metadata_store: Option<Arc<dyn MetadataStore>>,
    pub pre_rerank_k: usize,
    pub embedder_model_id: String,
    pub embedder_dim: usize,
    pub bm25_name: String,
    pub vector_name: String,
}

/// 启动 HTTP server，监听到 SIGTERM 优雅退出。
pub async fn serve(opts: &super::HttpOptions, state: AppState) -> Result<()> {
    let app = build_router(Arc::new(state));
    let listener = tokio::net::TcpListener::bind(opts.addr)
        .await
        .map_err(|e| RagError::Other(anyhow::anyhow!("bind {}: {}", opts.addr, e)))?;
    tracing::info!("HTTP server listening on http://{}", opts.addr);
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(|e| RagError::Other(anyhow::anyhow!("axum::serve: {}", e)))?;
    tracing::info!("HTTP server shutdown complete");
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };
    #[cfg(unix)]
    let terminate = async {
        if let Ok(mut s) = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        {
            s.recv().await;
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

/// 构造 axum Router；分离出来便于 integration test 直接调用而不真起监听。
pub fn build_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/corpus/list", get(corpus_list))
        .route("/search", post(search))
        .route("/index", post(index_trigger))
        .with_state(state)
}

// ─── /health ──────────────────────────────────────────────

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    embedder: String,
    embedder_dim: usize,
    vector: String,
    bm25: String,
    rerank_enabled: bool,
    enhancer: String,
    pre_rerank_k: usize,
}

async fn health(State(state): State<Arc<AppState>>) -> JsonResp<HealthResponse> {
    JsonResp(HealthResponse {
        status: "ok",
        embedder: state.embedder_model_id.clone(),
        embedder_dim: state.embedder_dim,
        vector: state.vector_name.clone(),
        bm25: state.bm25_name.clone(),
        rerank_enabled: state.reranker.is_some(),
        enhancer: state.enhancer.name().to_string(),
        pre_rerank_k: state.pre_rerank_k,
    })
}

// ─── GET /corpus/list ──────────────────────────────────────

#[derive(Serialize)]
struct CorpusDir {
    name: String,
    exists: bool,
    docs: usize,
}

#[derive(Serialize)]
struct CorpusListResponse {
    dirs: Vec<CorpusDir>,
}

async fn corpus_list() -> JsonResp<CorpusListResponse> {
    let candidates = ["official", "samples", "migration", "errors", "custom"];
    let dirs = candidates
        .iter()
        .map(|d| {
            let path = std::path::Path::new("corpus").join(d);
            let exists = path.exists();
            let docs = if exists {
                std::fs::read_dir(&path)
                    .map(|rd| {
                        rd.filter_map(|e| e.ok())
                            .filter(|e| !e.file_name().to_string_lossy().starts_with('.'))
                            .count()
                    })
                    .unwrap_or(0)
            } else {
                0
            };
            CorpusDir {
                name: d.to_string(),
                exists,
                docs,
            }
        })
        .collect();
    JsonResp(CorpusListResponse { dirs })
}

// ─── POST /search ──────────────────────────────────────────

#[derive(Deserialize)]
struct SearchRequest {
    query: String,
    #[serde(default = "default_k")]
    k: usize,
    /// 是否走 enhancer (HyDE)。默认 false（用 PassthroughEnhancer 行为）
    #[serde(default)]
    enhance_query: bool,
    /// 是否扩展到父 chunk
    #[serde(default)]
    expand_parent: bool,
}

fn default_k() -> usize {
    5
}

#[derive(Serialize)]
struct HitDto {
    chunk_id: String,
    score: f32,
    source: String,
    citation: Citation,
    /// 内容预览（前 500 字符，full 可在 Day 14 续返回）
    content_preview: String,
    /// 若 expand_parent 启用且找到父：父 chunk 预览
    parent_preview: Option<String>,
    parent_chunk_id: Option<String>,
}

#[derive(Serialize)]
struct SearchResponse {
    hits: Vec<HitDto>,
    latency_ms: u128,
    embedder: String,
    bm25: String,
    rerank: Option<String>,
    enhancer: String,
}

async fn search(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SearchRequest>,
) -> std::result::Result<JsonResp<SearchResponse>, HttpError> {
    let start = Instant::now();

    let eq = if req.enhance_query {
        state.enhancer.enhance(&req.query).await.map_err(HttpError::from)?
    } else {
        EnhancedQuery::passthrough(req.query.clone())
    };

    let retrieve_k = if state.reranker.is_some() {
        state.pre_rerank_k.max(req.k)
    } else {
        req.k
    };
    let hits = state
        .retriever
        .retrieve(&eq, retrieve_k)
        .await
        .map_err(HttpError::from)?;
    let hits = if let Some(rr) = &state.reranker {
        rr.rerank(&req.query, hits, req.k)
            .await
            .map_err(HttpError::from)?
    } else {
        hits.into_iter().take(req.k).collect()
    };

    // 可选扩展到父
    let expanded: Option<Vec<ExpandedHit>> =
        if req.expand_parent {
            match &state.metadata_store {
                Some(store) => {
                    let asm = ContextAssembler::new(store.clone());
                    Some(asm.expand_to_parent(hits.clone()).await.map_err(HttpError::from)?)
                }
                None => None,
            }
        } else {
            None
        };

    let hits_dto: Vec<HitDto> = hits
        .iter()
        .enumerate()
        .map(|(i, h)| {
            let citation = Citation::from(h);
            let preview: String = h.chunk.content.chars().take(500).collect();
            let (parent_preview, parent_chunk_id) = expanded
                .as_ref()
                .and_then(|exs| exs.get(i))
                .and_then(|e| e.parent.as_ref())
                .map(|p| {
                    let prev: String = p.content.chars().take(500).collect();
                    (Some(prev), Some(p.id.as_str().to_string()))
                })
                .unwrap_or((None, None));
            HitDto {
                chunk_id: h.chunk.id.as_str().to_string(),
                score: h.score,
                source: format!("{:?}", h.source).to_lowercase(),
                citation,
                content_preview: preview,
                parent_preview,
                parent_chunk_id,
            }
        })
        .collect();

    Ok(JsonResp(SearchResponse {
        hits: hits_dto,
        latency_ms: start.elapsed().as_millis(),
        embedder: state.embedder_model_id.clone(),
        bm25: state.bm25_name.clone(),
        rerank: state.reranker.as_ref().map(|r| r.model_id().to_string()),
        enhancer: state.enhancer.name().to_string(),
    }))
}

// ─── POST /index (stub) ────────────────────────────────────

#[derive(Deserialize)]
struct IndexRequest {
    #[allow(dead_code)]
    source: String,
}

#[derive(Serialize)]
struct IndexResponse {
    status: &'static str,
    message: String,
}

async fn index_trigger(
    Json(req): Json<IndexRequest>,
) -> std::result::Result<JsonResp<IndexResponse>, HttpError> {
    // Day 14 stub：仅返回提示，建议用户走 CLI `arkui-rag index --source ...` 命令
    Ok(JsonResp(IndexResponse {
        status: "stub",
        message: format!(
            "POST /index 在 Day 14 是 stub。请用 CLI 'arkui-rag index --source {}'。Week 4+ 接异步任务。",
            req.source
        ),
    }))
}

// ─── 错误适配 ────────────────────────────────────────────

struct HttpError(RagError);
impl From<RagError> for HttpError {
    fn from(e: RagError) -> Self {
        Self(e)
    }
}
impl IntoResponse for HttpError {
    fn into_response(self) -> axum::response::Response {
        let status = match &self.0 {
            RagError::NotImplemented(_) => StatusCode::NOT_IMPLEMENTED,
            RagError::Config(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        let body = serde_json::json!({
            "error": format!("{}", self.0),
            "kind": match self.0 {
                RagError::Io(_) => "io",
                RagError::SerdeJson(_) => "serde_json",
                RagError::Embedding(_) => "embedding",
                RagError::Retrieval(_) => "retrieval",
                RagError::Rerank(_) => "rerank",
                RagError::Chunker(_) => "chunker",
                RagError::Storage(_) => "storage",
                RagError::Config(_) => "config",
                RagError::NotImplemented(_) => "not_implemented",
                RagError::Other(_) => "other",
            },
        });
        (status, JsonResp(body)).into_response()
    }
}
