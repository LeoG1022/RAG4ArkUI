# arkui-rag-storage

**定位**：存储层接口 + 后端适配器。
- 向量索引 → `VectorStore` trait（Week 2 接 LanceDB）
- BM25 索引 → `BM25Index` trait（Week 2 接 Tantivy）
- 元数据 → `MetadataStore` trait（Week 2 接 SQLite）

Day 1 状态：纯 trait + `InMemoryStore` 占位（让其他 crate 能 import 类型）。完整决策见 [`docs/ADR-002-crate-structure.md`](../../docs/ADR-002-crate-structure.md)。

技术方案对应章节：§4.2 决策 4、§4.5 双轨知识库。
