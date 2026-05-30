# 39 — uninstall-script

> 日期：2026-05-30
> 涉及代码：`scripts/uninstall-binary.sh`（新）· `Makefile`（uninstall target）· `docs/MCP-INTEGRATION-CLAUDE-CODE.md`
> 类型：新建（反向安装脚本 · 装/卸/装幂等闭环）

## 本轮目标

补齐 `make install` 的反向操作 · 让用户「装 → 卸 → 装」一键幂等闭环。Round 37/38 都欠这一项 · 本轮 Round 39 收掉。

## Plan

### 设计：5 step 对称 install

`scripts/uninstall-binary.sh` 5 step 对应 `install-binary.sh` 的 8 step（不需要 source / PATH 检查 / 索引检查）：

```bash
1. binary: mv ~/.local/bin/arkui-rag → arkui-rag.uninstalled.<TS>（可恢复）
2. Claude CLI: claude mcp remove arkui-rag --scope user
3. Claude Desktop: Python del cfg['mcpServers']['arkui-rag'] + backup
4. opencode: Python del cfg['mcp']['arkui-rag'] + backup
5. 索引: 保留 + 提示用户手动 rm -rf 决定
```

### 关键设计

- **默认 dry-run** —— `--yes` 才真删 · 防误操作
- **binary `mv` 不 `rm`** —— 一行恢复 `mv arkui-rag.uninstalled.* arkui-rag`
- **配置文件 backup** —— 所有 config 改前 `cp` 到 `.bak.<TS>`
- **不动索引** —— `~/.arkui-rag/` 含 bm25 + index.json + 可能 100MB+ 用户数据 · 不自动删
- **空 mcpServers / mcp 节自动删** —— 移除 arkui-rag 后如果节空了 · 也删 · 保持 JSON 干净

### Makefile：2 target

```makefile
uninstall:                 # dry-run · 安全 · 显示要做什么
	bash scripts/uninstall-binary.sh $(ARGS)

uninstall-yes:             # 真删 · 直接 make uninstall-yes
	bash scripts/uninstall-binary.sh --yes
```

`make uninstall ARGS=--yes` 也可以但比较绕 · `uninstall-yes` 短。

### 文档：MCP-INTEGRATION 加「反向操作」节

「推荐一键路径」之后插入 · 列两个命令 + 5 step 操作清单 + 恢复方式（备份在 `.bak.*` / `.uninstalled.*`）。

### 替代方案权衡

| 选项 | 优点 | 缺点 | 选 |
|---|---|---|---|
| A · dry-run + `--yes`（本轮）| 防误操作 · CI-friendly · 可恢复 | 用户得跑两次（先看后真） | ✅ |
| B · 直接真删 | 一步 | 误跑灾难（三端 config 全清掉） | ❌ |
| C · `read -p` 交互 | 用户友好 | bash 工具非 tty hang · CI 不能跑 | ❌ |

选 A · 跟 Round 37 install 设计哲学一致（安全 > 便捷）。

## 对话摘要

| 轮次 | 用户 | Agent |
|---|---|---|
| 1 | 「先做高优先级」（Round 38 末尾 5 条残留里 🔴 高 = uninstall target）| 本轮：uninstall-binary.sh 5 step + Makefile 两 target + 实测装/卸/装幂等闭环 |

无方向调整 · 用户明确选最高优先级项。

## 改动要点

- **scripts/uninstall-binary.sh** 155 行 · 5 step：
  - 文件头注释说明设计要点（不用 rm 用 mv · 默认 dry-run · 不动索引）
  - Step 1-4 各自 dry-run / 真删两分支
  - Step 3-4 Python 安全合并 + 备份 + 空节自动删
  - 末尾完成提示分 dry-run / 真删两种输出（dry-run 提示「真执行加 --yes」· 真删提示「重启 client · 恢复 binary 命令」）
- **Makefile** 加 `uninstall: bash scripts/uninstall-binary.sh $(ARGS)` 和 `uninstall-yes: bash scripts/uninstall-binary.sh --yes`
- **docs/MCP-INTEGRATION-CLAUDE-CODE.md** 「推荐一键路径」节之后插入「反向操作 · make uninstall」小节（20 行）· 含 5 step 操作清单 + 恢复方式
- 与 Round 37/38 关系：37 立 install 框架（双端）· 38 加 opencode 扩到三端 · 39 补反向操作 · 闭环

## 验证结果

- 编译：N/A（脚本 + 文档）
- 集成测试：
  - `bash -n scripts/uninstall-binary.sh` ✓ 语法 OK
  - `bash scripts/uninstall-binary.sh`（dry-run）✓ 4 step 全识别要清理什么 + Step 5 索引保留提示（1.8M）
  - `bash scripts/uninstall-binary.sh --yes`（真执行）✓ 全 5 step 通过 · binary 重命名 + 三端 MCP config 节移除 + 全部备份
- **关键幂等闭环实测**：
  - A. `uninstall --yes` → 三端全干净（claude mcp list 没 arkui-rag · Desktop config 没节 · opencode config 没节 · binary 不存在）
  - B. `bash scripts/install-binary.sh` → 8 step 全过 · 三端重新 `✓ Connected`
  - 验证完全幂等

事后验证（用户 · 可选）：自己跑一次 `make uninstall` 看 dry-run · 不真删 · 看清流程后看是否需要 `make uninstall-yes`。

## 残留 / 下一轮

- [x] scripts/uninstall-binary.sh 5 step
- [x] Makefile uninstall + uninstall-yes
- [x] MCP-INTEGRATION 加反向操作节
- [x] 装/卸/装闭环幂等实测
- [x] 双轨归档 + STATUS
- [ ] **CI smoke test**：加 `make install && make uninstall-yes && make install` 闭环矩阵（macOS-only）防回归
- [ ] **可选 `--purge` flag**：明示要彻底清理时真 `rm -rf ~/.arkui-rag/`（当前只提示 · 不动）
- [ ] **Round 37 残留延续**：release-local.sh 加 codesign step
- [ ] **Round 38 残留延续**：classify-change.sh 分类边界（Makefile / Cargo.toml / mdbook/src 应判 meta · 已 3-4 轮残留）
- [ ] **README / USER-VERIFICATION 加 uninstall 段**：当前只在 MCP-INTEGRATION 提了 · 顶部 README 也可加一行 reminder
