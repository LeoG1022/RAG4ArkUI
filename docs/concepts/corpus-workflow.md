# Corpus 工作流（投喂文档 / 索引 / 分发）

> 触发：Round 47 v1.0.0 上线后 · 用户问「release 包里是啥」「私有文档要不要传 GitHub」「想分发给别人不用每个人都自己收集语料怎么做」。

## 一句话

`arkui-rag` 把 corpus（文档源文件）→ index（向量 + BM25 倒排）→ Top-K hits 三段拆开 · 每段都可以**纯本地** · 也可以**预打包分发**。release 包只发 binary（4MB） · 不含任何文档。

## 三段流水的角色

```
┌─ corpus ─────────┐  ┌─ index ──────────┐  ┌─ query ──────┐
│ 你的 .md/.ts/.kt │→ │ index.json       │→ │ Top-K hits   │
│ ArkUI-X 官方文档 │  │ bm25/*.idx (.gz) │  │ 给 Claude 用 │
│ 个人笔记         │  │ ~50-500MB        │  │ 200ms / 4s   │
└──────────────────┘  └──────────────────┘  └──────────────┘
   纯文本 · 几 MB         向量 + 倒排             实时计算
   1-100k chunk

   `arkui-rag index`       `arkui-rag query`
   首次 ~1-30 分钟          实时 ~ms-s
```

每段都有 **2 种获取方式**：

| 段 | 自己做 | 别人做好你 pull |
|---|---|---|
| corpus | 自己收集 .md / .ts / ... 放本地任意目录 | `arkui-rag corpus pull` 从 GitHub Release 拉打包 corpus |
| index | `arkui-rag index --source <corpus> --embedder onnx ...`（首次几分钟）| `arkui-rag index-pull`（未实装 · 见下）|
| model | `arkui-rag corpus model-pull bge-m3` 拉 BGE-M3 ONNX（2GB · 一次性）| 同 ↑（model-pull 即 pull）|
| query | 命令行 / MCP / HTTP / LSP | 同（本来就实时）|

## 业界用法（向 RAG 工具横向对比）

| 工具 | corpus 来源 | index 分发 | 备注 |
|---|---|---|---|
| **arkui-rag**（本项目）| 本地 + `corpus pull` GitHub Release | model 已支持 · index 待加 | 设计为 hybrid（本地 + 公共 pull）|
| LlamaIndex | 全本地 / 自己集成 | 一般不分发 · 用户自己 build | 框架 · 不管 corpus 来源 |
| LangChain | 同上 | 同上 | 同上 |
| Mem0 | SaaS 云端 | 云端共享 | 偏多用户共享 |
| OpenWebUI | 本地 | 全本地 | UI 工具 |
| Cursor / Cody | 私有源代码 | 全本地 · IDE 范围 | 单仓库 |

本项目特殊在 **「本地优先 + 可选公共 corpus pull」** 双轨。

## 本项目里怎么用

### 场景 A · 私有文档 · 完全本地

```bash
# 1. 你把私有文档放本地任意位置
mkdir -p ~/work/my-corpus
cp -r ~/Documents/work-wiki/*.md ~/work/my-corpus/
cp -r ~/work/projects/foo/docs/ ~/work/my-corpus/

# 2. 一次性建索引（首次 30 秒到几分钟 · 看文档量）
arkui-rag index \
    --source ~/work/my-corpus \
    --embedder onnx --model-path ~/.arkui-rag/models/bge-m3 \
    --index-path ~/work/my-corpus/.index.json \
    --bm25 tantivy

# 3. 配 Claude / opencode MCP（index-path 指向上面）
# 4. 重启 client · 调用就用真语义检索你的私有文档
```

**所有数据全在本地** · 不上传 · 不上 GitHub · 不传任何外网。

### 场景 B · 用公共 corpus + 私有补充

```bash
# 1. 拉公共 corpus（项目作者打包推到 GitHub Release）
arkui-rag corpus pull
# → 解压到 corpus/official/

# 2. 拉模型（一次性 · 2GB）
arkui-rag corpus model-pull bge-m3

# 3. 自己补充私有文档进 corpus/custom/
cp ~/internal-docs/*.md corpus/custom/

# 4. 整体索引
arkui-rag index --source corpus --embedder onnx ...
```

公共 corpus = 项目作者维护的 ArkUI-X / OpenHarmony 官方文档 · 用户**直接拉** · 不必自己爬 / 整理。

### 场景 C · maintainer 分发整套（用户零摩擦）

这是**未来要做的方向**（Round 47 用户提的新需求）：

