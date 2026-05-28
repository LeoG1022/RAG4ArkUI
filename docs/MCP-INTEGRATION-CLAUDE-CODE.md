# MCP 接入 Claude Code · 完整指南

> 把 RAG4ArkUI 接入 Claude Code，让 Claude 在对话中调用 4 个 ArkUI-X 检索工具。
> 对应 commit：Day 15 (`f4724cc`) + Day 19（本轮 · 接入验证）。

---

## 一图看懂

```
                    ┌─────────────────────────┐
                    │   Claude Code (CLI)     │
                    │   ~/.claude/mcp.json    │
                    └────────────┬────────────┘
                                 │ fork + stdio
                                 ▼
                ┌────────────────────────────┐
                │  arkui-rag serve --mcp     │
                │  (Rust 二进制 · stdio JSON-RPC)│
                └────────────┬───────────────┘
                             │ retriever / reranker / enhancer
                             ▼
                ┌────────────────────────────┐
                │  AppState (trait object)   │
                │  ├── HybridRetriever        │
                │  ├── InMemoryVectorStore    │
                │  │   或 LanceVectorStore    │
                │  ├── Tantivy BM25 / Memory  │
                │  ├── OnnxReranker / Mock    │
                │  └── ContextAssembler       │
                └────────────┬───────────────┘
                             ▼
                ┌────────────────────────────┐
                │  corpus/_index/*.json+lance│
                │  (Indexer 产物 · Day 2 起) │
                └────────────────────────────┘
```

Claude Code 启动子进程，通过 stdin/stdout 用 JSON-RPC 2.0 与 server 通信。
4 个工具暴露给 Claude，由 Claude 自主决定何时调用。

---

## 一、前置准备

### 1.1 编译 arkui-rag 二进制

```bash
# 默认配置（Mock embedder · 适合先跑通流程）
make build-mcp
# 产物：crates/target/release/arkui-rag

# 全功能配置（真实 BGE-M3 + Tantivy + LanceDB）
make build-full
# 需要先下载 BGE-M3 模型 · 见 docs/STATUS-day3-onnx-embedder.md
```

### 1.2 建索引

```bash
# 投放文档到 corpus/（参考 corpus/README.md）
cp /path/to/arkui-docs/*.md corpus/official/

# 建索引（Mock embedder · 适合 demo）
cargo run --features mcp -p arkui-rag-cli -- \
    index --source corpus

# 或真实 ONNX 索引
cargo run --features full -p arkui-rag-cli -- \
    index --source corpus \
    --embedder onnx --model-path ~/.arkui-rag/models/bge-m3 \
    --bm25 tantivy --vector lancedb
```

### 1.3 安装到 PATH（推荐）

```bash
# 把二进制放到 ~/bin 或 /usr/local/bin
cp crates/target/release/arkui-rag ~/bin/arkui-rag
# 或者：
sudo cp crates/target/release/arkui-rag /usr/local/bin/

# 验证
arkui-rag --version
# arkui-rag 0.0.1
```

---

## 二、配置 Claude Code

### 2.1 编辑 `~/.claude/mcp.json`

如果文件不存在，新建：

```json
{
  "mcpServers": {
    "arkui-rag": {
      "command": "arkui-rag",
      "args": [
        "serve",
        "--mcp",
        "--index-path", "/Users/me/RAG4ArkUI/corpus/_index/index.json"
      ]
    }
  }
}
```

**关键字段**：

| 字段 | 说明 |
|---|---|
| `command` | 二进制路径。如 PATH 已配 `arkui-rag` 直接写名字；否则用绝对路径 `/usr/local/bin/arkui-rag` |
| `args` | 至少含 `serve --mcp --index-path <你的索引产物绝对路径>` |

### 2.2 加全功能参数（可选）

