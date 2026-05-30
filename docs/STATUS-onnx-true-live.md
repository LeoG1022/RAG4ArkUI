# STATUS — onnx-true-live

> 配套 feature log：`feedback/features/rag4arkui-core/42-2026-05-30-onnx-true-live.md`
> 日期：2026-05-30

---

## 当前状态

**task #87 完全解锁 · MVP 完成度 92% → 100%** ✅

本阶段交付：
- Phase 3（用户）：下载 BGE-M3 ONNX 真模型（2.2GB · hf-mirror）· 6 文件齐全
- Phase 3（agent）：编 onnx binary（28MB · 含 ONNX Runtime 静态链接 · 44.5s）· 真索引（dim=1024 · 107 chunks · 144s）· 真检索 Top-1 完美命中
- Phase 4 量化对比：mock 3/6 → onnx **6/6** Top-1 命中率
- 多文档同步：mvp.md / onnx-chain.md / ROADMAP.md / USER-VERIFICATION.md 4 处「task #87 阻塞」→「已解锁」

意义：ROADMAP 上挂了 15+ 轮的 task #87 终于真正画上句号。从「ort 整链路 broken」（Round 40 起）→「ort 编过 + 我们代码 47 错」（Round 41 起）→「cargo build 真出 binary」（Round 41 末）→「用户下载真模型 + Top-1 100% 真语义命中」（本轮 Round 42）。

工程层 MVP 100%。剩下两项是用户操作（task #84 推 master 触发 book.yml · task #85 push v1.0.0 tag）· 不算工程范围。

## 输入契约

### 用户做的 Phase 3

```bash
# 1. 下载真模型（hf-mirror · 国内能爬墙环境最稳）
mkdir -p ~/.arkui-rag/models/bge-m3
cd ~/.arkui-rag/models/bge-m3
BASE="https://hf-mirror.com/BAAI/bge-m3/resolve/main/onnx"
curl --progress-bar -C - -L -o model.onnx_data "$BASE/model.onnx_data"  # 2.1GB
curl ... 其它 5 文件

# 验证：
ls -lh ~/.arkui-rag/models/bge-m3/
# model.onnx_data 2.2G · tokenizer.json 17M · sentencepiece.bpe.model 4.9M
# model.onnx 708K · Constant_7_attr__value 65K · config.json 698B
```

### Agent 做的 Phase 3 后续 + Phase 4

```bash
# 编 cli 含 onnx feature
cd crates && cargo build --release -p arkui-rag-cli \
    --features http,mcp,lsp,tantivy,typescript,corpus-pull,onnx

# 真索引
arkui-rag index --source ... \
    --embedder onnx --model-path ~/.arkui-rag/models/bge-m3 \
    --index-path ~/.arkui-rag/index-onnx.json --bm25 tantivy

# 真检索
arkui-rag query --text "..." \
    --embedder onnx --model-path ~/.arkui-rag/models/bge-m3 \
    --index-path ~/.arkui-rag/index-onnx.json --bm25 tantivy -k 3
```

### 不变项

- 默认 features 不含 onnx · master 默认 build 不受影响
- mock embedder 继续作 default fallback · 旧索引 (mock-384) 仍兼容
- CLI / API / 索引格式 / MCP 工具签名 全部不变

## 输出契约

### Phase 3 索引输出

```
✅ 索引完成
   embedder    : bge-m3            ← 不再 mock-384
   dim         : 1024              ← 不再 384
   vector      : memory
   bm25        : tantivy
   files       : 11
   chunks      : 107
   skipped     : 0
   elapsed_ms  : 144445             ← 真推理就这速度
   saved to    : /Users/leo/.arkui-rag/index-onnx.json
```

### Phase 4 量化对比

| 指标 | Mock 384-dim | **BGE-M3 1024-dim** |
|---|---|---|
| Top-1 命中 | 3/6（50%）| **6/6（100%）** |
| Latency p50 | ~48ms | ~4000ms |
| Latency p95 | ~70ms | ~4300ms |
| 推理硬件 | CPU + hash | CPU only（CoreML EP 未启 · Round 43 候选） |

