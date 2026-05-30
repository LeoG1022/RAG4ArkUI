# 38 — opencode-mcp

> 日期：2026-05-30
> 涉及代码：`scripts/install-binary.sh`（meta · 见 feedback/meta/19）· `docs/MCP-INTEGRATION-CLAUDE-CODE.md`
> 类型：新 client 集成（业务流程扩展 · 三端覆盖）

## 本轮目标

把 opencode（SST 开源 terminal agent · 已装 v1.15.10）加入 `make install` 自动配置的 MCP client 列表。从 2 端（Claude Code CLI + Claude Desktop GUI）扩到 3 端（+ opencode）。

承接 Round 37 install-binary.sh 7-step 架构 · 加 Step 8 复用既有「检测 + Python 安全合并 + 备份 + 验证」模式。

## Plan

### 设计：Step 8 复用 Step 7 模式

```bash
if command -v opencode; then
    if [[ -f ~/.config/opencode/opencode.json ]]; then
        # 备份 + Python 合并 mcp.arkui-rag
        # 验证 opencode mcp list | grep "✓ arkui-rag"
    elif [[ -d ~/.config/opencode ]]; then
        # 新建 opencode.json
    else
        echo "opencode config 目录不存在 · 跳过"
    fi
else
    echo "opencode 未装 · 跳过"
fi
```

### opencode 配置格式差异（关键）

WebFetch `opencode.ai/docs/mcp-servers` 确认：

| 维度 | Claude (CLI / Desktop) | opencode |
|---|---|---|
| 顶层节 | `mcpServers` | **`mcp`** |
| command 格式 | string + 独立 `args` 数组 | **数组**（命令名 + 所有 args 平铺）|
| 必填字段 | `command` | `command` + **`type: "local"`** |

Python 合并示例：

```python
cfg.setdefault('mcp', {})['arkui-rag'] = {
    'type': 'local',
    'command': [binary, 'serve', '--mcp',
                '--index-path', index_path, '--bm25', 'tantivy'],
    'enabled': True,
}
```

### 文档同步

`docs/MCP-INTEGRATION-CLAUDE-CODE.md` 「双端配置差异」表升级为「三端配置差异」· 加 opencode 行 + 单独 JSON 示例块强调差异化字段。

### 替代方案权衡

| 选项 | 优点 | 缺点 | 选 |
|---|---|---|---|
| A · 扩 install-binary.sh Step 8（本轮）| 一键覆盖三端 · 一致体验 | 脚本变长 ~40 行 | ✅ |
| B · 用户手动跑 `opencode mcp add` | 跟 Claude CLI 一致 | 该子命令 usage 未文档化 · 不可靠 | ❌ |
| C · 单独 `scripts/install-opencode.sh` | install-binary 不膨胀 | 用户得跑两条命令 · 维护两份合并逻辑 | ❌ |

选 A · install-binary.sh 单一职责契合（装 binary + 配三端 MCP）。

## 对话摘要

| 轮次 | 用户 | Agent |
|---|---|---|
| 1 | 「先做另一件事，MCP对接opencode」 | 探测 opencode 已装 v1.15.10 · 现有 opencode.json 仅含 plugin 节 |
| 2 | （观察 agent 进展）| WebFetch 官方 schema · Python 合并到 opencode.json · 手动 `opencode mcp list` 验证 ✓ connected |
| 3 | （隐式继续）| 沉淀到 install-binary.sh Step 8 + 文档化三端差异 + 归档 |

无方向调整 · 用户提需求 → agent 现场实施 + 沉淀。

## 改动要点

- `scripts/install-binary.sh` 末段加 Step 8（~40 行）：
  - `command -v opencode` 探测 · 没装跳过（友好降级）
  - `~/.config/opencode/opencode.json` 存在 → 备份 + Python 合并 + 验证
  - 不存在但目录在 → 新建 schema-完整 JSON
  - 目录都不在 → 跳过（用户首次启动 opencode 会自动创建）
- 完成提示加第 3 条「重启 opencode 如果用」
- `docs/MCP-INTEGRATION-CLAUDE-CODE.md` 「双端」→「三端」表 + opencode JSON 示例块（强调 `mcp` vs `mcpServers` / 数组 command / `type: "local"`）
- 与 Round 37 关系：37 = 装 binary + 配双端 Claude；38 = 加 opencode 第 3 端 · 同模式扩展

## 验证结果

- 编译：N/A（脚本 + 文档）
- 集成测试：`bash scripts/install-binary.sh` 8 step 全过 · Step 8 显示「✅ arkui-rag connected」
- 单独验证：`opencode mcp list` 输出 `● ✓ arkui-rag connected · /Users/leo/.local/bin/arkui-rag serve --mcp ...`
- 配置合并幂等：本轮跑了两次（手动 + 脚本）· 第二次覆盖第一次 · 保留 `$schema` + `plugin` 节不破坏

事后验证（用户）：退当前 opencode tui · 重跑 `opencode` · chat 内用 arkui_search_docs · 应返回 Top-K hits 跟 Claude 端一致。

## 残留 / 下一轮

- [x] install-binary.sh Step 8（opencode）
- [x] MCP-INTEGRATION 三端差异表 + opencode 示例
- [x] 双轨归档 + STATUS
- [ ] **用户验证**：重启 opencode tui · 在 chat 调 arkui_search_docs
- [ ] **Round 37 残留延续**：Makefile uninstall target 要覆盖三端
- [ ] **可选未来**：opencode mcp add 子命令 usage 官方文档完善后 · 切换到 CLI 命令路径（更标准 · 不依赖文件编辑）· 当前直接 JSON 合并够用
- [ ] **可选未来**：加 Cursor IDE MCP 支持作第 4 端（`.cursor/mcp.json` 项目级）
- [ ] **可选未来**：把 install-binary.sh 拆 modular（每个 client 一个 install function）· 加新 client 更轻量 · 当前 inline 第 4 个 client 时再重构
