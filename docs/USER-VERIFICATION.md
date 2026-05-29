# 用户端到端本地验证清单

> 用途：让你自己跑一遍 · 确认所有本地能力真活。
> 估计耗时：**首次 30 分钟**（含 cargo 编译）· 二次跑 5 分钟。
> 适用范围：~92% MVP 能力（不含 ONNX 真语义 embedding · task #87 阻塞）。

每步给出**命令** + **期望输出** + **失败时怎么办**。按顺序跑，前一步过了再跑下一步。

---

## 0. 环境准备（一次性）

| 工具 | 装法 | 验证 |
|---|---|---|
| Rust toolchain | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` | `cargo --version` 应显示 1.75+ |
| (lancedb 需要) protoc | `brew install protobuf`（macOS）/ `apt install protobuf-compiler`（Linux） | `protoc --version` 应显示 3.x 或 35+ |
| (mdBook 文档站需要) mdbook | `brew install mdbook` 或 `cargo install mdbook --locked` | `mdbook --version` 应显示 0.5+ |

### 跑工作树前置检查

```bash
cd /Users/leo/work/RAG4ArkUI   # 或你 clone 的路径
git status                      # 应该 clean（或只有 Cargo.lock）
bash scripts/preflight.sh       # 应该 PASS
```

---

## 1. 默认 features 编译（最快验证 · 2 分钟）

```bash
make check
```

**期望**：`Finished dev profile [unoptimized + debuginfo] target(s)` 无 error · 最多有 1 个 `unused_mut` warning（pre-existing · 不阻塞）。

**失败时**：粘 cargo error 给我。

---

## 2. 全 workspace 测试（4-5 分钟）

```bash
cd crates && cargo test --workspace --no-fail-fast 2>&1 | tail -25
```

**期望**：所有 `test result:` 行都是 `ok. X passed; 0 failed`。汇总应该是：
- **56 passed / 0 failed / 11 ignored**
- 11 个 ignored 是：1 ArkTS struct（pre-existing limitation · custom grammar 需求）+ 10 doctest（tokio_test dep 未添加）

**失败时**：把 FAILED 那行 + 后面 10 行 stderr 粘给我。

---

## 3. smoke 端到端（30 秒）

```bash
make smoke
```

**期望**：
```
═══ [4/4] 通过 ═══
  ✅ index + query 端到端跑通
🎉 Day 2 Mock RAG smoke PASS
```

**失败时**：跑 `bash scripts/demo-smoke.sh --keep --verbose` 看详细 + 保留 tmp 目录。

---

## 4. MCP 端到端（30 秒）

```bash
make mcp-demo
```

**期望**：
```
═══ [4/4] 解析响应 + 断言 ═══
  ✅ initialize: protocolVersion=2024-11-05
  ✅ tools/list: 4 个工具齐全
  ✅ tools/call: 返回 markdown 文本 + 命中 list.md
🎉 Day 19 MCP 端到端演示 PASS
```

**失败时**：`bash scripts/mcp-demo.sh --keep --verbose` 看 stderr 日志。

---

## 5. Release 二进制打包（首次 5 分钟）

```bash
make release-local-verify
```

**期望**：
```
[1/4] cargo build --release --features http,mcp,lsp,tantivy,typescript,corpus-pull
[2/4] 烟雾测试 --version → arkui-rag 0.0.1
[3/4] 暂存产物
[4/4] 打包为 .tar.gz
✅ Release artifact 完成
  产物 : dist/arkui-rag-v0.0.1-aarch64-apple-darwin.tar.gz (4.x MB)
━━━ 解压验证 ━━━
arkui-rag 0.0.1
✅ release tarball 端到端可用
```

产物在 `dist/`。把 binary 装到 PATH 可选：
```bash
cp dist/arkui-rag-v0.0.1-aarch64-apple-darwin/arkui-rag /usr/local/bin/
arkui-rag --version    # 全局可调
```

---

## 6. CLI 完整业务流（5 分钟）

### 6.1 拉 corpus（占位 URL 当前 404，用 `--from-file`）

```bash
# 把仓库自带的 mapping doc 当 corpus
mkdir -p /tmp/rag-verify/corpus
cp .claude/references/mapping-list.md \
   .claude/references/mapping-state.md \
   .claude/references/mapping-async.md \
   /tmp/rag-verify/corpus/

# 用 corpus pull --from-file 也可以（验证 pull 路径）
# 先打个本地 tarball：
mkdir -p /tmp/rag-verify/staging/test-corpus-v1/official
cp .claude/references/mapping-*.md /tmp/rag-verify/staging/test-corpus-v1/official/
(cd /tmp/rag-verify/staging && tar -czf /tmp/rag-verify/test-corpus-v1.tar.gz test-corpus-v1)

