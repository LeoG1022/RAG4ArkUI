#![doc = include_str!("../README.md")]

pub mod mock;

#[cfg(feature = "onnx")]
pub mod onnx;

#[cfg(feature = "onnx")]
pub mod onnx_embedder;

pub use mock::MockEmbedder;

#[cfg(feature = "onnx")]
pub use onnx::{EmbeddingModel, SharedEmbedding};

#[cfg(feature = "onnx")]
pub use onnx_embedder::OnnxEmbedder;
