//! LSP (Language Server Protocol) adapter —— Day 16 真活（minimal）。
//!
//! 协议：JSON-RPC 2.0 over stdio · **LSP framing**（Content-Length header 分隔消息）。
//! 不引入 tower-lsp 第三方依赖（与 mcp.rs 决策一致），手撸 framing + handlers。
//!
//! 实装的 method：
//! - `initialize` ✅ 真活（返回 capabilities）
//! - `initialized` ✅ notification（无响应）
//! - `shutdown` ✅ 真活（返回 null）
//! - `exit` ✅ notification（停 server）
//! - `arkui-rag/search` ✅ 真活（custom request · 接 retriever 返回 markdown）
//! - `arkui-rag/migrate` ⏳ stub
//! - `textDocument/hover` ⏳ stub（capability 声明 true · 返回 null · Day 16 续真活）
//!
//! 与 mcp.rs 关键差异：
//! - LSP 用 `Content-Length: N\r\n\r\n<body>` 包装（不是行分隔）
//! - LSP 有 `shutdown` + `exit` 双阶段优雅退出
//! - 暴露 hover/completion 等 capability（IDE 编辑器内联交互）
//!
//! IDE 接入（DevEco Studio / IntelliJ Plugin）通过自定义 request 调用：
//! ```text
//! Client → arkui-rag/search { query, top_k }
//! Server ← { content: { type: "markdown", text: "..." } }
//! ```

#![cfg(feature = "lsp")]

use crate::AppState;
use arkui_rag_core::{EnhancedQuery, Hit, RagError, Result};
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};

/// 启动 LSP stdio loop（Content-Length 包装的 JSON-RPC）。
pub async fn serve_stdio(state: AppState) -> Result<()> {
    let state = Arc::new(state);
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    let mut reader = BufReader::new(stdin);
    let mut writer = stdout;
    let mut shutdown_requested = false;

    tracing::info!(
        "LSP stdio server ready. embedder={} bm25={} vector={}",
        state.embedder_model_id,
        state.bm25_name,
        state.vector_name
    );

    while let Some(body) = read_message(&mut reader)
        .await
        .map_err(|e| RagError::Other(anyhow::anyhow!("LSP read: {}", e)))?
    {
        let (response, is_exit) = handle_body(&state, &body, &mut shutdown_requested).await;
        if let Some(resp) = response {
            write_message(&mut writer, &resp)
                .await
                .map_err(|e| RagError::Other(anyhow::anyhow!("LSP write: {}", e)))?;
        }
        if is_exit {
            tracing::info!("LSP exit received · stopping");
            break;
        }
    }
    Ok(())
}

/// 读取一条 LSP 消息：先解析 headers (Content-Length)，再读 body。
/// 返回 Ok(None) 表示 EOF。
async fn read_message(
    reader: &mut BufReader<tokio::io::Stdin>,
) -> std::io::Result<Option<String>> {
    let mut content_length: Option<usize> = None;
    loop {
        let mut header = String::new();
        let n = reader.read_line(&mut header).await?;
        if n == 0 {
            return Ok(None); // EOF
        }
        let trimmed = header.trim_end_matches(|c| c == '\r' || c == '\n');
        if trimmed.is_empty() {
            break;
        }
        if let Some(value) = trimmed.strip_prefix("Content-Length: ") {
            content_length = value.parse().ok();
        }
        // 其他 header（如 Content-Type）忽略
    }
    let len = match content_length {
        Some(n) => n,
        None => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "LSP: missing Content-Length header",
            ));
        }
    };
    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf).await?;
    let body = String::from_utf8(buf).map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::InvalidData, format!("utf8: {}", e))
    })?;
    Ok(Some(body))
}

async fn write_message(
    writer: &mut tokio::io::Stdout,
    body: &str,
) -> std::io::Result<()> {
    let bytes = body.as_bytes();
    let header = format!("Content-Length: {}\r\n\r\n", bytes.len());
    writer.write_all(header.as_bytes()).await?;
    writer.write_all(bytes).await?;
    writer.flush().await?;
    Ok(())
}

