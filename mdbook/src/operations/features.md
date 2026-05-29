# Cargo Features 全表

`arkui-rag-cli` 共暴露 10 个 Cargo features，全部 opt-in（默认 release 启用其中 6 个）。

## 默认 release features（6 项）

scripts/release-local.sh + .github/workflows/release.yml 默认带上：

| feature | 体积影响 | 用途 |
|---|---|---|
| `http` | + axum + tower (~2 MB) | `serve --http` |
| `mcp` | + tokio io-std (微) | `serve --mcp` |
| `lsp` | + tokio io-std (微) | `serve --lsp` |
| `tantivy` | + tantivy 0.22 (~3 MB) | `--bm25 tantivy` 真 BM25 |
| `typescript` | + tree-sitter + tree-sitter-typescript (~1.5 MB) | `index` 自动识别 .ets/.ts/.tsx |
| `corpus-pull` | + ureq + flate2 + tar (~3 MB) | `corpus pull --url|--from-file` |

总计：~11 MB release binary（macOS arm64 strip + thin-LTO + opt-level=3）。

## 默认不启用（4 项 · 用户按需开）

| feature | 体积影响 | 用途 | 阻塞 |
|---|---|---|---|
| `onnx` | + ort + tokenizers + ndarray (~300 MB ONNX Runtime 原生库) | 真语义 embedding（BGE-M3） | ⏳ Day 20c 待真活 |
| `kotlin` | + tree-sitter-kotlin (stub) | `index` 识别 .kt | tree-sitter-kotlin 包还没实装 |
| `swift` | + tree-sitter-swift (stub) | `index` 识别 .swift | 同上 |
| `lancedb` | + lance + arrow-* + protoc 编译期依赖 (~10 MB + 工具链) | `--vector lancedb` 嵌入式向量库 | ⏳ task #81：lance 0.17 内部 async 类型递归超限 · 需升 lancedb 0.10 → 0.20+ 主版本 |

## 启用方式

```bash
# 单 feature
cargo build --release -p arkui-rag-cli --features onnx

# 多 features（默认 release 已含 6 项 · 再加 onnx）
cargo build --release -p arkui-rag-cli --features http,mcp,lsp,tantivy,typescript,corpus-pull,onnx

# 一键全开（含 onnx · 需要预装 ONNX Runtime · 当前 lancedb 阻塞会报错）
cargo build --release -p arkui-rag-cli --features full
```

## 极小 binary 场景

不要任何协议、不要真 BM25、不要 corpus pull 的极小工具：

```bash
cargo build --release -p arkui-rag-cli
# binary ~3 MB · 仅支持 index/query 配 mock embedder + memory backend
```

适合：CI 流水线只跑 eval、嵌入式工具链、教学 demo。

## Feature 命名约定

- 所有 feature 都 lowercase + kebab-case
- 协议层用单词：`http` / `mcp` / `lsp`
- 后端用 crate 名：`tantivy` / `lancedb` / `onnx`
- 语言用 tree-sitter 标准：`typescript` / `kotlin` / `swift`
- 工具能力用动词短语：`corpus-pull`

## 维护：加新 feature 的清单

1. `crates/arkui-rag-<crate>/Cargo.toml` 加 `[features]` 行
2. `crates/arkui-rag-cli/Cargo.toml` 加转发 feature
3. 如果是默认 release 需要：`scripts/release-local.sh` `DEFAULT_FEATURES` + `.github/workflows/release.yml` `FEATURES` env
4. 更新本表
5. 更新 `docs/RELEASE.md` feature 表
