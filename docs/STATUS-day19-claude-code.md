# STATUS · Day 19 · Claude Code 接入验证

> 日期：2026-05-28
> 对应 commit：[本 commit · Day 19 接入验证]
> 对应 feature log：[`feedback/features/rag4arkui-core/18-2026-05-28-day19-claude-code.md`](../feedback/features/rag4arkui-core/18-2026-05-28-day19-claude-code.md)
> 上一阶段：[`STATUS-day15-mcp.md`](STATUS-day15-mcp.md)
> 下一阶段：`STATUS-day16-lsp.md` 或 `STATUS-day17-deveco.md`

> 🎯 **里程碑**：**Claude Code 接入完整就绪** · 用户文档 + demo 脚本双轨齐备。Week 5 接入路径 0.5/1 → 1/1。

---

## 当前状态

| 模块 | 变化 |
|---|---|
| `docs/MCP-INTEGRATION-CLAUDE-CODE.md` | **新增** · 用户视角完整指南 · 10 节 ~280 行 |
| `scripts/mcp-demo.sh` | **新增** · bash 端到端冒烟脚本（heredoc + EOF · 4 请求 + 3 断言） |
| `Makefile` | + `mcp-demo` target + help 输出 |
| `docs/ROADMAP.md` | 第 7 次实战 · 进度行同步 |
| 代码 | **0 改动**（纯文档 + 脚本） |
| 测试 | 数量不变（默认 49 · 全 feature ~80） |

---

## 输入契约

### 用户视角（接入指南覆盖）

```bash
# 1. 编译
make build-mcp                              # mock 模式（快）
make build-full                             # 全功能（onnx/tantivy/lancedb/http/mcp）

# 2. 建索引（投放 corpus → 一次性）
cargo run --features mcp -p arkui-rag-cli -- index --source corpus

# 3. 配置 ~/.claude/mcp.json
{
  "mcpServers": {
    "arkui-rag": {
      "command": "arkui-rag",
      "args": ["serve", "--mcp", "--index-path", "/abs/path/index.json"]
    }
  }
}

# 4. 重启 Claude Code · 工具自动可用
# 5. 自然语言触发：
#    "ArkUI-X 怎么做下拉刷新？"
#    → Claude 调 arkui_search_code → 返回示例
```

### 端到端验证（mcp-demo.sh）

```bash
make mcp-demo               # 默认（mock + memory）
bash scripts/mcp-demo.sh --keep       # 失败保留临时目录
bash scripts/mcp-demo.sh --verbose    # 显示 cargo + stderr
```

**4 步流程**：
1. 临时目录 + 投放 2 份 markdown + 建索引
2. 构造 4 个 JSON-RPC 请求
3. 启动 server · stdin 喂请求 · stdout 收响应
4. 解析 3 条响应（notifications 不响应）+ 3 个断言

---

## 输出契约

### `make mcp-demo` 预期输出

```
═══ [1/4] 准备 corpus + 建索引 ═══
  ✅ 索引就绪：/tmp/rag-mcp-demo-12345/index.json (35421 bytes)

═══ [2/4] 构造 4 个 JSON-RPC 请求 ═══
  ✅ 4 个请求就绪（initialize · notifications/initialized · tools/list · tools/call）

═══ [3/4] 启动 MCP server，喂请求，收响应 ═══

═══ [4/4] 解析响应 + 断言 ═══
  ✅ initialize: protocolVersion=2024-11-05, serverInfo.name=arkui-rag
  ✅ tools/list: 4 个工具齐全（search_docs/search_code/migrate_snippet/validate_api）
  ✅ tools/call: 返回 markdown 文本 + 命中 list.md

🎉 Day 19 MCP 端到端演示 PASS

下一步：
  1. 看 docs/MCP-INTEGRATION-CLAUDE-CODE.md 完整接入指南
  2. 配置 ~/.claude/mcp.json 接 Claude Code 体验
  3. 真实 corpus 用 --features full 启用 ONNX + LanceDB + Tantivy
```

### 接入指南内容速查

| 节 | 内容 |
|---|---|
| § 1 | mermaid 架构图（Claude Code → MCP server → AppState → corpus） |
| § 2 | 前置准备（编译 / 建索引 / 安装到 PATH） |
| § 3 | `~/.claude/mcp.json` 配置（基本 + 全功能） |
| § 4 | 在 Claude Code 中使用（自然语言 + 工具签名速查） |
| § 5 | 手工验证（不依赖 Claude Code · mcp-demo + 手工 JSON-RPC） |
| § 6 | 故障排查（4 个常见症状 + 排查步骤） |
| § 7 | 性能调优建议（按 corpus 规模推荐配置） |
| § 8 | 其他 Agent 接入（Cursor / OpenCode） |
| § 9 | 限制 & 未做（明确）|
| § 10 | 相关文档 + 反馈 |

---

## 验证手段

### 用户手动

```bash
# 1. 编译 + 单测
make check-mcp
cd crates && cargo test -p arkui-rag-server --features mcp     # 6 单测

# 2. 端到端 bash 演示
make mcp-demo
# 预期：30-90 秒（首次编译较慢，后续秒级）· 3 断言全过

# 3. Claude Code 真实接入
# 配 ~/.claude/mcp.json → 重启 Claude Code
# 对话："ArkUI-X 怎么做下拉刷新？"
# 预期：Claude 自动调 arkui_search_code · 返回基于 corpus 的代码示例

# 4. 故障诊断
bash scripts/mcp-demo.sh --keep --verbose     # 失败保留 + 详细输出
```

### 自动化