**真语义把所有相关段聚到 Top-K**（同一文件多 heading）· mock 是分散乱序。

## 验证手段

### 用户手动（已完成）

```bash
ls -lh ~/.arkui-rag/models/bge-m3/   # 6 文件齐全 + 总 ~2.2GB
arkui-rag query --text "@State 双向绑定" --embedder onnx ...   # Top-1 = mapping-state.md
```

### 自动化（agent 跑过 · 6 query 矩阵）

略 · 见 feature log 「实测结果」。

## 与上一阶段的关联性

| Round | 阶段 | 解决 |
|---|---|---|
| 40 | Phase 1 | ort 库自身 broken（load-dynamic 撞 vitis bug）· 改 Cargo.toml |
| 41 | Phase 2 | 47 个 rc.4 → rc.12 API 漂移 · `cargo build --features onnx` 真出 binary |
| **42（本轮）**| **Phase 3-4** | **用户下载真模型 + agent 真索引/检索 + 量化对比 + 多文档同步** |

Phase 3-4 是「让 binary 真用上」· 配合 Phase 1-2 的「让 binary 编出来」· 形成完整 task #87 闭环。

兼容性：
- mock embedder 继续可用 · 用户切换无破坏
- 旧 mock 索引（mock-384）vs 新 onnx 索引（bge-m3-1024）通过 embedder_model_id 区分 · 不会错用
- query 时 model_id 与索引 model_id 不匹配会清晰报错

破坏性变更：无。

性能：
- 索引时间：mock ~1s → onnx ~144s（100x 慢 · 但一次性 · 真索引就这速度）
- query 时间：mock ~48ms → onnx ~4000ms（80x 慢 · 但质量翻倍）
- 优化方向：启 ort/coreml feature → 预期 < 500ms（Round 43+）

## 完成度 / 下一阶段

### 达成度

| 项 | 状态 |
|---|---|
| Phase 3 模型下载（用户）| ✅ |
| Phase 3 真索引（agent）| ✅ |
| Phase 3 真检索 Top-1 命中（agent）| ✅ |
| Phase 4 量化对比 6 query | ✅ |
| 4 处文档同步「task #87 解锁」 | ✅ |
| 双轨归档 + STATUS（本文件）| ✅ |
| **B 步骤**：cp binary → ~/.local/bin + 改三端 MCP + 重启 | ⏳（下一步） |

### 下一阶段建议

立即（本会话继续 · B 步骤）：
- `cp crates/target/release/arkui-rag ~/.local/bin/arkui-rag`（28MB 含 onnx）
- `codesign --force --sign - ~/.local/bin/arkui-rag` 重签防 provenance
- 改三端 MCP 配置加 `--embedder onnx --model-path ~/.arkui-rag/models/bge-m3 --index-path ~/.arkui-rag/index-onnx.json`
- 重启 Claude CLI / Desktop / opencode
- 验证：Claude chat 调 arkui_search_docs · 返回真语义 hits

短期（agent · 1-2 round）：
- **启 `ort/coreml` feature**：4s → <500ms · Apple Silicon GPU 加速 · 用户体验质变
- **写 mapping-* corpus 真 GT 新 eval fixture** 替代 corpus/_eval/queries.yaml
- **CI 加 onnx build matrix** 防回归
- **README 加「真语义 RAG」徽章** v1.0.0 release 前

中期：
- 跑 100k chunks 大规模 benchmark · 看 onnx 真活的瓶颈在哪
- Reranker 真活集成（onnx_reranker 已修但 CLI 还没接 · Round 5 工作延续）
- 考虑 quantized BGE-M3 ONNX 版本 · binary 从 2.2GB → 600MB 左右

长期：
- 评估 candle 替代 ort 的可能性（一直挂残留）· 看 1.0 后是否值得切
- 加更多 embedder 支持（Qwen3-Embedding-0.6B / e5 等）· 模型选型多样化
- IDE 插件（DevEco / VSCode）接入 LSP server · 用户在 IDE 内拿真语义代码补全
