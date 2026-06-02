# 53 — corpus-build-pipeline（Round 49.6）

> 日期：2026-06-02
> 涉及文件：`.github/workflows/corpus-build.yml`（新）· `scripts/release-corpus.sh`（新）
> 类型：业务 pipeline（maintainer 自动化 + 用户消费链路闭环）

## 本轮目标

Round 49.5 第 1/2 半（50/51）+ Round 49.8（52）三轮把 corpus + index 的"agent 本地造"和"用户 cli 拉"两侧分别打通 · 但中间一段"maintainer 怎么把 1.2MB index 持续推到 GitHub Release"仍然是手敲 30 行 + gh release create。

本轮加 GitHub Actions workflow_dispatch 触发的 CI · 一行 `gh workflow run "Corpus Build"` 自动跑全量 build + 打包 + 推 release · 用户侧 `arkui-rag corpus index-pull` 默认 URL 真活。

## Plan

### 决策 A · 触发模式只 workflow_dispatch

不绑定 push tag · 不绑定 schedule cron：

| 触发 | 选 | 理由 |
|---|---|---|
| push tag corpus-v* | ❌ | 误推风险 · 一次 3.2h CI 浪费 |
| schedule（每周）| ❌ | ArkUI-X 文档更新频率不固定 · 跑空 |
| **workflow_dispatch** | ✅ | maintainer 显式 trigger · 看清楚 input 再跑 |

### 决策 B · 单 job · 不 matrix

跟 release.yml 4 平台 matrix 不同：corpus + index 是**数据制品** · 不是 binary · 单 Linux 跑出来用户在任何平台都能用（解压 .tar.gz）。

### 决策 C · CPU only 不冲突

Round 51 已实证 BGE-M3 + CoreML 不兼容 · macOS 本地也 CPU only。Linux runner 同理 · 速度 ~0.73 chunks/sec · zh-cn 子集（~5180 chunks）约 1.5-2h · zh-cn+en 双语约 3.2h。

### 决策 D · Cache 两条

1. **cargo registry + target**：加速重跑（首跑 cache miss 不亏 · 之后秒入 build）
2. **~/.arkui-rag/models/bge-m3 (2.3GB)**：key 固定 `bge-m3-onnx-v1` · 一次拉永久复用

### 决策 E · scripts/release-corpus.sh 抽离

不写 inline yaml step · 把 tar + sha256 提到独立脚本：
- 本地 maintainer 应急/备份/调试也能跑
- CI 改 workflow 时不动打包逻辑
- 本地 macOS BSD tar + CI Linux GNU tar 同一接口 · 内部分支处理 reproducibility

### 决策 F · 命名约定与 Round 49.8 路由对齐

| 文件 | 命名 |
|---|---|
| corpus tarball | `arkui-rag-corpus-{version}.tar.gz` |
| index tarball | `arkui-rag-index-{embedder}-{version}.tar.gz` |
| 校验 | `SHA256SUMS` |
| release tag | `corpus-{version}` |

与 Round 49.8 `default_index_url(embedder, version)` 和 `DEFAULT_CORPUS_URL` 完全对齐 · cli 拉默认 URL 即命中。

### 不动

- 既有 release.yml（binary 4 平台 matrix）不动
- 既有 book.yml / ci.yml 不动
- BGE-M3 模型 build 流程不动（仍 model-pull · 仍 onnx feature）
- collect-corpus.sh（Round 51 写的）不动 · CI 直接调
- cli 不动

## 对话摘要

| 轮次 | 用户 | Agent |
|---|---|---|
| 1 | 「go」（接 Round 49.8 后 agent 推荐 #97 Round 49.6 CI）| 设计单 job · workflow_dispatch · 抽 release-corpus.sh · cache 模型 + cargo · dry_run flag |

无方向调整。

## 改动要点

### 新增
- `.github/workflows/corpus-build.yml`（~130 行 · 11 步骤）
- `scripts/release-corpus.sh`（~120 行 · 本地 + CI 共用打包器）
- `feedback/meta/26-2026-06-02-corpus-build-ci.md`（meta 归档 · 工具脚本 + CI 元变更）
- `docs/STATUS-corpus-build-ci.md`
- `feedback/features/rag4arkui-core/53-2026-06-02-corpus-build-pipeline.md`（本文件）

### 修改
无（既有 release.yml / cli / collect-corpus.sh 都不动）。

### 不入 git
- `/tmp/release-test/` 本地脚本测试产物

## 验证结果

### 本地脚本
```bash
bash scripts/release-corpus.sh \
    --version v1.0.0 --embedder bge-m3 \
    --index-dir /Users/leo/tmp-index-pull2 \
    --corpus-dir corpus/official \
    --output-dir /tmp/release-test
# 🔧 BSD tar 检测到（macOS）· reproducible options 跳过（hash 跨机器可能不同）
# ═══ corpus 2.8M
# ═══ index 1.2M
# ═══ SHA256SUMS
#   47adf4c7d94a989f43683f9b8a8030cf346bf142e0c6934800b48334cea87235  arkui-rag-corpus-v1.0.0.tar.gz
#   fa723d6de4fe0da64847990bf5825d7f7d9cd2459aee13d70210c3d56f86205e  arkui-rag-index-bge-m3-v1.0.0.tar.gz
```

### YAML
```bash
ruby -ryaml -e "YAML.load_file('.github/workflows/corpus-build.yml')"   # ✓
```

### CI 真活
等用户 `gh workflow run "Corpus Build" -f version=v1.0.0-rc.1 -f dry_run=true` 触发首测。

## 残留 / 下一轮

- [x] scripts/release-corpus.sh 抽离 · 本地 + CI 共用
- [x] .github/workflows/corpus-build.yml workflow_dispatch + 11 步骤
- [x] cache cargo + BGE-M3
- [x] dry_run flag 支持 artifact-only 测试
- [x] GNU tar reproducible options + BSD tar fallback
- [x] 命名约定与 Round 49.8 cli 默认 URL 路由对齐
- [ ] **用户首跑** dry_run=true 验 workflow（~1.5h zh-cn 子集）
- [ ] **用户推真 release** dry_run=false 推 corpus-v1.0.0
- [ ] **Round 49.6.1** CI 真跑暴露 ubuntu runner 资源（7GB 内存 / 14GB 磁盘）紧张时升 ubuntu-22.04-large（8core/32GB · 收费）
- [ ] **Round 49.6.2** CI release 真活后 · 跑 `arkui-rag corpus index-pull` 不带 --from-file 验默认 URL
- [ ] **Round 49.7** OpenHarmony 收集后 build v1.1.0
- [ ] **Round 52** init wizard 把 model-pull + index-pull + MCP 配置串起来
- [ ] **长期** mirror 切换（gitee / 阿里云 OSS / Cloudflare R2 · 国内访问）
