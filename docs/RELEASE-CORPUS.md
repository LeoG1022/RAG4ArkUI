# 发布公共 corpus + index 流程（maintainer 用）

> 给项目 maintainer 准备的「整套分发」工作流。终端用户跑 `arkui-rag corpus pull` 就用上。
> 自 Round 49 PoC 起。

## 前置准备

| 工具 | 用法 |
|---|---|
| `arkui-rag` v1.0.0+ | `cargo build --release --features onnx` 或 `make install` |
| BGE-M3 模型 | `arkui-rag corpus model-pull bge-m3` |
| GitHub repo write 权限 | maintainer 账号 |

## 三步发布流程

### Step 1 · 收集 corpus

放进 `corpus/official/` · 按子项目分目录：

```
corpus/official/
├── arkui-x/        ← ArkUI-X 官方文档（Apache 2.0）
│   ├── LICENSE     ← 上游 LICENSE 必保留
│   └── *.md
├── openharmony/    ← OpenHarmony 官方文档（Apache 2.0）
│   ├── LICENSE
│   └── *.md
└── mapping/        ← 项目自维护 mapping doc（你的私货）
    ├── mapping-state.md
    └── ...
```

法务底线：
- **必须保留**上游 LICENSE 文件
- 只放 **Apache 2.0 / MIT / BSD / CC-BY 等可重分发**许可证的文档
- DevEco 文档之类 **不要拿**（华为官网 license 不许重分发）

> 当前 PoC：corpus/official/ 含 8 个 mapping doc（项目自带 · 100% 可重分发）作为流水线验证。Round 49.5 实际收 ArkUI-X / OpenHarmony 时按上面结构补。

### Step 2 · 本地 build index

```bash
# 用 BGE-M3 真语义 embedder + Tantivy BM25
arkui-rag index \
    --source corpus/official \
    --embedder onnx --model-path ~/.arkui-rag/models/bge-m3 \
    --index-path /tmp/dist-corpus-v<VER>/index.json \
    --bm25 tantivy
```

⚠️ **CoreML feature 在 BGE-M3 external data 模型上有 ort rc.12 加载 bug**（PoC 触发 · 报 `open file "...model.onnx/model.onnx_data" failed`）· 当前规避：
- 用 Round 42 build 时的 ~/.arkui-rag/index-onnx.json 重用
- 或在 build index 时单独编 no-coreml binary

Round 49.5+ 需修 CoreML + external data 路径解析 · 或暂用 CPU-only build。

### Step 3 · 打包 + 推 release

```bash
# corpus tarball
tar -czf arkui-rag-corpus-v<VER>.tar.gz -C . corpus/official/

# index tarball（含 index.json + bm25/）
mkdir -p index-stage/bm25
cp /tmp/dist-corpus-v<VER>/index.json index-stage/
cp -r ~/.arkui-rag/bm25/* index-stage/bm25/
tar -czf arkui-rag-index-bge-m3-v<VER>.tar.gz -C index-stage .

# SHA256SUMS
shasum -a 256 arkui-rag-corpus-v<VER>.tar.gz arkui-rag-index-bge-m3-v<VER>.tar.gz > SHA256SUMS

# 推 GitHub Release（命名约定）
gh release create corpus-v<VER> \
    arkui-rag-corpus-v<VER>.tar.gz \
    arkui-rag-index-bge-m3-v<VER>.tar.gz \
    SHA256SUMS \
    --title "Corpus v<VER>" \
    --notes "..."
```

## 版本兼容性矩阵

| binary | corpus | index (embedder=BGE-M3) |
|---|---|---|
| v1.0.0 | corpus-v1.0.0 | index-bge-m3-v1.0.0 |
| v1.0.x | corpus-v1.0.x | 同上（同 minor 内兼容）|
| v1.x.0 | corpus-v1.x.0 | index-bge-m3-v1.x.0（每个 minor 重 build）|
| v2.0.0 | corpus-v2.0.0 | index-bge-m3-v2.0.0（major 升级 · 索引格式可能 break）|

终端用户跑 `arkui-rag corpus pull` 拉 corpus · `arkui-rag index-pull`（Round 50 加）拉 index。

## Round 49 PoC 实测数据

| 项 | 大小 | 备注 |
|---|---|---|
| corpus（8 mapping doc） | 14 KB | seed PoC |
| index（11 文件 / 107 chunks）| 988 KB | Round 42 BGE-M3 真索引 |

外推：

| 真实场景 | corpus 预估 | index 预估 |
|---|---|---|
| ArkUI-X 文档（~500 .md）| ~3 MB | ~30 MB |
| OpenHarmony 文档（~5000 .md）| ~30 MB | ~300 MB |
| ArkUI-X + OpenHarmony 合并 | ~33 MB | ~330 MB |

GitHub Release 单文件 2GB 限制 · 远低于。

## 终端用户体验目标（Round 52 wizard 后）

```bash
arkui-rag init
# → 自动跑
#   1. arkui-rag corpus pull              (~30MB)
#   2. arkui-rag corpus model-pull bge-m3 (~2GB)
#   3. arkui-rag index-pull               (~300MB)
#   4. claude mcp add ...
#   5. 完成
```

约 3 分钟（含下载）。

## Round 49+ 后续工作

- Round 49.5 · 实际收 ArkUI-X / OpenHarmony 官方文档 · 跑 build
- Round 50 · 加 `arkui-rag index-pull` 命令
- Round 51 · maintainer CI 自动 re-build + 推 release（master 改 corpus/ 触发）
- Round 52 · 加 `arkui-rag init` wizard
- Round 53 · 终端用户视角文档
