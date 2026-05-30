# STATUS — onnx-rc12-api-fix

> 配套 feature log：`feedback/features/rag4arkui-core/41-2026-05-30-onnx-rc12-api-fix.md`
> 日期：2026-05-30

---

## 当前状态

**Phase 2 完成 · task #87 完全解锁**。47 个 rc.4 → rc.12 API 漂移错全清 · `cargo build --features onnx` 真编出 binary。

本阶段交付：
- `crates/arkui-rag-embedding/src/onnx.rs` 适配 rc.12（135 → 168 行）
- `crates/arkui-rag-embedding/src/reranker_onnx.rs` 同款（131 → ~150 行）
- 编译验证：47 → 8 → 2 → 0 error · `cargo build --features onnx` ✓ 5.65s
- 回归验证：workspace 71 passed / 0 failed · 默认 features `cargo check` ✓ 0.40s

意义：ROADMAP 上挂了 ~15 轮的 task #87「Day 20c blocker」**正式解锁**。从「ort 整链路 broken · 不知道怎么修」到「`cargo build --features onnx` 真出 binary · 等用户接真模型」。

距离 MVP 100% 只差 Phase 3-4（用户下载真模型 + agent 质量评估）。

## 输入契约

### Cargo 命令现在可用

```bash
# 之前（Round 40 Phase 1 前）
cargo check -p arkui-rag-embedding --features onnx
# error: could not compile `ort` (lib) due to 1 previous error  ← VitisAI bug

# Phase 1 后
cargo check -p arkui-rag-embedding --features onnx
# error: could not compile `arkui-rag-embedding` (lib) due to 47 errors  ← 我们代码 API 漂移

# Phase 2 后（本轮）
cargo check -p arkui-rag-embedding --features onnx
# Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.19s  ✓
cargo build -p arkui-rag-embedding --features onnx
# Finished `dev` profile  in 5.65s  ✓ ort 真链接
```

### API 代码契约（rc.4 → rc.12）

| 维度 | rc.4 写法 | rc.12 写法 |
|---|---|---|
| Error 处理 | `Session::builder()?` | `.map_err(ort_err("..."))` |
| `commit()` | `?` | `commit()`（返回 bool · 不是 Result）|
| `Tensor::from_array` | `from_array(arr)?` | `from_array((shape, data))` |
| `ort::inputs!` | `inputs![...]?` | `inputs![...]`（宏返回 Vec）|
| `try_extract_tensor` | 返回 ArrayView | 返回 `(&Shape, &[f32])` 元组 |
| `Shape::dims()` | 有 | 无 · 用 `Deref<Target = [i64]>` |
| `Session::run` | `&self` | `&mut self` · 用 `Mutex<Session>` |

### 不变项

- onnx feature 仍 opt-in
- Embedder / Reranker trait 接口完全不变（`async fn encode(&self, ...) -> Result<...>`）
- BGE-M3 / BGE-Reranker-v2-m3 模型路径约定不变
- 索引格式 / MCP 工具签名不变
- CLI `--embedder onnx --model-path ...` 参数不变

## 输出契约

### 编译产物

```
crates/target/debug/libarkui_rag_embedding-*.rlib  ← onnx feature 编出来的产物
                              + 静态链接的 ONNX Runtime（download-binaries）
```

binary size 增加：默认无 ort 时 cli 10.7MB · 含 onnx 后预计 50-100MB（含 ONNX Runtime C++）。

### 等 Phase 3 后输出（用户下载真模型）

```bash
$ arkui-rag index --source corpus --embedder onnx \
    --model-path ~/.arkui-rag/models/bge-m3 --bm25 tantivy

✅ 索引完成
   embedder    : bge-m3 (1024-dim)        ← 不再 mock-384
   dim         : 1024
   files       : ...
   chunks      : ...
```

```bash
$ arkui-rag query --text "@State 双向绑定" --embedder onnx \
    --model-path ~/.arkui-rag/models/bge-m3 \
    --index-path ~/.arkui-rag/index.json --bm25 tantivy -k 3

# [1] mapping-state.md · L24-34 · score=0.85   ← 真语义命中 · 不再 0.0164 平均分
# [2] mapping-list.md  · L17-30 · score=0.42
# [3] mapping-benchmark.md · L287-291 · score=0.31
```

## 验证手段

### Agent 已验证（本轮）

```bash
cd crates
cargo check -p arkui-rag-embedding --features onnx     # ✓
cargo build -p arkui-rag-embedding --features onnx     # ✓ 5.65s
cargo check --workspace                                  # ✓ 0.40s
cargo test --workspace --lib                             # 71 passed / 0 failed
```

