# 36 — tantivy-read-only

> 日期：2026-05-30
> 涉及代码：`crates/arkui-rag-storage/src/tantivy_bm25.rs` · `crates/arkui-rag-cli/src/main.rs`
> 类型：bug 修复（设计层 · 解锁多 Claude client 并发接入）

## 本轮目标

修一个**真活打通最后障碍**的设计 bug：

`arkui-rag serve --mcp/--http/--lsp` 启动时调 `TantivyBM25Index::open()` · 这个方法**总是**初始化 `IndexWriter`（拿目录独占写锁）· 即便 server 只做检索完全不写。

后果：
- Claude Code (CLI) 启动 arkui-rag MCP child → 拿到写锁
- Claude Desktop (GUI) 启动 arkui-rag MCP child → `Failed to acquire Lockfile: LockBusy` → server 立即 exit → Claude 看不到工具
- 任何「同时跑两个 Claude client 接同一 binary」的场景都坏

实战触发：用户已经按 Round 33-35 把 cli default features + claude mcp add + Claude Desktop 配置全做对了 · `claude mcp list` 一开始 `✓ Connected` · 但本会话 child（PID 96247）持有写锁后 · 用户在 terminal 新 launch claude → 第二个 child fork 失败 → `✗ Failed to connect`。

## Plan

修复策略：**`open` API 拆 reader / writer 两路 · 让 server 路径完全不持写锁 · 多 instance 共存**。

### 改动 1 · `TantivyBM25Index` 加 `open_read_only`

```rust
// before
pub fn open(dir: &Path) -> Result<Self> {
    // 总是创建 IndexWriter · 拿独占锁
}

// after
pub fn open(dir: &Path) -> Result<Self> {
    Self::open_with_mode(dir, true)   // 写模式 · 同 indexer 用
}
pub fn open_read_only(dir: &Path) -> Result<Self> {
    Self::open_with_mode(dir, false)  // 只读 · 多 instance 共存
}
fn open_with_mode(dir: &Path, writable: bool) -> Result<Self> {
    // ... 通用 setup ...
    let writer = if writable { Some(...) } else { None };
}
```

`writer: Option<Arc<Mutex<IndexWriter>>>` 包成 Option · None 时不持锁。

`upsert` / `delete` 加 `self.writer.as_ref().ok_or(...)` · read-only 误用会得到清晰错误而非 panic。

### 改动 2 · CLI `build_bm25` 加 `writable: bool`

```rust
fn build_bm25(kind, index_path, writable: bool) -> ... {
    match kind {
        Tantivy => if writable {
            TantivyBM25Index::open(&dir)
        } else {
            TantivyBM25Index::open_read_only(&dir)
        }
    }
}
```

6 处调用方：

| 函数 | writable | 理由 |
|---|---|---|
| `cmd_index` | true | 写索引 · 需独占写锁 |
| `cmd_query` | false | 只查 |
| `cmd_eval` | false | 只查 |
| `cmd_serve_mcp` | false | 只查 · 多 client 共存 |
| `cmd_serve_lsp` | false | 只查 |
| `cmd_serve_http` | false | 只查 |

### 改动 3 · 顺手修 pre-existing reader.reload() 缺失

跑测试发现 `delete_works` / `upsert_and_search_basic` **本来就坏**（master 也 fail · 不是本轮引入）。

根因：reader 用 `ReloadPolicy::OnCommitWithDelay` · commit 后**异步** reload · 立即调 `num_docs()` 看到旧值。

补：upsert / delete 之后调一次 `self.reader.reload()` · 同步刷新。生产 query 路径不影响（OnCommitWithDelay 仍工作）· 测试稳定。

1 行 fix × 2 处。本来可以独立 round · 但跟主修复同文件 · 顺手清掉避免 commit 时 cargo test 阻塞。

### 加 2 个新单元测试

```rust
#[tokio::test]
async fn read_only_allows_concurrent_multi_instance() {
    // 三个 reader 同时活 · 模拟 Claude Code + Desktop + manual stdio
}
#[tokio::test]
async fn read_only_upsert_returns_error() {
    // read_only 模式 upsert 必须报错 · 防误用
}
```

### 替代选项权衡

| 选项 | 优点 | 缺点 | 选了吗 |
|---|---|---|---|
| A · API 拆分 + Option<Writer>（本轮）| 类型签名清晰 · 误用 fail-fast | 多一个公开方法 | ✅ |
| B · Lazy IndexWriter（OnceCell）| 调用方无感 | 第一次 upsert 时延迟拿锁 · 行为隐性 | ❌ |
| C · serve 路径用独立 index 目录 | 不动代码 | 双倍磁盘 · corpus 更新两份 · 用户负担大 | ❌ |
| D · 仅修文档「同时只能一个 client」 | 0 代码 | 不解决 · 用户必踩 | ❌ |

