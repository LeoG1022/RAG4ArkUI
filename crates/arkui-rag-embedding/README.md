# arkui-rag-embedding

**定位**：文本嵌入（文本 → 向量）。给索引和检索两路用。

## Day 1 提供的实现

| 实现 | feature 要求 | 用途 |
|---|---|---|
| `MockEmbedder` | 默认 | 开发期占位，返回确定性伪随机向量；让 cargo check 不依赖 ONNX |
| `OnnxEmbedder`（§7.2 verbatim） | `--features onnx` | 真实 BGE-M3 推理 |

## Feature gate 原由

技术方案 §7.2 的代码依赖 `ort` (~300MB 原生库 + 长编译) + `tokenizers` + `ndarray`。Day 1 不想把这些依赖压在所有人头上，所以默认 feature 留空。验证骨架走 `make check`，验证 ONNX 真实可编译走 `make check-onnx`。

## 用法（Mock）

```rust
use arkui_rag_core::Embedder;
use arkui_rag_embedding::MockEmbedder;

# tokio_test::block_on(async {
let m = MockEmbedder::new(1024);
let v = m.encode_single("ArkUI-X 下拉刷新").await.unwrap();
assert_eq!(v.len(), 1024);
# });
```

## 用法（ONNX，Week 2 起）

```rust
# #[cfg(feature = "onnx")]
# {
use arkui_rag_embedding::onnx::EmbeddingModel;
use std::path::Path;

let m = EmbeddingModel::load(Path::new("/Users/you/.arkui-rag/models/bge-m3")).unwrap();
let arr = m.encode(&["query 1", "query 2"]).unwrap();  // ndarray::Array2<f32>
# }
```

注意：`OnnxEmbedder` 当前是技术方案 §7.2 verbatim 的同步 API，**尚未**包装为 `Embedder` trait 的 async 实现 —— 这是 Week 2 的 backlog 项（用 `tokio::task::spawn_blocking` 桥接）。

## 模型获取

模型不在仓库里，首次运行靠 CLI 拉取：

```bash
arkui-rag corpus model-pull --name bge-m3   # TODO Week 2 实现
```

存放位置（约定）：`~/.arkui-rag/models/bge-m3/{model.onnx, tokenizer.json}`。
