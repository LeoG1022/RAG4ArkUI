# 54 — ci-lint-cleanup

> 日期：2026-06-02
> 涉及代码：多 crate fmt + clippy 修复（详见 git diff stats · 32 文件）
> 类型：bug 修复（CI ci.yml fmt + clippy -D warnings 挂）

## 本轮目标

Round 49.6 commit `6b1f6df` push 到 GitHub 后 · ci.yml 跑 fmt + clippy `-D warnings` 全挂：

- `cargo fmt --check` → tantivy_bm25.rs 等多处 `.await.unwrap()` 链 fmt 不一致
- `cargo clippy -- -D warnings` → Rust 1.95 新加 lint 触发存量代码 warning 转 error

ci.yml 之前应该是绿的 · 但 toolchain 升到 1.95 后存量代码出新 warning · 加上本轮 Round 49.8 新写的 `from_file.is_some() / from_file.unwrap()` 模式被新 lint 抓 · 全炸。本轮统一修。

## Plan

### 决策 A · 不放宽 ci.yml 标准

宁可改代码 · 不放宽 `-D warnings` 门禁。原因：
- 放宽 = 持续退化 · 之后新 lint 仍会触发 · 一退再退
- 一次性修干净 + auto-fix 大半 · 工作量可控

### 决策 B · `cargo clippy --fix --allow-dirty` 一把过

可自动修的（13+ 个 warning）让工具修：
- `Default` 改 derive
- `vec![..]` → 数组直写
- `trim_end_matches(|c| ...)` → 数组
- 等等

剩下 5 类手工：

| 文件 | 问题 | 修法 |
|---|---|---|
| `arkui-rag-cli/src/main.rs` | `from_file.is_some() ... from_file.unwrap()` × 2 | 改 `if let Some(p) = from_file` |
| `arkui-rag-chunker/src/treesitter_base.rs` | `name_by_first_identifier` dead | `#[allow(dead_code)]` |
| `arkui-rag-chunker/src/typescript.rs` | field `lang` 未读 | `#[allow(dead_code)]` |
| `arkui-rag-storage/src/tantivy_bm25.rs` | `pub fn len` 无 `is_empty` | 加 `pub fn is_empty()` |
| `arkui-rag-storage/src/lib.rs` | `trait VectorStore::len` 无 `is_empty` | trait 加 default impl `is_empty` |
| `arkui-rag-storage/src/memory.rs` | `c.clone()` 切片 | `std::slice::from_ref(&c)` |

### 不动
- ci.yml 配置（仍 `-D warnings` 严格门禁）
- 业务逻辑（仅 lint cleanup · 不动行为）
- 既有 trait 接口（VectorStore 加 `is_empty` 是默认实现 · 不破坏 impl）

## 对话摘要

| 轮次 | 用户 | Agent |
|---|---|---|
| 1 | 「已经推了 报错 cargo fmt --check / clippy -D warnings」| 本地 reproduce · fmt auto-fix · clippy auto-fix + 手工 5 类 · 验全绿 |

无方向调整 · agent 自主选择"修代码而非放门禁"。

## 改动要点

### 自动 fmt + clippy --fix 修
- `cargo fmt --all` 32 文件统一格式（主要是 `.await.unwrap()` 链长度断行）
- `cargo clippy --fix --allow-dirty` 13+ 个 warning 自动修：
  - enum `Default` 改 derive + `#[default]`
  - `vec![lit, lit]` → `[lit, lit]`
  - `|c| c == '\r' || c == '\n'` → `['\r', '\n']`
  - 等等

### 手工修
- `arkui-rag-cli/src/main.rs` cmd_corpus_model_pull + cmd_corpus_index_pull 两处 `if from_file.is_some() { from_file.unwrap()... }` → `if let Some(p) = from_file { p.display()... }`
- `arkui-rag-chunker/src/treesitter_base.rs:142` 加 `#[allow(dead_code)]`（保留 helper · 未来用）
- `arkui-rag-chunker/src/typescript.rs:71` field `lang` 加 `#[allow(dead_code)]`（保留 · ChunkerDispatcher 未来 lang 选路时用）
- `arkui-rag-storage/src/tantivy_bm25.rs:124` 加 `pub fn is_empty(&self) -> bool { self.len() == 0 }`
- `arkui-rag-storage/src/lib.rs:35` trait VectorStore 加 default `is_empty()` impl（不破坏既有 impl）
- `arkui-rag-storage/src/memory.rs:302/303` `c.clone()` 切片改 `std::slice::from_ref(&c)`

### 不动
- ci.yml 仍 `cargo clippy --workspace --all-targets -- -D warnings`
- 业务 / cli 行为不变

## 验证结果

```bash
cargo fmt --all -- --check                              # ✓ PASS
cargo clippy --workspace --all-targets -- -D warnings   # ✓ Finished · 0 warning
cargo check --workspace                                  # ✓ PASS · 全 crate 编译过
```

CI 真活：等本 commit push 后 · ci.yml 应 fmt + clippy + test 全绿。

## 残留 / 下一轮

- [x] cargo fmt 全过
- [x] cargo clippy -D warnings 全过
- [x] 不动 ci.yml 门禁 · 修代码不退让
- [ ] **用户重 push** master 验 GitHub Actions ci.yml 真绿
- [ ] **用户触发 Corpus Build** workflow（Round 49.6 等待中）
- [ ] **长期**：Rust toolchain 再升级时 · 新 lint 同款方式 cleanup