/// 处理一条消息 body，返回 (Some(response_json) 或 None notification, is_exit)。
pub async fn handle_body(
    state: &Arc<AppState>,
    body: &str,
    shutdown_requested: &mut bool,
) -> (Option<String>, bool) {
    let req: Value = match serde_json::from_str(body) {
        Ok(v) => v,
        Err(e) => {
            return (
                Some(error_response(None, -32700, &format!("parse error: {}", e))),
                false,
            );
        }
    };
    let id = req.get("id").cloned();
    let method = req.get("method").and_then(|v| v.as_str()).unwrap_or("");
    let params = req.get("params").cloned().unwrap_or(json!({}));
    let is_notification = id.is_none();

    // exit notification 直接停 server
    if method == "exit" {
        return (None, true);
    }

    // shutdown 后只允许 exit · 其他 method 报错
    if *shutdown_requested && method != "exit" {
        if is_notification {
            return (None, false);
        }
        return (
            Some(error_response(
                id,
                -32600,
                "server is shutdown · only 'exit' allowed",
            )),
            false,
        );
    }

    let result: std::result::Result<Value, (i32, String)> = match method {
        "initialize" => Ok(handle_initialize(&params)),
        "initialized" => return (None, false),
        "shutdown" => {
            *shutdown_requested = true;
            Ok(Value::Null)
        }
        "arkui-rag/search" => arkui_search(state, &params).await,
        "arkui-rag/migrate" => arkui_migrate(state, &params).await,
        "textDocument/hover" => Ok(Value::Null), // stub · Day 16 续真活
        "textDocument/didOpen"
        | "textDocument/didChange"
        | "textDocument/didClose"
        | "$/cancelRequest"
        | "$/setTrace" => return (None, false), // 已知 notification · 忽略
        _ => Err((-32601, format!("method not found: {}", method))),
    };

    if is_notification {
        return (None, false);
    }

    let resp = match result {
        Ok(v) => success_response(id, v),
        Err((code, msg)) => error_response(id, code, &msg),
    };
    (Some(resp), false)
}

fn success_response(id: Option<Value>, result: Value) -> String {
    serde_json::to_string(&json!({
        "jsonrpc": "2.0",
        "id": id.unwrap_or(Value::Null),
        "result": result,
    }))
    .unwrap()
}

fn error_response(id: Option<Value>, code: i32, message: &str) -> String {
    serde_json::to_string(&json!({
        "jsonrpc": "2.0",
        "id": id.unwrap_or(Value::Null),
        "error": { "code": code, "message": message },
    }))
    .unwrap()
}

// ─── initialize ─────────────────────────────────────────────

fn handle_initialize(_params: &Value) -> Value {
    json!({
        "capabilities": {
            "textDocumentSync": 0,    // None · Day 16 simplification
            "hoverProvider": true,    // capability 声明，但 handler 是 stub
            "executeCommandProvider": {
                "commands": [
                    "arkui-rag.search",
                    "arkui-rag.migrate"
                ]
            }
        },
        "serverInfo": {
            "name": "arkui-rag",
            "version": env!("CARGO_PKG_VERSION"),
        }
    })
}

// ─── arkui-rag/search ──────────────────────────────────────

async fn arkui_search(
    state: &Arc<AppState>,
    params: &Value,
) -> std::result::Result<Value, (i32, String)> {
    let query = params
        .get("query")
        .and_then(|v| v.as_str())
        .ok_or((-32602, "missing param: query".into()))?;
    let top_k = params.get("top_k").and_then(|v| v.as_u64()).unwrap_or(5) as usize;

    let eq = EnhancedQuery::passthrough(query.to_string());
    let retrieve_k = if state.reranker.is_some() {
        state.pre_rerank_k.max(top_k)
    } else {
        top_k
    };
    let hits = state
        .retriever
        .retrieve(&eq, retrieve_k)
        .await
        .map_err(|e| (-32603, format!("retrieve: {}", e)))?;
    let hits = if let Some(rr) = &state.reranker {
        rr.rerank(query, hits, top_k)
            .await
            .map_err(|e| (-32603, format!("rerank: {}", e)))?
    } else {
        hits.into_iter().take(top_k).collect()
    };

    let text = render_hits_as_markdown(&hits);
    Ok(json!({
        "content": {
            "kind": "markdown",
            "value": text
        },
        "hits": hits.iter().map(|h| json!({
            "chunk_id": h.chunk.id.as_str(),
            "source": h.chunk.metadata.source,
            "heading_path": h.chunk.metadata.heading_path,
            "line_range": h.chunk.metadata.line_range,
            "score": h.score,
        })).collect::<Vec<_>>()
    }))
}

