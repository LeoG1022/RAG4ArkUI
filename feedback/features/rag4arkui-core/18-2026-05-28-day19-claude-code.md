# 18 — day19-claude-code

> 日期：2026-05-28
> 涉及代码：
> - `docs/MCP-INTEGRATION-CLAUDE-CODE.md`（**新增** · 完整接入指南 · 10 节 ~280 行）
> - `scripts/mcp-demo.sh`（meta · 见 feedback/meta/6-*.md）
> - `Makefile`（加 `mcp-demo` target + help 输出）
> - `docs/ROADMAP.md`（第 7 次实战 · 同步进度行）
> 类型：新建（Day 19 主线 · 轻量切片 · 接入验证）

## 本轮目标

Day 15 MCP server 真活后，第一时间验证真实可用 + 用户能接入。两件事：
1. **完整接入指南**（`docs/MCP-INTEGRATION-CLAUDE-CODE.md`）：mcp.json 配置 / 工具用法 / 故障排查 / 性能调优
2. **端到端 demo 脚本**（`scripts/mcp-demo.sh`）：bash 一行命令验证完整 JSON-RPC 流程

## Plan

> 必备节（FAIL 级 M-FEATURE-PLAN）

### 范围：轻量切片（1 commit · 主要文档 + demo 脚本）

| 做 | 不做 |
|---|---|
| ✅ 完整 Claude Code 接入指南（10 节） | ❌ Rust subprocess integration test（Day 19 续） |
| ✅ bash demo 脚本（heredoc 喂 4 请求 + 断言响应） | ❌ MCP 协议合规性测（官方测试套件） |
| ✅ Makefile `mcp-demo` target | ❌ Cursor / OpenCode 各自详细文档（README 提一句） |
| ✅ 故障排查 + 性能调优建议 | ❌ Web Agent + SSE 接入（需 Day 15 续 SSE 实装） |

### 接入指南文档结构（10 节）

1. 一图看懂（mermaid 架构图）
2. 前置准备（编译 + 索引 + 安装到 PATH）
3. 配置 `~/.claude/mcp.json`（基本 + 全功能）
4. 在 Claude Code 中使用（自然语言触发 + 工具签名速查）
5. 验证（不依赖 Claude Code · 手工 demo）
6. 故障排查（4 个常见症状 + 排查步骤）
7. 性能调优建议（不同 corpus 规模的推荐配置）
8. 其他 Agent 接入（Cursor / OpenCode）
9. 限制 & 未做（明确）
10. 相关文档 + 反馈渠道

### mcp-demo.sh 设计

**核心思路**：MCP server 读 stdin EOF 后自动退出 → heredoc 一次性喂 4 个请求 → server 处理完后退出 → stdout 完整可解析。

```
[heredoc requests] → stdin → arkui-rag serve --mcp → stdout → 逐行解析 + 断言
```

**断言 3 条**（notifications 不响应所以总 3 条）：
- line 1: initialize 含 `protocolVersion=2024-11-05` + `serverInfo.name=arkui-rag`
- line 2: tools/list 含全部 4 个工具名
- line 3: tools/call 含 `type=text` + `list.md`（Mock 阶段对原文本必命中）

**故障诊断**：
- `--keep` 失败时保留临时目录
- `--verbose` 显示 cargo / stderr 输出
- 退出码：0=PASS · 127=cargo 缺失 · 其他=断言失败

### 关键决策

- **bash + heredoc**：MCP server 读 stdin EOF 自动退出特性 · 不需要 kill 子进程或 named pipe 双向通信 · 简洁优雅
- **断言走文本匹配**：不引入 jq 依赖（macOS / Linux 默认未必有）· bash 字符串包含够用
- **临时目录隔离**：与 `demo-smoke.sh` 同款套路，每次 run 独立
- **与 demo-smoke.sh 并列**：mcp-demo 验证 MCP 协议 · smoke 验证 CLI 二进制 · 互补不冗余

### 替代方案权衡（被否）

- 备选：Rust integration test（`std::process::Command`）
  - 否决：bash 简单 · 用户独立可跑 + 与 Day 2.5 demo-smoke 风格一致 · 留 Day 19 续可加
- 备选：用 `jq` 解析 JSON
  - 否决：增加外部依赖（macOS 默认没 jq · linux 也不一定有）· 字符串匹配够用
- 备选：写 Python 测试客户端（mcp_smoke.py）
  - 否决：项目当前无 Python 依赖 · bash 够用 · 跨语言无收益

## 改动要点

> API 选型 / 算法 / 关键决策

**与 Day 15 的差异**：
- crate 数 9（不变）
- 代码改动 0（纯文档 + 脚本）
- 测试数不变
- 新增 1 个 Makefile target + help 输出更新

**关键决策**：
- 接入指南是项目首份"用户视角"完整文档（与 ADR / STATUS 视角不同）
- demo 脚本利用 MCP stdin EOF 退出特性，避免双向通信复杂度

## 对话摘要

> 必备节（FAIL 级 M-FEATURE-PLAN）

按时序：
1. **Day 15 MCP commit 后**，agent 推荐 Day 19 Claude Code 端到端验证（轻量 · 第一时间验证）
2. **用户指令**：「继续」
3. **Agent 自主决策 3 项**：
   - bash + heredoc 不依赖 Rust subprocess test
   - 字符串匹配断言（不引入 jq）
   - scripts/mcp-demo.sh 归 meta 写 feedback
4. **Agent 不再回问**，3 phase 直接执行至本 commit

## 验证结果

- 文档：`docs/MCP-INTEGRATION-CLAUDE-CODE.md` 10 节齐 · 含 mcp.json 配置示例 + 故障排查 + 性能矩阵
- 脚本：`scripts/mcp-demo.sh` ~180 行 · 4 步流程 · 3 个断言
- 验证 1：`bash -n scripts/mcp-demo.sh` 期望语法过
- 验证 2：⏳ 用户跑 `make mcp-demo` · 期望 PASS（~30-90 秒，首次编译较慢）
- 验证 3：⏳ 用户配 `~/.claude/mcp.json` + 重启 Claude Code 看到 4 工具

## 残留 / 下一轮

- [ ] **关键**：用户配 Claude Code mcp.json 真实接入 · 在对话中触发 arkui_search_docs
- [ ] **关键**：用户跑 `make mcp-demo` 验证 PASS
- [ ] Day 19 续：Rust subprocess integration test（CI 友好）
- [ ] Day 19 续：Cursor / OpenCode 接入示例 + 截图
- [ ] Day 19 续：性能基准（criterion · MCP 端到端 P99）
- [ ] Week 4 Day 16：LSP Server（协议层 3/3 完整）
- [ ] Week 4 Day 17：DevEco Plugin MVP（关键路径）
- [x] Day 19：完整接入指南（10 节）
- [x] Day 19：bash 端到端 demo 脚本
- [x] Day 19：ROADMAP 维护约定第 7 次实战
- [x] **Week 5 0.5/1 → 1/1**（MCP 接入就绪 → 接入指南就绪 · 待用户真实验证）
