#![doc = include_str!("../README.md")]

#[cfg(feature = "http")]
pub mod http;
pub mod lsp;
#[cfg(feature = "mcp")]
pub mod mcp;

#[cfg(feature = "http")]
pub use http::{build_router, serve as serve_http, AppState as HttpAppState};

// AppState 在 http feature 内定义；为统一对外暴露，alias 为 AppState
#[cfg(feature = "http")]
pub use http::AppState;

// 当只启 mcp 不启 http 时，AppState 类型不可用 → 给一个独立定义
#[cfg(all(feature = "mcp", not(feature = "http")))]
pub mod app_state_mcp_only {
    use arkui_rag_core::{QueryEnhancer, Reranker, Retriever};
    use arkui_rag_storage::MetadataStore;
    use std::sync::Arc;
    pub struct AppState {
        pub retriever: Arc<dyn Retriever>,
        pub reranker: Option<Arc<dyn Reranker>>,
        pub enhancer: Arc<dyn QueryEnhancer>,
        pub metadata_store: Option<Arc<dyn MetadataStore>>,
        pub pre_rerank_k: usize,
        pub embedder_model_id: String,
        pub embedder_dim: usize,
        pub bm25_name: String,
        pub vector_name: String,
    }
}
#[cfg(all(feature = "mcp", not(feature = "http")))]
pub use app_state_mcp_only::AppState;

#[cfg(feature = "mcp")]
pub use mcp::serve_stdio as serve_mcp_stdio;

/// 服务启动选项。`arkui-rag-cli` 的 `serve` subcommand 直接构造此结构传入。
#[derive(Debug, Clone, Default)]
pub struct ServeOptions {
    pub http: Option<HttpOptions>,
    pub mcp: bool,
    pub lsp: bool,
}

#[derive(Debug, Clone)]
pub struct HttpOptions {
    pub addr: std::net::SocketAddr,
}

impl Default for HttpOptions {
    fn default() -> Self {
        Self {
            addr: "127.0.0.1:7654".parse().expect("valid default addr"),
        }
    }
}
