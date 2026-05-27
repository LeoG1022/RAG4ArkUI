# arkui-rag-core

**定位**：纯接口 crate。定义 RAG 引擎的 trait 边界与共享类型，**不含**任何实现。所有 retriever / reranker / embedder / chunker / storage 后端通过实现本 crate 的 trait 接入。

## 公开 trait

| Trait | 角色 | 典型实现位置 |
|---|---|---|
| `Retriever` | 给定 query 返回 Top-K 候选 | `arkui-rag-retrieval`、`arkui-rag-storage` |
| `Reranker` | 给定 (query, hits) 重排 | `arkui-rag-retrieval` |
| `Embedder` | 文本 → 向量 | `arkui-rag-embedding` |
| `ASTChunker` | 文档 → chunks | `arkui-rag-chunker` |

所有 trait 都是 `async_trait::async_trait`，因为 RAG 流水线在 server 形态下必须异步。

## 共享类型

| 类型 | 用途 |
|---|---|
| `Chunk` / `ChunkId` / `ChunkMetadata` | 切分后的文本块及其元数据（platform / version / type / tags） |
| `Hit` / `Citation` | 检索结果及其来源引用 |
| `QueryIntent` / `EnhancedQuery` / `QueryFilters` | 查询的中间表示，跟 Query Router / Enhancer 配合 |
| `RagError` / `Result<T>` | 统一错误类型，跨 crate 共用 |

## Day 1 状态

- ✅ Trait 签名完整
- ✅ 类型 derive `Serialize/Deserialize` 以便序列化为 HTTP / MCP payload
- ⏳ 无具体实现（所有实现在其他 crate）

完整设计依据：[`docs/ADR-002-crate-structure.md`](../../docs/ADR-002-crate-structure.md)，原型见技术方案图 3 类图。
