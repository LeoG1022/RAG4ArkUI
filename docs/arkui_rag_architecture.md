# ArkUI-X RAG 系统 UML 架构图集

> 七张图覆盖：系统上下文 → 容器架构 → 组件设计 → 部署拓扑 → 索引流程 → 检索流程 → 错误飞轮

---

## 图 1：系统上下文图（C4-Level 1）

**视角**：RAG 系统在整个生态中的位置 —— 谁用它、它依赖什么。

```mermaid
graph TB
    subgraph users["消费者 Consumers"]
        Dev["👤 开发者<br/>使用 IDE / CLI / Agent"]
        Team["👥 团队<br/>共享知识库"]
    end

    subgraph clients["客户端 Clients"]
        DevEco["DevEco Studio<br/>Plugin"]
        VSCode["VSCode<br/>Extension"]
        JB["JetBrains<br/>Plugin"]
        Claude["Claude Code<br/>(MCP)"]
        Cursor["Cursor<br/>(MCP)"]
        OpenCode["OpenCode / Hermes<br/>(MCP)"]
        CLI["arkui-rag CLI"]
    end

    subgraph core["RAG Core 系统 (本地二进制)"]
        RAG["ArkUI-X RAG Core<br/>━━━━━━━━━━━━━<br/>本地检索引擎<br/>混合检索 + Reranker<br/>MCP/HTTP/LSP 协议"]
    end

    subgraph external["外部依赖 External"]
        LLM["☁️ LLM API<br/>Claude / 百炼 / DeepSeek"]
        Corpus["📚 Corpus Registry<br/>GitHub / 内网"]
        Models["🤖 Model Hub<br/>HuggingFace / ModelScope"]
        XDB["XDB 错误流水线<br/>(自研)"]
        UISG["UISG 语义图<br/>(自研)"]
    end

    Dev --> DevEco & VSCode & JB & CLI
    Dev --> Claude & Cursor & OpenCode
    Team --> Corpus

    DevEco -->|HTTP/LSP| RAG
    VSCode -->|HTTP/LSP| RAG
    JB -->|HTTP/LSP| RAG
    Claude -->|MCP stdio| RAG
    Cursor -->|MCP stdio| RAG
    OpenCode -->|MCP stdio| RAG
    CLI -->|in-process| RAG

    Corpus -.->|拉取知识库| RAG
    Models -.->|下载模型| RAG
    XDB -.->|错误回流| RAG
    UISG -.->|图谱注入| RAG

    DevEco -.->|生成调用| LLM
    Claude -.->|生成调用| LLM

    classDef coreBox fill:#FAEEDA,stroke:#854F0B,stroke-width:2px,color:#412402
    classDef clientBox fill:#E6F1FB,stroke:#185FA5,color:#042C53
    classDef extBox fill:#E1F5EE,stroke:#0F6E56,color:#04342C
    classDef userBox fill:#F1EFE8,stroke:#5F5E5A,color:#2C2C2A

    class RAG coreBox
    class DevEco,VSCode,JB,Claude,Cursor,OpenCode,CLI clientBox
    class LLM,Corpus,Models,XDB,UISG extBox
    class Dev,Team userBox
```

**关键洞察**：
- RAG Core 是中心节点，但**只做检索**，不做生成
- LLM 调用由各 Client 各自完成（关注点分离）
- XDB/UISG 是你们独有的飞轮输入

---

## 图 2：容器架构图（C4-Level 2）

**视角**：RAG Core 内部分成哪几个进程/服务/库。

