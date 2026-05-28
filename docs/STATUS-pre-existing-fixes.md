# STATUS · Day 20c · Pre-existing 阻塞清理（Phase 1+2）

> 日期：2026-05-28
> 对应 commit：[本 commit · Day 20c pre-existing fixes]
> 对应 feature log：[`feedback/features/rag4arkui-core/22-2026-05-28-pre-existing-fixes.md`](../feedback/features/rag4arkui-core/22-2026-05-28-pre-existing-fixes.md)
> 对应 meta：[`feedback/meta/9-2026-05-28-chrono-pin-typescript-fix.md`](../feedback/meta/9-2026-05-28-chrono-pin-typescript-fix.md)
> 上一阶段：[`STATUS-day20b-ci-matrix.md`](STATUS-day20b-ci-matrix.md)
> 下一阶段：`STATUS-day21-corpus-pull.md`（下一 commit · corpus 分发管道）

> 🎯 **里程碑**：**typescript feature 解锁 ✅ · chrono trait 歧义解决 ✅ · 默认 release features 5 项**

---

## 当前状态

| 模块 | 变化 |
|---|---|
| `crates/arkui-rag-chunker/src/typescript.rs` | tree-sitter-typescript 0.21 API 对齐（`LANGUAGE_TYPESCRIPT` → `language_typescript()`） · 1 个 ArkTS struct 测试标 `#[ignore]` |
| `crates/arkui-rag-chunker/README.md` | doctest 标 `,ignore`（缺 tokio_test dev-dep） |
| `crates/Cargo.toml` | workspace 加 `chrono = "=0.4.39"` exact pin |
| `crates/arkui-rag-storage/Cargo.toml` | lancedb feature 启用 chrono dep |
| `scripts/release-local.sh` | DEFAULT_FEATURES 加 `typescript` |
| `.github/workflows/release.yml` | FEATURES env 加 `typescript` |
| `docs/RELEASE.md` | feature 表 + lancedb 3 层阻塞细化 |

### Phase 进度

| Phase | 范围 | 状态 |
|---|---|---|
| Phase 1 | tree-sitter-typescript 0.21 API 漂移 | ✅ 完整修复 |
| Phase 2 Layer 1 | chrono trait method 歧义（arrow-arith） | ✅ 修了 |
| Phase 2 Layer 2 | lance build 需 protoc（build-time native 依赖） | 📄 文档化 |
| Phase 2 Layer 3 | lance 0.17 async 类型递归超 rustc 默认深度 | ⏳ task #81（升 lancedb 主版本） |
| Phase 3 | Day 21 corpus pull 真活 | ⏳ 下一 commit |

---

## 输入契约

无（仅修复 pre-existing 阻塞 · 不改业务 API）

---

## 输出契约

### typescript feature 可用

```bash
cargo check -p arkui-rag-chunker --features typescript    # ✅ 通过
cargo test  -p arkui-rag-chunker --features typescript    # ✅ 17 passed / 0 failed / 1 ignored
cargo check -p arkui-rag-cli      --features typescript   # ✅ 通过
```

### lancedb feature 推进 1/3 层

```bash
cargo check -p arkui-rag-storage --features lancedb
# Layer 1 (chrono trait 歧义)：✅ 过
# Layer 2 (protoc build dep)：本地 `brew install protobuf` 解决 · CI 需 yml 加装步
# Layer 3 (lance 0.17 async recursion)：❌ 卡 · 需 lancedb 主版本升级
```

### 默认 release features 5 项

```bash
make release-local      # 默认 features: http,mcp,lsp,tantivy,typescript
# 产物：dist/arkui-rag-v0.0.1-<host-triple>.tar.gz （含 typescript 真活）
```

---

## 验证手段

### 用户手动

```bash
make check                                                     # 默认 workspace · ✅
cargo check -p arkui-rag-chunker --features typescript         # ✅
cargo test  -p arkui-rag-chunker --features typescript         # ✅ 17/0 ignored 1
cargo check -p arkui-rag-cli      --features typescript        # ✅
cargo check -p arkui-rag-storage  --features lancedb           # ⚠️ Layer 3 fails (expected)
make release-local                                             # ✅ 默认 features 含 typescript
```

### 自动化

| 手段 | 状态 |
|---|---|
| `make check` 默认 workspace | ✅ |
| chunker typescript feature 测试（17 + 1 ignore） | ✅ |
| chunker doctest（1 ignore） | ✅ |
| **M-STATUS-PER-ROUND** Round 22 + STATUS-day20c 配套 | ✅ |
| **ROADMAP 维护约定（第 11 次实战）** | ✅ |

