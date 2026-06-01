# 51 — arkui-x-real-corpus（Round 49.5 第 2 半）

> 日期：2026-06-01
> 涉及代码：`crates/arkui-rag-embedding/src/onnx.rs` · `crates/arkui-rag-embedding/src/reranker_onnx.rs` · `crates/arkui-rag-cli/src/main.rs`
> 涉及目录：`corpus/official/arkui-x/`（新增 1066 文件）
> 涉及脚本：`scripts/collect-corpus.sh`（新增）
> 类型：业务里程碑 + bug fix（Round 49 PoC 真实化 · CoreML auto-detect 完善）

## 本轮目标

Round 49.5 第 1 半（feature 50）解了 CoreML + external data 的 index 路径 bug · 但只半解：query 路径仍要 env 才能跑（且 Round 50 报告的"query 200ms CoreML 加速"其实没真验过 BGE-M3）。

本轮第 2 半 · 在真 ArkUI-X 文档上做端到端：

1. 收集真 ArkUI-X 官方文档（gitcode.com/arkui-x/docs · 1072 .md · Apache 2.0 重分发 OK）
2. 写 `scripts/collect-corpus.sh` · maintainer 一键拉
3. build 真 index（应做小子集还是全量？）
4. **修完 CoreML bug**：query/serve 路径自动 fallback CPU（不再要 env）
5. 重打 corpus + index tarball · 端到端解压 → query 验证
6. OpenHarmony 仓库受网络/体积限制 · 决策延后

## Plan

### 决策 A · 收集策略：shallow clone + rsync .md only

```bash
git clone --depth=1 https://gitcode.com/arkui-x/docs   # 不带 history · 几分钟
rsync --include='*/' --include='*.md' --exclude='*' ...  # 只挑 .md
```

不要图片 / 不要 .git / 不要二进制资源 · 19MB 真够用（gzip 后 2.8MB · 远比预想的 5-30MB 还要紧凑）。

LICENSE 强制复制（Apache 2.0 重分发要求）· 见 `corpus/official/arkui-x/LICENSE`。

OpenHarmony 仓库 600MB+ · shallow clone 5 分钟超时仍然失败（673MB .git · HEAD 损坏）。换 partial clone `--filter=blob:none + sparse checkout` 是下一轮（Round 49.7）的事。

### 决策 B · build 子集 vs 全量

实测速度：CPU only（CoreML 跳过）+ BGE-M3 = **0.73 chunks/sec**。

| 范围 | files | 预估 chunks | 预估时间 |
|---|---|---|---|
| `quick-start/` 子集 | 19 | 130 | ~3 分钟 |
| `application-dev/` 半区 | 590 | ~4720 | ~108 分钟 |
| zh-cn 全量 | 647 | ~5180 | ~120 分钟 |
| zh-cn + en 双语 | ~1066 | ~8530 | ~3.2 小时 |

本轮选 **quick-start 子集**：
- 用户最常问的入门问题命中率高
- 3 分钟内可重现 · 适合本地开发 + CI 验证
- 全量 build 留给 Round 49.6 maintainer CI（Linux runner 跑一晚上）

### 决策 C · CoreML auto-detect

Round 50 用 env 绕过 · 但 query/serve 路径漏改。本轮**升级为自动检测**：

```rust
let env_disable = std::env::var("ARKUI_RAG_DISABLE_COREML").is_ok();
let has_external_data = std::fs::read_dir(model_dir)
    .map(|rd| {
        rd.filter_map(|e| e.ok()).any(|e| {
            e.file_name()
                .to_str()
                .map(|n| n.ends_with(".onnx_data") || n.ends_with("_data"))
                .unwrap_or(false)
        })
    })
    .unwrap_or(false);
let disable_coreml = env_disable || has_external_data;
```

判定逻辑：模型目录有 `*_data` 或 `*.onnx_data` 文件 → 跳 CoreML EP。BGE-M3 中招（`model.onnx_data` 2.2GB + `Constant_7_attr__value`）· 小模型（all-MiniLM 等单文件）不中招。

`cmd_index` 不再硬 set env（onnx.rs 自动判 · 不需要 cli 干预）· env 保留作 debug 兜底。

### 决策 D · 重打 tarball + 端到端验证

```bash
# 真 ArkUI-X .md → corpus tarball（替换 Round 50 PoC mapping seed）
tar -czf arkui-rag-corpus-v1.0.0.tar.gz -C $REPO corpus/official/
ls -lh   # 2.8MB

# 真 ArkUI-X 索引 → index tarball
tar -czf arkui-rag-index-bge-m3-v1.0.0.tar.gz index.json bm25/
ls -lh   # 1.2MB

# 模拟用户：解压 + query
tar -xzf .../corpus-v1.0.0.tar.gz
tar -xzf .../index-bge-m3-v1.0.0.tar.gz
arkui-rag query --text "ArkUI-X 怎么创建第一个应用" --embedder onnx ...
# ✓ Top-3 命中 start-overview.md / README.md（命中 "快速入门"）
```

### 不动

- 模型文件不动（保留 external data 结构 · auto-detect 不要求改模型）
- CLI 参数不动（用户不感知 env · 默认路径自动正确）
- BGE-M3 维度 / max_length 不动
- corpus/official/ 已有 PoC mapping seed 保留 · ArkUI-X 加入 `arkui-x/` 子目录

## 对话摘要

