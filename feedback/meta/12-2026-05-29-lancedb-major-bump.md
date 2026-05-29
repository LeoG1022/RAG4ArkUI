# 12 — lancedb-major-bump

> 日期：2026-05-29
> 触发：用户指令「执行 task #81」（升 lancedb 0.10 → 0.30 主版本 · 解锁向量库）
> 类型：工程结构调整（workspace deps 主版本升级 · 移除 chrono pin · feature dep 简化）

---

## 用户提出的要求

继 Day 20c 探查 onnx 链路被 ecosystem-level blocker 拦下后，转去做 task #81 —— 把 lancedb 0.10 升到 0.20+ 主版本，解锁 lance 0.17 内部 async 类型递归 bug。

## Agent 给出的修改建议

| 改动 | 类别 | 内容 |
|---|---|---|
| `crates/Cargo.toml` | meta（workspace 顶层） | bump lancedb 0.10 → 0.30 · arrow-array/arrow-schema 52 → 58 · 移除 `chrono = "=0.4.39"` pin（恢复 `0.4` 范围） |
| `crates/arkui-rag-storage/Cargo.toml` | business（crate manifest） | lancedb feature 移除 `dep:chrono`（不再需要） |
| `crates/arkui-rag-storage/src/lancedb_store.rs` | business（Rust 源） | `RecordBatchReader` trait cast · `open(dim=0)` 自动从 schema 推导 · `read_vector_dim_from_schema` helper · 3 个新单测 |
| `docs/RELEASE.md` | business（用户文档） | lancedb 行从 ❌ 改 ✅ + 注 protoc build 依赖 |

### 关键决策

1. **跨越式升级到 lancedb 0.30**：跳过中间版本 · 一次性对齐 upstream latest
2. **arrow 52 → 58 同步**：lancedb 0.30 强制要求 · 顺带修了 chrono trait 歧义
3. **移除 chrono pin**：新 arrow-arith 已修 quarter() 撞 trait method 的 bug · pin 多余
4. **dim auto-detect from schema**：让 CLI 「打开已有 lance dir 时无需用户传 dim」（等价于 SQLite 自动读 metadata）
5. **不加 lancedb 进默认 release**：binary 11 MB → 95 MB · 对大多数用户过载（in-memory + Tantivy 已够）

## 多轮互动

无 —— 用户给出「执行 task #81」后 agent 自主推进。

构建/运行时调试 3 次（agent 自己修，未涉及用户）：
- cargo check：2 个 RecordBatchReader cast 错误 → 加 trait object 显式 cast
- 端到端 `query --vector lancedb`：dim mismatch（store dim=0 · query dim=1）→ 加 dim auto-detect from schema
- 单测：3 个新 dim auto-detect 测全过

## 实际改动

- **接口变化**：`LanceVectorStore::open(uri, model_id, dim)` 新增「dim=0 自动从 schema 推导」语义（向后兼容 · 不破坏 dim>0 用法）
- **规则变化**：无
- **文件变化**：
  - 修：`crates/Cargo.toml`、`crates/arkui-rag-storage/Cargo.toml`、`crates/arkui-rag-storage/src/lancedb_store.rs`
  - 修：`docs/RELEASE.md`（lancedb 状态 ❌ → ✅）
- **配置变化**：
  - chrono 不再 `=0.4.39` 钉死 · 用 `^0.4` 范围
  - storage Cargo.toml 不再有 chrono optional dep
  - lancedb 主版本号 0.30（vs 之前 0.10）

## 执行生效后总结

### 实际产出

| 项 | 状态 |
|---|---|
| `cargo check -p arkui-rag-storage --features lancedb` | ✅ 通过（vs Day 21b 卡 Layer 3） |
| `cargo test -p arkui-rag-storage --features lancedb` | ✅ **13 passed**（10 原有 + 3 新 dim auto-detect） |
| `cargo build --release -p arkui-rag-cli --features ...,lancedb` | ✅ 95 MB binary（vs 11 MB 无 lancedb） |
| `arkui-rag index --vector lancedb` | ✅ 22 chunks → lance dir 134ms |
| `arkui-rag query --vector lancedb` | ✅ Top-3 正确命中 · lance KNN+FilteredRead 真活 |
| regression memory backend | ✅ 仍工作 |

### 前后对比

| 维度 | Day 21b 完成时 | task #81 完成时 |
|---|---|---|
| lancedb feature | ❌ Layer 3 卡死（lance 0.17 async 递归超 rustc 默认深度） | ✅ **完整真活** |
| `--vector lancedb` 端到端 | ❌ 编不过 | ✅ 真活 |
| LanceVectorStore tests | 0 通过（不能编） | **13 passed** |
| chrono workspace dep | `=0.4.39` exact pin | `0.4` 范围 |
| arrow 版本 | 52.2.0 | 58.x |
| lancedb 版本 | 0.10 | 0.30 |
| docs/RELEASE.md lancedb 状态 | ❌ | ✅ |

### 实测验证

```
$ cargo test -p arkui-rag-storage --features lancedb
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured

$ cargo build --release -p arkui-rag-cli --features corpus-pull,tantivy,lancedb
Finished `release` profile [optimized] target(s) in 3m 18s

$ arkui-rag index --source corpus --bm25 tantivy --vector lancedb
✅ files=3 chunks=22 vector=lancedb bm25=tantivy elapsed_ms=134

$ arkui-rag query --text "@State 双向绑定" --bm25 tantivy --vector lancedb -k 3
plan_summary="Projection(Take(CoalesceBatches(GlobalLimit(LanceFilter(SortExec(TopK)(KNNVectorDistance(FilteredRead)))))))"
✅ Top-3 命中 · [1] mapping-state.md L24-34 "状态选择决策"
```

### 残留 / 下一轮处理

- [ ] **思考**：加 lancedb 进默认 release 吗？（决策：暂不 · 95 MB binary 太大 · 用户显式 `--features lancedb`）
- [ ] CI release.yml 未来加 protoc 安装步骤（如要打 lancedb release artifact）
- [ ] mdBook 加 lancedb 真活使用章节
- [ ] benchmark 1k+ chunks 时 lancedb vs in-memory（验升级价值）
- [ ] Day 20c onnx 链路（task #87 ort 阻塞 · 等架构决策）
- [ ] Day 17 DevEco Plugin MVP
- [x] task #81：lancedb 0.10 → 0.30 主版本升级
- [x] arrow 52 → 58 同步
- [x] chrono pin 移除
- [x] LanceVectorStore API 适配（RecordBatchReader cast + dim auto-detect）
- [x] 3 个新单测覆盖 dim auto-detect
- [x] CLI 端到端 `--vector lancedb` 真活
- [x] docs/RELEASE.md feature 表 lancedb ❌ → ✅
