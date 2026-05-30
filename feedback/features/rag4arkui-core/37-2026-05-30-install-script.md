# 37 — install-script

> 日期：2026-05-30
> 涉及代码：`scripts/install-binary.sh`（新）· `Makefile`（install target）· `docs/MCP-INTEGRATION-CLAUDE-CODE.md` · `docs/USER-VERIFICATION.md`
> 类型：新建（一键安装脚本 · 业务流程封装）

## 本轮目标

把 Round 33-36 实战中累积的 4 个独立坑 · 全部封装在 `make install` 一行命令里 · 让下个用户从「`make release-local` 编完」到「Claude 真活调 arkui_search_docs」只需 90 秒（之前实战是 90 分钟含 debug）。

涵盖：

1. Round 34 默认 features 问题（已修代码 · 本轮文档强化）
2. Round 35 `claude mcp add` 命令（脚本自动调）
3. Round 36 多 client 并发（前置 binary 已修 · 脚本部署新版）
4. **Round 37 新：macOS provenance 杀 root-owned binary + 双端配置差异**

## Plan

### 设计：`install-binary.sh` 7 step

1. **校验 source binary** —— `crates/target/release/arkui-rag` 存在 + 可执行
2. **装到 BIN_DIR**（默认 `~/.local/bin`）—— cp + chmod + 备份既有版本
   - **加 ad-hoc codesign** —— macOS 必须 · 否则 cp 覆盖后 provenance 缓存触发 SIGKILL
3. **跑 --version 自检** —— 装好的 binary 必须 exit 0 + 输出版本号
4. **PATH 检查** —— BIN_DIR 在 PATH 就提示 OK · 不在就提示加 `~/.zshrc`
5. **索引检查** —— `~/.arkui-rag/index.json` 存在 · 否则提示用户先 `arkui-rag index`
6. **配 Claude Code CLI MCP** —— `claude mcp remove` + `claude mcp add --scope user` 用绝对路径 · 跑 `claude mcp list` 验证 `✓ Connected`
7. **配 Claude Desktop GUI MCP** —— Python 安全合并 `~/Library/Application Support/Claude/claude_desktop_config.json` · 备份 + 绝对路径

### Makefile：3 个 target

```makefile
install: release-local
    bash scripts/install-binary.sh

install-no-mcp: release-local
    bash scripts/install-binary.sh --skip-mcp
```

### 文档：MCP-INTEGRATION 顶部加 90 秒入口

旧文档要求用户逐节读 § 一编译 · § 二配置 · 5 个 sub-section 各自看完。新文档：

- 顶部「🚀 推荐一键路径」3 行命令解决
- 「macOS provenance 坑」专节解释为啥不 sudo cp 到 /usr/local/bin
- 「Claude Code CLI vs Claude Desktop GUI · 双端配置差异」表

`USER-VERIFICATION.md` Step 5 同步：「sudo cp」→「make install」+ 验证 checklist。

### 关键决策：必须 codesign 的实测发现

设计脚本时本以为「cp 到 ~/.local/bin 就够了」（user-owned 不触发 provenance）。第一次跑脚本：

```
Step 3 · 验证装好的 binary 能跑
❌ exit code 137
```

诊断：source `crates/target/release/arkui-rag` 也有 `com.apple.provenance` xattr · 但 source 能跑（exit 0）· cp 出来的不能跑。手动 17:44 第一次 cp 时也能跑。

复现：脚本 18:45 cp 覆盖 17:44 旧文件 · macOS 触发新 provenance 检查 · SIGKILL。

修法：`codesign --force --sign -`（ad-hoc 签名 · 不要 sudo · 不要证书）· 立即 exit 0。

并入脚本 Step 2 之后 · 第二次跑脚本端到端通过。

### 替代方案

| 选项 | 优点 | 缺点 | 选 |
|---|---|---|---|
| A · `~/.local/bin` + codesign（本轮）| 不 sudo · XDG 标准 · 长期稳 | 加一行 codesign | ✅ |
| B · `/usr/local/bin` + xattr -d com.apple.provenance | 跟旧文档一致 | 重装可能再被打标 | ❌ |
| C · `cargo install --path crates/arkui-rag-cli` | cargo 标准 · 自动 PATH | 重复编译 (.cargo/bin)· feature 选择不直观 | ❌ |
| D · brew formula | macOS 标准 | 维护 tap · 1.0+ 再做 | ❌ |

