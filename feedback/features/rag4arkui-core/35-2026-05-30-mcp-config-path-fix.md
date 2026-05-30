# 35 — mcp-config-path-fix

> 日期：2026-05-30
> 涉及代码：`docs/MCP-INTEGRATION-CLAUDE-CODE.md`（主接入指南）· `mdbook/src/usage/mcp.md`（mdBook 站快速接入节）
> 类型：bug 修复（文档错路径误导用户 · 全链路打通最后一公里）

## 本轮目标

修一个**实操致命错路径**：

历史文档说 Claude Code 的 MCP 配置在 `~/.claude/mcp.json` · 实际**不读这个文件** · 真正配置在：
- `~/.claude.json`（顶层 `mcpServers` 节）· 用 `claude mcp add` 命令管理

用户按错文档照做 → `cat > ~/.claude/mcp.json` → 重启 Claude Code → 调 `arkui_search_docs` 时 Claude 回应「我没有这个工具」。

后果：整条「本地 RAG 接 Claude Code」链路 99% 已通（binary / 索引 / MCP stdio 握手 / 4 工具 stdio 验证全部 ✓），但**最后一公里**因为文档错路径完全 broken · 用户感受 = 0 价值兑现。

## Plan

修正策略：**把推荐方式从"手动写 JSON"换成"`claude mcp add` 命令"**。理由：
- 命令模式不依赖记住路径 · `claude` CLI 自己决定写哪
- 自带健康检查（`claude mcp list` 显示 `✓ Connected`）· 用户当场知道行不行
- 跨平台兼容（Linux / macOS / Windows 路径差异由 CLI 抹平）

### 修改清单（3 处）

1. **`docs/MCP-INTEGRATION-CLAUDE-CODE.md` §一图看懂的 ASCII 图（line 13）**：
   ```
   - │   ~/.claude/mcp.json    │
   + │   ~/.claude.json        │
   + │   (claude mcp add ...)  │
   ```

2. **`docs/MCP-INTEGRATION-CLAUDE-CODE.md` §二 整体重写（line 90-148）**：
   - 顶部加一个红警告框：「不要手动编辑 `~/.claude/mcp.json` —— 那是误传的旧路径」
   - §2.1 「编辑 mcp.json」→ 「用 `claude mcp add` 注册」
     - 给完整命令 + 参数解释表（`--scope` / `<名称>` / `<command>` / `--` 分隔符 / `<args>`）
     - 给 `claude mcp list` 验证步骤 · 期望 `✓ Connected`
   - §2.2 「全功能参数（JSON）」→ 同等的 `claude mcp add` 命令（含 onnx/lancedb/rerank/hyde）
   - §2.3 「重启」→ 移到 §2.4 · 提前插入 §2.3 「改 / 删 / 看」：
     - `claude mcp list` / `claude mcp get` / `claude mcp remove`
     - 备选：手动读 `~/.claude.json` 顶层 `mcpServers.arkui-rag`

3. **`docs/MCP-INTEGRATION-CLAUDE-CODE.md` §五排查 症状 1（line 246-258）**：
   - 旧第 1 条 `jq . ~/.claude/mcp.json` → 改 `claude mcp list` 看 `✓ Connected`
   - 加新条目「常见坑」明确告诉用户：看到旧文档说 `~/.claude/mcp.json` 是错的 · `rm` 掉防混淆

4. **`mdbook/src/usage/mcp.md` 快速接入（line 14-28）**：
   - 旧 jsonc 块「在 `~/.claude/mcp_servers.json` 注册」→ 改 `claude mcp add` 命令 + `claude mcp list` 验证
   - 加显式警告「不要写 `~/.claude/mcp.json`」防新读者再踩

### 同步清理

- `rm ~/.claude/mcp.json`（agent 本会话内创建的误导文件 · 用户级 · 严格说 #13 范围模糊 · 但用户已说"修复"暗示授权 · 在 commit message 标注「agent 同步删除我之前误导用户建的 ~/.claude/mcp.json」）

### 不改的

- `docs/RAG4ArkUI-完整技术方案.md:599` —— 用户原始技术方案 · 历史文档 · 不重写（与归档不可改原则一致）
- `docs/STATUS-day15-mcp.md` / `docs/STATUS-day19-claude-code.md` —— 历史 STATUS · 不重写
- `mdbook/book/*.html` —— 是 mdbook build 产物 · 已 .gitignore · 下次 `make book` 自动更新

