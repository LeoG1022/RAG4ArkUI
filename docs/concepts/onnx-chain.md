# ONNX 链路

> 状态：当前**未真活** · ort 2.0 RC 编译 broken · 见 task #87 · 本文档为决策梳理快照（Round 40）。

## 一句话

把文本（query / chunk）经 BGE-M3 ONNX 模型推理为定长向量（384 或 1024 维 float32）· 给向量检索（cosine 相似度）用。

## 业界用法

ONNX（Open Neural Network Exchange）= 跨框架的模型格式 + Runtime · 让 PyTorch / TensorFlow / JAX 训练的模型能在 C++ / Rust / Web / 移动端**统一推理**。

| 库 | 出品 | 定位 |
|---|---|---|
| **ONNX Runtime** | Microsoft | 业界事实标准 · C++ 实现 · 多语言 binding |
| **ort** | pykeio | Rust crate 包 ONNX Runtime · 主流 Rust 选择 |
| **candle** | HuggingFace | Rust 原生推理（不依赖 ONNX Runtime）· 直接读 safetensors / GGUF |
| **sherpa-onnx** | k2-fsa | 语音场景 ONNX Runtime wrapper · 跨平台 |

RAG / embedding 场景几乎都用 ONNX Runtime（或 candle）跑：BGE-M3 / BGE-Reranker / E5 / Qwen-Embedding 等开源 embedding 模型都有 ONNX 版本发布。

业界惯用栈：

```
HuggingFace 训练
    ↓ 导出 .onnx + tokenizer.json
ONNX Runtime（或 candle）加载
    ↓ encode(text) → Vec<f32>
向量库（FAISS / Milvus / LanceDB / Pinecone）
    ↓ ANN 检索
Top-K 结果
```

## 本项目里怎么用

### 端到端链路位置

```
用户 query「ArkUI-X 双向绑定怎么写」
    ↓
[A] Embedder.encode("...")  ← ONNX 链路在这里
    现状: MockEmbedder 哈希假向量 (384 维)
    目标: OnnxEmbedder BGE-M3 真语义向量
    ↓ Vec<f32; 384 或 1024>
[B] HybridRetriever
    ├── 向量检索（cosine · InMemory 或 LanceDB）
    └── 关键词检索（Tantivy BM25）
    ↓ RRF 融合
[C] Top-K hits
    ↓
[D] Reranker（可选）── 也是 ONNX 链路
    ↓ Cross-encoder 二阶段重排
[E] Final results 给 Claude / opencode 等 agent
```

ONNX 链路涉及两个独立模型推理：

| 用途 | 模型 | 输出 | crate |
|---|---|---|---|
| **Embedder** | BGE-M3 | 384 / 1024 维向量 | `arkui-rag-embedding/src/onnx_embedder.rs` |
| **Reranker** | BGE-Reranker-v2-m3 | 相关性 score | `arkui-rag-embedding/src/onnx_reranker.rs` |

### 现状盘点（截至 Round 39）

**代码已写好** · 全 ~501 行 ONNX 代码 ready：

```
crates/arkui-rag-embedding/src/
├── onnx.rs              135 行 · EmbeddingModel（BGE-M3 推理 + mean_pooling + l2_normalize）
├── onnx_embedder.rs     107 行 · async Embedder trait 包装
├── onnx_reranker.rs     128 行 · Cross-encoder rerank 逻辑
├── reranker_onnx.rs     131 行 · BGE-Reranker-v2 推理
└── mock.rs               93 行 · MockEmbedder 哈希假向量（当前默认）
```

**broken 在 ort 库自身**（不是我们代码）：

```
$ cargo check -p arkui-rag-embedding --features onnx
error[E0609]: no field `SessionOptionsAppendExecutionProvider_VitisAI`
              on type `&'static OrtApi`
error: could not compile `ort` (lib) due to 1 previous error
```

ort 2.0.0-rc.4（我们 pin 的版本）内部代码引用了 VitisAI execution provider 字段 · 但底层 ONNX Runtime C 头里不存在 · ort 自身编译失败。

### 决策选项（Round 40 落地）

| 方案 | 工作量 | 风险 | 长期 |
|---|---|---|---|
| **A 升 ort 到最新 RC** | 1 天 | ⭐⭐⭐ | 看 ort 官方何时出 stable |
| B 退 ort 1.16 稳定版 | 2-3 天 | ⭐⭐ | 1.x → 2.x 大版本不兼容 · 重写 501 行 |
| C 换 candle | 4-5 天 | ⭐⭐⭐⭐ | 长期最优 · HuggingFace 原生 Rust · 一次到位 |
| D 接 OpenAI Embeddings API | 0.5 天 | ⭐ | 偏离「本地优先」核心 · 不归 MVP 范围 |

**Round 40 决策**：用户选 A（先试升 ort 最新 RC · 不行降级转 C）。

### 配套基础设施（已 ready）

ONNX 链路不只是 embedder 推理 · 还有：

- **模型下载**：`arkui-rag corpus model-pull bge-m3`（Round 21b 已实装 · 共用 corpus pull tar.gz 基础设施）
- **本地路径约定**：`~/.arkui-rag/models/bge-m3/{model.onnx, tokenizer.json}`
- **CLI 转发**：`arkui-rag index --embedder onnx --model-path ~/.arkui-rag/models/bge-m3`（Round 5 已实装）
- **三协议 server 路径**：`arkui-rag serve --mcp --embedder onnx ...` 同步生效

也就是说 · 一旦 A/B/C 任一方案让 OnnxEmbedder 编出来 · 整条链路立刻可用。

## 类比

| 类比 | 角色 |
|---|---|
| ONNX | Java 的 `.class` 字节码 —— 跨平台中间表示 |
| ONNX Runtime | JVM —— 跨平台运行字节码 |
| ort（Rust）/ candle | OpenJDK / GraalVM —— JVM 的不同实现 |
| BGE-M3 模型 | 一个 jar 包 —— 拿来即用的预编译资产 |
| MockEmbedder | 占位实现 —— 接口对但功能假 · 像 Java 里 `throw new NotImplementedException()` |

或者：

| 类比 | 角色 |
|---|---|
| Embedder | Google Translate API |
| query/chunk 文本 | 待翻译的句子 |
| Vec\<f32; 384\> | 翻译结果（向量空间里的「坐标」）|
| cosine 相似度 | 两个坐标的距离 |
| BGE-M3 vs Mock | 「真翻译」vs「随机生成数字」|

## 相关链接

- 现状代码：[`crates/arkui-rag-embedding/src/`](../../crates/arkui-rag-embedding/src/)
- 技术方案 §7.2：[`docs/RAG4ArkUI-完整技术方案.md`](../RAG4ArkUI-完整技术方案.md)（onnx.rs 源头）
- Task #87 跟踪：本文档「现状盘点」节
- 业界对比：[`docs/concepts/tree-sitter.md`](tree-sitter.md)（类似「跨语言 AST parser」选型）
- 决策上下文：`feedback/features/rag4arkui-core/40-2026-05-30-onnx-chain-decision.md`（Round 40 归档）
