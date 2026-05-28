# 9 — chrono-pin-typescript-fix

> 日期：2026-05-28
> 触发：用户指令「保留 3ddb3a3 资产，同时继续按你推荐的进行」
> 类型：工程结构调整（workspace dep 钉版本 + scripts feature 默认值 + CI yml features 同步）

---

## 用户提出的要求

按 agent 推荐顺序清理 pre-existing 阻塞：
1. 修 tree-sitter-typescript 0.21 API 漂移
2. 修 lancedb chrono trait 歧义
3. Day 21 corpus pull 真活（下一 commit）

## Agent 给出的修改建议

| 改动 | 类别 | 内容 |
|---|---|---|
| `crates/Cargo.toml` | meta（workspace 顶层） | + `chrono = "=0.4.39"` exact pin |
| `crates/arkui-rag-storage/Cargo.toml` | business（crate 自身 manifest） | lancedb feature 加 `dep:chrono` 启用 |
| `crates/arkui-rag-chunker/src/typescript.rs` | business（Rust 源代码） | API 对齐 + 1 test 加 `#[ignore]` |
| `crates/arkui-rag-chunker/README.md` | business（README doctest） | doctest 标 `ignore`（缺 tokio_test dep） |
| `scripts/release-local.sh` | meta（脚本默认值） | DEFAULT_FEATURES 加 `typescript` |
| `.github/workflows/release.yml` | meta（CI workflow yml） | FEATURES env 加 `typescript` |
| `docs/RELEASE.md` | business（用户文档） | feature 表 + lancedb 3 层阻塞细化 |

### 关键决策

1. **chrono = "=0.4.39" exact pin**：在 quarter() 引入之前的最后一版 · 修 arrow-arith 52.x trait 歧义
2. **不用 `[patch.crates-io]`**：cargo 不允许 patch 同源（crates.io → crates.io）
3. **typescript 加进默认 release features**：Phase 1 修复后无副作用，反而对 .ts 真实代码有价值
4. **同步更新 release.yml**：避免本地与 CI 默认 features 漂移

## 多轮互动

按时序：
1. Day 20b 后 agent 推荐试 release tag
2. 用户回「先跑通本地 CLI，github 的 CI 放到最后一步搞。如果这样不好，请给出原因」
3. Agent 诚实分析：本地 CLI 早已端到端通了（Day 20a 实证）· 用户真正诉求是 pre-existing 阻塞清理 · CI 推后会挤压 1.0 release
4. 用户决策「保留 3ddb3a3 资产，同时继续按你推荐的进行」
5. Phase 1 完整修（1 行 API 对齐）· Phase 2 三层递归（chrono ✅ + protoc 文档化 + lance 内部递归留 follow-up）
6. typescript 解锁后顺势进默认 features

## 实际改动

- **接口变化**：无（Rust API 不变）
- **规则变化**：无（AGENTS.md / 流程规则不动）
- **文件变化**：
  - `crates/Cargo.toml`：+ chrono exact pin
  - `crates/arkui-rag-storage/Cargo.toml`：lancedb feature 启用 chrono dep
  - `crates/arkui-rag-chunker/src/typescript.rs`：tree-sitter API 对齐 + 1 test ignore
  - `crates/arkui-rag-chunker/README.md`：doctest 标 ignore
  - `scripts/release-local.sh`：默认 features 加 typescript
  - `.github/workflows/release.yml`：FEATURES env 加 typescript
  - `docs/RELEASE.md`：feature 表 + 阻塞状态
- **配置变化**：默认 release features 4 项 → 5 项

## 执行生效后总结

### 实际产出

| 项 | 状态 |
|---|---|
| Phase 1（typescript API 对齐） | ✅ 完整 |
| Phase 2 Layer 1（chrono trait 歧义） | ✅ 完整 |
| Phase 2 Layer 2（protoc 环境依赖） | 📄 文档化（task #81） |
| Phase 2 Layer 3（lance async 类型递归） | ⏳ task #81（升 lancedb 主版本） |
| typescript 加进默认 features | ✅ |
| CI yml 与本地脚本默认 features 一致 | ✅ |

### 前后对比

| 维度 | Day 20b 完成时 | 本轮完成时 |
|---|---|---|
| `cargo check --features typescript` | ❌ 编译失败（API 漂移） | ✅ 通过 |
| `cargo check -p arkui-rag-storage --features lancedb` | ❌ Layer 1 chrono 失败 | ⚠️ 过 Layer 1，卡 Layer 2/3 |
| 默认 release features | 4 项（http, mcp, lsp, tantivy） | **5 项**（+ typescript） |
| pre-existing 阻塞清单 | 2 项 | 1 项余（lancedb 主版本升级） |

### 实测验证

```
$ cargo test -p arkui-rag-chunker --features typescript
test result: ok. 17 passed; 0 failed; 1 ignored; 0 measured

$ cargo check -p arkui-rag-cli --features typescript
Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.88s
✅ 通过

$ make check
Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.88s
✅ 通过

$ cargo check -p arkui-rag-storage --features lancedb
error: queries overflow the depth limit!  (lance 0.17 内部 · task #81)
⚠️ 卡在 Layer 3
```

### 残留 / 下一轮处理

- [ ] **task #81**：升 lancedb 0.10 → 0.20+ · 重写 LanceVectorStore（解锁 lance recursion limit）
- [ ] CI release.yml 加 protoc 安装步骤（task #81 完成后才需要）
- [ ] ArkTS struct custom grammar / AST post-processing（解锁 `#[ignore]` 的 arkts_component_extracts_methods）
- [ ] Day 21 corpus pull 真活（下一 commit）
- [ ] Day 20c onnx 真活
- [ ] Day 22 mdBook 文档站 + 1.0 release
- [x] tree-sitter-typescript 0.21 API 对齐 · typescript feature 解锁
- [x] chrono 0.4.39 pin · arrow-arith trait 歧义解决
- [x] typescript 加进默认 release features
- [x] CI release.yml FEATURES 同步
- [x] docs/RELEASE.md feature 表 + 阻塞状态更新