```mermaid
graph TB
    subgraph consumers["上游消费者"]
        IDE["IDE Plugins"]
        AGT["MCP Agents"]
    end

    subgraph binary["arkui-rag 单二进制 (Rust)"]
        direction TB

        subgraph protocol["协议适配层 Protocol Layer"]
            HTTP["HTTP/REST Server<br/>:7654 axum"]
            MCP["MCP Server<br/>stdio + SSE"]
            LSP["LSP Extension<br/>JSON-RPC"]
            CLIIF["CLI Interface<br/>clap"]
        end

        subgraph engine["引擎层 Engine Core"]
            QR["Query Router<br/>意图分类 + 路由"]
            QE["Query Enhancer<br/>HyDE + 改写"]
            RT["Retriever<br/>混合检索 + RRF"]
            RR["Reranker<br/>精排"]
            CA["Context Assembler<br/>父子扩展"]
        end

        subgraph inference["推理层 Inference"]
            ORT["ONNX Runtime"]
            EMB["Embedding<br/>BGE-M3 / Qwen3"]
            RNK["Reranker<br/>BGE-Reranker"]
        end

        subgraph storage["存储层 Storage"]
            VDB[("Vector DB<br/>LanceDB")]
            FTS[("BM25<br/>Tantivy")]
            KG[("Knowledge Graph<br/>SQLite")]
            META[("Metadata<br/>SQLite")]
        end

        subgraph index["索引管道 Indexing"]
            PRS["Document Parser<br/>Markdown/PDF/Code"]
            CHK["AST Chunker<br/>tree-sitter"]
            EMG["Embedder Pipeline"]
            IDX["Index Writer"]
        end

        subgraph corpus_mgr["Corpus 管理"]
            CM["Corpus Manager<br/>git/OCI pull"]
            WCH["File Watcher<br/>增量索引"]
        end
    end

    subgraph filesystem["本地文件系统"]
        FS["~/.arkui-rag/<br/>corpus/ + index/ + models/"]
    end

    IDE -->|HTTP| HTTP
    IDE -->|JSON-RPC| LSP
    AGT -->|MCP stdio| MCP

    HTTP --> QR
    MCP --> QR
    LSP --> QR
    CLIIF --> QR

    QR --> QE --> RT
    RT --> RR --> CA

    QE -.->|可选改写| EMB
    RT -->|向量查询| EMB
    RT --> VDB & FTS
    RT --> META
    RR --> RNK

    EMB --> ORT
    RNK --> ORT

    PRS --> CHK --> EMG --> IDX
    EMG --> EMB
    IDX --> VDB & FTS & KG & META

    CM --> PRS
    WCH --> PRS

    VDB & FTS & KG & META -.->|mmap| FS
    CM -.->|read/write| FS

    classDef protocolBox fill:#EEEDFE,stroke:#3C3489,color:#26215C
    classDef engineBox fill:#FAEEDA,stroke:#854F0B,color:#412402
    classDef inferenceBox fill:#FAECE7,stroke:#993C1D,color:#4A1B0C
    classDef storageBox fill:#E1F5EE,stroke:#0F6E56,color:#04342C
    classDef indexBox fill:#E6F1FB,stroke:#185FA5,color:#042C53
    classDef fsBox fill:#F1EFE8,stroke:#5F5E5A,color:#2C2C2A

    class HTTP,MCP,LSP,CLIIF protocolBox
    class QR,QE,RT,RR,CA engineBox
    class ORT,EMB,RNK inferenceBox
    class VDB,FTS,KG,META storageBox
    class PRS,CHK,EMG,IDX,CM,WCH indexBox
    class FS fsBox
```

**关键设计**：
- 协议层、引擎层、推理层、存储层**完全解耦**
- 同一个二进制可启动为 HTTP / MCP / LSP / CLI 任一形态
- 文件系统是唯一持久化（无 Docker 无数据库）

---

## 图 3：核心组件类图（C4-Level 3）

**视角**：Rust trait/struct 设计，关键扩展点。

