# LSP 协议（IDE 内联）· Day 16

`arkui-rag serve --lsp` 启动 Language Server Protocol stdio server，让 **DevEco Studio / IntelliJ Plugin / VSCode Extension** 等 IDE 内联使用 RAG。

与 MCP 的区别：
- MCP 给 Agent 用（Claude Code / Cursor 调工具）
- LSP 给 IDE 编辑器内联用（hover / completion / codeAction）

## 启动

```bash
arkui-rag serve --lsp \
    --index-path ./corpus/official/index.json \
    --bm25 tantivy
```

启动需要 `lsp` feature（默认 release 已含）。

## 协议帧（Content-Length framing）

```
Content-Length: 75\r\n
\r\n
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"capabilities":{}}}
```

注意 LSP 是 Content-Length 帧分隔 · 区别于 MCP 的行分隔。

## 实装 method

| Method | 状态 | 说明 |
|---|---|---|
| `initialize` | ✅ 真活 | 返回 capabilities（hoverProvider + executeCommandProvider） |
| `initialized` | ✅ notification | |
| `shutdown` | ✅ 真活 | 状态机：shutdown 后只允 `exit`，其它 method 报 -32600 |
| `exit` | ✅ notification | 停 server |
| `arkui-rag/search` | ✅ 真活 | **自定义 method**：传 `{query, top_k}` · 返回 markdown content + hits |
| `arkui-rag/migrate` | ⏳ stub | 待 LLM 接入 |
| `textDocument/hover` | ⏳ stub | capability=true 但返回 null（Day 16 续真活） |
| `textDocument/didOpen` `didChange` `didClose` | ✅ 已知 notification · 忽略 |
| `$/cancelRequest` `$/setTrace` | ✅ 已知 notification · 忽略 |

## IDE 接入示例（伪代码）

```typescript
const client = new LSPClient({
  command: "arkui-rag",
  args: ["serve", "--lsp", "--index-path", "/path/to/index.json", "--bm25", "tantivy"]
});

// 标准 LSP handshake
await client.sendRequest("initialize", {});
client.sendNotification("initialized", {});

// 自定义 method 调用
const result = await client.sendRequest("arkui-rag/search", {
  query: "下拉刷新",
  top_k: 5
});
// result.content.value = "# 📚 RAG4ArkUI 检索结果 · Top-5\n..."
// result.hits = [{ chunk_id, source, heading_path, line_range, score }, ...]
```

详细技术细节见 [STATUS-day16-lsp.md](https://github.com/LeoG1022/RAG4ArkUI/blob/master/docs/STATUS-day16-lsp.md)。

## 三协议互斥

```bash
arkui-rag serve --http --mcp     # ❌ 报错：互斥
arkui-rag serve --mcp --lsp      # ❌ 同上
arkui-rag serve                  # ❌ 必须指定一个
arkui-rag serve --lsp            # ✅
```

要同时跑多个协议：启多个独立进程。
