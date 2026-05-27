//! LSP adapter（Day 1 stub）。
//!
//! 提供 textDocument 上的自定义命令：arkui-rag/validateApi、arkui-rag/migrate
//! Week 4 接 tower-lsp。

use arkui_rag_core::{RagError, Result};

pub async fn serve() -> Result<()> {
    tracing::warn!("LSP server 是 Day 1 stub — 仅打印命令约定。");
    println!("LSP custom commands (planned, Week 4):");
    println!("  arkui-rag/validateApi(uri, range) → diagnostics[]");
    println!("  arkui-rag/migrate(uri, range, target) → workspaceEdit");
    Err(RagError::NotImplemented(
        "LSP server - 见 Week 4 backlog".into(),
    ))
}
