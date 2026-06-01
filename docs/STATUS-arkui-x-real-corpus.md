# STATUS — arkui-x-real-corpus

> 配套 feature log：`feedback/features/rag4arkui-core/51-2026-06-01-arkui-x-real-corpus.md`
> 日期：2026-06-01
> Round 49.5 第 2 半 — 用真 ArkUI-X 文档跑通端到端

---

## 当前状态

Round 49.5 第 1 半（Round 50）解锁了 index 路径的 CoreML+external data bug · 但 query/serve 漏改。本轮第 2 半：

- 收集真 ArkUI-X 文档（1066 .md · Apache 2.0 · 一键 `bash scripts/collect-corpus.sh`）
- 升级 CoreML 处理为 **auto-detect**：onnx.rs / reranker_onnx.rs 看模型目录是否有 `_data` / `.onnx_data` 文件自动跳 EP
- quick-start 子集 build（130 chunks · 3 分钟）· 真 ArkUI-X 入门内容
- 重打 tarball（corpus 2.8MB · index 1.2MB）· 端到端解压 → query → 命中 ✓
- OpenHarmony（600MB+）暂跳过 · 留 Round 49.7

ArkUI-X 路径完整跑通后 · 用户视角的 `arkui-rag index-pull` 命令（Round 49.8）有了真东西可拉。

## 输入契约

无 CLI 新参数 · 无环境变量要求（auto-detect 接管）。

向后兼容路径：
```bash
ARKUI_RAG_DISABLE_COREML=1   # 仍可强制禁用 · 用于 debug
                              # 不 set 也行 · onnx.rs 自动判
```

新依赖：`scripts/collect-corpus.sh` 调用系统的 `git` + `rsync`。两者 macOS/Linux 默认有。

## 输出契约

| 命令 / 路径 | 改变 | 数据 |
|---|---|---|
| `bash scripts/collect-corpus.sh --src arkui-x` | 新 | 拉 1066 .md 到 `corpus/official/arkui-x/` |
| `arkui-rag index --embedder onnx` | 无 env 也能跑 | CoreML 自动跳 · CPU only · 0.73 chunks/sec |
| `arkui-rag query --embedder onnx` | 无 env 也能跑 | CoreML 自动跳 · CPU only · 嵌入约 200-300ms / 查询 |
| `corpus/official/arkui-x/LICENSE` | 新 | Apache-2.0 文本 · 重分发要求 |

破坏性变更：无（旧 env `ARKUI_RAG_DISABLE_COREML=1` 仍兼容）。

## 验证手段

### Agent 本轮已做

```bash
# Phase A · 收集
git clone --depth=1 https://gitcode.com/arkui-x/docs /tmp/corpus-collect/arkui-x   # 19MB
rsync -a --include='*/' --include='*.md' --exclude='*' ...   # 1066 .md

# Phase B · 改 onnx.rs + reranker_onnx.rs auto-detect
cargo build --release ...,onnx     # 7m24s · finished OK

# Phase C · build quick-start
arkui-rag index --source corpus/official/arkui-x/zh-cn/application-dev/quick-start ...
# ✓ files=19 · chunks=130 · elapsed=179305ms · 130 chunks / 3 分钟

# Phase D · 重打 tarball
tar -czf arkui-rag-corpus-v1.0.0.tar.gz corpus/official/     # 2.8MB
tar -czf arkui-rag-index-bge-m3-v1.0.0.tar.gz index.json bm25/   # 1.2MB

# Phase E · 端到端模拟用户
mkdir -p /tmp/end2end-v2/{corpus,index}
tar -xzf .../corpus-v1.0.0.tar.gz -C /tmp/end2end-v2/corpus/
tar -xzf .../index-bge-m3-v1.0.0.tar.gz -C /tmp/end2end-v2/index/
arkui-rag query --text "ArkUI-X 怎么创建第一个应用" --embedder onnx \
    --model-path ~/.arkui-rag/models/bge-m3 \
    --index-path /tmp/end2end-v2/index/index.json --bm25 tantivy -k 3
# ✓ Top-3 命中 README.md "快速入门" + start-overview.md "开发准备"
```