| 手段 | 范围 | 状态 |
|---|---|---|
| mcp.rs 单测 × 6 | 协议层验证（Day 15 已有） | ✅ feature gated |
| `mcp-demo.sh` | CLI subprocess + stdio 喂请求 + 断言响应 | ✅ Day 19 新增 |
| **M-STATUS-PER-ROUND** | Round 18 + STATUS-day19 配套 | ✅ |
| **ROADMAP 维护约定（第 7 次实战）** | 进度行同步 | ✅ |
| 文档链接完整性 | M-LINK-DEAD 校验 | ✅ 默认 |

### 暂未自动化（明确缺口）

- ❌ Rust subprocess integration test（CI 友好版 · Day 19 续）
- ❌ Claude Code 端到端自动化测（需 fork Claude Code 子进程）
- ❌ Cursor / OpenCode 真实接入截图（README 提一句）
- ❌ MCP 协议合规性测（与官方测试套件对比）
- ❌ 性能基准（criterion · MCP P99）

---

## 与上一阶段（STATUS-day15-mcp）的关联性

### 增量

| 维度 | Day 15 完成时 | 本轮（Day 19）后 |
|---|---|---|
| Crate 数 | 9 | 不变 |
| 代码 | mcp.rs ~380 行 | **不变**（纯文档 + 脚本） |
| 用户文档 | 仅 STATUS-day15 内嵌示例 | + **完整接入指南**（10 节） |
| 端到端验证手段 | 仅 6 单测（feature gated） | + **bash demo 脚本**（subprocess） |
| Makefile target 数 | 多个 | + `mcp-demo` |

### 与 §4.4 协议层完整对齐进度

| 接口 | 状态 |
|---|---|
| MCP Server (stdio) 实装 | ✅ Day 15 |
| MCP Server (stdio) **用户接入指南** | **✅ Day 19（本轮）** |
| MCP Server (SSE) | ⏳ Day 15 续 |
| HTTP REST API | ✅ Day 14 |
| LSP Custom Commands | ⏳ Day 16 |

Claude Code 真实接入路径**用户文档侧补齐** → 协议层 from 实装到用户能用，闭环完成。

### 兼容性

- ✅ 无破坏性变更（纯文档 + 脚本）
- ✅ 与 Day 2.5 demo-smoke.sh 风格一致（同 Makefile 命名风格 + 同 `--keep` / `--verbose` 选项）
- ✅ 不引入新依赖（不用 jq · 不用 Python · 仅 bash + cargo）

---

## 完成度 / 下一阶段

### Day 19 完成度

| 项 | 状态 |
|---|---|
| 完整接入指南（10 节） | ✅ |
| bash 端到端 demo 脚本 | ✅ |
| Makefile `mcp-demo` target | ✅ |
| ROADMAP 维护约定第 7 次实战 | ✅ |
| Rust subprocess integration test | ⏳ Day 19 续 |
| Cursor / OpenCode 真实接入截图 | ⏳ Day 19 续 |
| 性能基准（criterion） | ⏳ Day 19 续 |

### 6 周路线图达成度更新

| 章节 | 状态 |
|---|---|
| Week 1: Rust 骨架 + tree-sitter + LanceDB + Tantivy + BGE-M3 | **7/7** ✅ |
| Week 2: 混合检索 + Reranker + HyDE + 评估集 | **4/4** ✅ |
| Week 3: HTTP + MCP + CLI | **3/3** ✅ |
| Week 4: IDE 插件（DevEco/IntelliJ） | **0/2** ⏳ |
| Week 5: Claude Code 接入 | **1/1** ✅（本轮 · 接入指南就绪 · 待真实验证）|
| Week 6: 发布 + 文档站 | **1/4** ✅（评估报告 ✓ · 跨平台二进制 / corpus 分发 / 文档站 ⏳） |

**总完成度估算：~70%**

### 下一阶段建议（按推荐优先级）

| 候选 | 价值 | 工作量 |
|---|---|---|
| 🟢 **Day 16 LSP Server**（协议层 3/3 完整） | tower-lsp · IDE 内联补全 + diagnostic · 协议层最后一项 | 2-3 commit |
| 🟢 Day 17 DevEco Plugin MVP（关键路径 · §4.3 主战场） | IDE 集成 · 完整业务闭环 | 5+ commit · 大工程 |
| 🟢 Day 18 VSCode Extension | 跨编辑器覆盖 | 3+ commit |
| 🟡 Day 8 tantivy-jieba | 中文 BM25 精度 | 0.5 commit |
| 🟡 Day 19 续 | Rust subprocess test + Cursor 截图 + 性能基准 | 1-2 commit |
| 🟡 Day 14/15 续 | POST /index 真活 + MCP SSE + resources/prompts | 1-2 commit |

**Agent 推荐**：**Day 16 LSP Server**。理由：
1. 协议层 3/3 完整收尾，与 Day 14/15 共享 AppState 抽象 · 工作量复用度高
2. 完成后 Week 3 + Week 4 协议层全部就位（HTTP + MCP + LSP）· 之后专注 IDE 层
3. tower-lsp 是 Rust 生态成熟方案 · 风险低
4. 工作量适中（2-3 commit）· 比 Day 17 DevEco（5+ commit · 大工程）更易完成

**备选**：直接跳到 **Day 17 DevEco Plugin**（关键路径主战场，但工作量大 · 适合连续多日投入）。

### 重要的"非完成"项

- ❌ Rust subprocess integration test（bash demo 已覆盖手动场景 · CI 时缺）
- ❌ Cursor / OpenCode 真实接入端到端验证（README 提了 · 没跑过）
- ❌ MCP SSE 传输（仅 stdio · Web Agent 场景）
- ❌ MCP resources / prompts 能力
- ❌ Claude Code 真实接入回归（用户需手动配 + 验证 · 自动化困难）
