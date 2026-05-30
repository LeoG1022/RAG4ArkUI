# 19 — opencode-mcp

> 日期：2026-05-30
> 触发：用户跳出 Round 37 收尾 D1-D4 计划 · 直接提诉求「MCP 对接 opencode」· 扩展 install-binary.sh Step 8
> 类型：Skill 变更（install-binary.sh 加 opencode 支持）· 流程扩展（三端客户端）

---

## 用户提出的要求

原文：

> 先做另一件事，MCP对接opencode

承接 Round 37 末尾「待你点 D1-D4」之后 · 用户主动跳到新需求：把 opencode (SST 开源 terminal agent) 也接入 arkui-rag MCP server。

## Agent 给出的修改建议

opencode v1.15.10 已装在用户机器（`/opt/homebrew/bin/opencode`）· 现有 `~/.config/opencode/opencode.json` 仅含 `plugin` 节。

opencode MCP 配置格式跟 Claude 不同（WebFetch opencode.ai/docs/mcp-servers 确认）：

| 维度 | Claude（CLI / Desktop）| opencode |
|---|---|---|
| 顶层节名 | `mcpServers` | `mcp` |
| `command` 字段 | string（CLI 用 args 数组分开）| **数组**（command + args 平铺）|
| 必填字段 | `command` | `command` + `type: "local"` |
| 可选字段 | `args` / `env` | `enabled` / `environment` / `timeout` |

实施方案：

1. **手动验证可行性**（已完成）—— Python 合并到 opencode.json · `opencode mcp list` 显示 `✓ arkui-rag connected`
2. **沉淀到 install-binary.sh Step 8** —— 复用 Step 7 (Claude Desktop) 同样模式：备份 + Python 安全合并 + 验证
3. **文档化** —— `docs/MCP-INTEGRATION-CLAUDE-CODE.md` 「双端配置差异」表升级为「三端配置差异」· 加 opencode 行 + 单独 opencode JSON 示例块（差异化字段强调）

替代方案权衡：

| 选项 | 优点 | 缺点 | 选 |
|---|---|---|---|
| A · 扩 install-binary.sh Step 8（本轮）| 自动化 · 一键覆盖三端 · 一致体验 | install 脚本变长 | ✅ |
| B · 用户手动跑 `opencode mcp add` | 跟 Claude CLI 一致 | opencode 该子命令实际语法未文档化（`--help` 显示存在但 usage 未列）· 不可靠 | ❌ |
| C · 单独 `scripts/install-opencode.sh` | install-binary 不膨胀 | 用户得跑两条命令 · 体验差 · 维护两份合并逻辑 | ❌ |

选 A · install-binary.sh 当下是「装 arkui-rag binary + 自动配三端 MCP」单一职责的工具 · Step 8 完全契合。

## 多轮互动

| 轮次 | 用户 | Agent |
|---|---|---|
| 1 | 「先做另一件事，MCP对接opencode」 | 探测 opencode 装了 + 配置文件路径 + WebFetch 官方 schema · 手动 Python 合并 · `opencode mcp list` ✓ connected |
| 2 | （隐式确认 · 没说不要继续）| 扩 install-binary.sh Step 8 自动化 + 文档化三端差异 + 双轨归档 |

无方向调整 · 用户提需求 → agent 现场实施 + 沉淀。

## 实际改动

- 接口变化：无
- 规则变化：install-binary.sh 从 7 step（仅两端 Claude）扩 8 step（加 opencode）· 推荐流程从 「Claude Code + Claude Desktop」改为「Claude Code + Claude Desktop + opencode」三端
- 文件变化：
  - **修改** `scripts/install-binary.sh` —— 加 Step 8 · ~40 行 · 检测 opencode 命令 + 配置文件路径 + Python 合并 + 验证 + 完成提示加重启 opencode 一条
  - **修改** `docs/MCP-INTEGRATION-CLAUDE-CODE.md` —— 「双端差异」表升级三端 · 加 opencode JSON 示例块强调 `command` 数组格式
- 配置变化：用户机器实际 `~/.config/opencode/opencode.json` 加 `mcp.arkui-rag` 节（agent 已直接写入）

## 执行生效后总结

### 实际产出

| 文件 | 改动 |
|---|---|
| `scripts/install-binary.sh` | +40 行（Step 8 opencode 自动配 · 完成提示扩三端 + 1）|
| `docs/MCP-INTEGRATION-CLAUDE-CODE.md` | +20 行（三端差异表 + opencode JSON 示例 + 验证）|
| `feedback/meta/19-...` | 本归档 |
| `feedback/features/.../38-...` | feature log |
| `docs/STATUS-opencode-mcp.md` | STATUS |

### 前后对比

| 维度 | Before（Round 37） | After（Round 38） |
|---|---|---|
| 支持 MCP client 数 | 2（Claude Code CLI + Claude Desktop GUI） | **3**（+ opencode）|
| install-binary.sh step 数 | 7 | 8 |
| MCP-INTEGRATION 差异表 | 双端 | 三端 |
| opencode 用户接入方式 | 没文档 · 用户得自己研究 | `make install` 自动包含 |

### 实测验证

- `bash scripts/install-binary.sh` 端到端 ✓ Step 8 自检显示「✅ arkui-rag connected」
- 手动 `opencode mcp list` 单独验证 ✓ `● ✓ arkui-rag connected`
- 配置文件合并幂等：本轮跑了两次（手动 + 脚本）· 第二次覆盖第一次 · 不破坏既有 `plugin` 节

事后验证（用户）：重启 opencode tui · 在 chat 内使用 arkui_search_docs 工具应当 work。

### 残留 / 下一轮处理

<!-- 用 - [ ] 标记未解决项，- [x] 标记已解决项；若无残留写"无" -->
- [x] install-binary.sh Step 8 加 opencode
- [x] MCP-INTEGRATION 文档加三端差异 + opencode JSON 示例
- [x] 双轨归档 + STATUS
- [ ] **用户验证 opencode 内调 arkui_search_docs** · 退当前 opencode tui · 重启 · 试 「用 arkui_search_docs 检索 X」
- [ ] Makefile uninstall target（Round 37 残留延续）· 反向操作要覆盖三端
- [ ] 长期：考虑 install-binary.sh 拆 modular（每个 client 一个 install function）· 让加新 client（如 Cursor）更轻量 · 当前是 inline · 第 4 个 client 时再重构