./crates/target/release/arkui-rag corpus pull \
    --from-file /tmp/rag-verify/test-corpus-v1.tar.gz \
    --target /tmp/rag-verify/corpus-pulled \
    --force
```

**期望**：
```
✅ corpus 拉取完成
   文件数    : 8（含 macOS AppleDouble · 实际 3 个 md）
```

### 6.2 模型 pull（用 fake tarball 验证路径 · 不下真模型）

```bash
mkdir -p /tmp/rag-verify/fake-model/bge-m3-v1/model
echo "fake" > /tmp/rag-verify/fake-model/bge-m3-v1/model/model.onnx
echo '{"vocab_size":250002}' > /tmp/rag-verify/fake-model/bge-m3-v1/tokenizer.json
(cd /tmp/rag-verify/fake-model && tar -czf /tmp/rag-verify/bge-m3-fake.tar.gz bge-m3-v1)

./crates/target/release/arkui-rag corpus model-pull \
    --name bge-m3 \
    --from-file /tmp/rag-verify/bge-m3-fake.tar.gz \
    --target /tmp/rag-verify/models/bge-m3 \
    --force
```

**期望**：
```
✅ 模型拉取完成
   model    : bge-m3
   目标     : /tmp/rag-verify/models/bge-m3
```

### 6.3 建索引（默认 in-memory + Tantivy BM25）

```bash
./crates/target/release/arkui-rag index \
    --source /tmp/rag-verify/corpus \
    --index-path /tmp/rag-verify/index.json \
    --bm25 tantivy
```

**期望**：
```
✅ 索引完成
   embedder    : mock-384
   files       : 3
   chunks      : 22
```

### 6.4 检索

```bash
./crates/target/release/arkui-rag query \
    --text "@State 双向绑定" \
    --index-path /tmp/rag-verify/index.json \
    --bm25 tantivy -k 3
```

**期望**：Top-3 命中 · 第 1 个应该是 `mapping-state.md L24-34`「状态选择决策」。

### 6.5 检索 + HyDE 改写

```bash
./crates/target/release/arkui-rag query \
    --text "@State 双向绑定" \
    --index-path /tmp/rag-verify/index.json \
    --bm25 tantivy --hyde mock -k 3
```

**期望**：Top-K 返回（mock embedder 下 HyDE 不会改变命中 · 但路径跑了）。

### 6.6 检索 + 父级扩展

```bash
./crates/target/release/arkui-rag query \
    --text "@State 双向绑定" \
    --index-path /tmp/rag-verify/index.json \
    --bm25 tantivy --expand-parent -k 2
```

**期望**：每个 hit 末尾多一行 `↳ parent (Mapping — ... L2-4): ...` 显示父级 chunk。

### 6.7 评估报告

```bash
./crates/target/release/arkui-rag eval \
    --queries corpus/_eval/queries.yaml \
    --index-path /tmp/rag-verify/index.json \
    --bm25 tantivy -k 5 \
    --report-path /tmp/rag-verify/eval-report.md
```

**期望**：
```
📊 跑评估：8 个 query
✅ 评估完成
   avg recall@5   : 0.000     ← 预期 · 因为 fixture GT chunk_id 与你 corpus 不匹配
   report saved   : /tmp/rag-verify/eval-report.md
```

打开 `cat /tmp/rag-verify/eval-report.md` 看报告结构（含 8 query 各自 recall + latency 分位）。

---

## 7. 三协议 server 验证（每个 30 秒）

⚠️ 这三个**互斥** · 一次只跑一个 · `Ctrl-C` 停。

### 7.1 HTTP

终端 1：
```bash
./crates/target/release/arkui-rag serve --http --addr 127.0.0.1:7654 \
    --index-path /tmp/rag-verify/index.json --bm25 tantivy
```

**期望**：log 出现 `HTTP server listening on 127.0.0.1:7654`。

终端 2：
```bash
curl http://127.0.0.1:7654/health
curl -X POST http://127.0.0.1:7654/search \
    -H "Content-Type: application/json" \
    -d '{"query":"@State 双向绑定","top_k":3}' | head -c 500
```

**期望**：`/health` 返回 `{"status":"ok",...}` · `/search` 返回 JSON 含 `hits` 数组。

`Ctrl-C` 停 server。

### 7.2 MCP（手动 stdio）

```bash
printf '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"manual","version":"1"}}}\n{"jsonrpc":"2.0","id":2,"method":"tools/list"}\n' \
    | ./crates/target/release/arkui-rag serve --mcp \
        --index-path /tmp/rag-verify/index.json --bm25 tantivy \
    2>/dev/null | head -c 800
