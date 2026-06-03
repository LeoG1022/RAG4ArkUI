//! MCP (Model Context Protocol) adapter —— Day 15 真活。
//!
//! 协议：JSON-RPC 2.0 over stdio（一行一消息）。不引入第三方 MCP SDK，
//! 手撸 4 个核心方法（initialize / notifications/initialized / tools/list / tools/call）
//! 保证依赖最少 + 协议兼容性最稳。
//!
//! 暴露 4 个工具（方案 §4.2 决策 2）：
//! - `arkui_search_docs` —— ✅ 真活，文档检索
//! - `arkui_search_code` —— ✅ 真活，代码示例检索（automatically prefer code chunk types）
//! - `arkui_migrate_snippet` —— ⏳ stub，需 LLM 调用，Week 4-5 续
//! - `arkui_validate_api` —— ⏳ stub，需 API 时效性检查器，Week 4 续
//!
//! 集成方式（Claude Code · ~/.claude/mcp.json）：
//! ```json
//! { "mcpServers": { "arkui-rag": { "command": "arkui-rag",
//!   "args": ["serve", "--mcp", "--index-path", "/path/to/index.json"] } } }
//! ```

#![cfg(feature = "mcp")]

use crate::AppState;
use arkui_rag_core::{EnhancedQuery, Hit, RagError, Result};
use arkui_rag_retrieval::ContextAssembler;
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

const PROTOCOL_VERSION: &str = "2024-11-05";
const SERVER_NAME: &str = "arkui-rag";

/// 启动 MCP stdio loop。每行从 stdin 读 JSON-RPC 请求，向 stdout 写响应。
pub async fn serve_stdio(state: AppState) -> Result<()> {
    let state = Arc::new(state);
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    let mut reader = BufReader::new(stdin).lines();
    let mut writer = stdout;

    tracing::info!(
        "MCP stdio server ready. embedder={} bm25={} vector={}",
        state.embedder_model_id,
        state.bm25_name,
        state.vector_name
    );

    while let Some(line) = reader
        .next_line()
        .await
        .map_err(|e| RagError::Other(anyhow::anyhow!("stdin read: {}", e)))?
    {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let resp = handle_line(&state, line).await;
        if let Some(json_text) = resp {
            writer
                .write_all(json_text.as_bytes())
                .await
                .map_err(|e| RagError::Other(anyhow::anyhow!("stdout write: {}", e)))?;
            writer
                .write_all(b"\n")
                .await
                .map_err(|e| RagError::Other(anyhow::anyhow!("stdout newline: {}", e)))?;
            writer
                .flush()
                .await
                .map_err(|e| RagError::Other(anyhow::anyhow!("stdout flush: {}", e)))?;
        }
    }
    Ok(())
}

/// 处理一行 JSON-RPC 请求。返回 Some(response_json) 或 None（notification 不响应）。
pub async fn handle_line(state: &Arc<AppState>, line: &str) -> Option<String> {
    let req: Value = match serde_json::from_str(line) {
        Ok(v) => v,
        Err(e) => {
            return Some(error_response(None, -32700, &format!("parse error: {}", e)));
        }
    };
    let id = req.get("id").cloned();
    let method = req.get("method").and_then(|v| v.as_str()).unwrap_or("");
    let params = req.get("params").cloned().unwrap_or(json!({}));

    // notifications/* 不响应
    let is_notification = id.is_none() || method.starts_with("notifications/");

    let result: std::result::Result<Value, (i32, String)> = match method {
        "initialize" => Ok(handle_initialize(&params)),
        "notifications/initialized" => return None,
        "tools/list" => Ok(handle_tools_list()),
        "tools/call" => handle_tools_call(state, &params).await,
        "ping" => Ok(json!({})),
        _ => Err((-32601, format!("method not found: {}", method))),
    };

    if is_notification {
        return None;
    }

    match result {
        Ok(v) => Some(success_response(id, v)),
        Err((code, msg)) => Some(error_response(id, code, &msg)),
    }
}

fn success_response(id: Option<Value>, result: Value) -> String {
    let resp = json!({
        "jsonrpc": "2.0",
        "id": id.unwrap_or(Value::Null),
        "result": result,
    });
    serde_json::to_string(&resp).unwrap()
}

fn error_response(id: Option<Value>, code: i32, message: &str) -> String {
    let resp = json!({
        "jsonrpc": "2.0",
        "id": id.unwrap_or(Value::Null),
        "error": { "code": code, "message": message },
    });
    serde_json::to_string(&resp).unwrap()
}

// ─── initialize ───────────────────────────────────────────

fn handle_initialize(_params: &Value) -> Value {
    json!({
        "protocolVersion": PROTOCOL_VERSION,
        "serverInfo": {
            "name": SERVER_NAME,
            "version": env!("CARGO_PKG_VERSION"),
        },
        "capabilities": {
            "tools": {}
        }
    })
}

// ─── tools/list ───────────────────────────────────────────

