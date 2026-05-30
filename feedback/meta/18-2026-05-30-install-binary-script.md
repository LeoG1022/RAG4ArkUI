# 18 — install-binary-script

> 日期：2026-05-30
> 触发：Round 33-36 实战中累积发现 4 个独立坑 · 用户最终诉求「两端 MCP 都能用」· 需要把坑封装成一键脚本 · 不能让下个用户重踩 4 次
> 类型：Skill 变更（scripts/install-binary.sh 新建）+ 流程优化（Makefile install target）

---

## 用户提出的要求

原文：

> 一验证好，go on

承接 Round 36 末尾 Agent 提议的 Round 37 计划 · 用户确认两端 Claude 接入 ✓ Connected 之后 · 让 agent 继续做：
1. `scripts/install-binary.sh` 一键脚本
2. Makefile `install` target
3. `docs/MCP-INTEGRATION-CLAUDE-CODE.md` 加章节（macOS provenance + Code/Desktop 双端差异）
4. `docs/USER-VERIFICATION.md` Step 5 改推荐 `make install`

## Agent 给出的修改建议

实战 Round 33-36 累积发现 4 个独立坑（每个新用户都会踩 · 不修文档下个用户也会被绊倒）：

1. 裸 `cargo build` 产物 features=[] —— Round 34 已修 Cargo.toml default
2. `~/.claude/mcp.json` 错路径 —— Round 35 已改用 `claude mcp add`
3. **Claude Code CLI vs Claude Desktop GUI 走不同配置文件** —— 本轮文档化 + 自动配
4. **macOS provenance 杀 root-owned binary** —— 本轮发现 + 落地一键预防

落地方案：

| 文件 | 内容 |
|---|---|
| `scripts/install-binary.sh` | 7 step 一键：cp 到 `~/.local/bin` → ad-hoc codesign → 验证 --version → PATH 检查 → 索引检查 → `claude mcp add` → 合并 Desktop JSON |
| `Makefile` | `install` 调上面脚本 · `install-no-mcp` 跳过自动配 |
| `docs/MCP-INTEGRATION-CLAUDE-CODE.md` | 顶部「🚀 推荐一键路径」+「macOS provenance 坑」+「Code/Desktop 双端配置差异」表 |
| `docs/USER-VERIFICATION.md` Step 5 | `sudo cp` → `make install` |

关键决策：

- **目标位置 = `~/.local/bin/`**（不是 `/usr/local/bin`）—— 用户级 · 避开 provenance · 长期稳
- **必须 ad-hoc codesign**（`codesign --force --sign -`）—— 实测发现：`cp` 覆盖既有 binary 后 macOS provenance 缓存导致 SIGKILL · 即便用户目录、即便 source 跑得起来。本会话内复现：手动 17:44 cp 跑 OK · 脚本 18:45 cp 跑 137 · 加 codesign 后 OK
- **MCP 配置都用绝对路径** —— Desktop 不继承 shell PATH · CLI 用绝对也更稳

替代方案权衡：

| 选项 | 优点 | 缺点 | 选了吗 |
|---|---|---|---|
| A · `~/.local/bin` + ad-hoc codesign（本轮）| 不 sudo · 长期稳 · XDG 标准 | 一行 codesign | ✅ |
| B · `sudo cp /usr/local/bin` + `xattr -d com.apple.provenance` | 跟旧文档一致 | 每次重装可能再被打标 · macOS 升级风险 | ❌ |
| C · `cargo install --path` | cargo 标准 | 装到 `~/.cargo/bin/` · 重复编译 | ❌ |
| D · brew formula | macOS 用户标准 | 维护 tap · 单 binary 不值 | ❌（1.0+ 考虑）|

## 多轮互动

| 轮次 | 用户 | Agent |
|---|---|---|
| 1 | Round 35-36 完成 · 提诉求「配好 Claude CLI 和 Claude app · 能使能 arkui-rag」 | 给「sudo cp 一键路径」但被 provenance 杀 |
| 2 | 「不要删 com.apple.provenance · 装到非系统目录 + PATH」 | 立刻执行：cp 到 ~/.local/bin · 改两端 MCP · 跑通 ✓ Connected |
| 3 | 「一验证好，go on」 | 本轮：把刚才手动步骤 + ad-hoc codesign 实战经验封装成 install-binary.sh + Makefile + 文档 |