```json
{
  "mcpServers": {
    "arkui-rag": {
      "command": "arkui-rag",
      "args": [
        "serve", "--mcp",
        "--index-path", "/Users/me/RAG4ArkUI/corpus/_index/index.json",
        "--embedder", "onnx",
        "--model-path", "/Users/me/.arkui-rag/models/bge-m3-onnx",
        "--bm25", "tantivy",
        "--vector", "lancedb",
        "--rerank", "onnx",
        "--reranker-model-path", "/Users/me/.arkui-rag/models/bge-reranker-v2-m3",
        "--hyde", "mock"
      ]
    }
  }
}
```

**注意**：二进制必须用对应 feature 编译（`build-full` 已开全）。

### 2.3 重启 Claude Code

```bash
# 退出当前 Claude Code 进程，重新启动
# Claude Code 启动时读 ~/.claude/mcp.json 并 fork 子进程
```

启动成功后，Claude Code 主进程会看到 `arkui-rag` MCP server，下方应出现 4 个工具：
- `arkui_search_docs`
- `arkui_search_code`
- `arkui_migrate_snippet`
- `arkui_validate_api`

---

## 三、在 Claude Code 中使用

### 3.1 自然语言触发

直接对 Claude 说：

```
> 我想在 ArkUI-X 里实现一个下拉刷新的列表，怎么写？
```

Claude 应该自动调用 `arkui_search_code`，返回相关代码示例，然后基于检索结果给你写代码。

```
> 帮我查 router.pushUrl 的 API 参数
```

Claude 调用 `arkui_search_docs`，返回 API 文档段落。

### 3.2 显式工具调用（debug 用）

如果想看 MCP 工具直接输出（不经 Claude 解释），可用 Claude Code 的 `/mcp` 指令（参考 Claude Code 文档）。

### 3.3 工具签名速查

| Tool | 必填参数 | 可选参数 |
|---|---|---|
| `arkui_search_docs` | `query` | `top_k` (默认 5) · `expand_parent` (默认 false) |
| `arkui_search_code` | `query` | `top_k` (默认 5) · `expand_parent` (默认 true) |
| `arkui_migrate_snippet` | `source_code` · `from` (`kmp\|android\|ios`) | `to` (默认 `arkui-x`) |
| `arkui_validate_api` | `code` | `platform` |

后两个是 stub（返回相关检索结果 + 提示），真活待 Week 4-5。

---

## 四、验证（不依赖 Claude Code · 手动测试）

### 4.1 端到端演示脚本

```bash
make mcp-demo
# 或：bash scripts/mcp-demo.sh
```

预期输出（节选）：

```
🔌 MCP server starting on stdio (JSON-RPC 2.0)
   embedder=mock-384 · bm25=memory · vector=memory · rerank=off
   tools: arkui_search_docs · arkui_search_code · arkui_migrate_snippet · arkui_validate_api

# ← 喂入 initialize
{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":"2024-11-05","serverInfo":{...},"capabilities":{"tools":{}}}}

# ← 喂入 tools/list
{"jsonrpc":"2.0","id":2,"result":{"tools":[{"name":"arkui_search_docs",...},...4 tools...]}}

# ← 喂入 tools/call arkui_search_docs
{"jsonrpc":"2.0","id":3,"result":{"content":[{"type":"text","text":"# 📚 文档检索 · Top-3\n\n..."}],"isError":false}}
```

### 4.2 手工模拟一次完整握手

```bash
# 启动 server（前台 · stdin 来自终端）
cargo run --features mcp -p arkui-rag-cli -- \
    serve --mcp --index-path corpus/_index/index.json

# 在终端粘贴（每行一个请求 · 回车送出）
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}
{"jsonrpc":"2.0","method":"notifications/initialized"}
{"jsonrpc":"2.0","id":2,"method":"tools/list"}
{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"arkui_search_docs","arguments":{"query":"下拉刷新","top_k":3}}}

# Ctrl-D（EOF）退出
```

### 4.3 单元测试

```bash
cargo test -p arkui-rag-server --features mcp
# 6 测试：initialize / tools_list / search_docs / unknown_method / notifications / migrate_stub
```

---

## 五、故障排查

### 症状 1：Claude Code 启动看不到 arkui-rag 工具

