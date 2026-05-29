# HTTP REST API（Day 14）

## 启动

```bash
arkui-rag serve --http --addr 127.0.0.1:7654 \
    --index-path ./corpus/official/index.json \
    --bm25 tantivy
```

启动需要 `http` feature（默认 release 已含）。

## 端点

### `GET /health`

```bash
curl http://127.0.0.1:7654/health
```

返回：
```json
{
  "status": "ok",
  "embedder": "mock-384",
  "embedder_dim": 384,
  "vector": "memory",
  "bm25": "tantivy",
  "rerank_enabled": false,
  "enhancer": "passthrough",
  "pre_rerank_k": 50
}
```

### `POST /search`

```bash
curl -X POST http://127.0.0.1:7654/search \
    -H "Content-Type: application/json" \
    -d '{"query": "@State 双向绑定", "top_k": 5}'
```

返回：
```json
{
  "hits": [
    {
      "chunk_id": "mapping-state.md#...",
      "score": 0.0294,
      "source": "hybrid",
      "citation": {
        "chunk_id": "...",
        "source": "mapping-state.md",
        "heading_path": ["Mapping — 状态、Effect 与生命周期", "状态选择决策"],
        "line_range": [24, 34],
        "score": 0.0294
      },
      "content_preview": "..."
    },
    ...
  ]
}
```

可选参数：
```json
{
  "query": "...",
  "top_k": 5,
  "filters": { "platform": "harmony" },
  "rerank": true,
  "expand_parent": true
}
```

### `POST /index`

```bash
curl -X POST http://127.0.0.1:7654/index \
    -H "Content-Type: application/json" \
    -d '{"source_path": "/path/to/new/docs"}'
```

（当前为 stub · 完整实装见 ROADMAP）

### `GET /corpus/list`

```bash
curl http://127.0.0.1:7654/corpus/list
```

列出 corpus/ 下的子目录与文档数。

## 安全考虑

当前 HTTP server **仅监听本地 127.0.0.1** · 没有 CORS / 鉴权 / TLS。  
公网部署需要额外做：
- TLS 终止（Nginx / Caddy reverse proxy）
- API key 鉴权（middleware）
- CORS（如果浏览器直接调用）

详见 [docs/STATUS-day14-http.md](https://github.com/keerecles/RAG4ArkUI/blob/master/docs/STATUS-day14-http.md)。
