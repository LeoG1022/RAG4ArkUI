# 52 — index-pull-command（Round 49.8）

> 日期：2026-06-01
> 涉及代码：`crates/arkui-rag-cli/src/main.rs`
> 类型：新建（cli 子命令 + 路径越界 hotfix）

## 本轮目标

端用户拿到 binary 后 · 自己跑 `arkui-rag index` build 索引要 3 分钟（quick-start 子集）到 3+ 小时（全量 zh-cn+en）· 而且现在 BGE-M3 + CoreML 不兼容只能 CPU 慢跑。Maintainer CI（Round 49.6）会预 build 推 release · 用户应该一行命令拉到本地立刻用。

本轮新增 `arkui-rag corpus index-pull` 子命令 · 镜像 `corpus pull` / `corpus model-pull` 既有架构 · 复用 `download_and_extract` 基础设施。

副作用 hotfix：`download_and_extract` 撞 tarball 顶层 `./` entry + macOS `/tmp` symlink → canonicalize path traversal 检查误报 · 一并修。

## Plan

### 决策 A · 命令位置 → `arkui-rag corpus index-pull`

继承既有命名约定：
- `corpus list` · 列出本地 corpus
- `corpus pull` · 拉 corpus tarball
- `corpus model-pull` · 拉模型 tarball
- **`corpus index-pull`（新）** · 拉预 build index tarball

为什么不是顶层 `arkui-rag index-pull`：保持子命令树扁平 · `corpus *` 组下三个 pull 类对称 · 用户记一个 `corpus` 前缀即可（Round 52 init wizard 会一行调三个）。

### 决策 B · URL 路由：embedder + version 两段

```
https://github.com/LeoG1022/RAG4ArkUI/releases/download/corpus-{version}/arkui-rag-index-{embedder}-{version}.tar.gz
```

CLI 默认 `--embedder bge-m3 --version v1.0.0` · 用户 `--url` 可整体覆盖。tag = `corpus-vX.Y.Z`（与 maintainer CI 推的 release 对齐）· file = `arkui-rag-index-{embedder}-{version}.tar.gz`（embedder 留扩展位 · 未来支持 qwen3-embedding-0.6b 等不冲突）。

同时把 `DEFAULT_CORPUS_URL` 也升到 v1.0.0 范式（PoC v0.0.1 占位换掉）。

### 决策 C · 默认目标 `~/.arkui-rag/index/`

跟 `~/.arkui-rag/models/<name>/`（Day 21b · 模型）+ `~/.arkui-rag/corpus/`（约定 · 未来 init wizard）对称：

| 内容 | 路径 |
|---|---|
| BGE-M3 模型 | `~/.arkui-rag/models/bge-m3/` |
| 预 build 索引 | `~/.arkui-rag/index/`（含 `index.json` + `bm25/`）|
| 原始 corpus | `~/.arkui-rag/corpus/`（未来 init wizard 写入）|

`query` 后 `--index-path ~/.arkui-rag/index/index.json` 即可。

### 决策 D · strip_components 默认 0

Round 51 重打的 tarball 顶层就是 `index.json + bm25/`（没有外层 wrap 目录）· strip 0 段直接落地。

对比 corpus tarball：`tar -czf ... -C $REPO corpus/official/` · 顶层是 `corpus/official/` · strip 1 剥外层。两者默认值不同合理。

### 决策 E · path traversal hotfix

撞 tarball 中 `./` 顶层 entry（GNU tar `tar -czf x.tgz -C dir .` 经典产物）：

```rust
// 修复前
if stripped.as_os_str().is_empty() { continue; }

// 修复后
if stripped.as_os_str().is_empty()
    || stripped == Path::new(".")
    || stripped == Path::new("./")
{ continue; }
```

为什么 macOS `/tmp` symlink 触发：
- `out_path = target.join(".")` · normalize 为 `target/.`
- `out_path.parent()` = `target`
- `parent.canonicalize()` = 物理路径（`/private/tmp/...`）
- `target.canonicalize()` 在 join `.` 前 normalize · 也是物理路径
- 但 `target` 自身（非 canonicalize）是 `/tmp/...` · starts_with 检查时一边 logical 一边 physical · false
- 结果：把无害的 `.` entry 当成恶意 path traversal 拒掉

干净修法是跳 `.` entry · path traversal 防御不动（仍守住真恶意 `../etc/passwd` 之类）。

### 不动

- `download_and_extract` 主体逻辑不动（只加 `.` skip）
- `cmd_corpus_pull` / `cmd_corpus_model_pull` 不动
- CLI 顶层结构不动（仍 `corpus <subcmd>`）
- `corpus-pull` feature gate 不动（同 model-pull 共用）

## 对话摘要

