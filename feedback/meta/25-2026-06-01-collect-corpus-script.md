# 25 — collect-corpus-script

> 日期：2026-06-01
> 触发：用户启动 Round 49.5 第 2 半（ArkUI-X / OpenHarmony 真文档收集）· 需要可重复的 maintainer 脚本
> 类型：工具脚本（新增 scripts/collect-corpus.sh）

---

## 用户提出的要求

> 「接上次 arkui-x文档仓库 https://gitcode.com/arkui-x/docs OpenHarmony docs：https://gitcode.com/openharmony/docs 重分发 OK · 启动 Round 49.5 第 2 半」

潜在需求：maintainer 之后多次重收集（升级版本 / CI 自动 re-build）· 不能每次都手敲一遍 git clone + rsync · 需要脚本固化。

## Agent 给出的修改建议

新增 `scripts/collect-corpus.sh`：
- shallow clone（`--depth=1`）· 不带 git history · 省时省盘
- rsync 只复制 `*.md` 文件 + LICENSE · 不要图片 / .git / 二进制资源
- 支持 `--src arkui-x|openharmony` 单收 · `--lang zh-cn|en|zh-cn en` 控语言
- 支持 `--clean` 清掉 `corpus/official/{arkui-x,openharmony}`
- 自动保留 LICENSE / README.md（Apache 2.0 重分发要求）
- 复用 clone 目录（再跑不会重拉 · 删 `$TMPDIR/corpus-collect/X/` 强制重拉）

替代方案：
- A · 不写脚本 · 每次手敲（被否：3 个月后忘细节 · CI 没法自动）
- B · 用 git submodule（被否：把 1066 文件灌进自家 repo history · 升级时 history 膨胀）
- **C · shallow clone + rsync 复制（本次选）**：corpus/official/arkui-x/ 当成普通文件管理 · 升级时直接覆盖 · 不留 git history

## 多轮互动

无 —— 用户给了 URL + 重分发授权 · agent 自主设计脚本接口。

## 实际改动

- 接口变化：新增 CLI 工具 `bash scripts/collect-corpus.sh [--src X] [--lang Y] [--clean]`
- 规则变化：无（不动 pre-commit / hook / classify）
- 文件变化：
  - 新增 `scripts/collect-corpus.sh`（121 行 · shebang + 注释 + 参数解析 + collect_one 函数 + 主循环）
- 配置变化：脚本内常量
  - `ARKUIX_REPO="https://gitcode.com/arkui-x/docs.git"`
  - `OPENHARMONY_REPO="https://gitcode.com/openharmony/docs.git"`
  - `CORPUS_DIR="$REPO_ROOT/corpus/official"`
  - `TMP_DIR="${TMPDIR:-/tmp}/corpus-collect"`

## 执行生效后总结

### 实际产出

| 项 | 内容 |
|---|---|
| 脚本 | `scripts/collect-corpus.sh` 可执行（+x）· bash -n 语法 OK |
| 单源测试 | `--src arkui-x --lang "zh-cn en"` → 1066 .md / 19MB / LICENSE 保留 ✓ |
| OpenHarmony | 仓库 600MB+ · 5 分钟超时 · 留 Round 49.7 改 partial clone |
| 默认行为 | 不带参数即全收 · LANGS="zh-cn en" · 双语全要 |

### 前后对比

| 操作 | Round 49（无脚本）| Round 49.5（有脚本）|
|---|---|---|
| 拉 ArkUI-X | 手敲 git clone + rsync · 30 行命令 · 易漏 LICENSE | `bash scripts/collect-corpus.sh --src arkui-x` |
| 升级到 v1.1.0 | 翻历史命令 / 重凭记忆 | 同样一行 · 输出可重现 |
| CI 自动 | 难（命令散在 round 笔记里）| 直接调用 |

### 实测验证

```bash
bash -n scripts/collect-corpus.sh        # ✓ 语法 OK
bash scripts/collect-corpus.sh --src arkui-x --lang "zh-cn en"
# 🌐 arkui-x: shallow clone（不带 history · 几分钟）
# Cloning into '/tmp/corpus-collect/arkui-x'...
#   📄 arkui-x/zh-cn
#   📄 arkui-x/en
#   ✅ arkui-x: 1066 files · 19M
```

LICENSE 自动复制到 `corpus/official/arkui-x/LICENSE` ✓（Apache-2.0 重分发要求）。

### 残留 / 下一轮处理

- [x] scripts/collect-corpus.sh 新增 · ArkUI-X 单源真活
- [ ] **Round 49.7**：OpenHarmony 需 partial clone（`--filter=blob:none + sparse checkout`）· 现脚本对 600MB+ 仓库不可靠 · 需加分支逻辑（OpenHarmony 走 partial clone · ArkUI-X 仍用 shallow clone）
- [ ] **Round 49.6**：把脚本接入 CI workflow `corpus-build.yml`
- [ ] **长期**：支持自定义 corpus 源（如 OEM 私有镜像）· 通过 `--repo` flag 注入
