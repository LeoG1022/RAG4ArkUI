# STATUS · Day 15 · MCP Server

> 日期：2026-05-28
> 对应 commit：[本 commit · Day 15 MCP]
> 对应 feature log：[`feedback/features/rag4arkui-core/17-2026-05-28-day15-mcp.md`](../feedback/features/rag4arkui-core/17-2026-05-28-day15-mcp.md)
> 上一阶段：[`STATUS-day14-http.md`](STATUS-day14-http.md)
> 下一阶段：`STATUS-day16-lsp.md` 或 `STATUS-day17-deveco.md` 或 `STATUS-day8-jieba.md`

> 🎯 **里程碑**：**协议层基本完整**（HTTP + MCP 双协议 · LSP Week 4 续）· **Claude Code 直接接入就绪** ⭐

---

## 当前状态

| 模块 | 变化 |
|---|---|
| `arkui-rag-server/src/mcp.rs` | **重写** ~380 行 · stdio JSON-RPC 主循环 + 4 tools + 6 单测 · feature gated |
| `arkui-rag-server/src/lib.rs` | 导出 serve_mcp_stdio · AppState alias · mcp-only 兜底类型定义 |
| `arkui-rag-cli/Cargo.toml` | `mcp = ["arkui-rag-server/mcp"]` · full 加入 mcp |
| `arkui-rag-cli/src/main.rs` | cmd_serve_mcp + cfg 双路径 · `--http`/`--mcp` 互斥校验 |
| `Makefile` | + `check-mcp` / `build-mcp` / `serve-mcp-demo` 三 target |
| `docs/ROADMAP.md` | 第 6 次实战 · 9 处进度行同步 · **Week 3 协议层 3/3 ⭐** |

### 测试覆盖

| 测试组 | 数量 |
|---|---|
| mcp.rs 单测（feature gated） | 6 · initialize / tools_list / search_docs / unknown_method / notifications / migrate_stub |
| **默认 features 累计** | 49（不变 · mcp 默认关） |
| **`--features mcp` 累计** | **55** |
| **`--features http,mcp` 累计** | **60** |
| 全 feature（`--features full`） | 约 **80** |

---

## 输入契约

### CLI 启动

```bash
# 编译 + 启动
make build-mcp
make serve-mcp-demo
# 或：
cargo run --features mcp -p arkui-rag-cli -- serve --mcp

# 全功能（含 onnx + tantivy + lancedb）
cargo run --features full -p arkui-rag-cli -- serve --mcp \
    --embedder onnx --model-path ~/.arkui-rag/models/bge-m3 \
    --bm25 tantivy --vector lancedb \
    --rerank onnx --reranker-model-path ~/.arkui-rag/models/bge-reranker \
    --hyde mock
```

### Claude Code 集成

```json
// ~/.claude/mcp.json
{
  "mcpServers": {
    "arkui-rag": {
      "command": "/usr/local/bin/arkui-rag",
      "args": [
        "serve", "--mcp",
        "--index-path", "/Users/me/RAG4ArkUI/corpus/_index/index.json"
      ]
    }
  }
}
```

Claude Code 自动启动子进程 + stdio 通信。

### JSON-RPC 协议

**Client → Server**（一行一消息）：
```json
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}
{"jsonrpc":"2.0","method":"notifications/initialized"}
{"jsonrpc":"2.0","id":2,"method":"tools/list"}
{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"arkui_search_docs","arguments":{"query":"下拉刷新","top_k":3}}}
```

**Server → Client**：
```json
{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":"2024-11-05",...}}
// (notification 不响应)
{"jsonrpc":"2.0","id":2,"result":{"tools":[...4 tools...]}}
{"jsonrpc":"2.0","id":3,"result":{"content":[{"type":"text","text":"# 📚 文档检索 · Top-3\n\n..."}],"isError":false}}
```

### 库 API

```rust
#[cfg(feature = "mcp")]
use arkui_rag_server::{AppState, serve_mcp_stdio, mcp::handle_line};

// 真启动 stdio 循环
serve_mcp_stdio(state).await?;

// 测试时直接调（不起循环）
let response: Option<String> = handle_line(&Arc::new(state), json_line).await;
```

---

## 输出契约

### tools/list 返回 4 个工具