| 轮次 | 用户 | Agent |
|---|---|---|
| 1 | 「按推荐做」（接 Round 51 收尾后的推荐 Round 49.8 index-pull）| 加 IndexPull · 复用 download_and_extract · 撞 `./` entry path traversal 误报 · 加 `.` skip · 端到端 query 命中真 ArkUI-X 内容 |

无方向调整 · agent 自主设计 CLI 接口 + URL 路由 + 默认目标 + bug hotfix。

## 改动要点

### 新增
- `CorpusOp::IndexPull` 枚举变体（7 字段：embedder / version / url / target / force / from_file / strip_components）
- `cmd_corpus_index_pull` async fn（~75 行 · URL 路由 + target 解析 + 复用 download_and_extract + 输出格式 + 下一步提示）
- `default_index_url(embedder, version) -> String`
- `default_index_target() -> Result<PathBuf>` → `~/.arkui-rag/index/`

### 修改
- `DEFAULT_CORPUS_URL`：v0.0.1 → v1.0.0 范式（PoC 占位 → 与 maintainer CI release 对齐）
- `download_and_extract`：路径越界检查加 `.` / `./` skip（path traversal hotfix）
- `run()` 在 CorpusOp match 加 IndexPull 分支

### 不入 git
- 测试解压目录 `/Users/leo/tmp-index-pull2/`

## 验证结果

### 编译
```bash
cargo build --release -p arkui-rag-cli \
    --features tantivy,http,mcp,lsp,onnx,corpus-pull,lancedb
# Finished release profile in 3m 42s ✓
```

### CLI 接口
```bash
arkui-rag corpus --help
#   ...
#   index-pull  Round 49.8：拉取预 build 好的 index tarball 到 ~/.arkui-rag/index/

arkui-rag corpus index-pull --help
# Usage: arkui-rag corpus index-pull [OPTIONS]
# Options:
#   --embedder <EMBEDDER>  [default: bge-m3]
#   --version <VERSION>    [default: v1.0.0]
#   --url <URL>            自定义 URL · 默认按 embedder + version 路由
#   --target <TARGET>      目标目录 · 默认 ~/.arkui-rag/index/
#   --force                强制覆盖已存在文件
#   --from-file <PATH>     跳过 HTTP 下载 · 从本地 tarball 解压
#   --strip-components <N> [default: 0]
```

### 端到端（本地 tarball · 等效用户拉 release）
```bash
arkui-rag corpus index-pull \
    --from-file /tmp/dist-corpus-v1.0.0/arkui-rag-index-bge-m3-v1.0.0.tar.gz \
    --target /Users/leo/tmp-index-pull2 --force
# ✅ index 拉取完成
#    embedder : bge-m3
#    version  : v1.0.0
#    大小     : 1.11 MB
#    文件数    : 207

arkui-rag query --text "ArkUI-X 怎么创建第一个应用" \
    --embedder onnx --model-path ~/.arkui-rag/models/bge-m3 \
    --index-path /Users/leo/tmp-index-pull2/index.json --bm25 tantivy -k 2
# ✅ Top-2:
#   [1] score=0.0164 README.md "快速开始 > 快速入门"
#   [2] score=0.0161 start-overview.md "开发准备 > 开发工具"
```

无 env · 无 model-pull · 无任何手工编辑 · 一行 `corpus index-pull` + 一行 `query` = 真活。

## 残留 / 下一轮

- [x] CorpusOp::IndexPull 加 + cmd_corpus_index_pull 实装
- [x] URL 路由 `corpus-{version}/arkui-rag-index-{embedder}-{version}.tar.gz`
- [x] 默认目标 `~/.arkui-rag/index/`
- [x] DEFAULT_CORPUS_URL v0.0.1 → v1.0.0 同步升级
- [x] path traversal hotfix（`.` / `./` entry skip · 修 macOS symlink 误报）
- [x] 端到端 --from-file 解压 + query 命中真内容
- [ ] **HTTP 真活验证**：等 Round 49.6 CI 推 release 后跑 `arkui-rag corpus index-pull` 无 `--from-file` · 验默认 URL 真活
- [ ] **macOS tar `._*` AppleDouble 文件**：本地 tar 打出来含 `._index.json` / `._bm25` 元数据文件 · 无害但占空间；Round 49.6 CI Linux runner 走 GNU tar 自动无此问题
- [ ] **SHA256 校验**：当前下载后不验 hash；Round 49.6 应同时推 SHA256SUMS · index-pull 加 `--verify-sha256` 可选
- [ ] **Round 52 init wizard**：连串调 model-pull + index-pull + 写 MCP 配置 · 一行 `arkui-rag init` 完事
- [ ] **mirror 切换**：默认 GitHub Releases · 国内访问慢 · 加 `--mirror gitee` 或读 `~/.arkui-rag/config.toml` 切镜像
