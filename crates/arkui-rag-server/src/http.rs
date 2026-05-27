//! HTTP/REST adapter（Day 1 stub）。
//!
//! 路由：POST /search、POST /index、GET /health、GET /corpus/list
//! Week 4 起接入真实 axum router。

use arkui_rag_core::{RagError, Result};

pub async fn serve(_opts: &super::HttpOptions) -> Result<()> {
    tracing::warn!("HTTP server 是 Day 1 stub — 仅打印路由约定，不实际监听。");
    println!("HTTP routes (planned, Week 4):");
    println!("  POST /search       —— 混合检索 + Rerank");
    println!("  POST /index        —— 触发索引");
    println!("  GET  /health       —— 健康检查");
    println!("  GET  /corpus/list  —— 列出 corpus 子目录");
    Err(RagError::NotImplemented(
        "HTTP server - 见 Week 4 backlog".into(),
    ))
}
