# 本地 RAG smoke v2 总结（Round 52 score-transparency 之后）

> 索引：`/Users/leo/tmp-index-pull2/index.json`（130 chunks · quick-start 19 文件子集）
> 模型：BGE-M3 ONNX · CPU only
> 时间：2026-06-03
> binary：arkui-rag 1.0.0（含 Round 52 三 score 暴露 + `--min-vector-score`）

---

## 关键改进 vs Phase A

Phase A 暴露的问题：

| 问题 | 现状 | 修法 |
|---|---|---|
| Score 全 0.0164 / 0.0161 / 0.0159（RRF rank-based）| ✅ 现在显示 rrf + vector + bm25 三列 | Hit 加 vector_score/bm25_score Optional 字段 |
| 负样本没过滤 | ✅ `--min-vector-score 0.7` 剔掉 | cli 加阈值参数 |
| 用户看不出"靠谱不靠谱" | ✅ vector cosine 0.81 vs 0.62 一眼分辨 | 三 score 透明输出 |

---

## vector cosine 分布（16 query · top-1）

| id | 类别 | query | top-1 vector | 命中合理 | 负样本判定 |
|---|---|---|---|---|---|
| a1 | 入门 | ArkUI-X 怎么创建第一个应用 | **0.8161** | ✅ README "快速入门" | 正 |
| a2 | 入门 | 什么是 ArkUI-X · 和 ArkUI 有什么区别 | **0.8438** | ✅ start-overview "基本概念" | 正 |
| a3 | 入门 | DevEco Studio 怎么装 ArkUI-X 插件 | **0.8669** | ✅ start-overview "IDE工具" | 正 |
| a4 | 入门 | ace tools 命令行工具用法 | **0.8441** | ✅ start-with-ace-tools "简介" | 正 |
| b1 | 工程 | ArkUI-X 项目目录结构 | 0.7799 | ✅ sdk-structure-guide | 正 |
| b2 | 工程 | Stage 模型 ability 是什么 | 0.7694 | ⚠️ 命中 iOS · 应包含 ets-stage | 正 |
| c1 | 多端 | 怎么在 Android 上跑 ArkUI-X 应用 | 0.8460 | ❌ 命中 sdk-structure 而非 ability-on-android | 正 |
| c2 | 多端 | ArkUI-X iOS 端怎么集成 | 0.8380 | ❌ 同 c1 错命中 | 正 |
| c3 | 多端 | ArkUI-X 不同平台行为差异 | 0.8514 | ❌ 错命中 start-overview | 正 |
| d1 | 进阶 | platform bridge 怎么用 | 0.8260 | ⚠️ top-2 命中 platform-bridge | 正 |
| d2 | 进阶 | 动态化加载 · 热更新 | 0.7293 | ✅ dynamic-introduction | 正 |
| d3 | 进阶 | ffi napi 调 C++ 接口 | 0.7391 | ✅ ffi-napi-introduction | 正 |
| d4 | 进阶 | ArkTS 双向绑定 $$ | 0.7250 | ✅ arkts-two-way-sync "$$语法" | 正 |
| e1 | 资源 | 怎么访问 resource 资源 | 0.7431 | ⚠️ top-3 才命中 resource-categories | 正 |
| **f1** | **负** | 今天天气怎么样 · 北京下雨吗 | **0.6470** | — 应过滤 | **负** |
| **f2** | **负** | 怎么炒西红柿炒鸡蛋 | **0.6181** | — 应过滤 | **负** |

### 阈值决策

- 正样本最低 **0.7250**（d4 双向绑定 top-1）
- 负样本最高 **0.6470**（f1 天气 top-1）
- **决策线 0.70**：正样本全留 · 负样本全剔 ✓

实测 `--min-vector-score 0.7`：
- a1-e1 14 个 query 命中 top-3 大部分保留（仅 d2 / d3 各保留 top-2/1）
- f1 / f2 直接"⚠️ 无命中"

---

## 命中质量再分析（130 chunks 子集）

| 类别 | 准确 | 偏弱 | 全错 |
|---|---|---|---|
| 入门 a1-a4 | 4/4 | 0 | 0 |
| 工程 b1-b2 | 1 | 1 | 0 |
| 多端 c1-c3 | 0 | 0 | 3 ⚠️ |
| 进阶 d1-d4 | 3 | 1 | 0 |
| 资源 e1 | 0 | 1 | 0 |
| 负样本 f1-f2 | 2/2（阈值后过滤）| — | — |

**子集 130 chunks 平均准确率（top-3 含 expected）约 65%**。多端类（c1/c2/c3）覆盖差 = quick-start 子集小，BGE-M3 对"Android"/"iOS"短词在中文文档里没强 signal，可能要扩大 build 范围（Phase B 在跑 590 files）才解。

---

## BM25 全程"—"

quick-start 子集所有 query 的 BM25 都没召回（显示 —）。

原因：
- query 短 + 中文 · Tantivy 默认 tokenizer 没装中文分词
- 19 文件 / 130 chunks 太小 · BM25 词频统计无效

修法：Round 53 候选
- 装中文分词器（jieba / lindera）· Tantivy `jieba-rs` feature
- 或 Phase B 全量 build 后看 BM25 是否好转

---

## 三 score 解读

| score | 含义 | 适合做啥 |
|---|---|---|
| **rrf** | RRF rank-based fusion 之后的融合 score（无信息量 · 仅排序）| 综合排名 · 不能阈值过滤 |
| **vector** | BGE-M3 cosine 真实相似度 0-1 | **阈值过滤负样本** · "靠谱不靠谱"判断 |
| **bm25** | Tantivy BM25 raw score · 0-15 | 关键词匹配 · 中文要装分词器才有用 |

---

## 推荐用户用法

### 默认（无脑跑）
```bash
arkui-rag query --text "..." --embedder onnx ...
```

### 排除负样本（推荐）
```bash
arkui-rag query --text "..." --min-vector-score 0.7 --embedder onnx ...
```

### MCP server 集成
当前 mcp.rs 输出还没透传 vector_score · Round 53 候选：让 MCP tool response 也带 vector_score · Claude Code 客户端能"过滤掉相似度太低的"。

---

## 待解（Phase B 还在跑 + 其他）

- [ ] Phase B 完成（590 files application-dev 全量）· 重跑 smoke 看覆盖度
- [ ] BM25 中文分词器（jieba / lindera）· 解 — 问题
- [ ] eval 命令用 vector_score 计算 Recall@K
- [ ] MCP / HTTP / LSP 也透传 vector_score
- [ ] reranker 启用后的 score 影响（cross-encoder 给绝对分数）
