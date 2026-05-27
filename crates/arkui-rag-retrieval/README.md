# arkui-rag-retrieval

**定位**：把 storage 后端（向量 + BM25）组合成业界标配的混合检索流水线：

```
Query → [向量检索 ∥ BM25] → RRF 融合 → Reranker 精排 → Top-N
```

技术方案对应：§1.4 检索层 / §2.4 检索流水线 / §6.2 模型 2、§7.1 模型 3。

## Day 1 状态

| 组件 | 状态 |
|---|---|
| `HybridRetriever`（实现 `Retriever`） | ⏳ stub：返回空 hits + warn 日志 |
| `RrfFusion` | ✅ 算法实现（纯函数，无依赖） |
| `CrossEncoderReranker`（实现 `Reranker`） | ⏳ stub |

## RRF 融合算法

```rust
score(d) = Σ over retrievers: 1 / (k + rank(d))
```

`k` 默认 60（Lin & Cormack 2009）。
