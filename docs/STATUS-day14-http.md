# STATUS · Day 14 · HTTP/REST Server

> 日期：2026-05-28
> 对应 commit：[本 commit · Day 14 HTTP]
> 对应 feature log：[`feedback/features/rag4arkui-core/16-2026-05-28-day14-http.md`](../feedback/features/rag4arkui-core/16-2026-05-28-day14-http.md)
> 上一阶段：[`STATUS-day11-parent-child.md`](STATUS-day11-parent-child.md)
> 下一阶段：`STATUS-day15-mcp.md`（推荐 · Claude Code 直接接入）或 `STATUS-day8-jieba.md`

> 🎯 **里程碑**：**协议层入门** · 关键路径起点突破 · 9 crate 能力可被外部消费。

---

## 当前状态

| 模块 | 变化 |
|---|---|
| `arkui-rag-server/src/http.rs` | **重写** ~260 行 · AppState + 4 handler · feature gated |
| `arkui-rag-server/src/lib.rs` | 导出 AppState / build_router / serve_http |
| `arkui-rag-server/tests/http_e2e.rs` | **新增** 5 集成测（tower::oneshot 不真起监听） |
| `arkui-rag-cli/src/main.rs` | Serve subcommand 全参 · cmd_serve_http 真活 · cfg 双路径 |
| `arkui-rag-cli/Cargo.toml` | feature http 转发 · full 加入 http |
| `Makefile` | + check-http / build-http / serve-demo |
| `docs/ROADMAP.md` | 第 5 次实战 · 9 处进度行同步 |

### 测试覆盖

| 测试组 | 数量 |
|---|---|
| http_e2e.rs（feature gated · tower oneshot） | 5（health/corpus_list/search/search+expand_parent/index-stub） |
| **默认 features 累计** | 49（不变 · http 默认关闭） |
| **`--features http` 累计** | **54** |
| 全 feature（`--features full`） | 约 **74** |

---

## 输入契约

### CLI 启动（用户视角）

```bash
# 1. 编译 + 启动（默认 mock embedder / memory bm25 / memory vector）
make build-http
make serve-demo
# 或直接：
cargo run --features http -p arkui-rag-cli -- serve --http

# 2. 全功能配置
cargo run --features full -p arkui-rag-cli -- serve --http \
    --addr 0.0.0.0:7654 \
    --embedder onnx --model-path ~/.arkui-rag/models/bge-m3 \
    --bm25 tantivy \
    --vector lancedb \
    --rerank onnx --reranker-model-path ~/.arkui-rag/models/bge-reranker \
    --hyde mock
```

### HTTP API（外部消费者视角）

```bash
# 健康检查
curl http://127.0.0.1:7654/health
# {"status":"ok","embedder":"mock-384","embedder_dim":384,"vector":"memory","bm25":"memory",
#  "rerank_enabled":false,"enhancer":"passthrough","pre_rerank_k":50}

# Corpus 子目录
curl http://127.0.0.1:7654/corpus/list
# {"dirs":[{"name":"official","exists":true,"docs":12}, ...]}

# 检索
curl -X POST http://127.0.0.1:7654/search \
     -H 'content-type: application/json' \
     -d '{"query":"下拉刷新","k":3,"enhance_query":false,"expand_parent":true}'
# {"hits":[...],"latency_ms":34,"embedder":"mock-384","bm25":"memory","rerank":null,"enhancer":"passthrough"}

# 触发索引（stub）
curl -X POST http://127.0.0.1:7654/index \
     -H 'content-type: application/json' \
     -d '{"source":"corpus"}'
# {"status":"stub","message":"... 请用 CLI ..."}
```

### 库 API

