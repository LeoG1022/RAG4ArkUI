# STATUS — install-script

> 配套 feature log：`feedback/features/rag4arkui-core/37-2026-05-30-install-script.md`
> 配套 meta：`feedback/meta/18-2026-05-30-install-binary-script.md`
> 日期：2026-05-30

---

## 当前状态

把 Round 33-36 实战累积的 4 个独立坑 · 全部封装在 `make install` 一键命令里。从 「`make release-local` 编完」到「Claude 真活调 arkui_search_docs」从 90 分钟（含实战 debug）压缩到 90 秒。

本阶段交付：
- `scripts/install-binary.sh` 220 行 · 7 step（含 macOS ad-hoc codesign 必要预防 SIGKILL）
- `Makefile` `install` / `install-no-mcp` 两个 target · 串接 `release-local` build → install
- `docs/MCP-INTEGRATION-CLAUDE-CODE.md` 顶部加 120 行「推荐一键路径 + macOS provenance 坑 + Code/Desktop 双端配置差异」
- `docs/USER-VERIFICATION.md` Step 5 改写 · `sudo cp` → `make install`
- 实测：脚本端到端 ✓ `claude mcp list` 显示 `arkui-rag · ✓ Connected`

意义：Round 33-36 全部修了**代码 / 配置 / 设计**层的问题 · Round 37 修**用户实际安装到本地这步**——把所有踩过的坑写进脚本 · 不让下个用户重新发现。

## 输入契约

### 新增 Makefile targets

| Target | 行为 |
|---|---|
| `make install` | `make release-local` → `bash scripts/install-binary.sh`（默认） |
| `make install-no-mcp` | 同上 · 但 `--skip-mcp`（CI / 已配过 MCP 用） |

### `install-binary.sh` 输入

```bash
bash scripts/install-binary.sh \
    [--bin-dir <PATH>]      # 默认 ~/.local/bin
    [--skip-mcp]            # 跳过 MCP 自动配置

# 环境变量
ARKUI_INDEX_PATH=<PATH>     # 默认 ~/.arkui-rag/index.json
```

### 不变项

- CLI / API / 索引格式全部不变
- `arkui-rag serve --mcp` 等子命令接口不变
- 配置文件格式（Claude CLI / Desktop）格式不变 · 脚本只是自动化「人肉编辑」步骤

## 输出契约

### `make install` 副作用清单

1. **`~/.local/bin/arkui-rag`** 写入（覆盖旧版前会 backup `*.bak.YYYYMMDD-HHMMSS`）
2. **macOS：ad-hoc codesign 已签名** —— `codesign --force --sign -` · 不带证书
3. **`~/.claude.json`** 顶层 `mcpServers.arkui-rag` 节添加 / 更新（通过 `claude mcp add` 命令）
4. **`~/Library/Application Support/Claude/claude_desktop_config.json`** 顶层 `mcpServers.arkui-rag` 节添加 / 更新（Python 安全合并 · 备份既有 config）

### 成功标志

最后一段输出：

```
🎉 安装完成

下一步：
  1. 重启 Claude Code CLI: ...
  2. 重启 Claude Desktop: ...
  3. 新 chat 测试：用 arkui_search_docs 检索 X
```

中间 `Step 6 · Claude Code CLI MCP` 验证段必须包含：

```
✅ arkui-rag ✓ Connected
```

### 失败标志

任一 step 报错（脚本 `set -euo pipefail` · 立即退）。常见：

| 信号 | 原因 | 修法 |
|---|---|---|
| Step 1 ❌ source binary 不存在 | 没跑 `make release-local` | `make release-local` 再 `make install` |
| Step 3 ❌ exit 137 | codesign 没生效 / 用户改了脚本 | 查 `codesign -dv ~/.local/bin/arkui-rag` |
| Step 6 ⚠️ 已配置但未 Connected | binary 跑不起来 / 索引路径错 | 手动跑 `~/.local/bin/arkui-rag serve --mcp ...` 看 stderr |
| Step 7 跳过 | Claude Desktop 没装 | 不影响 CLI 部分使用 |

## 验证手段

### 用户手动