```mermaid
classDiagram
    class RagEngine {
        -retriever: Box~dyn Retriever~
        -reranker: Box~dyn Reranker~
        -assembler: ContextAssembler
        -router: QueryRouter
        +search(query, options) Result~Context~
        +index(documents) Result~Stats~
    }

    class Retriever {
        <<trait>>
        +retrieve(query, top_k) Vec~Hit~
        +filter(metadata) Self
    }

    class HybridRetriever {
        -vector: VectorRetriever
        -bm25: BM25Retriever
        -fusion: RRFFusion
        +retrieve(query, top_k) Vec~Hit~
    }

    class VectorRetriever {
        -store: LanceDB
        -embedder: Arc~Embedder~
        +retrieve(query, top_k) Vec~Hit~
    }

    class BM25Retriever {
        -index: TantivyIndex
        +retrieve(query, top_k) Vec~Hit~
    }

    class Reranker {
        <<trait>>
        +rerank(query, hits) Vec~Hit~
    }

    class CrossEncoderReranker {
        -model: ONNXSession
        -tokenizer: Tokenizer
        +rerank(query, hits) Vec~Hit~
    }

    class Embedder {
        <<trait>>
        +encode(texts) Array2~f32~
        +encode_single(text) Vec~f32~
        +dim() usize
    }

    class ONNXEmbedder {
        -session: Session
        -tokenizer: Tokenizer
        -max_length: usize
        +encode(texts) Array2~f32~
    }

    class QueryRouter {
        -classifier: IntentClassifier
        +route(query) QueryIntent
    }

    class QueryEnhancer {
        -hyde: HyDEGenerator
        -extractor: EntityExtractor
        +enhance(query) EnhancedQuery
    }

    class ContextAssembler {
        -parent_expander: ParentChildMap
        +assemble(hits) Context
    }

    class Indexer {
        -parser: DocumentParser
        -chunker: ASTChunker
        -embedder: Arc~Embedder~
        -writer: IndexWriter
        +index(source) Stats
    }

    class ASTChunker {
        <<interface>>
        +chunk(content, lang) Vec~Chunk~
    }

    class ArkTSChunker {
        -ts_parser: TreeSitter
        +chunk(content) Vec~Chunk~
    }

    class KotlinChunker {
        -ts_parser: TreeSitter
        +chunk(content) Vec~Chunk~
    }

    class MarkdownChunker {
        +chunk(content) Vec~Chunk~
    }

    RagEngine *-- Retriever
    RagEngine *-- Reranker
    RagEngine *-- ContextAssembler
    RagEngine *-- QueryRouter
    RagEngine *-- QueryEnhancer

    Retriever <|.. HybridRetriever
    HybridRetriever *-- VectorRetriever
    HybridRetriever *-- BM25Retriever

    VectorRetriever ..> Embedder
    Reranker <|.. CrossEncoderReranker
    Embedder <|.. ONNXEmbedder

    Indexer *-- ASTChunker
    Indexer ..> Embedder
    ASTChunker <|.. ArkTSChunker
    ASTChunker <|.. KotlinChunker
    ASTChunker <|.. MarkdownChunker
```

**关键设计哲学**：
- Retriever/Reranker/Embedder 全部是 **trait**，可热插拔
- 多语言切分器走策略模式（ArkTS / Kotlin / Swift / Markdown）
- Embedder 被 VectorRetriever 和 Indexer 共享（同一个模型实例）

---

## 图 4：部署拓扑图

**视角**：用户机器上 RAG 系统的物理布局。

