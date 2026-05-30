# STATUS — uninstall-script

> 配套 feature log：`feedback/features/rag4arkui-core/39-2026-05-30-uninstall-script.md`
> 配套 meta：`feedback/meta/20-2026-05-30-uninstall-script.md`
> 日期：2026-05-30

---

## 当前状态

补齐 `make install` 的反向操作 · 完成「装 → 卸 → 装」幂等闭环。Round 37/38 都欠这条 · 本轮 Round 39 收掉。

本阶段交付：
- `scripts/uninstall-binary.sh` 155 行 · 5 step · 默认 dry-run · `--yes` 才真删
- `Makefile` `uninstall` + `uninstall-yes` 两 target
- `docs/MCP-INTEGRATION-CLAUDE-CODE.md` 「反向操作」节 · 20 行
- 实测：uninstall --yes → install 重新装回 → 三端 `✓ Connected` · 完全幂等

意义：用户测试 / 切换 / 清理时不用手动 5 步（删 binary · 编辑三处 config · 决定索引）· 一行 `make uninstall-yes` 搞定 + 全部可恢复（binary `.uninstalled.*` · config `.bak.*`）。

## 输入契约

### 新增 Makefile targets

| Target | 行为 |
|---|---|
| `make uninstall` | dry-run · 显示要删什么 · 不真删（推荐先看）|
| `make uninstall ARGS=--yes` | 真删（语法稍绕）|
| `make uninstall-yes` | 真删（推荐 · 简洁）|

### `uninstall-binary.sh` 输入

```bash
bash scripts/uninstall-binary.sh \
    [--yes]                  # 真执行（默认 dry-run）
    [--bin-dir <PATH>]       # 默认 ~/.local/bin
```

### 不变项

- `arkui-rag` CLI / API / 索引格式都不变
- `make install` 流程不变
- 配置文件格式不变（只是 del 已加的节）

## 输出契约

### 5 step 副作用清单（`--yes` 时）

1. `~/.local/bin/arkui-rag` mv 到 `arkui-rag.uninstalled.<TS>`（可恢复）
2. `~/.claude.json` 顶层 `mcpServers.arkui-rag` 移除（通过 `claude mcp remove`）
3. `~/Library/Application Support/Claude/claude_desktop_config.json` `mcpServers.arkui-rag` 节 del · `.bak.<TS>` 备份
4. `~/.config/opencode/opencode.json` `mcp.arkui-rag` 节 del · `.bak.<TS>` 备份
5. **索引保留** `~/.arkui-rag/` · 显示大小（如 1.8M）+ 提示 `rm -rf` 命令

### 友好降级

| 状态 | 行为 |
|---|---|
| binary 不存在 | 跳过 Step 1 · 继续后续 |
| Claude CLI 没装 | 跳过 Step 2 |
| Claude CLI 装了但 mcp list 没 arkui-rag | 跳过 Step 2 |
| Desktop config 不存在 | 跳过 Step 3 |
| Desktop config 没 arkui-rag 节 | 跳过 Step 3 |
| 同理 opencode | 跳过 Step 4 |
| 索引不存在 | Step 5 跳过 + 提示 |

### 完成提示

dry-run 完成：

```
🔍 DRY RUN 完成
   真执行: bash scripts/uninstall-binary.sh --yes
   或:     make uninstall
```

真删完成：

```
🎉 卸载完成

下一步（可选）：
  - 重启 Claude Code CLI 让配置生效
  - 重启 Claude Desktop
  - 重启 opencode
  - 清索引: rm -rf ~/.arkui-rag
  - 恢复 binary: mv ~/.local/bin/arkui-rag.uninstalled.<TS> ~/.local/bin/arkui-rag
```

## 验证手段

### 用户手动

```bash
# 1. 看要做什么（dry-run · 安全）
make uninstall

# 2. 真执行
make uninstall-yes

# 3. 验证三端干净
claude mcp list | grep arkui-rag      # 应该没输出
python3 -c "import json; print(json.load(open('$HOME/Library/Application Support/Claude/claude_desktop_config.json')).get('mcpServers',{}).get('arkui-rag'))"   # None
python3 -c "import json; print(json.load(open('$HOME/.config/opencode/opencode.json')).get('mcp',{}).get('arkui-rag'))"   # None

# 4. 重新装回（验证幂等）
make install
claude mcp list | grep arkui-rag      # ✓ Connected
```

### 自动化

本轮 agent 已实测装/卸/装闭环：
- A `bash scripts/uninstall-binary.sh --yes` ✓
- B 三端验证全干净 ✓
- C `bash scripts/install-binary.sh` 8 step 全过 · `✓ Connected` ✓

CI 残留（同 Round 37/38）：未加 `make install && make uninstall-yes && make install` 矩阵 step · 留作下一轮（macOS-only）。

## 与上一阶段的关联性

| Round | 解决 | install 链路 |
|---|---|---|
| 37 | install-script | 装 binary + 配 Claude 双端 |
| 38 | opencode-mcp | 加 opencode 第 3 端 |
| **39（本轮）**| uninstall-script | **反向操作 · 完成装/卸/装幂等闭环** |

层次：37 立 install 框架 · 38 扩 client · 39 补 uninstall · 闭环完整。

兼容性：
- 完全向后兼容：install/uninstall 互不依赖
- 任意 client 没装 / 没配 · uninstall 跳过该 step 不报错
- 用户手动 `claude mcp remove` 过的 · uninstall 也能 noop 跳过

破坏性变更：无。

性能：
- `uninstall --yes` 全程 ~1 秒（mv + 3 个 python del + 备份 cp）
- dry-run 更快（< 0.5 秒）· 适合 CI

## 完成度 / 下一阶段

### 达成度

| 项 | 状态 |
|---|---|
| uninstall-binary.sh 155 行 | ✅ |
| Makefile uninstall + uninstall-yes | ✅ |
| MCP-INTEGRATION 反向操作节 | ✅ |
| 装/卸/装幂等闭环实测 | ✅ |
| 双轨归档 + STATUS（本文件）| ✅ |
| 用户实际用过 | ⏳（用户视需 · 不必立即跑）|

### 下一阶段建议

立即：用户没必要立即跑 uninstall（当前用得好的话）· 留着将来切版本 / 测试时用

短期（agent 做 · 1 round）：
- **CI 闭环 smoke test**：`make install && make uninstall-yes && make install` 在 macOS-only matrix step
- **`--purge` flag**：彻底清理 · 真 `rm -rf ~/.arkui-rag/` · 用户明示要时
- **Round 37 残留延续**：`release-local.sh` 加 codesign step（dist tarball 解出也 self-signed）
- **Round 38 残留延续**：`classify-change.sh` 边界（Makefile / Cargo.toml / mdbook/src 应判 meta · 已 3-4 轮残留）

中期：
- README 顶部 + USER-VERIFICATION 都加一行 uninstall 提示（当前只 MCP-INTEGRATION 提）
- Linux/Windows 版本（Linux 应该直接能跑 · macOS 特性 codesign 不存在 · Windows 是 PowerShell 重写）

长期：
- `make purge` target · 全清（binary + 三端 config + 索引 + 备份文件）· 1.0 release 时考虑
- `make doctor` target · 诊断当前安装状态（用 install + uninstall 同款探测逻辑）
