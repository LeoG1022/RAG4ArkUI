# STATUS — tantivy-read-only

> 配套 feature log：`feedback/features/rag4arkui-core/36-2026-05-30-tantivy-read-only.md`
> 日期：2026-05-30

---

## 当前状态

`TantivyBM25Index` 拆分 reader / writer 双路径 · 解锁**多 Claude client 并发接同一个 arkui-rag binary**。

本阶段交付：
- `TantivyBM25Index::open_read_only()` 新 API · 不持 `IndexWriter` 锁
- `arkui-rag-cli` 6 处 `build_bm25` 按 writable 转发 · 仅 `cmd_index` 写模式
- 顺手修 `reader.reload()` pre-existing bug · 2 个 master 已坏测试现在通过
- 加 2 个新单元测试覆盖 multi-instance / read-only 误用
- 实测：Claude Desktop 内嵌的 claude-code（本会话 child）+ 第二个 binary instance 同时活 · 无 `LockBusy`

意义：Round 33-35 把「接 Claude Code」的**文档 + 配置**全打通了 · 但任何一刻只能一个 Claude client 用 · 实际不够。Round 36 才让 Claude Code 和 Claude Desktop 真正可以同时用 · 整条「本地 RAG 接 AI agent」链路才算 production-ready。

## 输入契约

### `TantivyBM25Index` 公开 API 变化

| 方法 | Before | After | 兼容性 |
|---|---|---|---|
| `open(dir)` | 总是 reader + writer | reader + writer（同前 · 写模式）| ✅ 向后兼容 |
| `open_read_only(dir)` | 不存在 | **新加** · 仅 reader · 不持锁 | 新功能 |
| `upsert(chunks)` | 永远可用 | read-only 模式返回 error | 误用 fail-fast |
| `delete(ids)` | 永远可用 | read-only 模式返回 error | 同上 |

### CLI 内部 helper

```rust
fn build_bm25(kind: Bm25Kind, index_path: &Path, writable: bool)
            -> anyhow::Result<(Arc<dyn BM25Index>, &'static str)>
```

加 `writable: bool` 第三参数。6 个 subcommand：

| Subcommand | writable | 行为 |
|---|---|---|
| `arkui-rag index` | true | 独占写锁 · 同目录此时不能跑 serve |
| `arkui-rag query` | false | 多 instance 共存 |
| `arkui-rag eval` | false | 同上 |
| `arkui-rag serve --mcp` | false | **关键**：Claude Code + Desktop 同跑 OK |
| `arkui-rag serve --http` | false | 同上 |
| `arkui-rag serve --lsp` | false | 同上 |

### 不变项

- CLI 用户接口完全不变（`--bm25 tantivy` 等参数语义不变）
- index.json / bm25 目录磁盘格式不变（同一个索引 reader/writer 都读得）
- BM25Index trait（pub）签名不变

## 输出契约

### 行为契约新增

- **多 instance 共存语义**：同一个 `~/.arkui-rag/bm25/` 目录可以被任意多个 `arkui-rag serve --bm25 tantivy` 进程同时打开（前提：都不调 `cmd_index`）
- **read-only 误用契约**：read-only 模式下调 upsert/delete 返回 `RagError::Storage("... open_read_only 模式打开 · upsert 不可用 · 重建索引请用 arkui-rag index")` · 不是 panic
- **commit 同步保证**：`upsert` / `delete` 返回时 reader 已 reload · 后续 `search` / `len()` 立刻看到最新数据（修 pre-existing `OnCommitWithDelay` 异步 bug）

### 输出格式不变

JSON-RPC 响应 / markdown 渲染 / hit 字段全部不变。

## 验证手段

### 用户手动

```bash
# 1. 重装 binary（必须 · 因为是代码层 fix）
sudo cp /Users/leo/work/RAG4ArkUI/crates/target/release/arkui-rag /usr/local/bin/

# 2. 退出 terminal claude CLI（PID 96460 之类 · 你启动的那个）
#    Ctrl-D 退 claude · 或 pkill -f "^claude$"

# 3. 重启 terminal claude
claude

# 4. 在 chat 内：
#    /mcp                              # 应该显示 arkui-rag ✓ Connected
#    用 arkui_search_docs 检索 X        # 应该实际调用并返回 hits

# 5. 同时验证 Desktop 也通（如果你也用 Desktop）
# 完全退出 Desktop (Cmd-Q 或 pkill -i "Claude") · 重开
# 在 Desktop chat 调 arkui_search_docs · 也应该真活
```