| 轮次 | 用户 | Agent |
|---|---|---|
| 1 | 上轮指示「按你推荐的做 然后挑合适的时机压缩」 | Round 50 修 CoreML + index 路径 bug · session 压缩 · 留 ArkUI-X collection 待续 |
| 2 | 「接上次 arkui-x文档仓库 https://gitcode.com/arkui-x/docs OpenHarmony docs：https://gitcode.com/openharmony/docs 重分发 OK · 启动 Round 49.5 第 2 半」 | shallow clone ArkUI-X 19MB / 1066 .md · OpenHarmony 600MB+ 超时 · 暂跳过 |
| 3 | （隐式持续）| 写 collect-corpus.sh · quick-start 19 files build 3 分钟成功 |
| 4 | （端到端验证暴露 query CoreML bug）| Round 50 修不全 · 升级为 auto-detect external data |

用户唯一直接决策：决策 1（A+B 双源）+ 决策 2（corpus+index 双发）+ 决策 3（版本绑定）已在 Round 49 给定 · 本轮自主决策子集策略 / auto-detect / OpenHarmony 延后。

## 改动要点

### 新增
- `corpus/official/arkui-x/`（1066 .md · 19MB · 含 LICENSE / zh-cn / en）
- `scripts/collect-corpus.sh`（maintainer 一键拉 · 支持 --src / --lang / --clean）
- `feedback/features/rag4arkui-core/51-2026-06-01-arkui-x-real-corpus.md`（本文件）
- `docs/STATUS-arkui-x-real-corpus.md`

### 修改
- `crates/arkui-rag-embedding/src/onnx.rs` `EmbeddingModel::load` env 检测 → auto-detect external data
- `crates/arkui-rag-embedding/src/reranker_onnx.rs` `RerankerModel::load` 同款
- `crates/arkui-rag-cli/src/main.rs` `cmd_index` 去掉硬 set env（onnx 自动判 · 不需要 cli 干预）

### tarball 重打（不入 git · 留在 /tmp/dist-corpus-v1.0.0/）
- `arkui-rag-corpus-v1.0.0.tar.gz` 14KB → **2.8MB**（PoC 8 seed + ArkUI-X 1066 .md）
- `arkui-rag-index-bge-m3-v1.0.0.tar.gz` 775KB → **1.2MB**（Round 50 PoC 68 chunks → quick-start 130 chunks）

## 验证结果

### 收集
```bash
bash scripts/collect-corpus.sh --src arkui-x
# ✅ arkui-x: 1066 files · 19M
```

### Build
```bash
arkui-rag index --source corpus/official/arkui-x/zh-cn/application-dev/quick-start \
    --embedder onnx --model-path ~/.arkui-rag/models/bge-m3 \
    --index-path /tmp/corpus-v1.0.0-arkuix/index.json \
    --bm25 tantivy
# ✅ files=19 · chunks=130 · elapsed=179305ms (~3 分钟 · CPU only · 0.73 chunks/sec)
```

### 端到端
```bash
tar -xzf .../arkui-rag-corpus-v1.0.0.tar.gz   # 1066 .md
tar -xzf .../arkui-rag-index-bge-m3-v1.0.0.tar.gz  # 130 chunks
arkui-rag query --text "ArkUI-X 怎么创建第一个应用" --embedder onnx ...
# ✅ Top-3:
#   [1] score=0.0164 README.md "快速开始 > 快速入门"
#   [2] score=0.0161 start-overview.md "开发准备 > 开发工具"
#   [3] score=0.0159 start-overview.md "开发准备"
```

✓ 真 ArkUI-X 文档 · 真嵌入 · 真返回结果 · 端到端解锁。

### CoreML auto-detect
- BGE-M3（含 model.onnx_data 2.2GB）→ auto skip CoreML · 走 CPU EP ✓
- 用户不需 set env · CLI 无新参数 ✓
- env `ARKUI_RAG_DISABLE_COREML=1` 仍可强制（保留作 debug 路径）✓

## 残留 / 下一轮

- [x] ArkUI-X 真文档收集（1066 .md · Apache 2.0 LICENSE 保留）
- [x] `scripts/collect-corpus.sh` maintainer 一键拉
- [x] quick-start 子集 build（130 chunks · 3 分钟）
- [x] CoreML auto-detect external data · 不再要 env
- [x] tarball 重打（corpus 2.8MB · index 1.2MB）· 端到端验证
- [ ] **Round 49.6**：maintainer CI 跑全量 ArkUI-X build（zh-cn + en · ~3.2 小时）· 推 `corpus-v1.0.0` Release
- [ ] **Round 49.7**：OpenHarmony 收集（partial clone --filter=blob:none + sparse checkout · 应付 600MB+ 大仓库）
- [ ] **Round 49.8**（占编号 50）：加 `arkui-rag index-pull` 命令 + `~/.arkui-rag/config.toml` 默认源 URL
- [ ] **Round 51**：maintainer CI 自动 re-build + 推 release（合并 49.6）
- [ ] **Round 52**：加 `arkui-rag init` wizard · 一键 model-pull + index-pull + 配置 MCP
- [ ] **Round 53**：终端用户视角文档（mdBook 用户向章节）
- [ ] **长期**：跟踪 ort 上游修 CoreML + external data bug · 修了之后可去掉 auto-detect 逻辑（恢复 BGE-M3 CoreML 加速）
