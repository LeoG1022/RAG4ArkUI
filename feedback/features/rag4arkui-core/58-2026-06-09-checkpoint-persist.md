# 58 — checkpoint-persist（Round 55）

> 日期：2026-06-09
> 涉及代码：`crates/arkui-rag-storage/src/lib.rs` · `crates/arkui-rag-storage/src/memory.rs` · `crates/arkui-rag-indexer/src/lib.rs` · `crates/arkui-rag-cli/src/main.rs`
> 类型：可靠性（防长 build 死掉清零）

## 本轮目标

Phase B v2 跑 application-dev 全量 build（590 files）· 17:00 启 · 21:51 进程死了 · build.log 整个不见 · index.json 没产出 · 3-5 小时算力作废。**Round 54 UTF-8 容错没能保住** —— 死因可能是 OOM killer / /tmp 清理 / swap 写穿 · 无 log 无法定 root cause。

教训：indexer 跑数小时**没有任何中间持久化** · 死了清零。本轮加 checkpoint 机制：每 N files 持久化一次 index.json · 死了下次重启最多丢最后一段。

## Plan

### 决策 A · VectorStore trait 加 `persist_checkpoint` 默认方法

```rust
async fn persist_checkpoint(&self, _path: Option<&Path>) -> Result<()> {
    Ok(())  // 默认 no-op
}
```

- InMemoryVectorStore override · 调既有 `save_to(path)` 写 index.json
- LanceVectorStore 用默认 no-op（upsert 已实时落盘）
- 默认 no-op = **零向后破坏** · 所有现有 impl 不改也能编译

### 决策 B · Indexer 加 `index_directory_with_checkpoint`

```rust
pub async fn index_directory_with_checkpoint(
    &self,
    source: &Path,
    checkpoint_every_files: usize,    // 0 = 关
    persist_path: Option<&Path>,
) -> Result<IndexStats>
```

老 `index_directory` 改成调 new method `every=0, path=None`（兼容）。

新方法内核：每处理 `every` files · flush buffer + 调 `vector.persist_checkpoint(path)` · log info「checkpoint files=N chunks=M」。失败时 warn 但继续 build（不让 persist 失败拖死整轮）。

### 决策 C · cli 加 `--checkpoint-every-files <N>` 默认 20

```bash
arkui-rag index --source corpus/... --checkpoint-every-files 20
```

为啥默认 20：
- application-dev 590 files / 20 = 29 次 persist · save_to 每次几百 ms · 总开销 < 1 min
- 死后最多丢最后 20 files chunks（~500 chunks）· 远比 Phase B v2 丢 16000+ chunks 强
- 用户可 `--checkpoint-every-files 0` 关（速度优先）

### 替代方案

- A · 只 in-memory checkpoint（Vec snapshot 不写盘）：死了仍清零 · 无用
- B · 每 batch 持久化（每 32 chunks · 默认 batch_size）：persist 开销过高 · 影响速度
- C · cli 层手动分批 + persist · 不动 indexer：跨 cli/indexer 越界 · 不优雅
- **D · trait 加默认 method + indexer 内置 checkpoint（本次选）**：扩展点干净 · LanceDB 零改动 · 老路径 0 退化

### 不动

- `index_directory` 老接口（默认调新方法 `every=0`）
- LanceDB 路径（trait 默认 no-op）
- BM25 / Tantivy 提交（已经自动 commit）
- 老 index.json 格式

## 对话摘要

| 轮次 | 用户 | Agent |
|---|---|---|
| 1 | 「phase B V2 咋样了」 | 报 Phase B v2 死了 · 提 4 个 plan |
| 2 | 「按推荐来」 | C1（tutorial 76 files 1h）+ C2（checkpoint dev）并行 |
| 3 | （build 完成通报）| install 新 binary · 写归档 |

## 改动要点

### 新增
- `VectorStore::persist_checkpoint` trait 默认方法
- `InMemoryVectorStore::persist_checkpoint` override
- `Indexer::index_directory_with_checkpoint(source, every, path)`
- CLI `Cmd::Index.checkpoint_every_files: usize` 字段 + `cmd_index` 参数

### 修改
- `Indexer::index_directory` 改成调新方法 `every=0, path=None`（兼容）

### 不动
- BM25 commit 路径（已 auto）
- LanceVectorStore（默认 no-op）
- Hit struct（Round 52 加的 vector_score / bm25_score 不动）

## 验证结果

```bash
cargo build --release ...,onnx     # ✓ 15m14s
~/.local/bin/arkui-rag index --help | grep checkpoint   # ✓ 参数显示
```

下一轮用新 binary 跑 Phase B v3 application-dev 全量 · checkpoint 默认 20 · 死了不丢全部。

## 残留 / 下一轮

- [x] VectorStore trait persist_checkpoint 加
- [x] InMemoryVectorStore override save_to
- [x] Indexer 加 with_checkpoint 方法
- [x] CLI --checkpoint-every-files 参数
- [x] 新 binary install + codesign
- [ ] **C1 tutorial 76 files build 完成**（用旧 binary · 无 checkpoint · 是个小子集风险低）
- [ ] **C3 = Phase B v3 application-dev 全量**：用新 binary checkpoint=20 · 死了重启不丢全部
- [ ] **resume from checkpoint**：当前死后重启需要 cli 知道"从 N file 起跳过已跑的"· Round 56 候选
- [ ] **MetadataStore::persist_checkpoint 也加**：让 chunk metadata 也 checkpoint（当前 only 向量）
- [ ] **BM25 commit 时机**：Tantivy 内部已 commit · checkpoint 之后立即可 query
- [ ] **CI workflow corpus-build.yml 加 checkpoint**：CI 全量 3.2h build 死了也不丢
