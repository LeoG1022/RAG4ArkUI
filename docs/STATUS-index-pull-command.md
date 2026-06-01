# STATUS — index-pull-command

> 配套 feature log：`feedback/features/rag4arkui-core/52-2026-06-01-index-pull-command.md`
> 日期：2026-06-01
> Round 49.8 — `arkui-rag corpus index-pull` 子命令

---

## 当前状态

Round 49.5 第 2 半（Round 51）让真 ArkUI-X corpus + 预 build index tarball 已经放在 `/tmp/dist-corpus-v1.0.0/`（2.8MB + 1.2MB）· 但用户拿不到。本轮加 cli 子命令 `corpus index-pull` 把"用户拉"路径打通：

- `corpus index-pull` 新子命令 · 镜像 model-pull / corpus pull 既有架构
- URL 路由：`corpus-{version}/arkui-rag-index-{embedder}-{version}.tar.gz`
- 默认目标：`~/.arkui-rag/index/`
- 复用 `download_and_extract`（HTTP / 本地文件 + tar.gz 解压 + path traversal 防御）
- Hotfix `download_and_extract`：`.` / `./` 顶层 entry skip（修 macOS `/tmp` symlink + canonicalize 误报）

`DEFAULT_CORPUS_URL` 同步升 v0.0.1 → v1.0.0（与 maintainer CI 待推的 release 对齐）。

## 输入契约

新 CLI 命令：

```bash
arkui-rag corpus index-pull [OPTIONS]

OPTIONS:
  --embedder <EMBEDDER>          [default: bge-m3]
  --version <VERSION>            [default: v1.0.0]
  --url <URL>                    自定义 URL（覆盖 embedder/version 路由）
  --target <TARGET>              默认 ~/.arkui-rag/index/
  --force                        覆盖已存在
  --from-file <PATH>             离线 · 跳 HTTP · 直接解压本地 tarball
  --strip-components <N>         [default: 0]
```

无破坏性接口变更 · 既有 `corpus list` / `corpus pull` / `corpus model-pull` 完全不动。

## 输出契约

| 路径 | 内容 |
|---|---|
| `~/.arkui-rag/index/index.json` | 向量索引（JSON · InMemoryVectorStore 序列化）|
| `~/.arkui-rag/index/bm25/` | Tantivy BM25 索引目录 |
| `~/.arkui-rag/index/._*` | macOS tar 附带的 AppleDouble 元数据（无害 · CI Linux 自动无）|

用户后续：
```bash
arkui-rag query --text '...' --embedder onnx \
    --model-path ~/.arkui-rag/models/bge-m3 \
    --index-path ~/.arkui-rag/index/index.json --bm25 tantivy
```

## 验证手段

### Agent 本轮已做

```bash
# Build
cargo build --release -p arkui-rag-cli \
    --features tantivy,http,mcp,lsp,onnx,corpus-pull,lancedb     # 3m 42s ✓

# CLI 接口
arkui-rag corpus --help                                          # 列出 index-pull ✓
arkui-rag corpus index-pull --help                               # 参数齐 ✓

# 端到端（--from-file 等效 release URL）
arkui-rag corpus index-pull \
    --from-file /tmp/dist-corpus-v1.0.0/arkui-rag-index-bge-m3-v1.0.0.tar.gz \
    --target /Users/leo/tmp-index-pull2 --force
# ✅ 207 files / 1.11 MB / strip 0

arkui-rag query --text "ArkUI-X 怎么创建第一个应用" \
    --embedder onnx --model-path ~/.arkui-rag/models/bge-m3 \
    --index-path /Users/leo/tmp-index-pull2/index.json --bm25 tantivy -k 2
# ✅ Top-2: README.md "快速入门" + start-overview.md "开发准备"
```

### 用户验证（HTTP 路径 · 待 Round 49.6 CI 推 release 后）

```bash
# 假设 v1.0.0 release 已发
arkui-rag corpus model-pull --name bge-m3       # 拉 BGE-M3 onnx（既有 Day 21b）
arkui-rag corpus index-pull                     # 拉预 build index（本轮新）
arkui-rag query --text "..." --embedder onnx \
    --index-path ~/.arkui-rag/index/index.json --bm25 tantivy
# 全程零本地 build
```

## 与上一阶段的关联性

| Round | 主题 | 关系 |
|---|---|---|
| Day 21b | model-pull 真活 | 本轮 index-pull 镜像同款架构 · 共用 download_and_extract |
| 49 PoC | corpus 分发流水线 PoC | 本轮把"用户拉"半边路径打通（Round 49 是 maintainer 打包 · 本轮是用户拉）|
| 51 | 真 ArkUI-X corpus + tarballs | 本轮用 51 产出的 1.2MB index tarball 做端到端 |
| **52（本轮）** | `corpus index-pull` 子命令 | 解锁 Round 49.6 maintainer CI release 真正可被用户消费 |

## 完成度 / 下一阶段

### 达成度

| 项 | 状态 |
|---|---|
| CorpusOp::IndexPull enum + 字段 | ✅ |
| cmd_corpus_index_pull 实装 | ✅ |
| default_index_url / default_index_target helper | ✅ |
| DEFAULT_CORPUS_URL v1.0.0 升级 | ✅ |
| download_and_extract `.` skip hotfix | ✅ |
| 端到端 --from-file + query 命中真内容 | ✅ |
| HTTP 路径真活验证 | ⏭ 等 Round 49.6 CI release |

### 下一阶段建议

**Round 49.6（maintainer CI）**：
- `.github/workflows/corpus-build.yml` 跑全量 ArkUI-X build
- 产物：`arkui-rag-corpus-v1.0.0.tar.gz` / `arkui-rag-index-bge-m3-v1.0.0.tar.gz` / `SHA256SUMS`
- 自动推 `corpus-v1.0.0` Release · 之后 `corpus index-pull` 默认 URL 真活
- 用 GNU tar（避免 macOS AppleDouble `._*` 文件污染）

**Round 49.7（OpenHarmony 收集）**：partial clone 后扩 `corpus/official/openharmony/` · build 第二份 index · v1.1.0 同款分发

**Round 52（init wizard）**：
```bash
arkui-rag init    # → model-pull bge-m3 + index-pull + 写 ~/Library/Application Support/Claude/claude_desktop_config.json
```

**安全增强**（可在 Round 49.6 一同做）：
- 加 `--verify-sha256 <hash>` 或自动从同 release 拉 SHA256SUMS 校验
- 当前 path traversal 防御已存在 · `.` skip 不弱化（仍守 `../etc/passwd` 等）

**长期**：mirror 切换 · `--mirror gitee` 或读 `~/.arkui-rag/config.toml` · 国内访问优化
