# STATUS · Day 16 · LSP Server

> 日期：2026-05-28
> 对应 commit：[本 commit · Day 16 LSP]
> 对应 feature log：[`feedback/features/rag4arkui-core/19-2026-05-28-day16-lsp.md`](../feedback/features/rag4arkui-core/19-2026-05-28-day16-lsp.md)
> 上一阶段：[`STATUS-day19-claude-code.md`](STATUS-day19-claude-code.md)
> 下一阶段：`STATUS-day17-deveco.md`（推荐 · DevEco Plugin MVP）或 `STATUS-day20-release.md`（发布相关）

> 🎯 **里程碑**：**协议层 3/3 完整收尾** ⭐（HTTP + MCP + **LSP** 三协议全部真活）· Week 4 协议层全部达成。

---

## 当前状态

| 模块 | 变化 |
|---|---|
| `arkui-rag-server/src/lsp.rs` | **重写** ~430 行 · Content-Length framing + JSON-RPC + 5 method + 6 单测 · feature gated |
| `arkui-rag-server/src/lib.rs` | cfg gate · 导出 serve_lsp_stdio |
| `arkui-rag-cli/Cargo.toml` | `lsp = ["arkui-rag-server/lsp"]` · full 加入 lsp |
| `arkui-rag-cli/src/main.rs` | cmd_serve_lsp + cfg 双路径 · **三协议互斥校验** |
| `Makefile` | + `check-lsp` / `build-lsp` / `serve-lsp-demo` |
| `docs/ADR-002` + `crates/README.md` + `docs/ROADMAP.md` | 速查表 + 路线图同步 · 协议层 3/3 ⭐ |

### 测试覆盖（本地实测）

| 测试组 | 数量 |
|---|---|
| lsp.rs 单测（feature gated） | **6** · initialize / search / shutdown→exit / hover stub / unknown method / notification |
| `cargo test -p arkui-rag-server --features lsp` | **6 通过 / 0 失败** |
| `cargo test -p arkui-rag-server --features http,mcp,lsp` | **12 单测 + 5 e2e = 17 通过 / 0 失败** |
| `make check` / `make check-lsp` | ✅ 通过 |

> 注：完整 `make test`（workspace 全 crate）残留 `arkui-rag-eval` / `arkui-rag-indexer` 两个 pre-existing 运行时 assertion 回归（Day 11 chunk 语义变更后失活），与本轮 LSP 无关，已挂 follow-up。

---

## 输入契约

### CLI 启动

```bash
# 编译
make build-lsp
make serve-lsp-demo
# 或：
cargo run --features lsp -p arkui-rag-cli -- serve --lsp

# 全功能配置
cargo run --features full -p arkui-rag-cli -- serve --lsp \
    --embedder onnx --model-path ~/.arkui-rag/models/bge-m3 \
    --bm25 tantivy --vector lancedb \
    --rerank onnx --reranker-model-path ~/.arkui-rag/models/bge-reranker
```

### IDE 接入示例（伪代码）

```typescript
// IDE LSP client（DevEco Plugin / VSCode Extension）
const client = new LSPClient({
    command: "arkui-rag",
    args: ["serve", "--lsp", "--index-path", "/path/to/index.json"]
});

// 标准 LSP handshake
await client.sendRequest("initialize", {});
client.sendNotification("initialized", {});

// 自定义 method 调用
const result = await client.sendRequest("arkui-rag/search", {
    query: "下拉刷新",
    top_k: 5
});
// result.content.value = "# 📚 RAG4ArkUI 检索结果 · Top-5\n\n..."
// result.hits = [{ chunk_id, source, heading_path, line_range, score }, ...]

// 优雅退出
await client.sendRequest("shutdown", null);
client.sendNotification("exit", null);
```

### JSON-RPC 帧格式（LSP framing）

```
Content-Length: 56\r\n
\r\n
{"jsonrpc":"2.0","id":1,"method":"initialize"}
```

每条消息前必带 `Content-Length: <bytes>\r\n\r\n` 头。其他 header（Content-Type 等）可选。

---

## 输出契约