```mermaid
graph TB
    subgraph dev_machine["开发者机器 (macOS / Linux / Windows)"]
        direction TB

        subgraph ide_layer["IDE 进程"]
            DEVECO_PROC["DevEco Studio<br/>(JVM 进程)"]
            VSCODE_PROC["VSCode<br/>(Node 进程)"]
            CLAUDE_PROC["Claude Code<br/>(Node 进程)"]
        end

        subgraph rag_proc["arkui-rag serve 常驻进程"]
            direction LR
            RAG_BIN["Rust 二进制<br/>10 MB"]
            RAG_MEM["内存常驻<br/>━━━━━━━<br/>BGE-M3: 600MB int8<br/>Reranker: 140MB<br/>索引 mmap: ~500MB<br/>━━━━━━━<br/>Total: ~1.3GB"]
        end

        subgraph fs_layer["~/.arkui-rag/"]
            CORPUS["corpus/<br/>官方文档 + 代码示例<br/>~500 MB"]
            INDEX["index/<br/>vectors.lance + bm25/<br/>~800 MB"]
            MODELS["models/<br/>BGE-M3 + Reranker<br/>~750 MB"]
            CONFIG["config.toml"]
        end

        subgraph network_layer["本地端口"]
            P7654["localhost:7654<br/>HTTP API"]
            STDIO["stdio<br/>MCP pipes"]
        end
    end

    subgraph cloud["云端 (按需调用)"]
        LLM_API["Anthropic / 阿里 / DeepSeek<br/>LLM API"]
        REGISTRY["Corpus Registry<br/>GitHub Releases"]
        HF["Model Hub<br/>HuggingFace"]
    end

    DEVECO_PROC -->|HTTP| P7654
    VSCODE_PROC -->|HTTP| P7654
    CLAUDE_PROC -->|fork + stdio| STDIO

    P7654 --> RAG_BIN
    STDIO --> RAG_BIN
    RAG_BIN <--> RAG_MEM

    RAG_BIN -.->|mmap| INDEX
    RAG_BIN -.->|read| CORPUS
    RAG_MEM -.->|加载| MODELS

    DEVECO_PROC -.->|LLM 调用| LLM_API
    CLAUDE_PROC -.->|LLM 调用| LLM_API

    RAG_BIN -.->|首次/更新| REGISTRY
    RAG_BIN -.->|首次/更新| HF

    classDef procBox fill:#EEEDFE,stroke:#3C3489,color:#26215C
    classDef ragBox fill:#FAEEDA,stroke:#854F0B,stroke-width:2px,color:#412402
    classDef fsBox fill:#E1F5EE,stroke:#0F6E56,color:#04342C
    classDef cloudBox fill:#E6F1FB,stroke:#185FA5,color:#042C53
    classDef netBox fill:#F1EFE8,stroke:#5F5E5A,color:#2C2C2A

    class DEVECO_PROC,VSCODE_PROC,CLAUDE_PROC procBox
    class RAG_BIN,RAG_MEM ragBox
    class CORPUS,INDEX,MODELS,CONFIG fsBox
    class LLM_API,REGISTRY,HF cloudBox
    class P7654,STDIO netBox
```

**关键事实**：
- 总内存占用 ~1.3GB（含模型）
- 总磁盘占用 ~2GB（首次安装后）
- LLM 调用由 IDE/Agent 各自管理，不经过 RAG 进程
- 离线可用（首次安装后不依赖网络）

---

## 图 5：索引流程图（Indexing Pipeline）

**视角**：文档怎么从原始格式变成可检索的索引。

```mermaid
flowchart TB
    Start(["arkui-rag index --source ./corpus"]) --> Discover

    Discover["📂 文件发现<br/>walk + ignore + 增量检测"] --> Dispatch{"按文件类型分发"}

    Dispatch -->|.ets / .ts| ArkTS["🌳 tree-sitter-typescript<br/>切分 Component / 方法"]
    Dispatch -->|.kt| Kotlin["🌳 tree-sitter-kotlin<br/>切分 class / function"]
    Dispatch -->|.swift| Swift["🌳 tree-sitter-swift<br/>切分 struct / func"]
    Dispatch -->|.md| MD["📝 Markdown AST<br/>切分 section + 代码块"]
    Dispatch -->|.pdf| PDF["📄 PDF 解析<br/>页 + 段落"]
    Dispatch -->|API 文档 JSON| API["🔧 结构化提取<br/>单 API 一 chunk"]

    ArkTS & Kotlin & Swift & MD & PDF & API --> Enrich

    Enrich["🏷️ 元数据增强<br/>━━━━━━━━━<br/>platform / version<br/>type / tags<br/>since / deprecated"] --> Hierarchy

    Hierarchy["🌲 父子层级建立<br/>━━━━━━━━━<br/>父 chunk: 完整 Component<br/>子 chunk: 单个方法"] --> Embed

    Embed["🧠 批量 Embedding<br/>BGE-M3 batch=32<br/>~140ms / batch"] --> Parallel

    Parallel --> WriteV
    Parallel --> WriteB
    Parallel --> WriteG
    Parallel --> WriteM

    WriteV[("LanceDB<br/>向量索引")]
    WriteB[("Tantivy<br/>BM25 索引")]
    WriteG[("SQLite<br/>API 关系图")]
    WriteM[("SQLite<br/>元数据 + 原文")]

    WriteV & WriteB & WriteG & WriteM --> Commit

    Commit["✅ 原子提交<br/>━━━━━━━<br/>所有索引同时更新<br/>失败可回滚"] --> Stats

    Stats["📊 输出统计<br/>━━━━━━━<br/>文件数 / chunk 数<br/>耗时 / 大小"] --> End(["索引完成"])

    Watch["👁️ File Watcher<br/>(可选常驻)"] -.->|增量| Discover

    classDef startBox fill:#F1EFE8,stroke:#5F5E5A,color:#2C2C2A
    classDef parseBox fill:#EEEDFE,stroke:#3C3489,color:#26215C
    classDef processBox fill:#FAEEDA,stroke:#854F0B,color:#412402
    classDef storeBox fill:#E1F5EE,stroke:#0F6E56,color:#04342C
    classDef decisionBox fill:#FAECE7,stroke:#993C1D,color:#4A1B0C

    class Start,End,Discover,Stats,Commit startBox
    class ArkTS,Kotlin,Swift,MD,PDF,API parseBox
    class Enrich,Hierarchy,Embed,Watch processBox
    class WriteV,WriteB,WriteG,WriteM storeBox
    class Dispatch,Parallel decisionBox
```