fn handle_tools_list() -> Value {
    json!({
        "tools": [
            {
                "name": "arkui_search_docs",
                "description": "检索 ArkUI-X 官方文档 / API 参考 / 迁移规则。返回 Top-K 文档段落 + 引用回链。",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "query": { "type": "string", "description": "自然语言查询，如 'router.pushUrl 怎么传参'" },
                        "top_k": { "type": "integer", "description": "返回数量，默认 5", "default": 5 },
                        "expand_parent": { "type": "boolean", "description": "扩展到父 chunk 显示完整上下文", "default": false }
                    },
                    "required": ["query"]
                }
            },
            {
                "name": "arkui_search_code",
                "description": "检索 ArkUI-X / ArkTS 代码示例（含 @Component struct · build() · @State 等）。",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "query": { "type": "string", "description": "代码场景描述或残缺片段" },
                        "top_k": { "type": "integer", "default": 5 },
                        "expand_parent": { "type": "boolean", "default": true }
                    },
                    "required": ["query"]
                }
            },
            {
                "name": "arkui_migrate_snippet",
                "description": "把源码（KMP/Android/iOS）迁移到 ArkUI-X。Day 15 仅返回相关迁移规则（不调 LLM 真生成代码）。",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "source_code": { "type": "string" },
                        "from": { "type": "string", "enum": ["kmp", "android", "ios"] },
                        "to": { "type": "string", "default": "arkui-x" }
                    },
                    "required": ["source_code", "from"]
                }
            },
            {
                "name": "arkui_validate_api",
                "description": "校验代码中使用的 ArkUI-X API 时效性 + 平台兼容性。Day 15 stub，Week 4 续接 API 版本检查器。",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "code": { "type": "string" },
                        "platform": { "type": "string", "enum": ["harmonyos", "android", "ios"] }
                    },
                    "required": ["code"]
                }
            }
        ]
    })
}

// ─── tools/call ───────────────────────────────────────────

async fn handle_tools_call(
    state: &Arc<AppState>,
    params: &Value,
) -> std::result::Result<Value, (i32, String)> {
    let name = params
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or((-32602, "missing tool name".into()))?;
    let args = params.get("arguments").cloned().unwrap_or(json!({}));
    match name {
        "arkui_search_docs" => arkui_search(state, &args, SearchMode::Docs).await,
        "arkui_search_code" => arkui_search(state, &args, SearchMode::Code).await,
        "arkui_migrate_snippet" => arkui_migrate(state, &args).await,
        "arkui_validate_api" => arkui_validate(&args),
        _ => Err((-32602, format!("unknown tool: {}", name))),
    }
}

#[derive(Debug, Clone, Copy)]
enum SearchMode {
    Docs,
    Code,
}

async fn arkui_search(
    state: &Arc<AppState>,
    args: &Value,
    mode: SearchMode,
) -> std::result::Result<Value, (i32, String)> {
    let query = args
        .get("query")
        .and_then(|v| v.as_str())
        .ok_or((-32602, "missing argument: query".into()))?;
    let top_k = args.get("top_k").and_then(|v| v.as_u64()).unwrap_or(5) as usize;
    let expand_parent = args
        .get("expand_parent")
        .and_then(|v| v.as_bool())
        .unwrap_or(matches!(mode, SearchMode::Code));

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

    // 可选 expand_parent
    let parents: Vec<Option<String>> = if expand_parent {
        if let Some(store) = &state.metadata_store {
            let asm = ContextAssembler::new(store.clone());
            let exp = asm
                .expand_to_parent(hits.clone())
                .await
                .map_err(|e| (-32603, format!("expand_parent: {}", e)))?;
            exp.iter()
                .map(|e| {
                    e.parent
                        .as_ref()
                        .map(|p| p.content.chars().take(800).collect::<String>())
                })
                .collect()
        } else {
            vec![None; hits.len()]
        }
    } else {
        vec![None; hits.len()]
    };

    let text = render_hits_as_markdown(&hits, &parents, mode);
    Ok(json!({
        "content": [
            { "type": "text", "text": text }
        ],
        "isError": false
    }))
}

fn render_hits_as_markdown(hits: &[Hit], parents: &[Option<String>], mode: SearchMode) -> String {
    let mut out = String::new();
    let label = match mode {
        SearchMode::Docs => "📚 文档检索",
        SearchMode::Code => "💻 代码检索",
    };
    out.push_str(&format!("# {} · Top-{}\n\n", label, hits.len()));
    if hits.is_empty() {
        out.push_str("⚠️  无命中。建议：\n- 检查 query 关键词\n- 用 `expand_parent: true` 扩展\n- 试 `arkui_search_docs` / `arkui_search_code` 互换\n");
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
        let body: String = h.chunk.content.chars().take(800).collect();
        out.push_str(&format!("```\n{}\n```\n", body));
        if let Some(parent_preview) = &parents[i] {
            out.push_str("\n**Parent context**:\n\n");
            out.push_str(&format!("```\n{}\n```\n", parent_preview));
        }
        out.push('\n');
    }
    out
}