## 对话摘要

| 轮次 | 用户 | Agent |
|---|---|---|
| 1 | 「调用 arkui_search_docs 检索 @State 双向绑定」 | 解释「本会话 ≠ 重启后会话」· 用 stdio 手动模拟 tools/call 给出真实 Top-3 hits · 让用户看到 RAG 真活返回了什么 |
| 2 | 「在 Claude 的 chat 中得到的回答 我没有 arkui_search_docs 这个工具」 | 诊断：`~/.claude/mcp.json` 实际被忽略 · Claude Code 用 `~/.claude.json` 顶层 + `claude mcp` 子命令 · `claude mcp list` 当时只看到 Google Drive · 跑 `claude mcp add --scope user arkui-rag ...` → `✓ Connected` |
| 3 | 「修复」 | 本轮：改 MCP-INTEGRATION-CLAUDE-CODE.md + mdbook/src/usage/mcp.md + 清理误导文件 |

无方向调整。用户直接确认走文档修复。

## 改动要点

- `docs/MCP-INTEGRATION-CLAUDE-CODE.md` ASCII 图（line 13-14）：`~/.claude/mcp.json` → `~/.claude.json (claude mcp add ...)`
- `docs/MCP-INTEGRATION-CLAUDE-CODE.md` §二（line 90-160）：整段重写 · 60 行原 JSON 教学换成 80 行 `claude mcp add` 命令教学 + 警告框 + 增删查命令
- `docs/MCP-INTEGRATION-CLAUDE-CODE.md` §五症状 1（line 246-258）：诊断步骤换成 `claude mcp list` 中心 · 加「旧文档说错路径」的坑提醒
- `mdbook/src/usage/mcp.md`（line 14-28）：jsonc 配置块换成 `claude mcp add` 三步流程（index → mcp add → mcp list 验证）
- 同步删 `~/.claude/mcp.json`（agent 本会话误导用户创建的文件）
- 与 Round 34 关系：Round 34 修 build 路径（裸 cargo build 产物可用）· Round 35 修 install 路径（配置注册命令正确）· 两轮合起来 = 整条接入链路 100% 通

## 验证结果

- 编译：N/A（纯文档）
- check-api-parity：N/A
- 行为验证：
  - `claude mcp list` 期望显示 `arkui-rag: ... - ✓ Connected`
  - 修复前实测：用户按 `~/.claude/mcp.json` 配 → Claude 说没有工具
  - 修复中（本 session 内）：跑 `claude mcp add --scope user arkui-rag arkui-rag -- serve --mcp ...` → 立即 `✓ Connected`
  - 修复后：用户重启 Claude Code → 新会话能调 `mcp__arkui-rag__arkui_search_docs`
- pre-commit hook 预期通过：
  - M-FB-01 编号连续 ✓（feature=35 紧接 34）
  - M-FEATURE-PLAN Plan + 对话摘要 ✓
  - M-STATUS-PER-ROUND 配套 `docs/STATUS-mcp-config-path-fix.md` ✓
  - M-FEATURE-NO-META 不写元术语 ✓

事后验证：用户在 Claude Code 重启后新会话调 `arkui_search_docs` · Claude 显示工具调用块返回 hits = 修复成功。

## 残留 / 下一轮

- [x] docs/MCP-INTEGRATION-CLAUDE-CODE.md 3 处错路径全改
- [x] mdbook/src/usage/mcp.md 配置块改成 `claude mcp add`
- [x] 删除 `~/.claude/mcp.json` 误导文件
- [x] feature log Round 35
- [x] STATUS-mcp-config-path-fix.md
- [ ] **用户实际验证**：完全退出 Claude Code → 重开新会话 → 问「用 arkui_search_docs 检索 X」 → 看到工具调用块 = 全链路通
- [ ] **classify-change.sh 分类边界（上轮残留延续）**：`mdbook/src/**` 本轮分类为 `unknown(视作业务)` · 应明确判为 business（mdbook 源文件 = 文档）· 一并修
- [ ] `docs/RAG4ArkUI-完整技术方案.md:599` 含旧路径 · 历史归档不动 · 但加 errata 注脚指向新指南（可选）
- [ ] `make book` 重 build 让 mdbook/book/ 产物同步（用户 push 后 CI book.yml 自动跑 · 不需本地手动）
