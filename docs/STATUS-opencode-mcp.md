# STATUS — opencode-mcp

> 配套 feature log：`feedback/features/rag4arkui-core/38-2026-05-30-opencode-mcp.md`
> 配套 meta：`feedback/meta/19-2026-05-30-opencode-mcp.md`
> 日期：2026-05-30

---

## 当前状态

把 opencode（SST 开源 terminal agent）加入 `make install` 自动配置的 MCP client 列表。`install-binary.sh` 从 7 step 扩到 8 step · 覆盖三端：Claude Code CLI + Claude Desktop GUI + opencode。

本阶段交付：
- `scripts/install-binary.sh` 新增 Step 8 · 自动合并 `~/.config/opencode/opencode.json` · ~40 行
- `docs/MCP-INTEGRATION-CLAUDE-CODE.md` 「双端配置差异」表升级为「三端」· 加 opencode JSON 示例块
- 实测：`opencode mcp list` 显示 `● ✓ arkui-rag connected`

意义：用户 MCP client 选型多样化（Claude 系两端 + opencode 开源端）· arkui-rag 一次配 · 三端都通。下个用 opencode 的用户跑 `make install` 也自动覆盖 · 无需手动研究 opencode.json 格式差异。

## 输入契约

### `make install` 流程扩展

| Step | 配的 client | 配置文件 |
|---|---|---|
| 6 | Claude Code CLI | `~/.claude.json`（`claude mcp add`）|
| 7 | Claude Desktop GUI | `~/Library/Application Support/Claude/claude_desktop_config.json`（Python 合并）|
| **8（本轮新增）** | **opencode** | `~/.config/opencode/opencode.json`（Python 合并）|

### 配置格式差异（用户须知）

opencode 跟 Claude 不一样 · 文档化在 MCP-INTEGRATION 「三端配置差异」表：

| 维度 | Claude 系（CLI / Desktop）| opencode |
|---|---|---|
| 顶层节名 | `mcpServers` | `mcp` |
| command 字段 | string | **数组**（含命令名 + 所有 args 平铺）|
| 必填 | `command` | `command` + `type: "local"` |
| 可选 | `args` / `env` | `enabled` / `environment` / `timeout` |

`make install` 自动处理这些差异 · 用户不用记。

### 不变项

- arkui-rag binary 自身（`~/.local/bin/arkui-rag` · Round 37 装的不变）
- `arkui-rag serve --mcp` CLI 接口
- 4 个 MCP 工具签名（arkui_search_docs / arkui_search_code / arkui_migrate_snippet / arkui_validate_api）
- 索引文件位置（`~/.arkui-rag/index.json`）

## 输出契约

### `make install` 输出新加段

```
═══ Step 8 · opencode MCP ═══
  ✅ 合并到 /Users/me/.config/opencode/opencode.json
  备份: /Users/me/.config/opencode/opencode.json.bak.YYYYMMDD-HHMMSS
  验证（opencode mcp list）:
  ✅ arkui-rag connected
```

### 完成提示加第 3 条

```
下一步：
  1. 重启 Claude Code CLI: ...
  2. 重启 Claude Desktop: ...
  3. 重启 opencode（如果用）: 退当前 opencode tui · 重跑 opencode
  4. 新 chat 测试：用 arkui_search_docs 检索 X
```

### 友好降级路径

| opencode 状态 | Step 8 行为 |
|---|---|
| 装了 + 有配置文件 | 备份 + 合并 + 验证 |
| 装了 + 仅有目录 | 新建 schema-完整 opencode.json |
| 装了 + 无配置目录 | 跳过 + 提示「用户首次启动 opencode 后会自动创建」|
| 没装 | 跳过 + 提示「opencode 未装」|

## 验证手段

### 用户手动

```bash
# 一键完整流程
make release-local && make install

# 验证三端
claude mcp list | grep arkui          # Claude CLI: ✓ Connected
opencode mcp list | grep arkui        # opencode: ✓ arkui-rag connected
# Claude Desktop GUI: 重启后 chat 调用工具

# 重启三端 → 新 chat 测试 arkui_search_docs
```

### 自动化

```bash
bash scripts/install-binary.sh
# 期望 Step 8 输出: ✅ arkui-rag connected
```

CI 残留（同 Round 37）：未加 install smoke test · 建议作 macOS-only matrix step。

## 与上一阶段的关联性

| Round | Slug | 解决 |
|---|---|---|
| 37 | install-script | 装 binary + 配 Claude 双端 |
| **38（本轮）**| opencode-mcp | + opencode 第 3 端 |

层次：Round 37 立起 7-step 框架 · Round 38 加 1 step · 不重构 · 复用既有「检测 + Python 安全合并 + 验证」pattern。

兼容性：
- 完全向后兼容：opencode 没装 / 没启用 → Step 8 跳过 · 不影响其它步骤
- 旧 opencode.json 配置：Python 安全合并 · 保留 `$schema` + `plugin` 等节
- 用户之前手动 `opencode mcp add` 的配置 · 脚本覆盖时备份 · 可还原

破坏性变更：无。

性能：Step 8 增加 ~200ms（cp + python json load/dump + opencode mcp list health check）· 整体 `make install` 仍 < 3 秒（不含 release build）。

## 完成度 / 下一阶段

### 达成度

| 项 | 状态 |
|---|---|
| Step 8 加 opencode 配置 | ✅ |
| 自检端到端跑通 | ✅ |
| 三端差异表 + opencode JSON 示例 | ✅ |
| 双轨归档 + STATUS | ✅ |
| 用户重启 opencode 验证 | ⏳ |

### 下一阶段建议

立即（用户做）：
1. 退当前 opencode tui · 重跑 `opencode`
2. chat 内问：「用 arkui_search_docs 检索 @State 双向绑定」
3. 期望返回 Top-K markdown（跟 Claude 端一致）

短期（agent 做 · 1-2 round）：
- Round 37 残留的 `Makefile uninstall` target 实施时要覆盖三端（Claude CLI + Desktop + opencode）
- Round 37 残留的 `release-local.sh` 加 codesign 仍 pending
- 文件名改：`docs/MCP-INTEGRATION-CLAUDE-CODE.md` → `docs/MCP-INTEGRATION.md`（现在覆盖三端 · 名字不准）· 留作可选 · 改名要更新所有引用

中期：
- 加 Cursor 作第 4 端（`.cursor/mcp.json` 项目级 · 跟 user level 不同）
- `install-binary.sh` 拆 modular：每个 client 一个 install function · 当前 inline 第 4 个 client 时再重构
- opencode 官方文档完善 `opencode mcp add` 子命令 usage 后 · 切到 CLI 路径（不依赖直接 JSON 编辑）

长期：
- 1.0 release 后看实际有多少用户用 opencode · 决定是否加 opencode-specific 工具（如果 opencode 有 Claude 没的 MCP 能力扩展）· 当前三端统一 schema
- 文档可考虑独立「MCP 接入指南」总篇 · 一篇 per client · 当前一篇 dump 三端 · 长期可能不便维护