**关键步骤**：
1. AST 切分 → 保留语义边界（绝不固定字符数切）
2. 元数据增强 → 后续可精准过滤
3. 父子层级 → 检索小、返回大
4. 并行写入 → 四个索引同时落盘
5. 原子提交 → 失败可回滚

---

## 图 6：检索流程时序图（Retrieval Sequence）

**视角**：一次完整检索调用，各组件如何协作。

```mermaid
sequenceDiagram
    autonumber
    actor User as 用户/Agent
    participant IDE as IDE Plugin
    participant API as HTTP/MCP Server
    participant Router as Query Router
    participant Enhancer as Query Enhancer
    participant Vec as Vector Retriever
    participant BM25 as BM25 Retriever
    participant Fusion as RRF Fusion
    participant Rerank as Reranker
    participant Assembler as Context Assembler
    participant Embed as ONNX Embedder
    participant LLM as LLM API (外部)

    User->>IDE: "ArkUI-X 怎么做下拉刷新"
    IDE->>API: POST /search {query, top_k=10, filters}

    rect rgb(240, 235, 250)
    Note over API,Router: 第1阶段 · 路由决策 (~10ms)
    API->>Router: classify(query)
    Router-->>API: intent=code_example<br/>complexity=simple
    end

    rect rgb(225, 245, 238)
    Note over API,Enhancer: 第2阶段 · Query 增强 (~100ms 可选)
    API->>Enhancer: enhance(query)
    Enhancer->>Embed: 调用本地小模型 (HyDE)
    Embed-->>Enhancer: 伪代码 + 实体
    Enhancer-->>API: enhanced_query
    end

    rect rgb(250, 238, 218)
    Note over API,BM25: 第3阶段 · 混合检索 (~40ms 并行)
    par 向量检索
        API->>Embed: encode(query)
        Embed-->>API: vec[1024]
        API->>Vec: search(vec, filters, top_50)
        Vec-->>API: hits_vector[50]
    and BM25 检索
        API->>BM25: search(tokens, filters, top_50)
        BM25-->>API: hits_bm25[50]
    end
    end

    rect rgb(250, 236, 231)
    Note over API,Fusion: 第4阶段 · 融合 (~5ms)
    API->>Fusion: rrf(hits_vector, hits_bm25)
    Fusion-->>API: hits_fused[50]
    end

    rect rgb(238, 237, 254)
    Note over API,Rerank: 第5阶段 · 精排 (~200ms)
    API->>Rerank: rerank(query, hits_fused)
    Rerank->>Embed: cross-encoder batch 推理
    Embed-->>Rerank: 50 个分数
    Rerank-->>API: hits_reranked[10]
    end

    rect rgb(225, 245, 238)
    Note over API,Assembler: 第6阶段 · Context 组装 (~5ms)
    API->>Assembler: assemble(hits_reranked)
    Assembler->>Assembler: 父 chunk 扩展<br/>元数据附加<br/>引用 ID 生成
    Assembler-->>API: context + citations
    end

    API-->>IDE: {results[10], citations, latency_ms: 360}
    
    rect rgb(255, 245, 245)
    Note over IDE,LLM: 第7阶段 · LLM 生成 (RAG 之外)
    IDE->>LLM: prompt + context
    LLM-->>IDE: streaming response
    end
    
    IDE-->>User: 流式回答 + 引用链接
```