async fn arkui_migrate(
    state: &Arc<AppState>,
    args: &Value,
) -> std::result::Result<Value, (i32, String)> {
    let source_code = args
        .get("source_code")
        .and_then(|v| v.as_str())
        .ok_or((-32602, "missing argument: source_code".into()))?;
    let from = args
        .get("from")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    // Day 15：检索相关迁移规则，不调 LLM 生成
    let query = format!("{} migration to ArkUI-X: {}", from, source_code);
    let eq = EnhancedQuery::passthrough(query);
    let hits = state
        .retriever
        .retrieve(&eq, 5)
        .await
        .map_err(|e| (-32603, format!("retrieve: {}", e)))?;

    let mut text = format!("# 🔄 迁移建议（{} → ArkUI-X）\n\n", from);
    text.push_str(
        "**⏳ 当前为 Day 15 stub**：仅返回相关迁移规则。Week 4-5 续接 LLM 调用真生成代码。\n\n",
    );
    text.push_str("## 相关迁移规则\n\n");
    let parents = vec![None; hits.len()];
    text.push_str(&render_hits_as_markdown(&hits, &parents, SearchMode::Code));
    Ok(json!({
        "content": [{ "type": "text", "text": text }],
        "isError": false
    }))
}

fn arkui_validate(args: &Value) -> std::result::Result<Value, (i32, String)> {
    let code = args
        .get("code")
        .and_then(|v| v.as_str())
        .ok_or((-32602, "missing argument: code".into()))?;
    let len = code.chars().count();
    let text = format!(
        "# 🔍 API 时效性校验\n\n**⏳ Day 15 stub**：API 版本检查器待 Week 4 续。\n\n输入代码长度：{} 字符\n\n建议：\n- 用 `arkui_search_docs` 查具体 API 是否存在\n- 用 `arkui_search_code` 查官方示例\n",
        len
    );
    Ok(json!({
        "content": [{ "type": "text", "text": text }],
        "isError": false
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
                        content: format!("fixture content for item {}", i),
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
                    vector_score: None,
                    bm25_score: None,
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
    async fn initialize_returns_protocol_version() {
        let state = test_state();
        let line = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#;
        let resp = handle_line(&state, line).await.unwrap();
        let v: Value = serde_json::from_str(&resp).unwrap();
        assert_eq!(v["jsonrpc"], "2.0");
        assert_eq!(v["id"], 1);
        assert_eq!(v["result"]["protocolVersion"], PROTOCOL_VERSION);
        assert_eq!(v["result"]["serverInfo"]["name"], "arkui-rag");
    }

    #[tokio::test]
    async fn tools_list_returns_4_tools() {
        let state = test_state();
        let line = r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#;
        let resp = handle_line(&state, line).await.unwrap();
        let v: Value = serde_json::from_str(&resp).unwrap();
        let tools = v["result"]["tools"].as_array().unwrap();
        assert_eq!(tools.len(), 4);
        let names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
        assert!(names.contains(&"arkui_search_docs"));
        assert!(names.contains(&"arkui_search_code"));
        assert!(names.contains(&"arkui_migrate_snippet"));
        assert!(names.contains(&"arkui_validate_api"));
    }

    #[tokio::test]
    async fn search_docs_returns_text_content() {
        let state = test_state();
        let line = r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"arkui_search_docs","arguments":{"query":"test","top_k":3}}}"#;
        let resp = handle_line(&state, line).await.unwrap();
        let v: Value = serde_json::from_str(&resp).unwrap();
        let content = v["result"]["content"].as_array().unwrap();
        assert_eq!(content.len(), 1);
        assert_eq!(content[0]["type"], "text");
        let text = content[0]["text"].as_str().unwrap();
        assert!(text.contains("Top-3"));
        assert!(text.contains("文档检索"));
        assert!(text.contains("doc/0.md"));
    }

    #[tokio::test]
    async fn unknown_method_returns_error() {
        let state = test_state();
        let line = r#"{"jsonrpc":"2.0","id":99,"method":"unknown_method"}"#;
        let resp = handle_line(&state, line).await.unwrap();
        let v: Value = serde_json::from_str(&resp).unwrap();
        assert_eq!(v["error"]["code"], -32601);
        assert!(v["error"]["message"]
            .as_str()
            .unwrap()
            .contains("method not found"));
    }

    #[tokio::test]
    async fn notifications_have_no_response() {
        let state = test_state();
        let line = r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;
        let resp = handle_line(&state, line).await;
        assert!(resp.is_none(), "notifications should not be responded to");
    }

    #[tokio::test]
    async fn migrate_stub_returns_with_warning() {
        let state = test_state();
        let line = r#"{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"arkui_migrate_snippet","arguments":{"source_code":"fun main(){}","from":"kmp"}}}"#;
        let resp = handle_line(&state, line).await.unwrap();
        let v: Value = serde_json::from_str(&resp).unwrap();
        let text = v["result"]["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("Day 15 stub"));
        assert!(text.contains("kmp"));
    }
}
