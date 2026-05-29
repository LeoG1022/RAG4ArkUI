# MCP 协议（Claude Code / Cursor）· Day 15

`arkui-rag serve --mcp` 启动一个 Model Context Protocol stdio server，让 **Claude Code / Cursor / OpenCode** 等 AI agent 直接调用 RAG 检索。

## 快速接入

```bash
# 1. 启 MCP server（独立进程 · 不要后台跑）
arkui-rag serve --mcp \
    --index-path ./corpus/official/index.json \
    --bm25 tantivy
```

```jsonc
// 2. 在 ~/.claude/mcp_servers.json 注册
{
  "mcpServers": {
    "arkui-rag": {
      "command": "/usr/local/bin/arkui-rag",
      "args": [
        "serve", "--mcp",
        "--index-path", "/path/to/corpus/official/index.json",
        "--bm25", "tantivy"
      ]
    }
  }
}
```

完整接入指南：[Claude Code MCP 接入指南](../reference/mcp-integration.md)。

## 暴露的 4 个工具

| 工具 | 用途 |
|---|---|
| `arkui_search_docs(query, top_k, expand_parent)` | 检索 ArkUI-X 官方文档 / API 参考 / 迁移规则 |
| `arkui_search_code(query, mode, top_k)` | 检索代码示例（@Component struct · build() · @State 等） |
| `arkui_migrate_snippet(source_code, from, to)` | 迁移建议（KMP → ArkUI-X 等） |
| `arkui_validate_api(code)` | API 时效性 + 平台兼容性校验 |

## JSON-RPC 帧

```
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"claude","version":"1"}}}
{"jsonrpc":"2.0","id":2,"method":"tools/list"}
{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"arkui_search_docs","arguments":{"query":"router.pushUrl 传参","top_k":5}}}
```

每行一个 JSON 对象（行分隔 framing · 区别于 LSP 的 Content-Length framing）。

## 实测演示

```bash
make mcp-demo   # 启 server + 喂 4 请求 + 断言响应
```

详见 [STATUS-day15-mcp.md](https://github.com/keerecles/RAG4ArkUI/blob/master/docs/STATUS-day15-mcp.md) 和 [scripts/mcp-demo.sh](https://github.com/keerecles/RAG4ArkUI/blob/master/scripts/mcp-demo.sh)。
