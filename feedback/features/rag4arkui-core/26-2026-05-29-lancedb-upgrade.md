# 26 — lancedb-upgrade

> 日期：2026-05-29
> 涉及代码：
> - `crates/Cargo.toml`：bump lancedb 0.10 → 0.30 · arrow 52 → 58 · 移除 chrono pin
> - `crates/arkui-rag-storage/Cargo.toml`：lancedb feature 移除 chrono dep（不再需要）
> - `crates/arkui-rag-storage/src/lancedb_store.rs`：API 适配（`RecordBatchReader` trait cast）+ `open(dim=0)` 自动从 schema 推导 dim + `read_vector_dim_from_schema` helper + 3 个新单测
> - `docs/RELEASE.md`：lancedb 行从 ❌ 改为 ✅
> 类型：bug 修复 + 主版本升级（task #81 · 解锁 lancedb feature 完整可用）

## 本轮目标

清除 Day 21b 残留的 **lancedb Layer 3 阻塞**（`lance 0.17` 内部 async 类型递归超 rustc 默认深度限制），同时解锁 LanceDB 嵌入式向量库的完整端到端：

```bash
arkui-rag index --vector lancedb     # ✅ 真活
arkui-rag query --vector lancedb     # ✅ KNN + FilteredRead 真活
```

完成后 default release features 暂不加 lancedb（binary 体积 11 MB → 95 MB · 太大 · 需用户显式 `--features lancedb` 启）。

## Plan

> 必备节（FAIL 级 M-FEATURE-PLAN）

### 主版本升级清单

| 依赖 | Day 21b | task #81 |
|---|---|---|
| `lancedb` | 0.10 | **0.30**（21 minor 大跨越） |
| `arrow-array` | 52 | **58**（6 major） |
| `arrow-schema` | 52 | **58** |
| `chrono` | `=0.4.39`（exact pin） | **`0.4`**（移除 pin · 新版 arrow-arith 已修 quarter() 歧义） |

### API 破坏性变更（仅 2 处需要适配）

#### 1. `RecordBatchIterator` → `Box<dyn RecordBatchReader + Send>` cast

lancedb 0.30 引入新 `Scannable` trait · `Box<RecordBatchIterator<...>>` 不再自动满足。手动 cast：

```rust
// Before (0.10)
conn.create_table(TABLE_NAME, Box::new(iter)).execute()
table.add(Box::new(iter)).execute()

// After (0.30) · 显式 cast 到 trait object
let reader: Box<dyn RecordBatchReader + Send> = Box::new(iter);
conn.create_table(TABLE_NAME, reader).execute()
table.add(reader).execute()
```

#### 2. `LanceVectorStore::open(dim=0)` 自动从 schema 推导

**端到端跑出 bug**：CLI query 路径用 `dim=0` 占位 open，但 lancedb_store 直接当 0 用 → 报 `query dim 1 != store dim 0`。

修法：`open` 时
- table 已存在 → 从 Arrow schema 反推 dim（新 helper `read_vector_dim_from_schema`）
- table 不存在 + dim=0 → 报错（不能无中生有）
- table 不存在 + dim>0 → 建空表用此 dim
- table 已存在 + dim>0 且与 schema 不一致 → 报错（防错配）

### 三层 lancedb 阻塞 final 进度

| Layer | Day 21b 状态 | task #81 状态 |
|---|---|---|
| 1. chrono trait method 歧义 | ✅ 修了（chrono 0.4.39 pin） | ✅ pin 移除（arrow-arith 58 已修） |
| 2. lance build 需 protoc | 📄 文档化（brew install） | 📄 仍需用户预装 protoc |
| 3. lance 0.17 async 类型递归超限 | ❌ 致命 | ✅ **修了**（升 lance → 自带的现代版本无递归 bug） |

### 替代方案权衡（被否）

- 备选 1：保持 lancedb 0.10 · 等 upstream 自己修 lance recursion
  - 否决：时间线不可控 · 已知 bug 早就该升
- 备选 2：升 lancedb 但保持 chrono pin
  - 否决：arrow-arith 58 已修 · pin 多余反而限制未来
- 备选 3：把 lancedb 加进默认 release features
  - 否决：binary 11 MB → 95 MB（lance 引擎本身 ~84 MB） · 大多数用户用 in-memory + Tantivy 已够
- 备选 4：用 qdrant / chromadb 替代 lancedb
  - 否决：lancedb 已 embedded（vs qdrant 需 server）· 完整切换是 4-6h 重大工程

### 测试策略

