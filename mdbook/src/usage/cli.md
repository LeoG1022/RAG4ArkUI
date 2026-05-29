# CLI 命令参考

`arkui-rag` 是单文件二进制，5 个顶级子命令：

| 命令 | 用途 | 文档 |
|---|---|---|
| `serve` | 启动常驻服务（HTTP / MCP / LSP 三者互斥） | [HTTP](http.md) · [MCP](mcp.md) · [LSP](lsp.md) |
| `index` | 对指定目录建索引（输出 JSON · 可选 Tantivy + LanceDB 后端） | 见下 |
| `query` | 检索一次并打印 Top-K 命中 | 见下 |
| `corpus` | Corpus 管理（list / pull / model-pull） | [Corpus 管理](corpus.md) |
| `eval` | 跑检索质量评估（Day 6） | [Corpus 管理](corpus.md#评估集) |

## 全局选项

```bash
arkui-rag --version    # 打印版本
arkui-rag --help       # 打印 help
arkui-rag <command> --help    # 打印 subcommand help
```

## `index`

```bash
arkui-rag index \
    --source ./corpus/official \
    --index-path ./corpus/official/index.json \
    [--embedder mock|onnx]            \  # 默认 mock-384（无外部依赖）
    [--model-path ~/.arkui-rag/.../bge-m3]  \  # 仅 --embedder onnx 时用
    [--bm25 memory|tantivy]           \  # 默认 memory（小语料）· tantivy 推荐
    [--vector memory|lancedb]            \  # 默认 memory · lancedb 暂阻塞（task #81）
    [--include "*.md,*.ts,*.ets"]        # 文件 glob · 默认所有 markdown
```

输出含 IndexStats：files / chunks / skipped / elapsed_ms / saved-to。

## `query`

```bash
arkui-rag query \
    --text "下拉刷新" \
    --index-path ./corpus/official/index.json \
    [--bm25 tantivy] \
    [-k 5]                    \  # Top-K · 默认 5
    [--rerank none|cross-encoder] \  # Day 5 OnnxReranker
    [--pre-rerank-k 50]       \  # rerank 前的召回数
    [--reranker-model-path ~/.arkui-rag/models/bge-reranker]  \
    [--reranker-model-id bge-reranker-v2-m3] \
    [--hyde none|mock]        \  # Day 7 HyDE 改写器
    [--expand-parent]         \  # Day 11 父子扩展
```

输出 Top-K 命中（source / heading_path / line_range / score / preview）。

## `eval`

```bash
arkui-rag eval \
    --queries corpus/_eval/queries.yaml \
    --index-path ./corpus/official/index.json \
    [--bm25 tantivy]        \
    [-k 5]                  \
    [--report-path reports/eval-2026-05-30.md] \
    [全部 query 同样的 reranker/hyde 参数]
```

产出 markdown 报告：recall@k / MRR / 延迟分位数（p50 / p99）。

## Cargo features 对照

| feature | 解锁的 subcommand 能力 |
|---|---|
| `tantivy`（默认 release） | `index/query --bm25 tantivy` |
| `lancedb` | `index/query --vector lancedb`（当前 task #81 阻塞） |
| `onnx` | `index/query --embedder onnx --model-path ...` |
| `typescript`（默认 release） | `index` 自动识别 `.ets/.ts/.tsx` |
| `http`（默认 release） | `serve --http` |
| `mcp`（默认 release） | `serve --mcp` |
| `lsp`（默认 release） | `serve --lsp` |
| `corpus-pull`（默认 release） | `corpus pull --url|--from-file` |

完整对照见 [Cargo Features 全表](../operations/features.md)。
