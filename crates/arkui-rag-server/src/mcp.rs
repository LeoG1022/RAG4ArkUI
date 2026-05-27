//! MCP (Model Context Protocol) adapter（Day 1 stub）。
//!
//! 4 个工具规约见 README。Week 4 接 MCP Rust SDK。

use arkui_rag_core::{RagError, Result};

pub async fn serve_stdio() -> Result<()> {
    tracing::warn!("MCP server 是 Day 1 stub — 仅打印工具约定，不监听 stdio。");
    println!("MCP tools (planned, Week 4):");
    println!("  arkui_search_docs(query, platform_filter, top_k)");
    println!("  arkui_search_code(query, mode, top_k)");
    println!("  arkui_migrate_snippet(source_code, from, to)");
    println!("  arkui_validate_api(code)");
    Err(RagError::NotImplemented(
        "MCP server - 见 Week 4 backlog".into(),
    ))
}
