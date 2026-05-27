#![doc = include_str!("../README.md")]

pub mod mock;

#[cfg(feature = "onnx")]
pub mod onnx;

pub use mock::MockEmbedder;

#[cfg(feature = "onnx")]
pub use onnx::{EmbeddingModel, SharedEmbedding};
