# Corpus 管理

## 子目录约定

`corpus/` 5 个子目录，对应不同语料类别（详见 [ADR-003](../adrs/003-corpus.md)）：

| 子目录 | 内容 |
|---|---|
| `official/` | ArkUI-X / OpenHarmony 官方文档 |
| `samples/` | 官方代码示例 |
| `migration/` | KMP / Android / iOS → ArkUI-X 迁移规则 |
| `errors/` | 错误-修复 pair 库 |
| `custom/` | 项目私有 |

## 列表查看

```bash
arkui-rag corpus list
```

输出：
```
corpus/ 子目录：
  ✅ official   (3 个文档)
  ❌ samples    (0 个文档)
  ❌ migration  (0 个文档)
  ❌ errors     (0 个文档)
  ❌ custom     (0 个文档)
```

## 拉取默认包（Day 21）

```bash
# 从 GitHub Releases 默认 URL
arkui-rag corpus pull --target ./corpus/official

# 自定义 URL（gitcode mirror / 自建镜像 / 私有 corpus）
arkui-rag corpus pull --url https://example.com/my-corpus.tar.gz --target ./corpus/custom

# 离线场景：跳过 HTTP 从本地文件解压
arkui-rag corpus pull --from-file ./downloaded.tar.gz --target ./corpus/official

# 兼容不同 tarball 结构（默认 strip 1 段 · 即剥外层 wrap 目录）
arkui-rag corpus pull --strip-components 0   # tarball 内已经是裸文件
arkui-rag corpus pull --strip-components 2   # tarball 内有 2 层 wrap 目录

# 覆盖非空目标目录
arkui-rag corpus pull --force --target ./corpus/official
```

### Tarball 格式

```
arkui-rag-corpus-vX.Y.Z.tar.gz
└── arkui-rag-corpus-vX.Y.Z/           <-- strip_components=1 剥掉
    ├── README.md                      # 元信息（来源、版本、文档数）
    ├── components/
    │   ├── list.md
    │   └── ...
    └── api/
```

### 安全机制

- Path traversal 检查：恶意 tarball 含 `../` 类越界路径会被拒绝
- 500 MB 下载上限：防恶意 HTTP 响应吃光内存
- 180s HTTP 超时

详细技术细节见 [STATUS-corpus-pull.md](https://github.com/keerecles/RAG4ArkUI/blob/master/docs/STATUS-corpus-pull.md)。

## 拉取模型（Day 21b）

`corpus model-pull` 共用 corpus pull 的 HTTP + tar.gz 基础设施，把 ONNX 模型拉到 `~/.arkui-rag/models/<name>/`：

```bash
# 一键拉默认 BGE-M3（按 name 路由到 GitHub Releases 的 models-v1 tarball）
arkui-rag corpus model-pull --name bge-m3

# 拉 reranker
arkui-rag corpus model-pull --name bge-reranker-v2-m3

# 自定义 URL（gitcode mirror / HuggingFace 直链 / 自建镜像）
arkui-rag corpus model-pull \
    --name custom \
    --url https://example.com/my-model.tar.gz \
    --target ~/.arkui-rag/models/custom

# 离线场景：从本地 tarball 解压
arkui-rag corpus model-pull --name bge-m3 --from-file ./bge-m3-onnx-v1.tar.gz
```

拉完后用真模型跑 index/query（需 `onnx` feature 编译）：

```bash
arkui-rag index --source ./corpus/official --embedder onnx --model-path ~/.arkui-rag/models/bge-m3 ...
arkui-rag query --text "..." --embedder onnx --model-path ~/.arkui-rag/models/bge-m3 ...
```

### 已知模型名 → 默认 URL 路由

| `--name` | 默认 URL |
|---|---|
| `bge-m3` | `https://github.com/keerecles/RAG4ArkUI/releases/download/models-v1/bge-m3-onnx-v1.tar.gz` |
| `bge-reranker-v2-m3` | `https://github.com/keerecles/RAG4ArkUI/releases/download/models-v1/bge-reranker-v2-m3-onnx-v1.tar.gz` |
| 其它 | 报错 · 用 `--url` 自定义 |

⚠️ 默认 URL 当前为占位 · 用户首次准备 ONNX 模型 tarball + push GitHub Release `models-v1` 后真活。

### Tarball 格式约定

```
bge-m3-onnx-v1.tar.gz
└── bge-m3-onnx-v1/                          <-- strip_components=1 剥掉
    ├── model/model.onnx                     # 主模型（或 model.fp16.onnx）
    ├── tokenizer.json                       # HuggingFace tokenizer 标准格式
    ├── special_tokens_map.json
    └── config.json
```

## 评估集

`corpus/_eval/queries.yaml` 是检索质量评估集（Day 6）。跑评估：

```bash
arkui-rag eval --queries corpus/_eval/queries.yaml --index-path ./corpus/official/index.json --bm25 tantivy -k 5
```

报告输出到 `reports/eval-<timestamp>.md`，含 recall@k / MRR / 延迟分位数。