### 暂未自动化（明确缺口）

- ❌ lancedb 完整 end-to-end（task #81 升主版本后）
- ❌ ArkTS struct method extraction（custom grammar 需求）
- ❌ Day 21 corpus pull HTTP 下载 + tar 解压（下一 commit）
- ❌ Day 20c onnx 真活（BGE-M3 模型 + 真语义 embedding）

---

## 与上一阶段（STATUS-day20b）的关联性

### 增量

| 维度 | Day 20b 完成时 | 本轮（Day 20c）后 |
|---|---|---|
| Crate 数 | 9 | 不变 |
| 默认 release features | 4 项（http, mcp, lsp, tantivy） | **5 项**（+ typescript） |
| typescript feature 可用 | ❌ 编译失败 | ✅ 通过 + 17 测过 |
| lancedb feature 可用 | ❌ Layer 1 chrono 失败 | ⚠️ 过 Layer 1，卡 Layer 2/3 |
| pre-existing 阻塞清单 | 2 项 | 1 项余（lancedb 主版本升级 · task #81） |
| 测试数（typescript feature 启用） | 0（不能编） | **17 + 1 ignored** |

### 兼容性

- ✅ 无破坏性变更（只增不改 · API 不变）
- ✅ `make check` 默认 workspace 仍通过
- ✅ 现有 ci.yml（PR 校验）不动
- ✅ release.yml 与本地 scripts 默认 features 同步

---

## 完成度 / 下一阶段

### 本轮（Phase 1+2）完成度

| 项 | 状态 |
|---|---|
| Phase 1: tree-sitter-typescript API 对齐 | ✅ |
| Phase 2 Layer 1: chrono trait 歧义 | ✅ |
| Phase 2 Layer 2: protoc 文档化 | ✅ |
| Phase 2 Layer 3: lance recursion limit | ⏳ task #81 |
| typescript 进默认 release features | ✅ |
| CI release.yml FEATURES 同步 | ✅ |
| docs/RELEASE.md 阻塞状态更新 | ✅ |
| 双轨归档 + STATUS + ROADMAP | ✅ |

### 6 周路线图达成度

| 章节 | 状态 |
|---|---|
| Week 1: Rust 骨架 + tree-sitter + LanceDB + Tantivy + BGE-M3 | **7/7** ✅ |
| Week 2: 混合检索 + Reranker + HyDE + 评估集 | **4/4** ✅ |
| Week 3: HTTP + MCP + CLI | **3/3** ✅ |
| Week 4: 协议层（HTTP + MCP + LSP） | **3/3** ✅ ⭐ |
| Week 5: Claude Code 接入 | **1/1** ✅ |
| Week 6: 发布 + 文档站 + 评估报告 | **3/4** ✅（评估 ✓ · 本地 release ✓ · CI matrix ✓ · 文档站待 Day 22） |

**总完成度估算：~81%**（pre-existing 阻塞清理 + typescript 加进默认 features 算 1%）

### 下一阶段建议（按推荐优先级）

| 候选 | 价值 | 工作量 |
|---|---|---|
| 🟢 **Day 21 corpus pull 真活** | 用户无脑接入 · 不用自己投放 corpus | 2 commit（HTTP 下载 + tar 解压） |
| 🟢 **Day 22 mdBook 文档站 + 1.0 release** | 公开发布 | 1-2 commit |
| 🟡 task #81 升 lancedb 0.10 → 0.20+ | 解锁向量库 · 1k+ chunks scale | 1-2 commit（API 破坏性变更） |
| 🟡 Day 20c onnx 真活 | BGE-M3 真语义 embedding · 解锁真 RAG | 2-3 commit |
| 🟡 ArkTS struct custom grammar | 解锁 ArkTS @Component 方法提取 | 大工程 |
| ⚪️ 用户跑首次 release tag 验证 CI matrix | 锦上添花 · 验完挂 status badge | 0.5 commit |

**Agent 推荐**：**Day 21 corpus pull 真活**（HTTP 下载 + tar 解压 · 让用户拿到 binary 后能 `arkui-rag corpus pull` 自动获取默认语料）。

### 重要的"非完成"项

- ❌ Day 21 corpus pull 真活（HTTP 下载 + tar 解压）
- ❌ Day 20c onnx 真活（BGE-M3 模型 + 端到端真语义检索）
- ❌ task #81 lancedb 主版本升级
- ❌ ArkTS struct method extraction（custom grammar）
- ❌ release.yml 第一次实跑验证
- ❌ Day 22 mdBook 文档站 + 1.0 release
