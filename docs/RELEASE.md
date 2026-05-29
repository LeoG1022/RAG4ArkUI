# RAG4ArkUI · Release 与分发指南

> Day 20a + 20b：本地端到端 + CI matrix 自动 release（4 平台）已落地。

---

## 当前 Release 状态（Day 20b）

| 维度 | 状态 |
|---|---|
| 本地 host 平台 release artifact | ✅ Day 20a · `make release-local` |
| **CI matrix（4 平台自动 build）** | ✅ **Day 20b · `.github/workflows/release.yml`** |
| **tag 触发自动上传 GitHub Releases** | ✅ Day 20b · push tag `v*` 即触发 |
| Apple Silicon arm64（aarch64-apple-darwin） | ✅ 本地实测通过 + CI matrix |
| Apple Intel x86_64（x86_64-apple-darwin） | ⏳ CI matrix 待第一次 release 验证 |
| Linux x86_64 GNU（x86_64-unknown-linux-gnu） | ⏳ CI matrix 待第一次 release 验证 |
| Windows x86_64 MSVC（x86_64-pc-windows-msvc） | ⏳ CI matrix 待第一次 release 验证 |
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

默认 features：**`http,mcp,lsp,tantivy,typescript,corpus-pull`**（约 8 MB）

| feature | 是否默认 | 原因 |
|---|---|---|
| `http` | ✅ | HTTP/REST 协议（Day 14） |
| `mcp` | ✅ | MCP stdio 协议（Day 15） |
| `lsp` | ✅ | LSP stdio 协议（Day 16） |
| `tantivy` | ✅ | 真 BM25 倒排索引（Day 4） |
| `typescript` | ✅ 已默认（Day 20c Phase 1 修了 0.21 API 漂移 · ArkTS struct 方法提取需 custom grammar 留 follow-up） |
| `corpus-pull` | ✅ 已默认（Day 21 新增 · `arkui-rag corpus pull --url|--from-file` HTTP 下载 + tar.gz 解压）|
| `lancedb` | ✅ | task #81 完整解锁（lancedb 0.10 → 0.30 + arrow 52 → 58 + chrono pin 移除 + LanceVectorStore dim auto-detect from schema）· 用户需预装 `brew install protobuf`（build-time） |
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

## CI matrix 自动 release（Day 20b · 已实装）

workflow：[`.github/workflows/release.yml`](../.github/workflows/release.yml)

```text
触发：git push tag v*  →  GitHub Actions 4 平台 matrix
       ├─ aarch64-apple-darwin       (macos-14)
       ├─ x86_64-apple-darwin        (macos-13)
       ├─ x86_64-unknown-linux-gnu   (ubuntu-latest)
       └─ x86_64-pc-windows-msvc     (windows-latest · git-bash)
       ↓
每个 matrix job 跑 scripts/release-local.sh （host triple 原生 build · 不跨编）
       ↓ artifacts (tar.gz)
release job 合并 SHA256SUMS + softprops/action-gh-release 上传到 Release page
```

### 用户操作（推 1.0 release 时）

```bash
# 1. 确保本地 master 与 GitHub 同步（gitcode 仅 mirror · 不跑 release）
git push github master

# 2. 打 tag
git tag v0.0.2
git push github v0.0.2

# 3. （可选）同步 tag 到 gitcode mirror
git push gitcode master --tags

# 4. 等 5-15 分钟看 GitHub Actions Release workflow 完成
#    → Releases 页面会自动出现 v0.0.2 + 4 个 tarball + SHA256SUMS
```

### 双 remote 配置（GitHub 主 · gitcode mirror）

```bash
# 方案 A：保留两个独立 remote 名
git remote add gitcode git@gitcode.com:keerecles/RAG4ArkUI.git
# 现在 push 需要分别：
git push github master
git push gitcode master

# 方案 B：origin 单 push 到两个 URL（推荐 · 一次 push 同步两端）
git remote set-url --add --push origin git@gitcode.com:keerecles/RAG4ArkUI.git
# 现在 git push origin 同步推到 github + gitcode
```

注意：release.yml **只在 GitHub 跑**（gitcode 不跑 GitHub Actions），即使代码 mirror 到 gitcode，Release artifact 也仅出现在 GitHub Releases 页面。

### 第一次 release 已知 risk

- ⚠️ Windows runner 上 tantivy build 路径未实测（tantivy 0.22+ 应跨平台 · 第一次跑可能浮出小问题）
- ⚠️ macOS x86_64 runner（macos-13）已被 GitHub 标 deprecation · 长期需切到 macos-14 + `--target=x86_64-apple-darwin` 跨编
- ⚠️ `dtolnay/rust-toolchain@stable` 第一次跑要装 toolchain · 比 cache hit 慢 ~2 分钟
- ⚠️ tag 写错（如 `0.0.2` 不带 `v`）→ workflow 不触发

### 失败重试

```bash
# 删本地 tag + 远端 tag
git tag -d v0.0.2
git push --delete github v0.0.2

# 修复问题 + 重打 tag + 重新 push
git tag v0.0.2
git push github v0.0.2
```

或者用 workflow_dispatch 手动触发（在 Actions 页面填 tag 名）。
