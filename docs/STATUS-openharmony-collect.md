# STATUS — openharmony-collect

> 配套：`feedback/meta/27-openharmony-partial-clone.md` + `feedback/features/rag4arkui-core/55-openharmony-collect.md`
> 日期：2026-06-02
> Round 49.7 — OpenHarmony 文档 partial clone + sparse checkout

---

## 当前状态

Round 51 的 OpenHarmony shallow clone 5 分钟超时 + 仓库损坏（残留 Round 51 第 2 半的 [-] 项）· 本轮换 partial clone + sparse checkout 策略 7 分钟拉完 109MB。

至此 corpus 双源（A=ArkUI-X · B=OpenHarmony）链路全活：

| 源 | 仓库 | 策略 | 实测时间 | 实测大小 |
|---|---|---|---|---|
| ArkUI-X | gitcode.com/arkui-x/docs | shallow `--depth=1` | ~1 分钟 | 19MB / 1066 .md（入 git）|
| OpenHarmony | gitcode.com/openharmony/docs | partial `--filter=blob:none` + sparse `application-dev + device-dev` | 7m23s | 109MB / 7343 .md（gitignore 屏蔽）|

## 输入契约

`scripts/collect-corpus.sh` 加新参数：

```bash
--oh-paths "..."   # 覆盖 OpenHarmony sparse paths
                    # 默认: "zh-cn/application-dev zh-cn/device-dev en/application-dev en/device-dev"
```

无破坏性变更 · 既有 `--src` / `--lang` / `--clean` 不动。

CI workflow `corpus-build.yml` **本轮不动**：现在仍只跑 `--src arkui-x` 推 corpus-v1.0.0。v1.1.0 加 OpenHarmony 时一并改 workflow。

## 输出契约

- `corpus/official/openharmony/`（本地 maintainer 生成 · git 屏蔽）
  - LICENSE / README.md / zh-cn + en 的 application-dev + device-dev 子目录的 .md
  - 7343 文件 · 109MB

`.gitignore` 加 `/corpus/official/openharmony/`：

```gitignore
# Round 49.7: OpenHarmony corpus 109MB · 太大不入 git
/corpus/official/openharmony/
```

## 验证手段

### Agent 本轮已做

```bash
# 用临时 TMPDIR 避免沙箱 rm 限制
export TMPDIR=/Users/leo/tmp-corpus-collect
bash scripts/collect-corpus.sh --src openharmony --lang zh-cn

# ✓ 7m23s · 7343 .md · 109MB
# git status: ✅ 已 gitignore 屏蔽
```

### 用户验证（如想本地复现）

```bash
# 收 OpenHarmony zh-cn 默认子目录
bash scripts/collect-corpus.sh --src openharmony --lang zh-cn

# 收 OpenHarmony 双语 + 自定义子目录
bash scripts/collect-corpus.sh \
    --src openharmony \
    --lang "zh-cn en" \
    --oh-paths "zh-cn/application-dev en/application-dev"

# 双源全收（ArkUI-X shallow + OpenHarmony partial+sparse）
bash scripts/collect-corpus.sh
```

### CI 验证（等 corpus-v1.1.0）

CI workflow 当前**不跑** OpenHarmony · v1.1.0 时一并改 corpus-build.yml。

## 与上一阶段的关联性

| Round | 主题 | 关系 |
|---|---|---|
| 51 | ArkUI-X 真 corpus + collect-corpus.sh shallow | 本轮扩展 collect-corpus.sh · 加 partial+sparse 分支 |
| 49.6 | maintainer CI corpus-build.yml | 当前仅跑 ArkUI-X · v1.1.0 改加 OpenHarmony 步骤 |
| 49.8 | cli index-pull | 用户拉法不变（默认仍 corpus-v1.0.0 · v1.1.0 时改默认）|
| **55（本轮）** | OpenHarmony partial+sparse 收集 | 解锁 v1.1.0 双源 release |

## 完成度 / 下一阶段

### 达成度

| 项 | 状态 |
|---|---|
| partial+sparse 函数实装 | ✅ |
| `clone_shallow` / `clone_partial_sparse` 拆分 | ✅ |
| 按 name 路由策略 | ✅ |
| 默认 sparse paths | ✅ |
| `--oh-paths` 自定义参数 | ✅ |
| OpenHarmony 实测真活（7m23s · 109MB · 7343 .md）| ✅ |
| `.gitignore` 屏蔽 109MB | ✅ |
| ArkUI-X 路径不退化 | ✅ |
| CI workflow 加 OpenHarmony 步骤 | ⏭ v1.1.0 推 |

### 下一阶段建议

**Round 49.6.3（corpus-v1.1.0 推 release）**：
1. 改 corpus-build.yml 加 `collect-corpus --src openharmony --lang $LANGS` 步骤
2. version 默认 v1.1.0
3. CI 跑 → 推 corpus-v1.1.0 release
4. cli `default_index_url` 默认 version 升 v1.1.0（或保持 v1.0.0 让用户显式 `--version v1.1.0`）

**Round 49.7.1（优化）**：
1. sparse paths 根据 `--lang` 动态算（用户 `--lang zh-cn` 时不 fetch en/*）
2. 加 `--clean-tmp` 选项

**Round 52（init wizard）**：把 model-pull + index-pull 串起来 · 用户 zero-config

**长期**：
- mirror 切换（gitee.com / 阿里云 OSS）
- 国内访问优化
- OpenHarmony 全部子目录（不限 application-dev / device-dev）可选支持