无方向调整 · 用户主动给出更优方案（不删 provenance · 改路径）后 agent 落地。

## 实际改动

- 接口变化：无（CLI / API 不动）
- 规则变化：推荐安装路径从 `/usr/local/bin/`（旧）改为 `~/.local/bin/`（新）· 多处文档同步
- 文件变化：
  - **新建** `scripts/install-binary.sh` —— 220 行 · 7 step 含 macOS codesign + 双端 MCP 自动配
  - **修改** `Makefile` —— 加 `install` + `install-no-mcp` 两个 target
  - **修改** `docs/MCP-INTEGRATION-CLAUDE-CODE.md` —— 顶部 +120 行（推荐路径 + provenance 坑 + 双端差异表）
  - **修改** `docs/USER-VERIFICATION.md` Step 5 · `sudo cp` → `make install`
- 配置变化：无

## 执行生效后总结

### 实际产出

| 文件 | 改动 |
|---|---|
| `scripts/install-binary.sh` | 新建 220 行 · 7 step（codesign + MCP 双端）|
| `Makefile` | +8 行（install / install-no-mcp） |
| `docs/MCP-INTEGRATION-CLAUDE-CODE.md` | +75 行（推荐路径 + provenance + 双端差异）|
| `docs/USER-VERIFICATION.md` | +20 / -4（Step 5 改 make install）|

### 前后对比

| 维度 | Before（Round 36 前） | After（Round 37） |
|---|---|---|
| 推荐安装命令 | `sudo cp dist/.../arkui-rag /usr/local/bin/` | `make install`（一行）|
| 自动配 Claude CLI MCP | 用户手动 `claude mcp add` | 脚本自动 |
| 自动配 Claude Desktop MCP | 用户手动编辑 JSON | 脚本自动合并 + 备份 |
| macOS provenance 坑 | 用户必踩 · 静默 SIGKILL · debug 半小时 | 脚本 ad-hoc codesign 直接预防 |
| Code/Desktop 配置差异说明 | 无文档 | docs/MCP-INTEGRATION 表格 |
| 用户从 0 接入耗时 | 实战 90 分钟（含 debug）| 90 秒（`make release-local && make install`）|

### 实测验证

- `bash -n scripts/install-binary.sh` ✓ 语法 OK
- `bash scripts/install-binary.sh --help` ✓ 显示完整 usage
- `bash scripts/install-binary.sh` 端到端 ✓：
  - Step 1-2 cp + codesign · binary 装到 ~/.local/bin
  - Step 3 `arkui-rag --version` exit 0
  - Step 4 PATH 解析对了
  - Step 5 索引存在
  - Step 6 `claude mcp list` ✓ Connected
  - Step 7 Desktop config 合并 + 备份 OK
- 关键发现 codesign 必要性：脚本第一次跑（不带 codesign）SIGKILL 137 · 加 `codesign --force --sign -` 后立即 exit 0 · 已并入脚本

### 残留 / 下一轮处理

- [x] scripts/install-binary.sh 写完 + 自检通过
- [x] Makefile install + install-no-mcp 加好
- [x] MCP-INTEGRATION 加章节
- [x] USER-VERIFICATION Step 5 改 make install
- [x] 双轨归档（meta/18 + feature/37）+ STATUS
- [ ] **classify-change.sh 分类边界** · Makefile 被分类 `unknown(视作业务)` · 应判 meta（基础设施）· 跟前轮 `crates/**/Cargo.toml` / `mdbook/src/**` 边界一并修
- [ ] **CI 加 `make install` smoke test** · 防脚本回归
- [ ] **release-local.sh 加 codesign step** · dist/ tarball 解出的 binary 也 self-signed · 用户下载 release 不踩 provenance
- [ ] **Makefile 加 `uninstall` target** · 反向操作（删 binary + remove MCP 双端配置）· 方便切换 / 测试