**排查**：
1. 检查 `~/.claude/mcp.json` JSON 语法（`jq . ~/.claude/mcp.json`）
2. 检查 `command` 是否在 PATH 内（`which arkui-rag`）
3. 检查二进制是否含 mcp feature：`arkui-rag serve --mcp 2>&1 | head -3`，应看到 "🔌 MCP server starting on stdio"
4. 看 Claude Code 日志（macOS：`~/Library/Logs/Claude/`）

### 症状 2：JSON-RPC 响应有 error -32601

**含义**：method not found · 检查协议版本

修复：升级 Claude Code · 检查 mcp.json `protocolVersion`

### 症状 3：search 返回 0 hits

**排查**：
1. 索引是否构建：`ls -la <index-path>`
2. 索引产物 embedder_model_id 与 server 配置一致？看 `arkui-rag serve --mcp` 启动 stderr
3. Mock embedder 阶段：query 必须与 corpus 中某 chunk 文本接近完全一致才命中（cosine=1）
4. 真实 ONNX：检查 model 是否真实下载（`du -sh ~/.arkui-rag/models/`）

### 症状 4：tools/call 报 -32603 internal error

**含义**：检索流水线内部错误

**排查**：
1. 看 server stderr 输出
2. 用 `arkui-rag query --text "..." --index-path ...` CLI 单独验证流水线
3. 检查 vector backend 一致性（用 lancedb 建的索引不能用 memory backend 查）

---

## 六、性能调优建议

| 场景 | 推荐配置 |
|---|---|
| Demo / 小 corpus (< 1k chunks) | mock embedder · memory bm25 · memory vector |
| 真实使用 (1k-10k chunks) | onnx embedder · tantivy bm25 · memory vector |
| 大规模 (> 10k chunks) | onnx embedder · tantivy bm25 · **lancedb vector** |
| 高质量召回 | + onnx rerank（额外 ~200ms 延迟） |
| 自然语言场景 | + hyde mock（query 改写为 ArkTS 代码风格） |

延迟参考（M2 Pro / Mock 全部）：
- search_docs 端到端：5-20 ms
- 启用 ONNX embedder：+ 50-200 ms（首次 cold start）
- 启用 ONNX rerank：+ 100-500 ms（取决于 batch size）
- 启用 LanceDB：与 memory 持平（小规模）

---

## 七、其他 Agent 接入（Cursor / OpenCode）

Cursor 用同样的 mcp 配置（`~/.cursor/mcp.json` · 路径不同但格式一致）。

OpenCode / Hermes 配置见各自 README，核心都是 `command + args + stdio`。

---

## 八、限制 & 未做（明确）

| 限制 | 原因 | 何时修 |
|---|---|---|
| `arkui_migrate_snippet` 仅返回相关规则 | 需 LLM 抽象 + 多模型适配（§3.3） | Week 4-5 |
| `arkui_validate_api` 仅 stub | 需 API 时效性检查器 | Week 4 |
| 不支持 MCP SSE 传输 | stdio 是 Claude Code 默认 · SSE 是 Web Agent 场景 | Day 15 续 |
| 不支持 MCP resources / prompts 能力 | 工具优先 · resources 价值小 | Day 15 续 |
| 一进程只能 stdio 或 HTTP 二选一 | stdio 专用 · HTTP 占端口 | 设计意图 · 不修 |

---

## 九、相关文档

- [`docs/STATUS-day15-mcp.md`](STATUS-day15-mcp.md) — Day 15 实现细节
- [`docs/STATUS-day14-http.md`](STATUS-day14-http.md) — HTTP server 实现（同款 AppState）
- [`docs/ADR-002-crate-structure.md`](ADR-002-crate-structure.md) — server crate 设计依据
- [`docs/RAG4ArkUI-完整技术方案.md`](RAG4ArkUI-完整技术方案.md) §4.2 决策 2 — 协议层设计

---

## 十、反馈

如遇问题：
1. 先看 §5 故障排查
2. 跑 `make mcp-demo` 排除环境问题
3. 看 `feedback/features/rag4arkui-core/` 各 Round 归档了解历史决策
4. 提 issue 到 repository（带 server stderr 输出）