### 自动化

```bash
cd crates && cargo test -p arkui-rag-storage --features tantivy --lib
# 期望：14 passed / 0 failed
# 含新加：
#   - read_only_allows_concurrent_multi_instance（3 reader 共存）
#   - read_only_upsert_returns_error（误用 fail-fast）
# 顺手修：
#   - delete_works · upsert_and_search_basic（master 本就坏 · 现在通过）
```

CI 残留：本轮测试需要 `--features tantivy` · 但 `make check` / `cargo test --workspace` 默认 features 跑不到。CI 需补一个矩阵 step（feature log 残留列出）。

## 与上一阶段的关联性

| Round | Slug | 解决层 |
|---|---|---|
| 33 | concepts-archive-rule | Agent 行为规则 |
| 34 | cli-default-features | **Build** 路径（裸 cargo build 产物可用） |
| 35 | mcp-config-path-fix | **Install** 路径（claude mcp add 命令对） |
| **36（本轮）** | tantivy-read-only | **Runtime** 路径（多 client 并发） |

层次递进：build 通了 → 配置写对了 → runtime 多 client 共存。任何一层挡道 · Claude 都看不到 arkui_search_docs。Round 36 把最后一层打通。

兼容性：
- API 层：`open()` 不变 · 完全向后兼容
- CLI 层：`build_bm25` 加参数 · 但是内部 helper · 不暴露给用户
- 数据层：磁盘格式不变 · 现有索引文件无需重建
- 协议层：MCP / HTTP / LSP JSON-RPC 响应不变

破坏性变更：无。

性能：
- read-only 模式启动比 writer 模式略快（少一次 `index.writer(WRITER_HEAP_BYTES)` 分配）· 微秒级
- search 路径完全相同 · 无变化
- upsert / delete 多一次 `reader.reload()` 同步刷新 · 微秒级 · 换取测试稳定 + 立即一致性

## 完成度 / 下一阶段

### 达成度

| 项 | 状态 |
|---|---|
| `open_read_only()` API | ✅ |
| `Option<Writer>` 字段重构 | ✅ |
| `upsert`/`delete` 误用检查 | ✅ |
| `reader.reload()` 同步 fix | ✅ |
| CLI 6 处 writable 转发 | ✅ |
| 2 个新单元测试 | ✅ |
| 2 个 pre-existing 测试 fix | ✅ |
| 实测多 instance 共存 | ✅ |
| 用户重装 + 验证 | ⏳（等用户 `sudo cp`） |
| Claude CLI/Desktop 双端真调 arkui_search_docs | ⏳（等用户重启 client） |

### 下一阶段建议

立即（用户做）：
1. `sudo cp /Users/leo/work/RAG4ArkUI/crates/target/release/arkui-rag /usr/local/bin/`
2. 退出当前 terminal claude（PID 96460）+ 重启
3. `/mcp` 命令看 `✓ Connected`
4. chat 试 `用 arkui_search_docs 检索 X`

短期（agent 做 · 1-2 round）：
- `docs/MCP-INTEGRATION-CLAUDE-CODE.md` 加章节「多 client 共存说明」+ Claude Desktop 配 `claude_desktop_config.json` 的差异 + `command` 必须绝对路径（launchd PATH 不继承 shell）
- CI `.github/workflows/ci.yml` 加 `cargo test --features tantivy` step · 防 multi-instance 测试被 default features 漏过

中期：
- 看 `ReloadPolicy::OnCommitWithDelay` 是否改 `OnCommitWithoutDelay`（trade-off：写吞吐 vs 一致性 · 当前 `reader.reload()` 显式同步 · 行为正确但有 redundancy）
- 给 `arkui-rag serve` 加 `--readonly` 显式 flag（当前默认 read-only · 但用户可能想未来加「热重建索引」之类用例 · 那时需要 writer · 显式 flag 防意外）

长期：
- 看 lancedb 也是不是同样问题（`LanceVectorStore` 是否多 instance OK · `--vector lancedb` 多 client 共存测一下）
- 看 `corpus pull` 期间 server 正在跑的并发安全 · 当前 corpus pull 是写 corpus 目录 · 不动 index · 但下个版本可能加自动 reindex