```

**期望**：2 行 JSON-RPC 响应 · 第 1 行含 `"protocolVersion":"2024-11-05"` · 第 2 行 `tools/list` 返回 4 个 tool。

### 7.3 LSP（Content-Length framing）

```bash
printf 'Content-Length: 75\r\n\r\n{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"capabilities":{}}}' \
    | ./crates/target/release/arkui-rag serve --lsp \
        --index-path /tmp/rag-verify/index.json --bm25 tantivy \
    2>/dev/null | head -c 400
```

**期望**：响应有 `Content-Length: ...\r\n\r\n` header · body JSON 含 `"hoverProvider":true` `"executeCommandProvider":{...}`。

---

## 8. lancedb feature（可选 · 首次 cargo build 10+ 分钟）

如果想验证 lancedb 真活：

```bash
# 编译（首次很慢 · lance + arrow 是大依赖）
cd crates && cargo build --release -p arkui-rag-cli --features corpus-pull,tantivy,lancedb 2>&1 | tail -3

# 用 --vector lancedb 重建索引
../crates/target/release/arkui-rag index \
    --source /tmp/rag-verify/corpus \
    --index-path /tmp/rag-verify/index-lance.json \
    --bm25 tantivy --vector lancedb

# 检索
../crates/target/release/arkui-rag query \
    --text "@State 双向绑定" \
    --index-path /tmp/rag-verify/index-lance.json \
    --bm25 tantivy --vector lancedb -k 3
```

**期望**：log 出现 `KNNVectorDistance(FilteredRead)` 真向量索引 · Top-K 与 in-memory 结果一致。

**失败时**：通常是 protoc 没装 · `brew install protobuf` 后重试。

---

## 9. mdBook 文档站（可选 · 30 秒）

```bash
make book-serve   # 自动开浏览器 http://localhost:3000
```

**期望**：浏览器打开 RAG4ArkUI 文档站 · 13 个 page 都能点 · 全文搜索可用。

`Ctrl-C` 停。

---

## ✅ 验证全过的标志

| 步骤 | 通过标志 |
|---|---|
| 1 make check | dev profile 编完 |
| 2 cargo test --workspace | 56 passed / 0 failed |
| 3 make smoke | 🎉 PASS |
| 4 make mcp-demo | 🎉 PASS |
| 5 make release-local-verify | tarball 4.x MB + ✅ 端到端可用 |
| 6 CLI 7 子步骤 | 都返回预期输出 |
| 7 三协议 | 各自响应正确 |
| 8 lancedb（可选） | KNN log + Top-K 一致 |
| 9 mdBook（可选） | 浏览器站点可用 |

**全过 = MVP ~92% 真活 dogfood 起来**。

---

## 故障常见原因

| 现象 | 通常原因 | 修法 |
|---|---|---|
| `cargo: command not found` | rust 没装 | 跑 0 节 |
| `protoc not found` | protobuf 没装 · lancedb feature 失败 | `brew install protobuf` |
| `mdbook: command not found` | mdbook 没装 | `brew install mdbook` |
| 端口 7654 占用 | 上次 serve 没退 | `lsof -i :7654` 看占用进程 + kill · 或换 `--addr 127.0.0.1:8765` |
| MCP/LSP 返回空 | server 立即 EOF 退出（stdin 不挂） | 用 `printf '...' \| arkui-rag serve` 形式 · 不是裸跑 |
| corpus pull `--url` 默认 404 | GitHub Release `corpus-v0.0.1` 还没推 | 用 `--from-file` 或自定义 `--url` |
| eval recall=0 | fixture GT chunk_id 不匹配你的 corpus | 是预期 · 改 `corpus/_eval/queries.yaml` 用你 corpus 真有的 chunk_id |

---

## 跑完之后

如果全过，把 `dist/arkui-rag-v0.0.1-<TRIPLE>/arkui-rag` 复制到 `/usr/local/bin/` 当真工具用：

```bash
sudo cp dist/arkui-rag-v0.0.1-aarch64-apple-darwin/arkui-rag /usr/local/bin/
arkui-rag --version    # 全局可调
```

之后就可以在任何项目目录跑 `arkui-rag index` / `arkui-rag query` / `arkui-rag serve --mcp` 接 Claude Code 等 agent。

有任何步骤 fail · 把命令 + 完整 stderr 贴给 agent 帮你定位。
