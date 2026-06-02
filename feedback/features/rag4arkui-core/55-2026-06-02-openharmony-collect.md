# 55 — openharmony-collect（Round 49.7）

> 日期：2026-06-02
> 涉及文件：`scripts/collect-corpus.sh`（partial+sparse 分支）· `.gitignore`（屏蔽 OpenHarmony 文件）
> 类型：业务里程碑（B 源 OpenHarmony 收集链路打通）

## 本轮目标

Round 51 把 A 源（ArkUI-X）拉通了 · B 源（OpenHarmony）卡在 600MB+ 仓库 shallow clone 5 分钟超时 · HEAD 损坏。本轮换 partial clone + sparse checkout 策略让 B 源真活：

1. `collect-corpus.sh` 加 partial+sparse 分支（仅对 OpenHarmony 用）
2. 默认 sparse paths：`application-dev` + `device-dev` 双语
3. 测：实测 7 分钟拉完 109MB / 7343 .md
4. 109MB 太大不入 git · `.gitignore` 屏蔽 · CI 自己拉
5. corpus-v1.1.0 路径就绪（v1.0.0 只含 ArkUI-X · v1.1.0 加 OpenHarmony）

## Plan

### 决策 A · 仅 OpenHarmony 走 partial+sparse

ArkUI-X 380MB shallow clone 实证 OK（Round 51 验过）· 不动。
OpenHarmony 600MB+ 仓库才需要 partial+sparse。

按 `name` 路由：

```bash
case "$name" in
    openharmony)
        clone_partial_sparse "$repo" "$clone_dir" "$OH_SPARSE_LIST" ;;
    *)
        clone_shallow "$repo" "$clone_dir" ;;
esac
```

### 决策 B · 默认 sparse paths

```
zh-cn/application-dev zh-cn/device-dev en/application-dev en/device-dev
```

- `application-dev`：应用开发文档（开发者最常用）
- `device-dev`：设备开发文档（OS 底层 · 多平台适配场景）
- 排除：release-notes / glossary / contribute / security 等（不上 RAG）

用户可 `--oh-paths "..."` 自定义。

### 决策 C · 不入 git + .gitignore 屏蔽

109MB / 7343 .md 入 git · 后续每次 clone repo 都拉 · 不健康。

替代方案：
- A · git LFS：LFS 配额 · 用户额外 `git lfs pull` · 太重
- B · 拆 git submodule：把 corpus 单独 repo · 跟 ArkUI-X 19MB 入 git 模式不一致
- **C · .gitignore 屏蔽 + CI 拉（本次选）**：simpler · CI 已经有 collect 步骤 · 自动走

ArkUI-X 19MB 仍入 git（小 · 已 commit · 不动）。

### 决策 D · CI workflow 无改动

`corpus-build.yml` 既有第 7 步：

```yaml
- name: Collect ArkUI-X official docs
  run: bash scripts/collect-corpus.sh --src arkui-x --lang "${{ inputs.langs }}"
```

要加 OpenHarmony：

```yaml
- name: Collect OpenHarmony official docs
  run: bash scripts/collect-corpus.sh --src openharmony --lang "${{ inputs.langs }}"
```

但本轮**先不改 CI**（v1.0.0 仅 ArkUI-X · v1.1.0 才加 OpenHarmony · 留下轮统一改）。

### 不动
- `cmd_corpus_index_pull` 默认 URL 路由（仍 v1.0.0）
- ArkUI-X shallow 路径
- corpus-v1.0.0 范围（仅 ArkUI-X · 不含 OpenHarmony）

## 对话摘要

| 轮次 | 用户 | Agent |
|---|---|---|
| 1 | 「CI已经ok，继续round 49.7」| 加 clone_partial_sparse 函数 · 按 name 路由 · 默认 sparse paths application-dev+device-dev · 实测 7m23s 109MB · .gitignore 屏蔽 · CI workflow 暂不动留 v1.1.0 |

无方向调整。

## 改动要点

### 新增
- `scripts/collect-corpus.sh` 内：
  - `OPENHARMONY_SPARSE_PATHS` 常量
  - `clone_shallow()` 函数（封装既有逻辑）
  - `clone_partial_sparse()` 函数（partial + sparse）
  - `collect_one()` 按 name 路由 clone 策略
  - `--oh-paths "..."` CLI 参数
- `feedback/meta/27-openharmony-partial-clone.md`（meta 归档）
- `feedback/features/rag4arkui-core/55-openharmony-collect.md`（本文件）
- `docs/STATUS-openharmony-collect.md`

### 修改
- `.gitignore` 加 `/corpus/official/openharmony/`（109MB 屏蔽 · 与 ArkUI-X 19MB 入 git 反差对待）

### 不动
- ArkUI-X shallow 路径（既有 OK）
- corpus-build.yml workflow（v1.1.0 再统一加）
- cli `corpus index-pull` 默认 URL

## 验证结果

### 收集
```bash
TMPDIR=/Users/leo/tmp-corpus-collect \
bash scripts/collect-corpus.sh --src openharmony --lang zh-cn

# ═══ Collect Corpus (zh-cn) ═══
# 🌐 partial clone --filter=blob:none --no-checkout（meta only · ~30MB）
# 🎯 sparse-checkout 限定子目录：zh-cn/application-dev zh-cn/device-dev en/application-dev en/device-dev
#   📦 checkout master（拉指定子目录 blob）
#   📄 openharmony/zh-cn
#   ✅ openharmony: 7343 files · 109M
# 7:23.38 total
```

### corpus/official/ 现状
```
arkui-x/        19M  · 1066 .md · 入 git (commit bdc39b4)
mapping/        36K  · 8  .md · 入 git
openharmony/   109M  · 7343 .md · gitignore 屏蔽（本轮新）
example-mapping.md   入 git
arkuix-best-practices.md  入 git
─────────────────
总               127M / 8417 .md
```

### .gitignore
```bash
git status --porcelain | grep openharmony && echo "未屏蔽" || echo "✅ 已屏蔽"
# ✅ 已屏蔽
```

## 残留 / 下一轮

- [x] OpenHarmony partial+sparse 真活
- [x] 默认 sparse paths（application-dev + device-dev）
- [x] `--oh-paths` 自定义参数
- [x] `.gitignore` 屏蔽 109MB
- [x] ArkUI-X 路径不退化
- [ ] **CI workflow 加 OpenHarmony 步骤**：v1.1.0 时 corpus-build.yml 加 `collect-corpus --src openharmony` 步骤
- [ ] **CI 真活验证**：用户跑 Corpus Build workflow · ubuntu runner 上 partial+sparse 真活
- [ ] **优化 1 · 双语 sparse 动态算**：当前 sparse 不论 LANGS 都 fetch en/* · 应根据 lang 动态算
- [ ] **优化 2 · `--clean-tmp`**：collect-corpus.sh 加清 $TMPDIR 选项 · 不靠手工删
- [ ] **corpus-v1.1.0 路径**：v1.0.0 推后 · build v1.1.0 含 OpenHarmony
- [ ] **mirror 切换**：gitee.com/openharmony/docs 等国内镜像支持