### initialize 响应

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "capabilities": {
      "textDocumentSync": 0,
      "hoverProvider": true,
      "executeCommandProvider": {
        "commands": ["arkui-rag.search", "arkui-rag.migrate"]
      }
    },
    "serverInfo": {
      "name": "arkui-rag",
      "version": "0.0.1"
    }
  }
}
```

### `arkui-rag/search` 响应

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "content": {
      "kind": "markdown",
      "value": "# 📚 RAG4ArkUI 检索结果 · Top-3\n\n## [1] `list.md` · L9-11 · score=0.0163\n\n**Heading**: List > 下拉刷新\n\n```\nArkUI-X 用 Refresh 组件实现下拉刷新。\n```\n\n..."
    },
    "hits": [
      {
        "chunk_id": "list.md#List/下拉刷新@9",
        "source": "list.md",
        "heading_path": ["List", "下拉刷新"],
        "line_range": [9, 11],
        "score": 0.0163
      }
    ]
  }
}
```

### shutdown / exit 双阶段

```
Client → shutdown        Server ← null
Client → arkui-rag/search Server ← error -32600 "server is shutdown · only 'exit' allowed"
Client → exit            (no response · server stops)
```

### 错误响应

JSON-RPC 2.0 错误码：
- -32700 parse error
- -32600 invalid request（shutdown 后调其他 method）
- -32601 method not found
- -32602 invalid params
- -32603 internal error

---

## 验证手段

### 用户手动

```bash
# 1. 默认编译（不拉 lsp）
make check
make test                                  # 默认 49 测试

# 2. lsp feature 编译（快 · 仅 tokio + serde_json）
make check-lsp
cd crates && cargo test -p arkui-rag-server --features lsp
# 期望 6 单测全过

# 3. CLI 启动 + 手工 LSP framing
make build-lsp
# 启动（前台）
cargo run --features lsp -p arkui-rag-cli -- serve --lsp

# 4. 真实 IDE 接入：DevEco Plugin / VSCode Extension（Day 17 续）
```

### 自动化

| 手段 | 范围 | 状态 |
|---|---|---|
| lsp.rs 单测 × 6 | initialize / search / shutdown→exit / hover stub / unknown / notification | ✅ feature gated |
| **M-STATUS-PER-ROUND** | Round 19 + STATUS-day16 配套 | ✅ |
| **ROADMAP 维护约定（第 8 次实战）** | 9 处进度行同步 | ✅ |

### 暂未自动化（明确缺口）

- ❌ Framing IO 层集成测（Rust subprocess + Content-Length 包装）· Day 16 续
- ❌ textDocument/hover 真活（结合 corpus + 位置追踪）
- ❌ textDocument/completion 真活（内联补全）
- ❌ textDocument/codeAction（"Migrate to ArkUI-X" 按钮）
- ❌ publishDiagnostics 推送（API 时效性警告）
- ❌ DevEco Plugin / VSCode Extension 真实接入端到端
- ❌ LSP 协议合规性测（与官方 lsp-types crate 对比）

---

## 与上一阶段（STATUS-day19）的关联性

### 增量

| 维度 | Day 19 完成时 | 本轮（Day 16）后 |
|---|---|---|
| Crate 数 | 9 | 不变 |
| 协议层数 | 2（HTTP + MCP） | **3（HTTP + MCP + LSP）** ⭐ |
| Agent 接入 | Claude Code（MCP） · IDE HTTP | + **IDE LSP**（DevEco / VSCode / IntelliJ） |
| CLI feature 数 | 9 | **10**（+ lsp） |
| 测试数（lsp 启用） | — | **6 单测** |
| Week 4 完成度 | 2/3 | **3/3** ⭐ |

### 与 §4.2 决策 2 完整对齐

| 接口 | 状态 |
|---|---|
| MCP Server (stdio) | ✅ Day 15 |
| MCP Server (SSE) | ⏳ Day 15 续 |
| HTTP REST API | ✅ Day 14 |
| **LSP Custom Commands** | **✅ Day 16（本轮）** ⭐ |

方案 §4.2 决策 2 协议层三件套**全部完成**。

### 业界基线 + 协议层全套

