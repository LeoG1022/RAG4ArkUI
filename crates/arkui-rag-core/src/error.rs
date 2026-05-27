//! 统一错误类型。所有 crate 都向上 `?` 到 `RagError`。

use thiserror::Error;

pub type Result<T, E = RagError> = std::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum RagError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("serde_json error: {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[error("embedding error: {0}")]
    Embedding(String),

    #[error("retrieval error: {0}")]
    Retrieval(String),

    #[error("rerank error: {0}")]
    Rerank(String),

    #[error("chunker error: {0}")]
    Chunker(String),

    #[error("storage error: {0}")]
    Storage(String),

    #[error("config error: {0}")]
    Config(String),

    #[error("not implemented (Day 1 stub): {0}")]
    NotImplemented(String),

    #[error("other: {0}")]
    Other(#[from] anyhow::Error),
}
