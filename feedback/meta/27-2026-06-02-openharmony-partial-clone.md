# 27 — openharmony-partial-clone

> 日期：2026-06-02
> 触发：Round 49.7 OpenHarmony 收集 · 51 的 shallow clone 5 分钟超时 + HEAD 损坏（673MB · 全量 clone 太大）
> 类型：工具脚本 + 工程结构调整（.gitignore）

---

## 用户提出的要求

> 「CI已经ok，继续round 49.7」（接 Round 51 残留 · 当时 OpenHarmony 600MB+ 仓库 shallow clone 5 分钟超时 · HEAD 损坏 · 留 partial clone + sparse checkout 待 Round 49.7 解）

潜在需求：
1. 让 `collect-corpus.sh` 能可靠拉 OpenHarmony · 不再因为仓库大就 fail
2. 不污染 ArkUI-X（小仓库）的现有 shallow clone 路径
3. CI workflow 也能跑（partial clone 在 ubuntu runner 上 OK）

## Agent 给出的修改建议

**A. `scripts/collect-corpus.sh` 拆分 clone 策略**

| 仓库 | 策略 | 命令 |
|---|---|---|
| ArkUI-X（380MB）| shallow（既有）| `git clone --depth=1` |
| **OpenHarmony（600MB+，本轮新）** | **partial + sparse** | `git clone --filter=blob:none --no-checkout --depth=1` + `git sparse-checkout set ...` |

原理：
- `--filter=blob:none`：跳 blob（文件内容）下载 · 只拉 meta（~30MB）
- `--no-checkout`：不展开工作树
- `sparse-checkout set zh-cn/application-dev en/application-dev ...`：限定子目录
- `git checkout master`：仅 fetch 这些子目录的 blob

**B. 默认 sparse paths**

```
zh-cn/application-dev zh-cn/device-dev en/application-dev en/device-dev
```

覆盖最常用的开发者文档 · 排除 release-notes / glossary 等冷门内容。用户可 `--oh-paths "..."` 自定义。

**C. 不入 git**

OpenHarmony 109MB / 7343 .md · 超 git 健康 repo size。决策：
- `.gitignore` 加 `/corpus/official/openharmony/`
- maintainer 跑 `bash scripts/collect-corpus.sh --src openharmony` 本地获取
- CI corpus-build.yml 已经在 collect-corpus 步骤自动拉 · 不需要改 workflow

ArkUI-X 19MB 仍入 git（小 · 已 commit · 用户克隆即可看 demo）· 不动。

### 替代方案

- A · sparse 但 full clone：cooperate 但浪费（仍下 600MB blob）
- B · 仍 shallow 全量：本轮 Round 51 已实证超时
- **C · partial + sparse（本次选）**：仅拉指定子目录 blob · 7 分钟拉完 109MB
- D · 入 git LFS：依赖 LFS 配额 · 用户克隆要 LFS pull · 太重

### 关键决策

| 决策 | 选择 | 理由 |
|---|---|---|
| ArkUI-X 策略 | shallow（不动）| 380MB · 实证 OK |
| OpenHarmony 策略 | partial + sparse | 600MB+ · 实证 7 分钟 109MB |
| sparse 默认 | application-dev + device-dev | 开发文档核心 · 排除 release-notes |
| 自定义 sparse | `--oh-paths "..."` | maintainer 想跑特定子集时灵活 |
| 入 git？| ❌ OpenHarmony / ✅ ArkUI-X | 109MB vs 19MB 差 5.7× · git 健康 |
| CI workflow 改不改 | 不改 | 既有 collect-corpus.sh 调用即自动走新分支 |

## 多轮互动

无 —— 用户「继续 Round 49.7」后 agent 自主设计 partial clone 策略 + .gitignore + 测试。

## 实际改动

- 接口变化：`scripts/collect-corpus.sh` 加 `--oh-paths "..."` 自定义 OpenHarmony sparse paths
- 规则变化：无（不动 pre-commit / hook / classify）
- 文件变化：
  - 修改 `scripts/collect-corpus.sh`（拆 `clone_shallow` / `clone_partial_sparse` · `collect_one` 按 name 路由）
  - 修改 `.gitignore` 加 `/corpus/official/openharmony/`
- 配置变化：
  - 脚本内常量 `OPENHARMONY_SPARSE_PATHS` 默认值

## 执行生效后总结

### 实际产出

| 项 | 内容 |
|---|---|
| 脚本升级 | `clone_partial_sparse` 函数 · `--filter=blob:none + sparse-checkout` |
| 仓库实测 | OpenHarmony zh-cn application-dev + device-dev：**7m23s · 7343 .md · 109MB** |
| 不入 git | `.gitignore` 加 `/corpus/official/openharmony/` |
| CI 兼容 | corpus-build.yml 既有 collect-corpus 步骤直接走新分支 · 无 yaml 改动 |

### 前后对比

| 操作 | Round 51（shallow 全量）| Round 49.7（partial + sparse）|
|---|---|---|
| OpenHarmony 拉法 | `git clone --depth=1`（600MB+ · 5 分钟超时 · HEAD 损坏）| `git clone --filter=blob:none --no-checkout --depth=1 + sparse-checkout` |
| 实际下行 | 5 分钟超时 + 失败 | 7m23s · 109MB · 7343 .md |
| 用户控制 | 无 | `--oh-paths "..."` 自定义子集 |
| 入 git | 不能（损坏）| 不（.gitignore）|
| ArkUI-X 路径 | 380MB shallow OK | 不动（继续 shallow）|

### 实测验证

```bash
TMPDIR=/Users/leo/tmp-corpus-collect \
bash scripts/collect-corpus.sh --src openharmony --lang zh-cn

# ═══ Collect Corpus (zh-cn) ═══
# 🌐 partial clone --filter=blob:none --no-checkout（meta only · ~30MB）
# 🎯 sparse-checkout 限定子目录：zh-cn/application-dev zh-cn/device-dev en/application-dev en/device-dev
#   📦 checkout master（拉指定子目录 blob）
#   📄 openharmony/zh-cn
#   ✅ openharmony: 7343 files · 109M
# ═══ 总览 ═══
#   总 .md 文件数: 8417
#   corpus/official/ 总大小: 127M

# 7:23.38 total（含 git fetch + sparse checkout + rsync）
```

### 残留 / 下一轮处理

- [x] partial + sparse 实装 · OpenHarmony 真活
- [x] `.gitignore` 屏蔽 109MB OpenHarmony 文件
- [x] CI workflow 无改动（既有 collect 步骤自动走新分支）
- [ ] **CI 验证**：用户触发 Corpus Build workflow 跑 OpenHarmony · 看 ubuntu runner 上 partial+sparse 真活
- [ ] **优化 1**：sparse paths 当前包含 en/* · 即使 `--lang zh-cn` 也 fetch en blob（浪费带宽）· 应根据 LANGS 动态算 sparse paths
- [ ] **优化 2**：collect-corpus.sh 加 `--clean-tmp` 清 $TMPDIR/corpus-collect · 不靠手工删
- [ ] **OpenHarmony v1.1.0**：合并 ArkUI-X + OpenHarmony 推 corpus-v1.1.0 release（Round 49.6.3）
- [ ] **长期**：mirror 切换支持（gitee.com/openharmony/docs 等）
