# 快速开始

## 5 分钟跑通本地 CLI

### 1. 下载

```bash
# 从 GitHub Releases 下（macOS Apple Silicon 示例）
curl -LO https://github.com/LeoG1022/RAG4ArkUI/releases/download/v1.0.0/arkui-rag-v1.0.0-aarch64-apple-darwin.tar.gz
tar -xzf arkui-rag-v1.0.0-aarch64-apple-darwin.tar.gz
cd arkui-rag-v1.0.0-aarch64-apple-darwin
./arkui-rag --version    # arkui-rag 0.0.1
```

其它平台：见 [Release 与分发](operations/release.md)。

### 2. 拉取默认 corpus（一键 · Day 21 真活）

```bash
./arkui-rag corpus pull --target ./corpus/official
```

或离线：

```bash
./arkui-rag corpus pull --from-file ./my-corpus.tar.gz --target ./corpus/official
```

### 3. 建索引

```bash
./arkui-rag index \
    --source ./corpus/official \
    --index-path ./corpus/official/index.json \
    --bm25 tantivy
```

输出：
```
✅ 索引完成
   embedder    : mock-384
   bm25        : tantivy
   files       : 3
   chunks      : 22
   elapsed_ms  : 132
```

### 4. 检索

```bash
./arkui-rag query \
    --text "@State 双向绑定" \
    --index-path ./corpus/official/index.json \
    --bm25 tantivy -k 5
```

返回 Top-K 命中 + 引用溯源（source / heading_path / line_range / score）。

### 5. 启常驻服务

按集成方式三选一：

```bash
# Claude Code / Cursor 用（MCP stdio）
./arkui-rag serve --mcp --index-path ./corpus/official/index.json --bm25 tantivy

# IDE 插件用（HTTP REST）
./arkui-rag serve --http --addr 127.0.0.1:7654 --index-path ./corpus/official/index.json --bm25 tantivy

# DevEco / IntelliJ inline 提示用（LSP stdio）
./arkui-rag serve --lsp --index-path ./corpus/official/index.json --bm25 tantivy
```

详细接入：
- [HTTP REST API](usage/http.md)
- [MCP 协议（Claude Code / Cursor）](usage/mcp.md)
- [LSP 协议（IDE 内联）](usage/lsp.md)

## 开发者：从源码

```bash
git clone https://github.com/LeoG1022/RAG4ArkUI
cd RAG4ArkUI
make install-rust       # 检查 / 提示安装 rust 工具链
make check              # cargo check --workspace
make release-local      # 编译 release tarball 到 dist/
```
