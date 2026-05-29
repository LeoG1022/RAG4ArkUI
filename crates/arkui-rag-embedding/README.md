# arkui-rag-embedding

**定位**：文本嵌入（文本 → 向量）。给索引和检索两路用。

## 当前阶段提供的实现（Day 5）

### Embedder

| 实现 | feature 要求 | 用途 |
|---|---|---|
| `MockEmbedder` | 默认 | 开发期占位，返回确定性伪随机向量；让 cargo check 不依赖 ONNX |
| `EmbeddingModel`（§7.2 verbatim） | `--features onnx` | 底层同步 ONNX 推理 API（直接迁移自方案文档） |
| `OnnxEmbedder`（Day 3 新增） | `--features onnx` | 实现 `Embedder` trait 的 async wrapper，内部 `spawn_blocking` 桥接 |

### Reranker（Day 5 新增）

| 实现 | feature 要求 | 用途 |
|---|---|---|
| `RerankerModel` | `--features onnx` | 底层同步 cross-encoder ONNX 推理（BGE-Reranker-v2-m3） |
| **`OnnxReranker`** | `--features onnx` | 实现 `Reranker` trait 的 async wrapper |

`OnnxReranker` 用法：

```rust,ignore
# #[cfg(feature = "onnx")]
# tokio_test::block_on(async {
use arkui_rag_core::Reranker;
use arkui_rag_embedding::OnnxReranker;
use std::path::Path;

let rr = OnnxReranker::load(
    Path::new("/Users/you/.arkui-rag/models/bge-reranker-v2-m3-onnx"),
    "bge-reranker-v2-m3",
).unwrap();
// hits 来自 HybridRetriever
let reranked = rr.rerank("query text", hits, 5).await.unwrap();
# });
```

## Feature gate 原由

技术方案 §7.2 的代码依赖 `ort` (~300MB 原生库 + 长编译) + `tokenizers` + `ndarray`。Day 1 不想把这些依赖压在所有人头上，所以默认 feature 留空。验证骨架走 `make check`，验证 ONNX 真实可编译走 `make check-onnx`。

## 用法（Mock）

```rust,ignore
use arkui_rag_core::Embedder;
use arkui_rag_embedding::MockEmbedder;

# tokio_test::block_on(async {
let m = MockEmbedder::new(1024);
let v = m.encode_single("ArkUI-X 下拉刷新").await.unwrap();
assert_eq!(v.len(), 1024);
# });
```

## 用法（ONNX，Day 3 起 trait 实现就绪）

**底层同步 API**（`EmbeddingModel`）：
```rust,ignore
# #[cfg(feature = "onnx")]
# {
use arkui_rag_embedding::EmbeddingModel;
use std::path::Path;

let m = EmbeddingModel::load(Path::new("/Users/you/.arkui-rag/models/bge-m3")).unwrap();
let arr = m.encode(&["query 1", "query 2"]).unwrap();  // ndarray::Array2<f32>
# }
```

**Async trait 实现**（`OnnxEmbedder`，推荐）：
```rust,ignore
# #[cfg(feature = "onnx")]
# tokio_test::block_on(async {
use arkui_rag_core::Embedder;
use arkui_rag_embedding::OnnxEmbedder;
use std::path::Path;

let emb = OnnxEmbedder::load(Path::new("/Users/you/.arkui-rag/models/bge-m3"), "bge-m3").unwrap();
let v = emb.encode_single("ArkUI-X 下拉刷新").await.unwrap();
assert_eq!(v.len(), emb.dim());
# });
```

`OnnxEmbedder` 内部用 `tokio::task::spawn_blocking` 把同步推理移到 blocking 线程池，
不会阻塞 tokio runtime 的 worker；与上层 `HybridRetriever`、`Indexer` 无缝集成。

## 模型获取（Day 3 阶段：手动 + CLI 提示）

CLI 的 `arkui-rag corpus model-pull` 目前还是 stub，但执行后会打印**完整的手动获取步骤**：

```bash
# 1. 拉 BGE-M3
git lfs install
git clone https://huggingface.co/BAAI/bge-m3 ~/.arkui-rag/models/bge-m3
git clone https://huggingface.co/BAAI/bge-reranker-v2-m3 ~/.arkui-rag/models/bge-reranker-v2-m3
# 或国内镜像：
git clone https://www.modelscope.cn/Xorbits/bge-m3.git ~/.arkui-rag/models/bge-m3

# 2. 导出 ONNX（一次性）
pip install optimum[onnxruntime]
optimum-cli export onnx --model ~/.arkui-rag/models/bge-m3 \
    --task feature-extraction --opset 17 ~/.arkui-rag/models/bge-m3-onnx
optimum-cli export onnx --model ~/.arkui-rag/models/bge-reranker-v2-m3 \
    --task text-classification --opset 17 ~/.arkui-rag/models/bge-reranker-v2-m3-onnx
```

存放位置（约定）：`~/.arkui-rag/models/bge-m3-onnx/{model.onnx, tokenizer.json}`。
真实下载实装是 Week 2-3 backlog。