```rust
#[cfg(feature = "http")]
use arkui_rag_server::{AppState, build_router, serve_http, HttpOptions};

let state = AppState {
    retriever: Arc<dyn Retriever>,
    reranker: Option<Arc<dyn Reranker>>,
    enhancer: Arc<dyn QueryEnhancer>,
    metadata_store: Option<Arc<dyn MetadataStore>>,
    pre_rerank_k: 50,
    embedder_model_id: "bge-m3".into(),
    embedder_dim: 1024,
    bm25_name: "tantivy".into(),
    vector_name: "lancedb".into(),
};
let opts = HttpOptions { addr: "127.0.0.1:7654".parse()? };
serve_http(&opts, state).await?;

// 或测试时用 build_router 直接构造 axum Router
let app = build_router(Arc::new(state));
```

---

## 输出契约

### `SearchResponse` JSON Schema

```json
{
  "hits": [
    {
      "chunk_id": "list.md#List/下拉刷新@10",
      "score": 0.85,
      "source": "hybrid",
      "citation": {
        "chunk_id": "list.md#List/下拉刷新@10",
        "source": "list.md",
        "heading_path": ["List", "下拉刷新"],
        "line_range": [10, 12],
        "score": 0.85
      },
      "content_preview": "ArkUI-X 用 Refresh 组件实现下拉刷新。...",
      "parent_preview": "# List\n\n## 下拉刷新\n...",
      "parent_chunk_id": "list.md#List@1"
    }
  ],
  "latency_ms": 34,
  "embedder": "mock-384",
  "bm25": "memory",
  "rerank": null,
  "enhancer": "passthrough"
}
```

### 错误响应

```
HTTP 400/500/501
Content-Type: application/json

{
  "error": "embedding error: ...",
  "kind": "embedding"
}
```

错误码映射：
| RagError variant | HTTP status |
|---|---|
| `NotImplemented` | 501 |
| `Config` | 400 |
| 其他 | 500 |

### 命令行启动日志

```
🚀 HTTP server starting on http://127.0.0.1:7654
   embedder=mock-384 · bm25=memory · vector=memory · rerank=off
   routes: GET /health · GET /corpus/list · POST /search · POST /index (stub)
INFO arkui_rag_server::http: HTTP server listening on http://127.0.0.1:7654
```

---

## 验证手段

### 用户手动

```bash
# 1. 默认编译（不拉 axum）
make check
make test                            # 默认 49 测试

# 2. http feature 编译（首次拉 axum + hyper ~1-2 分钟）
make check-http
cd crates && cargo test -p arkui-rag-server --features http
# 期望 5 集成测全过

# 3. 端到端 demo（需先 arkui-rag index）
cd crates && cargo run -p arkui-rag-cli -- index --source ../corpus
cd ..
make serve-demo                      # 启动 server

# 另开终端：
curl http://127.0.0.1:7654/health | jq
curl http://127.0.0.1:7654/corpus/list | jq
curl -X POST http://127.0.0.1:7654/search \
     -H 'content-type: application/json' \
     -d '{"query":"下拉刷新","k":3}' | jq

# Ctrl+C 优雅退出
```

### 自动化

| 手段 | 范围 | 状态 |
|---|---|---|
| http_e2e.rs × 5 | health/corpus_list/search/expand_parent/index-stub | ✅ feature gated |
| **M-STATUS-PER-ROUND** | Round 16 + STATUS-day14 配套 | ✅ |
| **ROADMAP 维护约定（第 5 次实战）** | 9 处进度行同步 | ✅ |

### 暂未自动化（明确缺口）

- ❌ 真起 listener 的 end-to-end（监听端口 → reqwest 调）—— 集成测用 oneshot，不验证 TCP 层
- ❌ 并发压测（criterion + Vegeta）
- ❌ 优雅退出测试（SIGTERM 信号）
- ❌ OpenAPI schema 文件 + 自动生成 client
- ❌ CORS / 鉴权 / rate limit
- ❌ POST /index 真活 + 进度查询

---

## 与上一阶段（STATUS-day11-parent-child）的关联性

### 增量

