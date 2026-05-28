# RAG4ArkUI · Release 与分发指南

> Day 20 本地 CLI 端到端分发版本。CI matrix 自动 release 留 Day 20 续。

---

## 当前 Release 状态（Day 20）

| 维度 | 状态 |
|---|---|
| 本地 host 平台 release 二进制 | ✅ 可用（`make release-local`） |
| Apple Silicon arm64（aarch64-apple-darwin） | ✅ 本地实测通过 |
| 其它平台（x86_64-darwin / linux / windows） | ⏳ 待 CI matrix 启动（Day 20 续） |
| GitHub Releases 自动上传 | ⏳ Day 20 续（需 `release.yml` workflow） |
| 安装脚本（`curl ... | sh`） | ⏳ Day 22 文档站时一并提供 |

---

## 一键本地打包

```bash
make release-local
```

产物：
```
dist/
├── arkui-rag-v0.0.1-<TARGET_TRIPLE>.tar.gz   # 6-8 MB
└── SHA256SUMS                                # 校验和
```

每个 tarball 含：
- `arkui-rag`：可执行 binary（静态链接 + strip · macOS 仅依赖 libSystem / libiconv）
- `INSTALL.txt`：用户向快速上手 6 步指南
- `LICENSE`：MIT
- `README.md`：项目说明

## 打包 + 自验证

```bash
make release-local-verify
```

会顺手解压到 `/tmp` 并跑 `--version`，确保 tarball 真能用。

## Features 组合策略（Day 20）

默认 features：**`http,mcp,lsp,tantivy`**（约 6.7 MB）

| feature | 是否默认 | 原因 |
|---|---|---|
| `http` | ✅ | HTTP/REST 协议（Day 14） |
| `mcp` | ✅ | MCP stdio 协议（Day 15） |
| `lsp` | ✅ | LSP stdio 协议（Day 16） |
| `tantivy` | ✅ | 真 BM25 倒排索引（Day 4） |
| `typescript` | ❌ | pre-existing：tree-sitter-typescript 0.21 API 漂移阻塞编译 |
| `lancedb` | ❌ | pre-existing：arrow-arith / chrono trait method 歧义阻塞编译 |
| `onnx` | ❌ | 体积 +~300MB + 需运行时 ONNX Runtime 原生库 · 单独 release 分发 |

未来 Day 20 续：
- 修复 `typescript` / `lancedb` 后加入 default features
- 单独发布 `arkui-rag-full-v<VERSION>-<TARGET>.tar.gz` 含 ONNX（需用户额外装 ONNX Runtime）

## 自定义 features 打包

```bash
# 仅 HTTP + MCP（无 LSP，无 BM25 真后端）
bash scripts/release-local.sh --features http,mcp

# 全 feature（前提：先修 typescript / lancedb pre-existing 阻塞）
bash scripts/release-local.sh --features full
```

## 端到端验证已通过的路径

`make release-local-verify` 在本地实测通过：

| 步骤 | 验证 |
|---|---|
| `cargo build --release --features http,mcp,lsp,tantivy` | ✅ 37s · 6.7 MB arm64 |
| `tar -xzf ... && ./arkui-rag --version` | ✅ `arkui-rag 0.0.1` |
| `arkui-rag index --bm25 tantivy` | ✅ 3 md → 22 chunks · 132ms |
| `arkui-rag query --text "..."` | ✅ Top-K 命中 + 引用溯源 |
| `serve --http` + `curl /health /search` | ✅ JSON 响应正确 |
| `serve --mcp` + `initialize`/`tools/list` | ✅ 4 tools 暴露 |
| `serve --lsp` + Content-Length `initialize` | ✅ capabilities 返回 |

详见 `feedback/features/rag4arkui-core/20-2026-05-28-day20-release-local.md`。

## 用户拿到 tarball 后的快速流程

抄自 INSTALL.txt（每个 tarball 都含）：

```bash
# 1. 下载并解压
tar -xzf arkui-rag-v0.0.1-aarch64-apple-darwin.tar.gz
cd arkui-rag-v0.0.1-aarch64-apple-darwin

# 2. 验证
./arkui-rag --version

# 3. 准备 corpus（任意含 .md 的目录都行）
mkdir -p ~/my-corpus && echo "# Hello" > ~/my-corpus/test.md

# 4. 建索引
./arkui-rag index \
    --source ~/my-corpus \
    --index-path ~/my-corpus/index.json \
    --bm25 tantivy

# 5. 检索
./arkui-rag query \
    --text "hello" \
    --index-path ~/my-corpus/index.json \
    --bm25 tantivy -k 3

# 6. 启 MCP 接 Claude Code（参考 docs/MCP-INTEGRATION-CLAUDE-CODE.md）
./arkui-rag serve --mcp \
    --index-path ~/my-corpus/index.json \
    --bm25 tantivy
```

## CI matrix 自动 release（Day 20 续）

计划 workflow：

```yaml
# .github/workflows/release.yml （未来）
on:
  push:
    tags: ['v*']
jobs:
  build:
    strategy:
      matrix:
        target:
          - aarch64-apple-darwin
          - x86_64-apple-darwin
          - x86_64-unknown-linux-gnu
          - x86_64-pc-windows-msvc
    runs-on: ${{ matrix.os }}
    steps:
      - run: cargo build --release --features http,mcp,lsp,tantivy --target ${{ matrix.target }}
      - run: bash scripts/release-local.sh --skip-build
      - uses: softprops/action-gh-release@v2
        with:
          files: dist/*
```

阻塞 / 待办：
- 本地 `make release-local-verify` 持续通过（已就绪）
- gitcode vs github 决策（meta-4 残留项）
- ONNX Runtime cross-compile 链接路径（onnx feature 进 release matrix 时需要）