## 对话摘要

| 轮次 | 用户 | Agent 决策 |
|---|---|---|
| 1 | Round 35-36 后诉求「配好 Claude CLI 和 Claude app · 能使能 arkui-rag」 | 给「sudo cp 一键路径」· 用户跑后 provenance 杀 |
| 2 | 「不要删 com.apple.provenance · 装到非系统目录 + PATH」 | 立刻执行：cp 到 ~/.local/bin · 改两端 MCP 配置 · ✓ Connected |
| 3 | 「一验证好，go on」 | 本轮：封装手动步骤 + 实测发现的 codesign 步骤 进 install-binary.sh + Makefile install + 文档 |

无方向调整。用户主动给出更优方案 · agent 落地。

## 改动要点

- **scripts/install-binary.sh** 220 行 · 7 step：
  - 设计要点写在文件头注释里 · 给未来读者解释为啥不 `/usr/local/bin`
  - Step 2 必备 codesign 块（macOS-only · 用 `uname -s` gate）· 含 inline 注释解释复现条件
  - Step 6 调 `claude mcp remove` 容忍未配（`|| true`）· `add` 用绝对路径 + `--scope user`
  - Step 7 Python 安全合并 JSON · 不覆盖原有 mcpServers + 备份既有 config
  - 末尾给清晰下一步 3 件事
- **Makefile** 加 `install: release-local` + `install-no-mcp` · 把 release build 串到一键安装
- **docs/MCP-INTEGRATION-CLAUDE-CODE.md** 顶部插入「推荐一键路径 + macOS provenance + 双端差异表」120 行 · § 一加引导句「能用 make install 跳到下一节」
- **docs/USER-VERIFICATION.md** Step 5 「sudo cp」改成 `make install` + 验证 checklist + 警告框
- 与 Round 36 关系：36 改了 binary（解锁多 client 并发）· 37 改了**部署 binary 的流程**（让用户实际拿到新 binary 且能跑）· 36 和 37 合起来 = 整条「代码改完 → 用户机器上真运行」闭环

## 验证结果

- 编译：N/A（脚本 + 文档 + Makefile · 无 Rust 改动）
- 单元测试：N/A
- 集成测试：`bash scripts/install-binary.sh` 端到端跑通 ✓
  - 7 step 全部通过
  - Step 6 `claude mcp list` 输出 `arkui-rag: ... ✓ Connected`
  - Step 7 Desktop config 合并成功 + 备份 OK
- 实测发现：codesign 步骤必须 · 第一次跑脚本（不带 codesign）SIGKILL 137 · 加上后 exit 0 · 已并入正式脚本
- 文档新章节链接 + 结构 OK · TOC 隐式有效

事后验证：用户重启 Claude CLI + Desktop 后两端 chat 调 arkui_search_docs · 都返回 Top-K hits = 文档化的流程对了。

## 残留 / 下一轮

- [x] scripts/install-binary.sh 220 行 + 自检 ✓
- [x] Makefile install + install-no-mcp
- [x] docs/MCP-INTEGRATION-CLAUDE-CODE.md 加章节
- [x] docs/USER-VERIFICATION.md Step 5 改写
- [x] 双轨归档 + STATUS
- [ ] **用户跑 `make install` 验证**：不必（agent 已端到端跑通） · 但下次构建后用户应直接 `make install` 而非手 cp
- [ ] **CI 加 install 端到端 smoke test** · 防脚本回归 · 建议作 macOS-only matrix step
- [ ] **release-local.sh 加 codesign step** · dist/ tarball 内 binary 也 self-sign · 用户下载 release 解压后立即可跑
- [ ] **Makefile uninstall target** · `claude mcp remove arkui-rag` + 删 ~/.local/bin/arkui-rag + 删 Desktop config 节
- [ ] **MCP-INTEGRATION 的 TOC** · 顶部章节多了之后导航要更新（暂用 GitHub markdown 自动 TOC 也行）
