# 20 — uninstall-script

> 日期：2026-05-30
> 触发：Round 37/38 残留高优先级项 · 用户选「先做高优先级」
> 类型：Skill 变更（scripts/uninstall-binary.sh 新建 · Makefile uninstall target）

---

## 用户提出的要求

原文：

> 先做高优先级

承接 Round 38 末尾 Agent 列出的 5 条残留 · 用户选最高优先级 = `Makefile uninstall target`（要覆盖三端：Claude Code CLI + Claude Desktop + opencode）。

## Agent 给出的修改建议

`make install` 已经一行装到位 · 但没反向命令。用户测试 / 切换 / 清理时只能手动 5 步（删 binary · `claude mcp remove` · 编辑 Desktop config · 编辑 opencode config · 决定是否清索引）· 易漏 · 易误。

设计：`scripts/uninstall-binary.sh` 对称 `install-binary.sh` · 5 step（对应 install 的 8 step 浓缩）：

| Uninstall Step | 对应 Install Step | 操作 |
|---|---|---|
| 1 binary | Install 1-3 | `mv` → `.uninstalled.<时间戳>`（可恢复 · 不 `rm`）|
| 2 Claude CLI | Install 6 | `claude mcp remove arkui-rag --scope user` |
| 3 Claude Desktop | Install 7 | Python `del cfg['mcpServers']['arkui-rag']` · 先备份 |
| 4 opencode | Install 8 | Python `del cfg['mcp']['arkui-rag']` · 先备份 |
| 5 索引保留 | — | 提示用户手动 `rm -rf ~/.arkui-rag/` 决定 |

关键设计决策：

- **默认 dry-run** · `--yes` 才真删 —— 防误操作 · 用户先看清要做什么
- **binary `mv` 不 `rm`** —— 用户恢复一行命令 · `mv arkui-rag.uninstalled.* arkui-rag`
- **配置文件备份** —— `.bak.<时间戳>` · 想还原 `cp` 回来
- **不动索引** —— `~/.arkui-rag/` 含 bm25 倒排 + index.json · 可能 1.8MB+ · 用户数据 · 不能自动删
- **AGENTS.md #13 自洽** —— 配置文件 `del key` 操作算可逆（备份在）· binary `mv` 算可逆 · 不属于「Agent 禁止主动执行不可逆操作」

`Makefile` 加两个 target：

- `make uninstall`（默认 dry-run）· 支持 `ARGS=--yes` 真删
- `make uninstall-yes`（语法糖 · 直接真删）

### 替代方案权衡

| 选项 | 优点 | 缺点 | 选 |
|---|---|---|---|
| A · dry-run + `--yes` 守门（本轮）| 防误操作 · 用户先看清 | 用户得跑两次 | ✅ |
| B · 直接真删 | 一步到位 | 误跑灾难（清掉所有 MCP 配置 · 用户还得手动恢复）| ❌ |
| C · 交互式 `read -p "确认?(y/N)"` | 用户友好 | bash 工具非 tty 时 hang · CI 不能跑 | ❌ |
| D · 真 `rm` binary | 干净 | 用户改主意要重装得 `make release-local`（慢）| ❌ |

选 A · 安全 + 可恢复 + CI-friendly。

## 多轮互动

| 轮次 | 用户 | Agent |
|---|---|---|
| 1 | 「先做高优先级」（承接 Round 38 末尾「🔴 高 · Makefile uninstall · 1 round」）| 本轮实施：uninstall-binary.sh 5 step + Makefile 两 target + 文档加节 + 实测装/卸/装幂等 |

无方向调整 · 用户明确选最高优先级。

## 实际改动

- 接口变化：新增 `make uninstall` / `make uninstall-yes` Makefile target
- 规则变化：用户体验从「install only · 卸载靠手动」升级为「装/卸/装幂等闭环」
- 文件变化：
  - **新建** `scripts/uninstall-binary.sh` —— 155 行 · 5 step · 默认 dry-run + `--yes` 真删
  - **修改** `Makefile` —— 加 `uninstall` + `uninstall-yes` 两个 target
  - **修改** `docs/MCP-INTEGRATION-CLAUDE-CODE.md` —— 「推荐一键路径」后加「反向操作 · make uninstall」节
- 配置变化：无（脚本运行时改用户配置 · 但有备份）

## 执行生效后总结

### 实际产出

| 文件 | 改动 |
|---|---|
| `scripts/uninstall-binary.sh` | 新建 155 行 · 5 step（binary + 三端 MCP + 索引提示）|
| `Makefile` | +6 行（uninstall + uninstall-yes）|
| `docs/MCP-INTEGRATION-CLAUDE-CODE.md` | +20 行（反向操作节）|

### 前后对比

| 维度 | Before（Round 38）| After（Round 39）|
|---|---|---|
| 卸载方式 | 手动 5 步（删 binary + 三处 config 编辑 + 决定索引）| `make uninstall-yes` 一行 |
| 误操作防护 | 无 | 默认 dry-run · `--yes` 才真删 |
| 恢复方式 | 无法恢复（手动 `rm` 不可逆）| binary `mv` 可恢复 · config 有 `.bak.*` 备份 |
| 索引数据 | 容易跟着删 | 明确保留 + 提示用户决定 |
| CI 可用 | 不适用 | dry-run mode 适合 CI smoke test |
| 三端覆盖 | N/A | Claude CLI + Desktop + opencode 全覆盖 |

### 实测验证

- `bash -n scripts/uninstall-binary.sh` ✓ 语法 OK
- `bash scripts/uninstall-binary.sh`（dry-run）✓ 4 step 全识别要清理什么 + Step 5 索引保留提示
- `bash scripts/uninstall-binary.sh --yes`（真执行）✓：
  - Step 1 binary `mv` 到 `.uninstalled.20260530-203432`
  - Step 2 `Removed MCP server arkui-rag from user config`
  - Step 3 Desktop config 节移除 + 备份
  - Step 4 opencode config 节移除 + 备份
  - Step 5 索引保留提示（1.8M）
- **关键闭环验证**：uninstall --yes → 验证三端干净 → `make install` 重新装回 → Step 8 全部 `✅ Connected` · **install/uninstall 完全幂等**

### 残留 / 下一轮处理

- [x] scripts/uninstall-binary.sh 5 step
- [x] Makefile uninstall + uninstall-yes
- [x] MCP-INTEGRATION 加反向操作节
- [x] 实测装/卸/装闭环幂等
- [x] 双轨归档 + STATUS
- [ ] **Round 37 残留延续**：release-local.sh 加 codesign step（独立 round）
- [ ] **Round 38 残留延续**：classify-change.sh 分类边界（Makefile / Cargo.toml / mdbook/src 应判 meta · 已 3 轮残留）
- [ ] **CI smoke test**：加 `make install && make uninstall-yes && make install` 幂等矩阵（macOS only）
- [ ] **可选**：`uninstall-binary.sh` 加 `--purge` flag · 真删索引（用户明示要彻底清理时）