fn render_hits_as_markdown(hits: &[Hit]) -> String {
    let mut out = String::new();
    out.push_str(&format!("# 📚 RAG4ArkUI 检索结果 · Top-{}\n\n", hits.len()));
    if hits.is_empty() {
        out.push_str("⚠️  无命中。\n");
        return out;
    }
    for (i, h) in hits.iter().enumerate() {
        let heading = if h.chunk.metadata.heading_path.is_empty() {
            "(root)".to_string()
        } else {
            h.chunk.metadata.heading_path.join(" > ")
        };
        let lines = h
            .chunk
            .metadata
            .line_range
            .map(|(a, b)| format!("L{}-{}", a, b))
            .unwrap_or_else(|| "L?".to_string());
        out.push_str(&format!(
            "## [{}] `{}` · {} · score={:.4}\n\n",
            i + 1,
            h.chunk.metadata.source,
            lines,
            h.score
        ));
        out.push_str(&format!("**Heading**: {}\n\n", heading));
        let body: String = h.chunk.content.chars().take(500).collect();
        out.push_str(&format!("```\n{}\n```\n\n", body));
    }
    out
}

// ─── arkui-rag/migrate (stub) ─────────────────────────────

async fn arkui_migrate(
    _state: &Arc<AppState>,
    params: &Value,
) -> std::result::Result<Value, (i32, String)> {
    let source_code = params
        .get("source_code")
        .and_then(|v| v.as_str())
        .ok_or((-32602, "missing param: source_code".into()))?;
    let from = params.get("from").and_then(|v| v.as_str()).unwrap_or("unknown");
    let _ = source_code;
    Ok(json!({
        "status": "stub",
        "message": format!("arkui-rag/migrate ({} → arkui-x) 是 Day 16 stub · 需 LLM 真生成代码 · Week 4-5 续", from)
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use arkui_rag_core::{
        Chunk, ChunkId, ChunkMetadata, ChunkType, HitSource, PassthroughEnhancer, Retriever,
    };
    use async_trait::async_trait;

    struct StubRetriever;

    #[async_trait]
    impl Retriever for StubRetriever {
        fn name(&self) -> &str {
            "stub"
        }
        async fn retrieve(&self, _q: &EnhancedQuery, top_k: usize) -> Result<Vec<Hit>> {
            Ok((0..top_k)
                .map(|i| Hit {
                    chunk: Chunk {
                        id: ChunkId::new(format!("doc/{}.md#root@{}", i, i + 1)),
                        content: format!("content {}", i),
                        metadata: ChunkMetadata {
                            source: format!("doc/{}.md", i),
                            heading_path: vec![format!("Section {}", i)],
                            line_range: Some((i as u32 + 1, i as u32 + 10)),
                            r#type: ChunkType::CodeExample,
                            ..ChunkMetadata::default()
                        },
                    },
                    score: 1.0 - (i as f32 * 0.1),
                    source: HitSource::Hybrid,
                })
                .collect())
        }
    }

    fn test_state() -> Arc<AppState> {
        Arc::new(AppState {
            retriever: Arc::new(StubRetriever),
            reranker: None,
            enhancer: Arc::new(PassthroughEnhancer),
            metadata_store: None,
            pre_rerank_k: 50,
            embedder_model_id: "mock-64".into(),
            embedder_dim: 64,
            bm25_name: "memory".into(),
            vector_name: "memory".into(),
        })
    }

    #[tokio::test]
    async fn initialize_returns_capabilities() {
        let state = test_state();
        let mut sd = false;
        let body = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#;
        let (resp, exit) = handle_body(&state, body, &mut sd).await;
        assert!(!exit);
        let v: Value = serde_json::from_str(&resp.unwrap()).unwrap();
        assert_eq!(v["jsonrpc"], "2.0");
        assert_eq!(v["id"], 1);
        assert_eq!(v["result"]["serverInfo"]["name"], "arkui-rag");
        assert_eq!(v["result"]["capabilities"]["hoverProvider"], true);
        let commands = v["result"]["capabilities"]["executeCommandProvider"]["commands"]
            .as_array()
            .unwrap();
        assert_eq!(commands.len(), 2);
    }

    #[tokio::test]
    async fn arkui_search_returns_markdown() {
        let state = test_state();
        let mut sd = false;
        let body = r#"{"jsonrpc":"2.0","id":2,"method":"arkui-rag/search","params":{"query":"test","top_k":3}}"#;
        let (resp, exit) = handle_body(&state, body, &mut sd).await;
        assert!(!exit);
        let v: Value = serde_json::from_str(&resp.unwrap()).unwrap();
        assert_eq!(v["result"]["content"]["kind"], "markdown");
        let text = v["result"]["content"]["value"].as_str().unwrap();
        assert!(text.contains("Top-3"));
        assert!(text.contains("doc/0.md"));
        let hits = v["result"]["hits"].as_array().unwrap();
        assert_eq!(hits.len(), 3);
        assert_eq!(hits[0]["chunk_id"], "doc/0.md#root@1");
    }

    #[tokio::test]
    async fn shutdown_then_exit_flow() {
        let state = test_state();
        let mut sd = false;

        // shutdown returns null result
        let body = r#"{"jsonrpc":"2.0","id":3,"method":"shutdown"}"#;
        let (resp, exit) = handle_body(&state, body, &mut sd).await;
        assert!(!exit);
        assert!(sd, "shutdown_requested 应为 true");
        let v: Value = serde_json::from_str(&resp.unwrap()).unwrap();
        assert_eq!(v["result"], Value::Null);

        // 之后非 exit method 应被拒
        let body2 = r#"{"jsonrpc":"2.0","id":4,"method":"arkui-rag/search","params":{"query":"x"}}"#;
        let (resp2, exit2) = handle_body(&state, body2, &mut sd).await;
        assert!(!exit2);
        let v2: Value = serde_json::from_str(&resp2.unwrap()).unwrap();
        assert!(v2["error"]["message"].as_str().unwrap().contains("shutdown"));

        // exit notification → is_exit=true
        let body3 = r#"{"jsonrpc":"2.0","method":"exit"}"#;
        let (resp3, exit3) = handle_body(&state, body3, &mut sd).await;
        assert!(exit3);
        assert!(resp3.is_none());
    }

    #[tokio::test]
    async fn hover_is_stub_returns_null() {
        let state = test_state();
        let mut sd = false;
        let body = r#"{"jsonrpc":"2.0","id":5,"method":"textDocument/hover","params":{}}"#;
        let (resp, _) = handle_body(&state, body, &mut sd).await;
        let v: Value = serde_json::from_str(&resp.unwrap()).unwrap();
        assert_eq!(v["result"], Value::Null);
    }

    #[tokio::test]
    async fn unknown_method_returns_error() {
        let state = test_state();
        let mut sd = false;
        let body = r#"{"jsonrpc":"2.0","id":6,"method":"foo/bar"}"#;
        let (resp, _) = handle_body(&state, body, &mut sd).await;
        let v: Value = serde_json::from_str(&resp.unwrap()).unwrap();
        assert_eq!(v["error"]["code"], -32601);
    }

    #[tokio::test]
    async fn notification_no_response() {
        let state = test_state();
        let mut sd = false;
        // initialized is notification
        let body = r#"{"jsonrpc":"2.0","method":"initialized","params":{}}"#;
        let (resp, exit) = handle_body(&state, body, &mut sd).await;
        assert!(!exit);
        assert!(resp.is_none());
        // textDocument/didOpen also notification（已知 LSP method · 忽略）
        let body2 = r#"{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{}}"#;
        let (resp2, _) = handle_body(&state, body2, &mut sd).await;
        assert!(resp2.is_none());
    }

    // 注：framing（Content-Length header）的 IO 层验证留 Day 16 续做集成测；
    // 当前 6 个 handle_body 单测已覆盖业务路由 + 协议状态机（initialize/shutdown/exit/notification/error）。
}
