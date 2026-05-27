#![doc = include_str!("../README.md")]

pub mod dispatcher;
pub mod markdown;

#[cfg(feature = "treesitter")]
mod treesitter_base;

#[cfg(feature = "typescript")]
pub mod typescript;

#[cfg(feature = "kotlin")]
pub mod kotlin;

#[cfg(feature = "swift")]
pub mod swift;

pub use dispatcher::ChunkerDispatcher;
pub use markdown::MarkdownChunker;

#[cfg(feature = "typescript")]
pub use typescript::TypeScriptChunker;

#[cfg(feature = "kotlin")]
pub use kotlin::KotlinChunker;

#[cfg(feature = "swift")]
pub use swift::SwiftChunker;