### 用户验证（如想本地复现）

```bash
# 1. 自己拉 ArkUI-X
bash scripts/collect-corpus.sh --src arkui-x

# 2. 子集 build（3 分钟）
mkdir -p /tmp/my-corpus
arkui-rag index \
    --source corpus/official/arkui-x/zh-cn/application-dev/quick-start \
    --embedder onnx --model-path ~/.arkui-rag/models/bge-m3 \
    --index-path /tmp/my-corpus/index.json --bm25 tantivy

# 3. 查
arkui-rag query --text "怎么写第一个 ArkUI-X 应用" \
    --embedder onnx --model-path ~/.arkui-rag/models/bge-m3 \
    --index-path /tmp/my-corpus/index.json --bm25 tantivy -k 5
```

## 与上一阶段的关联性

| Round | 主题 | 关系 |
|---|---|---|
| 47 | v1.0.0 + CoreML 21× 加速 | 本轮发现：BGE-M3 + external data 跟 CoreML 不兼容 · 21× 加速对 BGE-M3 不可用 · 等 ort 修 |
| 49 | corpus 分发 PoC | 本轮真实化（用真 ArkUI-X 替换 mapping seed）|
| 50 | CoreML disable env（index 路径）| 本轮升级为 auto-detect · 同时覆盖 query/serve 路径 |
| **51（本轮）** | ArkUI-X 真 corpus + auto-detect 完善 | Round 49.5 Phase 2 完成 |

## 完成度 / 下一阶段

### 达成度

| 项 | 状态 |
|---|---|
| ArkUI-X 真文档收集（1066 .md · LICENSE 保留）| ✅ |
| `scripts/collect-corpus.sh` 自动化 | ✅ |
| onnx.rs / reranker_onnx.rs auto-detect external data | ✅ |
| cli main.rs 不再硬 set env | ✅ |
| quick-start 子集 build（130 chunks · 3 分钟）| ✅ |
| corpus + index tarball 重打 | ✅ |
| 端到端解压 → query 命中真 ArkUI-X 内容 | ✅ |
| OpenHarmony（600MB+）| ⏭ 留 Round 49.7（partial clone） |
| zh-cn + en 双语全量 build | ⏭ 留 Round 49.6（CI Linux runner 3.2 小时） |

### 下一阶段建议

**Round 49.6（maintainer CI）**：
- 加 `.github/workflows/corpus-build.yml` · Linux runner 跑全量 ArkUI-X build
- 用 GitHub Cache action 缓存 BGE-M3 模型（2.3GB）· 避免每次 re-pull
- 产物：corpus-v1.0.0.tar.gz / index-bge-m3-v1.0.0.tar.gz / SHA256SUMS · 自动推 `corpus-v1.0.0` Release

**Round 49.7（OpenHarmony 收集）**：
```bash
git clone --filter=blob:none --no-checkout https://gitcode.com/openharmony/docs
cd docs
git sparse-checkout init --cone
git sparse-checkout set zh-cn/application-dev en/application-dev   # 只挑核心子目录
git checkout master
```
预计 50-100MB · 远小于 全 clone 的 673MB。

**Round 49.8（占编号 50）**：
- 加 `arkui-rag index-pull` 子命令（类比 `model-pull`）
- `~/.arkui-rag/config.toml` 加默认源 URL（GitHub Releases / 国内镜像可切）
- README + mdBook 更新一键拉取章节

**长期跟踪**：
- 等 ort 上游修 CoreML + external data bug · 现 `_data` 命名启发式可去
- 当 BGE-M3 CoreML 跑通后 · 索引速度从 0.73 chunks/sec → ~15+ chunks/sec（21× 加速回归）· 全量 build 3.2h → ~10 分钟