| 维度 | 状态 |
|---|---|
| §1.6 共识 1 Hybrid | ✅ Day 4 |
| §8.5 共识 2 Reranker | ✅ Day 5 |
| §8.5 共识 3 引用溯源 | ✅ Day 2 |
| §8.5 共识 4 Eval-Driven | ✅ Day 6 |
| §1.4 Parent-Child | ✅ Day 11 |
| §4.2 决策 2 协议层 HTTP | ✅ Day 14 |
| §4.2 决策 2 协议层 MCP | ✅ Day 15 |
| **§4.2 决策 2 协议层 LSP** | **✅ Day 16（本轮）** ⭐ |

### 兼容性

- ✅ 无破坏性变更
- ✅ lsp feature 默认关 · 老编译路径不变
- ✅ AppState trait object 与 HTTP/MCP 共享
- ✅ 三协议互斥校验 · 单进程占一个 stdio/端口

---

## 完成度 / 下一阶段

### Day 16 完成度

| 项 | 状态 |
|---|---|
| LSP Content-Length framing | ✅ |
| 5 真活 method + 2 stub + 5 已知忽略 | ✅ |
| shutdown 状态机 | ✅ |
| 6 单测覆盖 | ✅ |
| CLI serve --lsp + feature 转发 + 三协议互斥 | ✅ |
| Makefile + 文档 + ROADMAP | ✅ |
| textDocument/hover 真活 | ⏳ Day 16 续 |
| textDocument/completion / codeAction / diagnostics | ⏳ Day 16 续 |
| Framing IO 集成测 | ⏳ Day 16 续 |

### 6 周路线图达成度更新

| 章节 | 状态 |
|---|---|
| Week 1: Rust 骨架 + tree-sitter + LanceDB + Tantivy + BGE-M3 | **7/7** ✅ |
| Week 2: 混合检索 + Reranker + HyDE + 评估集 | **4/4** ✅ |
| Week 3: HTTP + MCP + CLI | **3/3** ✅ |
| **Week 4: 协议层（HTTP + MCP + LSP）** | **3/3** ✅ ⭐ |
| Week 5: Claude Code 接入 | **1/1** ✅ |
| Week 6: 发布 + 文档站 + 评估报告 | **1/4** ✅ |

**总完成度估算：~75%**

### 下一阶段建议（按推荐优先级）

| 候选 | 价值 | 工作量 |
|---|---|---|
| 🟢 **Day 17 DevEco Plugin MVP**（强推 · §4.3 主战场） | 关键路径 · IDE 集成完整业务闭环 · ArkUI-X 主舞台 | 5+ commit · 大工程 |
| 🟢 Day 20 跨平台二进制构建 | 解锁分发（macOS/linux/windows） · Week 6 起点 | 1-2 commit |
| 🟢 Day 18 VSCode Extension | 跨编辑器覆盖 · 比 DevEco 工作量小 | 3+ commit |
| 🟡 Day 16 续 | textDocument/hover/completion 真活 · publishDiagnostics | 1-2 commit |
| 🟡 Day 8 tantivy-jieba | 中文 BM25 精度 | 0.5 commit |
| 🟡 Day 21 model-pull 真活 | corpus 分发管道 | 1 commit |

**Agent 推荐**：**Day 20 跨平台二进制构建**（轻量 · 解锁分发）。理由：
1. 协议层完整后 · 用户需要分发的二进制
2. 工作量小（1-2 commit · 主要是 GitHub Actions release workflow）
3. 完成后整个 RAG4ArkUI 可被 Mac / Linux / Windows 用户开箱即用
4. 之后再上 Day 17 DevEco（大工程）or Day 21 model-pull

**备选**：**Day 17 DevEco Plugin MVP**（关键路径主战场，但工作量大）。

### 重要的"非完成"项

- ❌ LSP textDocument/* 真活（hover / completion / codeAction / diagnostics）
- ❌ LSP framing IO 集成测
- ❌ DevEco Plugin / VSCode Extension 真实接入端到端
- ❌ MCP SSE 传输（Day 15 续）
- ❌ 公网部署 CORS / 鉴权 / TLS（Day 14 续）