### 用户验证（Phase 3）

```bash
# 1. 下载真模型（首次几百 MB · 网络好 ~5 分钟）
arkui-rag corpus model-pull bge-m3

# 2. 编 cli 含 onnx feature
cd crates && cargo build --release -p arkui-rag-cli --features http,mcp,lsp,tantivy,typescript,corpus-pull,onnx

# 3. 装到 PATH（按 Round 37 同样的 make install · 加 onnx 参数）
# 或手动 cp + codesign

# 4. 真索引
arkui-rag index --source <你的 corpus> \
    --embedder onnx --model-path ~/.arkui-rag/models/bge-m3 \
    --index-path ~/.arkui-rag/index-onnx.json --bm25 tantivy

# 5. 真检索 + 对比
arkui-rag query --text "@State 双向绑定" \
    --embedder onnx --model-path ~/.arkui-rag/models/bge-m3 \
    --index-path ~/.arkui-rag/index-onnx.json --bm25 tantivy -k 3
# 期望 Top-1 是 mapping-state.md · 不是 mapping-benchmark.md（mock 时的错排）
```

### 自动化（Phase 4）

```bash
arkui-rag eval --queries corpus/_eval/queries.yaml \
    --index-path ~/.arkui-rag/index-onnx.json --bm25 tantivy \
    --embedder onnx --model-path ~/.arkui-rag/models/bge-m3 \
    --report-path /tmp/eval-onnx.md
# 期望 recall@5 显著高于 mock embedder（mock 时为 0 · onnx 期望 0.5+）
```

## 与上一阶段的关联性

| Round | 阶段 | 解决 |
|---|---|---|
| 40 | task #87 Phase 1 | ort 库 broken 根因（load-dynamic + vitis bug）+ Cargo.toml 改 + 概念归档 |
| **41（本轮）**| task #87 Phase 2 | rc.4 → rc.12 API 漂移修（47 → 0 错）|
| ⏳ Phase 3 | 用户做 | 下载真模型 + 端到端测试 |
| ⏳ Phase 4 | agent 做 | mock vs onnx 质量评估 |

Phase 1 + 2 = 「让 binary 编出来」· Phase 3-4 = 「让 binary 真用上」。

兼容性：
- 默认 features 不受影响 · workspace check / build / test 全过
- onnx feature 用户：API 漂移已修 · 新构建产物可用
- 旧索引（mock embedder 产）跟新 binary 兼容（索引格式不变）· 但 mock vs onnx 索引不能混（embedder_model_id 不同）

破坏性变更：无（onnx feature 之前根本编不过 · 没有「破坏」可言）。

## 完成度 / 下一阶段

### 达成度

| 项 | 状态 |
|---|---|
| onnx.rs 适配 rc.12 | ✅ |
| reranker_onnx.rs 适配 rc.12 | ✅ |
| 47 → 0 errors | ✅ |
| `cargo build --features onnx` 真链接 | ✅ |
| workspace tests 无回归 | ✅ |
| 默认 features 不受影响 | ✅ |
| 双轨归档 + STATUS | ✅ |
| Phase 3 用户跑真模型 | ⏳ |
| Phase 4 mock vs onnx 质量评估 | ⏳ |

### 下一阶段建议

立即（用户 · Phase 3 必做）：
1. 跑 `arkui-rag corpus model-pull bge-m3`（首次 ~2GB · 网络好 5 分钟）
2. 重 build cli 含 onnx feature
3. 跑真索引 + 真检索 · 看 Top-1 是否是预期文档

短期（agent · Phase 4 · 1 round）：
- `arkui-rag eval` 在 corpus/_eval/queries.yaml 跑 mock 和 onnx 两次 · 输出对比 markdown
- 更新 `docs/concepts/mvp.md`「~92%」→「100%」
- 多处 STATUS / ROADMAP / USER-VERIFICATION 把「task #87 阻塞」描述更新为「已解锁」
- `make install` install-binary.sh 加 `--with-onnx` flag 自动 build + 装 onnx 版 binary

中期：
- Reranker 端到端（onnx_reranker 已修但还没接 CLI · 看 Round 5 Reranker 主线工作进度）
- CI 加 `cargo build --features onnx` matrix step
- 性能 benchmark：mock 索引 1000 chunks / onnx 索引同 1000 chunks · 看 throughput 差异

长期：
- 看 ort 何时出真正 stable 2.0 · 升级再 commit 一次（可能还要修 API 漂移）
- 评估 candle 替代 ort 的可能性（Round 39 备选 C）· 看 1.0 后是否值得切换