选 A · API 显式 · 误用早死。

## 对话摘要

| 轮次 | 用户 | Agent |
|---|---|---|
| 1 | 「terminal 中启动 Claude CLI 也报错 没看到 arkui_search_docs」 | 诊断：本会话（Claude Desktop 启的 claude-code）的 arkui-rag child PID 96247 持 tantivy 写锁 · 用户 terminal claude CLI fork 第二个 arkui-rag 拿不到锁 → `claude mcp list` 显示 `✗ Failed to connect` · 给三个修法选项（A 修代码 / B 杀本会话 child / C 独立 index）|
| 2 | 选 A 修代码根治 | 本轮实施：tantivy_bm25.rs 加 open_read_only · cli main.rs 6 处 writable bool · 顺手修 reader.reload() pre-existing bug · 加 2 个测试 · 实测多 instance 共存 |

无方向调整 · 用户直接选 A。

## 改动要点

- `crates/arkui-rag-storage/src/tantivy_bm25.rs`：
  - `writer: Arc<Mutex<IndexWriter>>` → `Option<Arc<...>>`
  - `open()` 不变（向后兼容 · 写模式）· 新加 `open_read_only()` · 共用 `open_with_mode()` 内部实现
  - `upsert` / `delete` 加 `writer.as_ref().ok_or(...)` 检查
  - `upsert` / `delete` 末尾加 `self.reader.reload()` —— **pre-existing fix**（master 的 delete_works 本就坏 · 与主修复同文件 · 顺手清）
  - 加 `read_only_allows_concurrent_multi_instance` + `read_only_upsert_returns_error` 两个单元测试
- `crates/arkui-rag-cli/src/main.rs`：
  - `build_bm25(kind, index_path, writable: bool)` · `build_tantivy(index_path, writable)`
  - 6 处调用：cmd_index → true · 其它 5 处 → false
- 与 Round 35 关系：35 修 install 路径（用 `claude mcp add` 命令）· 36 修 runtime 路径（多 client 共存）· 36 之前 35 的「同时跑 Code + Desktop」即便配置全对也会一个 client 看不到工具 · 36 之后真正可用

## 验证结果

- 编译：`cargo build --release -p arkui-rag-cli` ✓ Finished 21.70s · 产物 10.7MB
- 单元测试：`cargo test -p arkui-rag-storage --features tantivy --lib` ✓ **14 passed / 0 failed**
  - 新加 2 个：`read_only_allows_concurrent_multi_instance` + `read_only_upsert_returns_error` 都通过
  - 顺手修复 2 个 pre-existing：`delete_works` + `upsert_and_search_basic` 现在通过
- 实操验证：
  - 本会话 child PID 96247 持写锁（旧 binary · open 模式）
  - 跑新 binary（read-only）`tools/list` → 完整返回 4 工具 · 无 `LockBusy` · 多 instance 共存 ✓
- check-api-parity：N/A（核心存储改动 · 无业务规则适用）

事后验证：用户 `sudo cp` 重装 + 退他的 terminal claude (PID 96460) + 重启 → `claude mcp list` 应显示 `✓ Connected` → 新会话调 arkui_search_docs 真活返回 hits。

## 残留 / 下一轮

- [x] tantivy_bm25.rs 加 open_read_only · writer 改 Option
- [x] cli main.rs build_bm25 加 writable 参数 · 6 处调用方
- [x] upsert/delete 内 reader.reload() 修 pre-existing 测试 bug
- [x] 加 2 个 multi-instance 测试
- [x] 实测多 instance 共存（无 LockBusy）
- [ ] **用户重装 + 验证**：`sudo cp /Users/leo/work/RAG4ArkUI/crates/target/release/arkui-rag /usr/local/bin/` + 退 PID 96460 terminal claude + 重启 + `claude mcp list` 看 `✓ Connected` + chat 试调用 arkui_search_docs
- [ ] 文档更新：`docs/MCP-INTEGRATION-CLAUDE-CODE.md` 加段「多 client 共存」说明（Round 35 之后的下一坑）· 含 Claude Desktop 配 `claude_desktop_config.json` 路径差异
- [ ] CI 加 `cargo test --features tantivy` step（之前 ci.yml 只跑 default features · 这些测试 CI 跑不到）
- [ ] 看长期：`ReloadPolicy::OnCommitWithDelay` 改 `OnCommitWithoutDelay` 是否更合适（trade-off：吞吐 vs 测试 / 立即一致性）