1. ✅ `cargo check -p arkui-rag-storage --features lancedb`（升级后能编）
2. ✅ `cargo test -p arkui-rag-storage --features lancedb`（10 → 13 测过 · 加 3 新 dim auto-detect 测）
3. ✅ `cargo build --release -p arkui-rag-cli --features corpus-pull,tantivy,lancedb`（CLI 端编通）
4. ✅ 端到端：`index --vector lancedb` + `query --vector lancedb` 全过（Top-3 正确命中 + lance 内部 KNN+FilteredRead）
5. ✅ 回归：memory backend 仍工作

## 改动要点

> API 选型 / 算法 / 关键决策

### 与 Day 21b 的差异

- crate 数 9（不变）
- CLI 子命令（不变）
- 默认 release features 6 项（不变 · lancedb 仍不进默认）
- binary 大小不变（默认 release 不含 lancedb）
- `cargo build --features ...,lancedb` 时 binary 11 MB → 95 MB
- 新增 3 个单测（13 total）
- 解锁了 `--vector lancedb` 端到端

### 关键决策

1. **bump 到 lancedb 0.30 latest**（不 pin 中间版本）：直接对齐 upstream · 减少未来升级阻力
2. **arrow 52 → 58 同步升级**：lancedb 0.30 要求 · 顺带修了 chrono 歧义
3. **dim auto-detect from schema**：让 CLI load 路径无需用户传 dim · 等价于 SQLite 「打开已有 db 自动读 metadata」
4. **3 个新单测覆盖 dim auto-detect**：`open_with_dim_zero_reads_from_schema` / `open_with_dim_zero_no_table_fails` / `open_dim_mismatch_with_existing_schema_fails`
5. **不加 lancedb 进默认 release**：binary 95 MB 对大多数用户过载（in-memory + Tantivy 已够 · lancedb 是「1k+ chunks 时才用」的可选项）

## 对话摘要

> 必备节（FAIL 级 M-FEATURE-PLAN）

按时序：
1. **Day 20c 探查后 onnx 链路 blocker**（ort 2.0 RC 整链路 broken · 挂 task #87）
2. **Agent 推荐改做 task #81（lancedb 升级）**
3. **用户指令**：「执行 task #81」
4. **Agent 自主决策 6 项**：
   - bump lancedb 0.30 latest（不 pin 中间）
   - arrow 52 → 58 同步
   - chrono pin 移除（试）
   - 2 处 `RecordBatchReader` cast 修编译
   - dim auto-detect from schema 修运行时 bug
   - 3 个新单测覆盖 dim auto-detect
5. **构建调试 2 次**：
   - cargo check：2 个 RecordBatchReader cast 错误 · 修
   - 端到端 query：浮出 dim mismatch · 修 LanceVectorStore::open 加 dim auto-detect
6. **最终端到端通过** · lance 内部 KNN 真活

## 验证结果

- ✅ `cargo test -p arkui-rag-storage --features lancedb`：**13 passed / 0 failed**（10 原有 + 3 新 dim auto-detect）
- ✅ `cargo build --release -p arkui-rag-cli --features corpus-pull,tantivy,lancedb`：通过（3m18s 增量 · 95 MB binary）
- ✅ `arkui-rag index --vector lancedb`：22 chunks → lance dir 写入 134ms
- ✅ `arkui-rag query --vector lancedb`：Top-3 命中正确 · lance 内部跑 `KNNVectorDistance(FilteredRead)`
- ✅ regression `arkui-rag query` memory backend：仍工作

## 残留 / 下一轮

- [ ] 加 `lancedb` 到 `make build-full` / `release-local.sh` full-features 集合？（决策：先不 · 95 MB binary 太大 · 用户显式 `--features lancedb` 开）
- [ ] CI release.yml 加 protoc 安装步骤（如未来要打 lancedb release）：`apt install protobuf-compiler` / `brew install protobuf` / `choco install protoc`
- [ ] mdBook 加 lancedb 真活使用章节
- [ ] benchmark：1k+ chunks 时 lancedb vs in-memory 性能对比（验证升级价值）
- [ ] Day 20c onnx 真活（task #87 ort 链路阻塞 · 需先决策 ort 1.16 vs candle vs sherpa-onnx）
- [ ] Day 17 DevEco Plugin MVP
- [x] task #81：lancedb 0.10 → 0.30 主版本升级
- [x] arrow 52 → 58 同步
- [x] chrono pin 移除
- [x] `RecordBatchReader` cast 修编译
- [x] dim auto-detect from schema 修运行时
- [x] 3 个新单测覆盖
- [x] CLI 端到端 `--vector lancedb` 真活
- [x] docs/RELEASE.md feature 表 ❌ → ✅