**关键时序约束**：
- RAG Core 端到端：~360ms（含可选 HyDE）
- 不含 HyDE：~260ms
- 第 3 阶段必须并行（向量 + BM25）
- 第 5 阶段是精度关键，但延迟主导

---

## 图 7：错误飞轮闭环（XDB 集成）

**视角**：你们最大差异化护城河 —— 错误自动回流的工程闭环。

```mermaid
flowchart LR
    subgraph dev["开发态"]
        DEV["👤 开发者写代码"]
        IDE["IDE Plugin"]
    end

    subgraph rag_loop["RAG 检索 + 生成"]
        Q["用户 Query"]
        RAG["RAG Core 检索"]
        GEN["LLM 生成代码"]
    end

    subgraph build["构建/运行"]
        COMPILE["ArkTS 编译"]
        RUN["运行/调试"]
        XTS["XTS 自验证<br/>AutoUI 验证"]
    end

    subgraph xdb_layer["XDB 错误捕获"]
        XDB_C["XDB Bridge<br/>编译错误捕获"]
        XDB_R["XDB Bridge<br/>运行时错误捕获"]
        XDB_U["UISG 一多合规<br/>检测"]
    end

    subgraph fix_loop["修复闭环"]
        SUG["LLM 生成修复方案"]
        APPLY["应用修复"]
        VERIFY["二次验证"]
    end

    subgraph corpus_grow["语料飞轮"]
        REVIEW["人工/自动审核"]
        PAIR["错误-修复 Pair<br/>结构化"]
        INDEX_INC["增量索引"]
        ERR_CORPUS[("📚 corpus/errors/<br/>独有错误知识库")]
    end

    DEV --> IDE --> Q --> RAG --> GEN --> COMPILE
    COMPILE -->|✅ 成功| RUN --> XTS
    COMPILE -->|❌ 失败| XDB_C
    RUN -->|❌ 崩溃| XDB_R
    XTS -->|❌ 不合规| XDB_U

    XDB_C & XDB_R & XDB_U --> SUG
    SUG --> APPLY --> VERIFY
    VERIFY -->|✅ 修复成功| REVIEW
    VERIFY -->|❌ 仍失败| SUG

    REVIEW --> PAIR --> INDEX_INC --> ERR_CORPUS
    ERR_CORPUS -.->|检索时召回<br/>避免重蹈覆辙| RAG

    classDef devBox fill:#E6F1FB,stroke:#185FA5,color:#042C53
    classDef ragBox fill:#FAEEDA,stroke:#854F0B,stroke-width:2px,color:#412402
    classDef buildBox fill:#EEEDFE,stroke:#3C3489,color:#26215C
    classDef errBox fill:#FCEBEB,stroke:#A32D2D,color:#501313
    classDef fixBox fill:#E1F5EE,stroke:#0F6E56,color:#04342C
    classDef growBox fill:#FAECE7,stroke:#993C1D,stroke-width:2px,color:#4A1B0C

    class DEV,IDE devBox
    class Q,RAG,GEN ragBox
    class COMPILE,RUN,XTS buildBox
    class XDB_C,XDB_R,XDB_U errBox
    class SUG,APPLY,VERIFY fixBox
    class REVIEW,PAIR,INDEX_INC,ERR_CORPUS growBox
```

**飞轮逻辑**：
1. 每次错误被 XDB 捕获 → 自动进入修复循环
2. 修复成功的 case → 结构化为 错误↔修复 pair
3. 增量索引到 corpus/errors/ → 下次类似错误检索时直接命中
4. **越用越聪明**，竞争对手永远没有你们的真实错误数据

---

## 图 8：协议适配状态图（MCP/HTTP/LSP 三模式）

**视角**：同一个二进制如何切换形态服务不同消费者。