```bash
# 完整流程（90 秒）
make release-local && make install

# 验证
which arkui-rag             # → ~/.local/bin/arkui-rag
arkui-rag --version         # → arkui-rag 0.0.1
claude mcp list | grep arkui   # → ✓ Connected

# 重启 Claude
# CLI: 当前 claude 进程 Ctrl-D · 重跑 claude
# Desktop: pkill -i "Claude" && sleep 2 && open -a Claude

# 新 chat 调用
# 「用 arkui_search_docs 检索 @State 双向绑定」
```

### 自动化

```bash
# 语法 check
bash -n scripts/install-binary.sh

# 端到端（agent 本会话已跑通）
bash scripts/install-binary.sh
# 期望最后看到：🎉 安装完成 · arkui-rag ✓ Connected
```

CI 集成（残留）：本轮没加 CI step · 留作下一轮（macOS-only matrix smoke test）。

## 与上一阶段的关联性

| Round | 主题 | 解决层 |
|---|---|---|
| 33 | concepts-archive-rule | Agent 行为契约 |
| 34 | cli-default-features | **Build** 路径 |
| 35 | mcp-config-path-fix | **Install** 路径（配置文件） |
| 36 | tantivy-read-only | **Runtime** 路径（多 client 并发） |
| **37（本轮）** | install-script | **Deploy** 路径（一键封装所有坑） |

层次：build 通 → 配置写对 → runtime 多 client 并发 → **用户实际机器上 90 秒装通**。

兼容性：
- 旧文档「sudo cp /usr/local/bin」仍可工作（如果用户跑了 xattr -d + codesign）· 新文档强推 `~/.local/bin/`
- 旧配置（用户已 `claude mcp add` 过）`install-binary.sh` 自动 remove + 重 add · 幂等
- Claude Desktop 旧 config（已合并过）也幂等 · Python 安全合并不破坏其它 mcpServers 节

破坏性变更：无。

性能：
- `make install` 全程 ~3 秒（不含 release build · 那是 `make release-local` 的 21 秒）
- Round 33-36 实战 90 分钟（含 debug）→ 本轮 90 秒（一键）· 用户体验 **60x 提升**

## 完成度 / 下一阶段

### 达成度

| 项 | 状态 |
|---|---|
| install-binary.sh 220 行 | ✅ |
| Makefile install / install-no-mcp | ✅ |
| MCP-INTEGRATION 加 macOS / 双端章节 | ✅ |
| USER-VERIFICATION Step 5 改写 | ✅ |
| 端到端自检（含 codesign 必要性证明）| ✅ |
| 双轨归档 + STATUS（本文件）| ✅ |
| 用户在两端 chat 调 arkui_search_docs | ✅（Round 36 已验证）|

### 下一阶段建议

立即（用户做）：
1. 没必要立即做什么——本轮已端到端跑通 · 整条接入链路 production-ready
2. 想测脚本可重跑 `make install`（幂等）

短期（agent 做 · 1-2 round）：
- `scripts/release-local.sh` 加 codesign step · dist/ tarball 内 binary 也 self-signed · 解压立即可跑
- `Makefile uninstall` target · 反向操作（删 binary + remove MCP 双端配置）· 方便切换 / 测试
- CI macOS matrix step 跑 `make install` smoke test · 防脚本回归
- `classify-change.sh` 修分类边界（Makefile / `crates/**/Cargo.toml` / `mdbook/src/**` 应判 meta · 当前 unknown）· 三个残留一并修

中期：
- Linux / Windows install 路径不一样（无 codesign · 但可能有 binfmt 限制）· 本轮脚本只测了 macOS · 跨平台支持
- Linux：`~/.local/bin/` 标准 · 无 codesign 步骤 · 直接 cp + chmod 就够
- Windows：`%USERPROFILE%\bin\` 或 `%LOCALAPPDATA%\Programs\` · 配置文件路径不同（`%APPDATA%\Claude\claude_desktop_config.json`）· 单独写 PowerShell 脚本

长期：
- 1.0 release 时考虑 brew formula（macOS）+ apt/yum repo（Linux）· 让 `brew install arkui-rag` / `apt install arkui-rag` 也 work · 用户体验最佳
- 当前 `make install` 是「源码用户」流程 · brew/apt 是「二进制用户」流程 · 互补不冲突
