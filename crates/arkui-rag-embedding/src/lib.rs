#![doc = include_str!("../README.md")]

pub mod mock;

#[cfg(feature = "onnx")]
pub mod onnx;

#[cfg(feature = "onnx")]
pub mod onnx_embedder;

#[cfg(feature = "onnx")]
pub mod reranker_onnx;

#[cfg(feature = "onnx")]
pub mod onnx_reranker;

pub use mock::MockEmbedder;

#[cfg(feature = "onnx")]
pub use onnx::{EmbeddingModel, SharedEmbedding};

#[cfg(feature = "onnx")]
pub use onnx_embedder::OnnxEmbedder;

#[cfg(feature = "onnx")]
pub use onnx_reranker::OnnxReranker;

#[cfg(feature = "onnx")]
pub use reranker_onnx::{RerankerModel, SharedReranker};
