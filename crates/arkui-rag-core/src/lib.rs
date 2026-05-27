#![doc = include_str!("../README.md")]

pub mod chunk;
pub mod chunker;
pub mod embedder;
pub mod error;
pub mod hit;
pub mod query;
pub mod reranker;
pub mod retriever;

pub use chunk::{Chunk, ChunkId, ChunkMetadata, ChunkType, Platform};
pub use chunker::ASTChunker;
pub use embedder::Embedder;
pub use error::{RagError, Result};
pub use hit::{Citation, Hit};
pub use query::{EnhancedQuery, QueryFilters, QueryIntent};
pub use reranker::Reranker;
pub use retriever::Retriever;