```mermaid
stateDiagram-v2
    [*] --> 启动检测

    启动检测 --> 加载模型: 探测硬件 + 配置
    加载模型 --> 加载索引: BGE-M3 + Reranker → 内存
    加载索引 --> 协议选择: mmap LanceDB + Tantivy

    协议选择 --> HTTP模式: --http
    协议选择 --> MCP模式: --mcp --stdio
    协议选择 --> LSP模式: --lsp
    协议选择 --> CLI模式: query / index

    state HTTP模式 {
        [*] --> 监听7654
        监听7654 --> 处理REST请求
        处理REST请求 --> 返回JSON
        返回JSON --> 监听7654
    }

    state MCP模式 {
        [*] --> 监听stdio
        监听stdio --> 解析JSONRPC
        解析JSONRPC --> 调用工具
        调用工具 --> 返回工具结果
        返回工具结果 --> 监听stdio
    }

    state LSP模式 {
        [*] --> initialize握手
        initialize握手 --> 注册命令
        注册命令 --> 处理textDocument
        处理textDocument --> 发送notification
        发送notification --> 处理textDocument
    }

    state CLI模式 {
        [*] --> 解析参数
        解析参数 --> 执行命令
        执行命令 --> 输出结果
        输出结果 --> [*]
    }

    HTTP模式 --> 优雅关闭: SIGTERM
    MCP模式 --> 优雅关闭: stdin EOF
    LSP模式 --> 优雅关闭: shutdown
    CLI模式 --> [*]

    优雅关闭 --> [*]
```

**核心价值**：
- 同一份代码、同一份模型、同一份索引
- 4 种协议无缝切换
- 部署时只需选择一个 flag

---

## 附：完整端到端代码生成时序（业务视角）

**视角**：用户视角看一次"迁移 KMP 代码到 ArkUI-X"的完整链路。

```mermaid
sequenceDiagram
    actor U as 开发者
    participant IDE as DevEco Plugin
    participant RAG as ArkUI-X RAG Core
    participant LLM as Claude API
    participant XDB as XDB Bridge
    participant FS as 文件系统

    U->>IDE: 选中 KMP ViewModel 代码<br/>右键 "迁移到 ArkUI-X"
    IDE->>IDE: 提取源代码 + 当前光标上下文

    Note over IDE,RAG: ① RAG 检索阶段
    IDE->>RAG: POST /search<br/>query=源代码片段<br/>mode=kmp2arkuix<br/>top_k=5
    RAG->>RAG: HyDE → 混合检索 → Rerank
    RAG-->>IDE: 5 个迁移参考代码<br/>+ ArkTS API 文档<br/>+ 历史错误规避

    Note over IDE,LLM: ② LLM 生成阶段
    IDE->>LLM: 拼装 prompt<br/>(system + RAG context + 源代码)
    LLM-->>IDE: 流式返回 ArkTS 代码

    Note over IDE,FS: ③ 应用与验证
    IDE->>FS: 写入新文件 / 显示 diff
    U->>IDE: Apply diff
    IDE->>FS: 保存代码
    FS->>FS: ArkTS 编译

    alt 编译成功
        FS-->>U: ✅ 迁移完成
    else 编译失败
        FS->>XDB: 捕获编译错误
        XDB->>RAG: POST /search<br/>query=错误信息<br/>mode=error_fix
        RAG-->>XDB: 历史修复方案
        XDB->>LLM: 请求修复
        LLM-->>IDE: 修复代码
        IDE->>FS: 应用修复
        FS-->>U: ✅ 修复完成
        XDB->>RAG: 错误-修复 pair 入库
    end
```

---

## 架构图阅读顺序建议

```
理解全局 → 图 1 (上下文)
深入内部 → 图 2 (容器)
代码层面 → 图 3 (类图)
物理部署 → 图 4 (部署)
索引流程 → 图 5 (索引)
检索流程 → 图 6 (时序)
独家壁垒 → 图 7 (飞轮)
协议形态 → 图 8 (状态)
业务闭环 → 附图   (端到端)
```

九张图构成完整的 RAG 系统蓝图，从产品视角到代码细节全部覆盖。