| Tool name | 状态 | 输入 schema 必填 |
|---|---|---|
| `arkui_search_docs` | ✅ 真活 | `query` |
| `arkui_search_code` | ✅ 真活 | `query` |
| `arkui_migrate_snippet` | ⏳ stub | `source_code` + `from` |
| `arkui_validate_api` | ⏳ stub | `code` |

每个都含完整 `inputSchema` JSON Schema（Claude Code 自动生成 prompt 用）。

### tools/call 输出（markdown text）

```markdown
# 📚 文档检索 · Top-3

## [1] `list.md` · L9-11 · score=0.0163

**Heading**: List > 下拉刷新

\`\`\`
ArkUI-X 用 Refresh 组件实现下拉刷新。
\`\`\`

**Parent context**:

\`\`\`
# List

## 下拉刷新
ArkUI-X 用 Refresh 组件实现下拉刷新。
\`\`\`

## [2] `router.md` · L9-12 · score=0.0156
...
```

### 错误响应（JSON-RPC 2.0 规范）

| 错误码 | 含义 |
|---|---|
| -32700 | Parse error（JSON 不合法） |
| -32601 | Method not found |
| -32602 | Invalid params |
| -32603 | Internal error |

```json
{"jsonrpc":"2.0","id":1,"error":{"code":-32601,"message":"method not found: foo"}}
```

### 启动日志（stderr 不污染 stdout）

```
🔌 MCP server starting on stdio (JSON-RPC 2.0)
   embedder=mock-384 · bm25=memory · vector=memory · rerank=off
   tools: arkui_search_docs · arkui_search_code · arkui_migrate_snippet · arkui_validate_api
INFO arkui_rag_server::mcp: MCP stdio server ready. embedder=mock-384 bm25=memory vector=memory
```

---

## 验证手段

### 用户手动

```bash
# 1. 默认编译（不拉 mcp）
make check
make test                                  # 默认 49 测试

# 2. mcp feature 编译（快 · 仅 tokio + serde_json）
make check-mcp
cd crates && cargo test -p arkui-rag-server --features mcp
# 期望 6 单测全过

# 3. CLI 启动 + JSON-RPC 手工测
make build-mcp
# 启动（前台）
cargo run --features mcp -p arkui-rag-cli -- serve --mcp < /dev/stdin

# 4. 真实接入：配 Claude Code mcp.json + 重启 Claude Code → 应能看到 4 个工具可用
```

### 自动化

| 手段 | 范围 | 状态 |
|---|---|---|
| mcp.rs 单测 × 6 | initialize/tools_list/search/error/notification/migrate | ✅ feature gated |
| **M-STATUS-PER-ROUND** | Round 17 + STATUS-day15 配套 | ✅ |
| **ROADMAP 维护约定（第 6 次实战）** | 9 处进度行同步 | ✅ |

### 暂未自动化（明确缺口）

- ❌ MCP SSE 传输（Web Agent 场景 · Day 15 续）
- ❌ MCP resources 能力（暴露 corpus 文件作为 readable resource）
- ❌ MCP prompts 能力（暴露 prompt 模板）
- ❌ migrate / validate 真活（需 LLM 抽象 · 方案 §3.3）
- ❌ Claude Code 端到端自动化测试（需 fork Claude Code 进程）
- ❌ 协议合规性测试（与 MCP 官方测试套件对比）

---

## 与上一阶段（STATUS-day14-http）的关联性

### 增量

| 维度 | Day 14（HTTP）后 | 本轮（Day 15）后 |
|---|---|---|
| Crate 数 | 9 | 不变 |
| 协议层数 | 1（HTTP） | **2（HTTP + MCP）** |
| Agent 接入 | 仅 curl / 自研 HTTP client | **+ Claude Code / Cursor / OpenCode 原生支持** |
| CLI feature 数 | 8 | **9**（+ mcp） |
| 测试数（mcp 启用） | — | **6 单测** |
| Week 3 完成度 | 2/3 | **3/3** ⭐ |

### 业界基线对照（持续更新）

