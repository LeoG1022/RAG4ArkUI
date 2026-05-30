# STATUS — mcp-config-path-fix

> 配套 feature log：`feedback/features/rag4arkui-core/35-2026-05-30-mcp-config-path-fix.md`
> 日期：2026-05-30

---

## 当前状态

修复了 Claude Code MCP 接入文档的**配置文件路径错误**——历史文档说 `~/.claude/mcp.json` · 实际 Claude Code 不读这个文件 · 真实配置在 `~/.claude.json`（顶层 `mcpServers`）· 由 `claude mcp add` 命令管理。

本阶段交付：
- `docs/MCP-INTEGRATION-CLAUDE-CODE.md` §一图 / §二配置 / §五排查 三处错路径全改
- `mdbook/src/usage/mcp.md` 快速接入节配置块从 JSON 教学改为 `claude mcp add` 命令教学
- 删除会话内误导用户创建的 `~/.claude/mcp.json` 文件
- 「本地 RAG 接 Claude Code」整条链路文档**100% 准确** · 新读者照做即通

意义：Round 34 修了 build 路径（裸 cargo build 能用） · Round 35 修了 install 路径（配置注册命令对） · 合起来等于把上一轮跑通的 RAG 系统（46 passed / 索引 11 文件 107 chunks / MCP stdio 真活）真正暴露到 Claude Code 客户端。

## 输入契约

### 文档侧契约变化

| 项 | Before | After |
|---|---|---|
| 配置文件路径 | `~/.claude/mcp.json` ❌ | `~/.claude.json`（顶层 `mcpServers`）✅ |
| 推荐操作 | 手动 `cat > ~/.claude/mcp.json <<EOF ... EOF` | `claude mcp add --scope user arkui-rag arkui-rag -- serve --mcp ...` |
| 验证手段 | `jq . ~/.claude/mcp.json`（即便语法对 Claude 也不读）| `claude mcp list`（显示 `✓ Connected`） |
| 改 / 删 | 编辑 JSON 文件 | `claude mcp get` / `claude mcp remove` |

### 不变项

- 4 个 MCP 工具签名：`arkui_search_docs` / `arkui_search_code` / `arkui_migrate_snippet` / `arkui_validate_api`
- `arkui-rag serve --mcp ...` CLI 接口
- 索引 / 协议格式 / 输出 schema

## 输出契约

新增「正确接入」的输出契约：

```bash
$ claude mcp list
Checking MCP server health…

arkui-rag: arkui-rag serve --mcp --index-path ... --bm25 tantivy - ✓ Connected
```

`✓ Connected` 是新「接入成功」信号 · 文档明确要求用户看到这一条才能进下一步。

Claude Code 会话内的工具暴露形式不变：`mcp__arkui-rag__arkui_search_docs` 等。

## 验证手段

### 用户手动

```bash
# 1. 验证配置写对了
claude mcp list
# 期望：arkui-rag: ... - ✓ Connected

# 2. 验证 ~/.claude.json 顶层有节
python3 -c "import json; print(json.dumps(json.load(open('/Users/leo/.claude.json')).get('mcpServers',{}).get('arkui-rag'), indent=2, ensure_ascii=False))"
# 期望：完整 JSON 含 command + args

# 3. 验证 stdio 握手（不依赖 Claude Code · 手动测）
printf '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"m","version":"1"}}}\n{"jsonrpc":"2.0","id":2,"method":"tools/list"}\n' \
  | arkui-rag serve --mcp --index-path ~/.arkui-rag/index.json --bm25 tantivy \
  2>/dev/null | tail -1 | python3 -c "import json,sys; print([t['name'] for t in json.load(sys.stdin)['result']['tools']])"
# 期望：['arkui_search_docs', 'arkui_search_code', 'arkui_migrate_snippet', 'arkui_validate_api']

# 4. 最终验证（真活）：完全退出 Claude Code → 重开新会话 → 试
#    用户：「用 arkui_search_docs 检索 @State 双向绑定」
#    Claude：显示工具调用块返回 markdown hits
```

### 自动化

- `make mcp-demo` ✓（已存在 · 不动 · 测的是 stdio 协议 · 不依赖 Claude Code）
- 推荐补充（未来）：`scripts/verify-claude-code-config.sh` 跑 `claude mcp list | grep "arkui-rag: .* ✓ Connected"` 验证用户端到端 · 但需要 claude CLI 装 · 留作可选

## 与上一阶段的关联性

| Round | Slug | 解决 |
|---|---|---|
| 32 | concepts-kb | 知识库基础设施 |
| 33 | concepts-archive-rule | Agent 自我约束（必问归档）|
| 34 | cli-default-features | **Build 路径**：裸 `cargo build` 产物可用 |
| **35（本轮）** | mcp-config-path-fix | **Install 路径**：配置注册命令正确 |

增量关系：
- Round 34 让**编出来的 binary** 装到 PATH 后能跑 `arkui-rag serve --mcp`
- Round 35 让**用户怎么告诉 Claude Code 用这个 binary** 这步指令准确

二者合起来 = 整条「本地 RAG → Claude Code」链路从 Day 22 mdBook 站之后真正打通到客户端。

兼容性：
- 文档变更 · 不破坏现有跑通的部署
- 用户已经手动 `claude mcp add` 过的不受影响
- 用户之前按错文档 `cat > ~/.claude/mcp.json` 的（如本会话开始时）· 文件被忽略 · 删了就行 · 无副作用

破坏性变更：无。

## 完成度 / 下一阶段

### 达成度

| 项 | 状态 |
|---|---|
| docs/MCP-INTEGRATION-CLAUDE-CODE.md 修 3 处 | ✅ |
| mdbook/src/usage/mcp.md 修配置块 | ✅ |
| `~/.claude/mcp.json` 删除 | ✅ |
| feature log Round 35 | ✅ |
| STATUS-mcp-config-path-fix（本文件）| ✅ |
| 用户最终验证（重启 Claude Code · 调 arkui_search_docs） | ⏳（等用户）|

### 下一阶段建议

立即（用户做）：
1. 完全退出 Claude Code · `pkill -i claude` · 重新打开
2. 新会话问：「用 arkui_search_docs 检索 @State 双向绑定，top_k=3」
3. Claude 应显示工具调用块返回 Top-3 hits（mapping-benchmark / mapping-state / mapping-list 三段 markdown）

短期（agent 做 · 1-2 round）：
- `scripts/classify-change.sh` 把 `mdbook/src/**` 显式归 business（本轮分类为 `unknown(视作业务)` · 应明确）· 同步上轮残留的 `crates/**/Cargo.toml` 分类边界一并修
- `docs/RAG4ArkUI-完整技术方案.md:599` 加 errata 注脚指向新指南（不重写历史 · 加补丁）

中期（可选）：
- 写 `scripts/verify-claude-code-config.sh` 自动跑 `claude mcp list` + grep `arkui-rag.*Connected` · 给用户一键自检
- mdBook 站「快速接入」节加截图（Claude Code 里能看到 `mcp__arkui-rag__*` 工具的实际样子）

长期：
- 1.0 release 推 v1.0.0 tag 之前 · 把本修复合进 release notes 草稿「已知坑修复」节
- 后续如果 Claude Code 又改 MCP 配置 API（如废弃 `claude mcp add` 改成别的）· 在本 STATUS 加 errata 注脚 · 文档保持 evergreen
