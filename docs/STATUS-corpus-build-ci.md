# STATUS — corpus-build-ci

> 配套 meta：`feedback/meta/26-2026-06-02-corpus-build-ci.md`
> 日期：2026-06-02
> Round 49.6 — maintainer CI 全量 corpus + index build · 推 GitHub Release

---

## 当前状态

Round 49.8 加了 cli `corpus index-pull` · 默认 URL 指向 `releases/download/corpus-v1.0.0/arkui-rag-index-bge-m3-v1.0.0.tar.gz` · 但还没真的有这个 release。本轮加 CI workflow + 打包脚本 · 闭环 maintainer "一键推 release · 用户 一行拉" 的整条链路。

不实际跑 CI build（3.2h · 等用户决定何时 trigger）· 但 workflow 文件就位 · scripts/release-corpus.sh 本地已通过测试。

## 输入契约

### 命令（maintainer）

**本地**（备份 / 应急 / 测试 release 包）：
```bash
# 假设本地已 build 完毕（index.json + bm25/ 在 INDEX_DIR）
bash scripts/release-corpus.sh \
    --version v1.0.0 \
    --embedder bge-m3 \
    --index-dir /path/to/index-dir \
    --corpus-dir corpus/official \
    --output-dir /tmp/dist
# → /tmp/dist/{arkui-rag-corpus-v1.0.0.tar.gz, arkui-rag-index-bge-m3-v1.0.0.tar.gz, SHA256SUMS}
```

**GitHub Actions**（推荐路径）：
```bash
gh workflow run "Corpus Build" \
    -f version=v1.0.0 \
    -f embedder=bge-m3 \
    -f langs="zh-cn" \
    -f dry_run=false
# 或 GitHub UI · Actions → Corpus Build → Run workflow
```

`dry_run=true` 时只 upload-artifact 不推 release · 适合首跑验证 workflow。

### 输入参数

| input | 类型 | 默认 | 用途 |
|---|---|---|---|
| version | string | v1.0.0 | 决定 tag name `corpus-{version}` + tarball 文件名 |
| embedder | string | bge-m3 | 决定 index tarball 文件名 |
| langs | string | zh-cn | 空格分隔语言 · 影响 collect-corpus.sh + build 时间 |
| dry_run | boolean | false | true=只 artifact 不推 release |

## 输出契约

### CI 产物（推到 release）

| 文件 | 内容 |
|---|---|
| `arkui-rag-corpus-{version}.tar.gz` | corpus/official/（ArkUI-X 全文档 · 含 LICENSE）|
| `arkui-rag-index-{embedder}-{version}.tar.gz` | index.json + bm25/（Tantivy 索引）|
| `SHA256SUMS` | 两个 tarball 的 sha256 hash |

### Release 元数据（自动生成）

- tag: `corpus-{version}`
- title: `Corpus + Index {version}`
- body: 用户拉法说明 + 校验命令 + License 声明
- draft: false / prerelease: false

### 用户消费（Round 49.8 cli）

```bash
arkui-rag corpus model-pull --name bge-m3
arkui-rag corpus index-pull --version v1.0.0
arkui-rag query --text "..." --embedder onnx \
    --model-path ~/.arkui-rag/models/bge-m3 \
    --index-path ~/.arkui-rag/index/index.json --bm25 tantivy
```

破坏性变更：无（既有 release.yml / book.yml / ci.yml 不动 · 仅新增 corpus-build.yml）。

## 验证手段

### Agent 本轮已做

```bash
# 1. release-corpus.sh 本地真跑
bash scripts/release-corpus.sh \
    --version v1.0.0 --embedder bge-m3 \
    --index-dir /Users/leo/tmp-index-pull2 \
    --corpus-dir corpus/official \
    --output-dir /tmp/release-test
# ✓ corpus 2.8M · index 1.2M · SHA256SUMS 两条

# 2. YAML 语法
ruby -ryaml -e "YAML.load_file('.github/workflows/corpus-build.yml')"   # ✓
```

### 用户验证（推 CI 真活）

```bash
# Step 1 · 测 workflow 真活（dry_run 不推 release）
gh workflow run "Corpus Build" -f version=v1.0.0-rc.1 -f dry_run=true
gh run watch                                  # 观察 ~1.5h（zh-cn 子集）
# 看 GitHub UI artifact corpus-v1.0.0-rc.1 含三个文件 ✓

# Step 2 · 真推 v1.0.0
gh workflow run "Corpus Build" -f version=v1.0.0 -f dry_run=false

# Step 3 · 用户视角拉
arkui-rag corpus index-pull --version v1.0.0
arkui-rag query --text "ArkUI-X 怎么创建第一个应用" ...
# 应命中真 ArkUI-X 内容
```

## 与上一阶段的关联性

| Round | 主题 | 关系 |
|---|---|---|
| 20b | release.yml binary 4 平台 matrix | 本轮镜像同款风格 · 仅 1 个 Linux job（不需 matrix）|
| 21b | corpus model-pull 真活 | 本轮 CI 调 model-pull 拉 BGE-M3（cache miss 时）|
| 49 PoC | corpus 分发 PoC | 本轮把 PoC 自动化 · `gh workflow run` 替代手动 30 行命令 |
| 51 | 真 ArkUI-X 1066 .md + Round 49.8 index-pull 命令 | 本轮 closes 整条 maintainer→user 链路 |
| **52（本轮）** | maintainer CI + release-corpus.sh | 解锁用户 `corpus index-pull` 默认 URL 真活 |

## 完成度 / 下一阶段

### 达成度

| 项 | 状态 |
|---|---|
| scripts/release-corpus.sh（本地 + CI 共用）| ✅ |
| .github/workflows/corpus-build.yml workflow_dispatch | ✅ |
| Cache cargo registry + target | ✅ |
| Cache BGE-M3 模型（避免重 pull 2.3GB）| ✅ |
| GNU tar reproducible options | ✅ |
| dry_run flag · artifact-only 测试模式 | ✅ |
| Release auto-create + 文档生成 | ✅ |
| YAML 语法校验 | ✅ |
| 本地脚本端到端测试 | ✅ |
| CI 真跑（3.2h）| ⏭ 用户触发 |
| 推 corpus-v1.0.0 真 release | ⏭ 用户验证后 dry_run=false |

### 下一阶段建议

**立即（用户操作）**：

1. **首跑测 workflow 真活（zh-cn 子集约 1.5h）**：
   ```bash
   gh workflow run "Corpus Build" -f version=v1.0.0-rc.1 -f dry_run=true
   ```
   验 artifact 含三个文件、cache 真存到。

2. **推真 release**：
   ```bash
   gh workflow run "Corpus Build" -f version=v1.0.0 -f dry_run=false
   ```
   完了后 `arkui-rag corpus index-pull --version v1.0.0` 测默认 URL。

**之后**：

- **Round 49.7（OpenHarmony 收集）**：扩 `corpus/official/openharmony/` · CI build v1.1.0
- **Round 49.6.1（CI 真跑暴露问题）**：可能命中 ubuntu runner 7GB 内存 / 14GB 磁盘限制 · BGE-M3 2.3GB + corpus + cargo target ≈ 11GB · 紧 · 若 OOM 用 `runs-on: ubuntu-22.04-large` (8core/32GB · 收费但秒级)
- **Round 52（init wizard）**：CI release 真活后 · `arkui-rag init` 串 model-pull + index-pull + 写 MCP 配置 · 新用户 zero-config
- **长期**：mirror 切换（gitee / 阿里云 OSS / Cloudflare R2）· 国内访问优化
