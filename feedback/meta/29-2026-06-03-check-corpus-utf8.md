# 29 — check-corpus-utf8

> 日期：2026-06-03
> 触发：feature/57 配套 · 防御未来类似 UTF-8 build 失败
> 类型：工具脚本（新增 scripts/check-corpus-utf8.sh）

---

## 用户提出的要求

> 「要先解决 index 失败问题」（接 Round 52 Phase B 3.2h build 撞 UTF-8 错挂掉）

潜在需求：
1. 知道哪个文件是非 UTF-8
2. 自动批量修复（不要手工 iconv 一个个）
3. CI / collect-corpus 后能自动 check 防患于未然

## Agent 给出的修改建议

新增 `scripts/check-corpus-utf8.sh`：
- 扫指定目录所有 .md · python 解码 · 输出非 UTF-8 文件位置 + 上下文
- `--fix` flag：尝试 GBK / GB18030 iconv 自动转 UTF-8
- 退出码 0/1/2（CI 友好）

### 替代方案

- A · `file` 命令：BSD 上不可靠（README.md 误判 ISO-8859）
- B · `iconv -f UTF-8 -t UTF-8`：BSD 上同样不可靠（127/128 误报）
- **C · python3 UnicodeDecodeError（本次选）**：标准库 · 跨平台准确 · 系统自带

### 关键决策

| 决策 | 选择 | 理由 |
|---|---|---|
| 扫描器 | python3 系统自带 | 不依赖第三方装 |
| 修复编码尝试 | GBK 优先 · GB18030 fallback | 中文遗留最常见 |
| 默认行为 | 不 fix · 只 report | 防止误改 · `--fix` 显式启用 |
| 退出码 | 0/1/2 | CI hook 友好 |

## 多轮互动

无 —— 配 feature/57 root cause fix 同时加这个防御工具 · agent 主动设计。

## 实际改动

- 接口变化：新增 `bash scripts/check-corpus-utf8.sh [--path X] [--fix]`
- 规则变化：无（不强制接入 pre-commit · 仅手动 / CI 调）
- 文件变化：
  - 新增 `scripts/check-corpus-utf8.sh`（~80 行 · python 扫 + iconv fix）
- 配置变化：无

## 执行生效后总结

### 实际产出

| 项 | 内容 |
|---|---|
| 默认扫 | `corpus/official/` 所有 .md |
| 自定义路径 | `--path corpus/official/openharmony` |
| 自动 fix | `--fix` 试 GBK / GB18030 iconv |
| 输出 | 非 UTF-8 文件 + 字节位置 + hex + 上下文 |

### 前后对比

| 操作 | Round 52 前 | Round 54（本次）|
|---|---|---|
| build 撞 UTF-8 错 | `error: io error: stream did not contain valid UTF-8` 死 | indexer lossy fallback · 继续 |
| 找罪魁文件 | 没工具 · python ad-hoc | `bash scripts/check-corpus-utf8.sh` 一行 |
| 批量修复 | 手工 iconv 一个个 | `--fix` 自动 |
| CI 防御 | 无 | CI 可加 `check-corpus-utf8` 步骤 · 0=pass / 1=fail / 2=err |

### 实测验证

```bash
bash scripts/check-corpus-utf8.sh
# ═══ 扫 corpus/official 下所有 .md ═══
# ✅ 全部 UTF-8 · 无需修复（罪魁文件已 fix）

bash scripts/check-corpus-utf8.sh --path corpus/official/arkui-x   # ✓
```

### 残留 / 下一轮处理

- [x] scripts/check-corpus-utf8.sh 工具脚本
- [x] python 扫 + iconv fix 双功能
- [x] 退出码友好（CI 集成预留）
- [ ] **scripts/collect-corpus.sh 集成**：拉完 corpus 后自动跑 check（避免新 corpus 出问题）
- [ ] **CI workflow corpus-build.yml 集成**：在 cargo build 之前先跑 check（提前失败）
- [ ] **pre-commit hook 集成**：commit 含 corpus/* 时自动跑 check（防止 commit 进坏文件）
- [ ] **--fix 不能识别的编码**：UTF-16 BE/LE · BIG5（繁中）等 · 后续补
- [ ] **OpenHarmony 109MB 拉完后跑一次**（gitignore 屏蔽 · 本地有 · 看是否含 GBK）