| 维度 | Day 11 完成时 | 本轮（Day 14）后 |
|---|---|---|
| Crate 数 | 9 | 不变 |
| Server 协议 | 全 stub（HTTP/MCP/LSP） | **HTTP 真活**（MCP/LSP 仍 stub） |
| 消费者类型 | 仅 CLI | CLI + HTTP（IDE 插件 / curl / 外部 agent） |
| CLI feature 数 | 7 | **8**（+ http） |
| 测试数（默认） | 49 | 49（不变） |
| 测试数（http 启用） | — | **54** |

### 兼容性

- ✅ 无破坏性变更
- ✅ HTTP feature 默认关 · 老用户编译路径不变
- ✅ `serve` subcommand 之前是 stub · 现在 Day 14 起 `--http` 真活、`--mcp` `--lsp` 仍 stub
- ✅ AppState 用 trait object · 任何后端组合都能注入

### 业界基线对照（再次更新）

| 业界共识 | 状态 |
|---|---|
| §1.6 共识 1 Hybrid 检索 | ✅ Day 4 |
| §8.5 共识 2 Reranker 分水岭 | ✅ Day 5 |
| §8.5 共识 3 引用溯源 | ✅ Day 2 |
| §8.5 共识 4 Eval-Driven | ✅ Day 6 |
| §1.4 Parent-Child 索引 | ✅ Day 11 |
| **协议层暴露（HTTP）** | **✅ Day 14（本轮）** |

---

## 完成度 / 下一阶段

### Day 14 完成度

| 项 | 状态 |
|---|---|
| http.rs 真活（4 端点） | ✅ |
| AppState trait object 抽象 | ✅ |
| CLI Serve subcommand 真活 | ✅ |
| 5 集成测覆盖 | ✅ |
| Makefile build-http / serve-demo | ✅ |
| ROADMAP 维护约定第 5 次实战 | ✅ |
| POST /index 真活 | ⏳ Day 14 续 |
| OpenAPI schema | ⏳ Day 14 续 |
| CORS / 鉴权 | ⏳ Week 4+ |

### 6 周路线图达成度更新

| 章节 | 状态 |
|---|---|
| Week 1: Rust 骨架 + tree-sitter + LanceDB + Tantivy + BGE-M3 | **7/7** ✅ |
| Week 2: 混合检索 + Reranker + HyDE + 评估集 | **4/4** ✅ |
| Week 3: HTTP + MCP + CLI | **2/3** ✅ HTTP ✓ · CLI ✓ · **MCP ⏳** |

**总完成度估算：~60%**

### 下一阶段建议（按推荐优先级）

| 候选 | 价值 | 工作量 |
|---|---|---|
| 🟢 **Day 15 MCP Server**（强推 · 关键路径） | **Claude Code 直接接入** · 方案 §4.2 决策 2 一等公民 | 3-4 commit |
| 🟢 Day 16 LSP Server | IDE 内联补全 / diagnostic | 2-3 commit |
| 🟢 Day 14 续 | POST /index 真活 + OpenAPI schema + 鉴权 | 1-2 commit |
| 🟡 Day 8 tantivy-jieba | 中文 BM25 精度 | 0.5 commit |
| 🟡 Day 12 Query Router | 不同 query 走不同流水线 | 1 commit |

**Agent 推荐**：**Day 15 MCP Server**。理由：
1. 方案 §4.2 决策 2 明确："MCP 是 2025 年的事实标准，必须作为一等公民支持"
2. 完成后 **Claude Code / Cursor / OpenCode 直接接入**，差异化护城河兑现
3. 与 Day 14 HTTP server 共享 AppState 抽象，工作量复用度高
4. 关键路径下一站（Day 17 DevEco Plugin 之前的协议层完整化）

### 重要的"非完成"项

- ❌ MCP / LSP 协议（Day 15 / 16）
- ❌ POST /index 真活（任务异步化 + 进度回报）
- ❌ OpenAPI / TypeScript client 自动生成
- ❌ 真起 listener 的 E2E 测试（只有 oneshot）
- ❌ 公网部署的 CORS / 鉴权 / TLS