| 业界共识 | 状态 |
|---|---|
| §1.6 共识 1 Hybrid | ✅ Day 4 |
| §8.5 共识 2 Reranker | ✅ Day 5 |
| §8.5 共识 3 引用溯源 | ✅ Day 2 |
| §8.5 共识 4 Eval-Driven | ✅ Day 6 |
| §1.4 Parent-Child | ✅ Day 11 |
| §4.2 决策 2 协议层（HTTP+MCP） | ✅ Day 14 + Day 15 |

### 与 §4.2 决策 2 完整对齐

方案文档：
> "MCP（Model Context Protocol）是 2025 年的事实标准，必须作为一等公民支持。
>  向上暴露三套接口：MCP Server (stdio + SSE) · HTTP REST API · LSP Custom Commands"

| 接口 | 状态 |
|---|---|
| MCP Server (stdio) | ✅ Day 15 |
| MCP Server (SSE) | ⏳ Day 15 续 |
| HTTP REST API | ✅ Day 14 |
| LSP Custom Commands | ⏳ Day 16 |

### 兼容性

- ✅ 无破坏性变更
- ✅ mcp / http feature 独立可启 · 也可同时启（一进程对外双协议）
- ✅ `--http` 与 `--mcp` CLI 互斥（前者占端口 · 后者占 stdio）
- ✅ AppState trait object 设计 · 切换 SDK 零成本

---

## 完成度 / 下一阶段

### Day 15 完成度

| 项 | 状态 |
|---|---|
| MCP stdio JSON-RPC 主循环 | ✅ |
| 4 tools（2 真 + 2 stub）+ inputSchema | ✅ |
| 6 单测覆盖（含 error / notification） | ✅ |
| CLI serve --mcp + feature 转发 | ✅ |
| Makefile + 文档 + ROADMAP | ✅ |
| MCP SSE 传输 | ⏳ Day 15 续 |
| migrate / validate 真活 | ⏳ 需 LLM 抽象 |
| MCP resources / prompts | ⏳ Day 15 续 |

### 6 周路线图达成度更新

| 章节 | 状态 |
|---|---|
| Week 1: Rust 骨架 + tree-sitter + LanceDB + Tantivy + BGE-M3 | **7/7** ✅ |
| Week 2: 混合检索 + Reranker + HyDE + 评估集 | **4/4** ✅ |
| Week 3: HTTP + MCP + CLI | **3/3** ✅ ⭐ |
| Week 4: IDE 插件 (DevEco/IntelliJ) | **0/2** ⏳ |
| Week 5: Claude Code 接入 | **0.5/1** ⏳ MCP 接入就绪，待验证 |
| Week 6: 发布 + 文档站 | **1/4** ✅ |

**总完成度估算：~65%**

### 下一阶段建议（按推荐优先级）

| 候选 | 价值 | 工作量 |
|---|---|---|
| 🟢 **Day 17 DevEco Plugin MVP**（强推 · 方案 §4.3 主战场） | IDE 内集成完整 RAG 能力 | 5+ commit · 大工程 |
| 🟢 Day 16 LSP Server | 协议层 3/3 完整收尾 | 2-3 commit |
| 🟢 Day 19 Claude Code 端到端验证 | 验证 Day 15 MCP 真实可用 | 1 commit |
| 🟡 Day 18 VSCode Extension | 跨编辑器覆盖 | 3+ commit |
| 🟡 Day 8 tantivy-jieba | 中文 BM25 精度 | 0.5 commit |

**Agent 推荐**：**Day 19 Claude Code 端到端验证**（轻量 · 关键路径下一站）。理由：
1. Day 15 完成后第一时间验证 MCP 接入 · 风险最小化
2. 工作量小（1 commit · 主要文档 + demo 脚本）
3. 用户可立即用 Claude Code 体验完整能力
4. 之后再上 Day 17 DevEco（大工程）or Day 16 LSP

**备选**：**Day 16 LSP Server**（协议层 3/3 完整 · 与 Day 14/15 共享 AppState）。

### 重要的"非完成"项

- ❌ MCP SSE 传输（仅 stdio）
- ❌ MCP resources / prompts 能力
- ❌ migrate / validate 真活（需 LLM 抽象）
- ❌ Claude Code 端到端自动化测试
- ❌ HTTP + MCP 同进程双协议运行（当前 `--http` `--mcp` 互斥；可改为多 future select 但 stdio 仍是 MCP 专用）
