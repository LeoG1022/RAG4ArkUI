#![doc = include_str!("../README.md")]

pub mod evaluator;
pub mod report;
pub mod types;

pub use evaluator::{load_queries, Evaluator};
pub use report::render_markdown;
pub use types::{EvalConfig, EvalQuery, EvalResult, EvalSummary};