```bash
# 用户体验目标：
arkui-rag init               # 一键完成
                             #   ↓
                             # 1. 拉 corpus  (~5MB · GitHub Release)
                             # 2. 拉 model   (~2GB · HuggingFace mirror)
                             # 3. 拉 index   (~50MB · GitHub Release)
                             # 4. 配三端 MCP
                             # 5. 完成 · 直接可用
```

需要实装的（**Round 48+ candidate**）：

1. **新命令 `arkui-rag index-pull`**：从 GitHub Release 下载预 built 索引 + bm25/ tarball · 解压到 `~/.arkui-rag/`
2. **新命令 `arkui-rag init`**：wizard 串起 corpus pull + model pull + index pull + install-binary 配三端
3. **maintainer 工作流**（新一轮）：
   - 收集 ArkUI-X / OpenHarmony 官方文档源（git submodule / curl 下载脚本）
   - 跑 `arkui-rag index --embedder onnx ...` build 索引
   - 打包 `corpus-v1.0.0.tar.gz` + `index-bge-m3-v1.0.0.tar.gz` 推 GitHub Release
   - CI 化（master 改 corpus/ 时自动重 build + 推 release）

## 三类文档 vs 上 / 不上 GitHub 速查

| 场景 | 放哪 | 上 GitHub | 分发方式 |
|---|---|---|---|
| 公司内部 / 私有 | 本地任意目录 | ❌ | 不分发（自用） |
| 个人笔记 / 学习 | 本地 | ❌ | 不分发 |
| 公共 corpus（项目作者维护） | 仓库 `corpus/` + 推 Release `corpus-v*.tar.gz` | ✅ | `arkui-rag corpus pull` |
| **公共 index**（预 built · 让用户零等待）| **推 Release `index-v*.tar.gz`** | ✅ | **`arkui-rag index-pull`**（未实装 · Round 48+）|

## 数据规模 vs 配置选型

| corpus 规模 | 推荐 backend | index 时间 | query 时间（CoreML）|
|---|---|---|---|
| < 1k chunks | memory + tantivy（默认）| ~5 秒 | ~200ms |
| 1k - 10k | memory + tantivy | ~30 秒 | ~200ms |
| 10k - 100k | + `--vector lancedb` | ~5 分钟 | ~250ms |
| 100k+ | lancedb + 拆 corpus 子集 + 多索引路由 | 10+ 分钟 | ~500ms |

Mac 16GB 跑 10k chunks（mapping doc 量级 ×100 倍）轻松。

## 类比

| 已知 | corpus / index 对应 |
|---|---|
| Docker Hub（公共 image）| GitHub Release 上的 corpus / index tarball |
| 私有 Docker registry | 本地 corpus + 本地 index · 不外传 |
| `docker pull` | `arkui-rag corpus pull` / `arkui-rag index-pull` |
| `docker build` | `arkui-rag index --source ...` |
| `docker run image` | `arkui-rag query` / `serve --mcp` |

或者：

| 已知 | 类比 |
|---|---|
| 浏览器（chrome.exe）| `arkui-rag` binary · release 发的就是它 |
| 浏览器书签 / 历史 | corpus + index · 全本地 · 浏览器自己管 |
| 书签同步服务（Chrome Sync）| `corpus pull` / `index-pull`（可选公共） |

## 与本项目其它概念的关系

| 相关 | 关系 |
|---|---|
| [ONNX 链路](onnx-chain.md) | embedder 是 corpus → index 时用的引擎 · 当前默认 BGE-M3 |
| [MVP](mvp.md) | corpus 工作流是 MVP 「让用户用上自己文档」的核心闭环 |
| [GitHub Pages 部署](github-pages-deploy.md) | 跟 corpus 无关 · 只发文档站本身 |
| [mdBook](mdbook.md) | mdBook 是项目文档（给开发者看）· corpus 是 RAG 语料（给 Claude 看）· 别混 |

## 相关链接

- 当前 corpus pull 实现：`crates/arkui-rag-cli/src/main.rs` `cmd_corpus_pull` / `cmd_corpus_model_pull`
- 默认 corpus URL：`https://github.com/LeoG1022/RAG4ArkUI/releases/download/corpus-v0.0.1/arkui-rag-corpus-v0.0.1.tar.gz`（**未推**）
- 模型路径约定：`~/.arkui-rag/models/<name>/`
- 索引路径约定：`~/.arkui-rag/index.json` + `~/.arkui-rag/bm25/`
- **Round 48+ 分发计划**：`feedback/features/rag4arkui-core/<下一个>-corpus-bundle-distribution.md`（待启动）
