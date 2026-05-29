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

## 评估集

`corpus/_eval/queries.yaml` 是检索质量评估集（Day 6）。跑评估：

```bash
arkui-rag eval --queries corpus/_eval/queries.yaml --index-path ./corpus/official/index.json --bm25 tantivy -k 5
```

报告输出到 `reports/eval-<timestamp>.md`，含 recall@k / MRR / 延迟分位数。
