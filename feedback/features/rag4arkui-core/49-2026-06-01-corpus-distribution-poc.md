# 49 — corpus-distribution-poc

> 日期：2026-06-01
> 涉及代码：`corpus/official/`（seed 8 文件）· `docs/RELEASE-CORPUS.md`（新）
> 类型：新功能 PoC（corpus + index 整套分发流水线验证）

## 本轮目标

承接 Round 48 corpus-workflow 概念归档 · 用户决策：

- 决策 1：corpus 来源 = **A（ArkUI-X）+ B（OpenHarmony）**
- 决策 2：分发包 = **corpus + index 双发**（model 单独）
- 决策 3：版本绑定 = arkui-rag v1.0.0 ↔ corpus-v1.0.0 ↔ index-bge-m3-v1.0.0

本轮 PoC：跑通「seed corpus → build index → 打包两份 tarball → 用户解压用」流水线 · 不依赖外部 ArkUI-X / OpenHarmony 仓库（Round 49.5 实际收集时再做）。

## Plan

### 4 步 PoC

1. **seed corpus**：把 `.claude/references/mapping-*.md` + arkuix-best-practices.md + example-mapping.md cp 到 `corpus/official/mapping/` 和 `corpus/official/` 根目录 · 8 文件 · 64KB · **100% 项目自身可重分发**
2. **build index**：本应用本 binary `arkui-rag index --embedder onnx ...` 真重 build
   - ⚠️ 跑出 **ort 2.0.0-rc.12 + CoreML EP + BGE-M3 external data bug**：`open file "...model.onnx/model.onnx_data" failed: Not a directory`
   - 规避：用 Round 42 已 build 的 `~/.arkui-rag/index-onnx.json`（11 文件 / 107 chunks · 内容兼容 mapping seed）
3. **打包**：
   - `arkui-rag-corpus-v1.0.0.tar.gz` · 14KB
   - `arkui-rag-index-bge-m3-v1.0.0.tar.gz` · 988KB
4. **maintainer 文档**：`docs/RELEASE-CORPUS.md` 写完整 3 步发布流程（收集 / build / 推 release）+ 法务底线 + 版本兼容性矩阵 + 大小估算外推

### 外推数据规模（用户 step 4 决策依据）

| 真实场景 | corpus 预估 | index 预估 |
|---|---|---|
| ArkUI-X 文档（~500 .md）| ~3 MB | ~30 MB |
| OpenHarmony 文档（~5000 .md）| ~30 MB | ~300 MB |
| A+B 合并 | ~33 MB | ~330 MB |

GitHub Release 单文件 2GB 限制 · 远低于。可行。

### CoreML + external data 兼容 bug（Round 49.5 关键残留）

```
WARN ort::logging: CoreMLExecutionProvider::GetCapability,
     number of partitions supported by CoreML: 195
     number of nodes in the graph: 1247
     number of nodes supported by CoreML: 886
error: 加载 ONNX 模型失败 ...:
     open file ".../model.onnx/model.onnx_data" failed: Not a directory
```

- CoreML EP 能识别 graph（886/1247 nodes 可加速）
- 但加载 model.onnx external data 路径解析 bug：把 model.onnx 当目录前缀

3 个修法（Round 49.5 / 50 之前必须解决一个）：

| 修法 | 工作量 | 影响 |
|---|---|---|
| A · 用单独 CPU-only binary build index · CoreML binary query | 加 `Makefile build-index-binary` · 30 分钟 | binary 数翻倍 |
| B · 把 BGE-M3 single-file 化（合并 external data 回 model.onnx）| Python script + onnx 包 · 1 round | model 文件变 2.2GB single | 简化部署 |
| C · 等 ort 上游修 bug · 提 issue | 时间不可控 | 等 |

选 B 推荐（项目长期最简）· 但 Round 49.5 可先用 A 临时方案。

### 不做（留 Round 49.5+）

- 没收 ArkUI-X / OpenHarmony 真文档（用户决策 1 的 A+B）· 需用户提供 git URL + 法务确认
- 没加 `arkui-rag index-pull` 命令（Round 50）
- 没写 collect-corpus.sh 脚本（Round 49.5 写 · 跟 ArkUI-X 收集合并）
- 没修 ort + CoreML + external data bug（Round 49.5 前必须解决）
- 没推 corpus-v1.0.0 GitHub Release（等 Round 49.5 真 corpus 后再推）

## 对话摘要

| 轮次 | 用户 | Agent |
|---|---|---|
| 1 | 决策 1 = A+B · 决策 2 = corpus+index · 决策 3 = 版本绑定 | 本轮 PoC：seed 8 文件 + build/打包/估算 + maintainer 文档 + 暴露 CoreML 兼容 bug |

## 改动要点

- `corpus/official/`：seed 8 文件（mapping-*.md + arkuix-best-practices.md + example-mapping.md）· 复制自 `.claude/references/`
- `docs/RELEASE-CORPUS.md`：3 步发布流程 · 法务底线 · 版本矩阵 · 大小估算
- 没改代码 · 没改 workflow · 没推 GitHub Release（Round 49.5 才推）

与 Round 48 关系：Round 48 概念归档「场景 C · maintainer 分发整套」节描绘的设计 · Round 49 PoC 验证设计可行 · Round 49.5+ 真 build。

## 验证结果

- corpus tarball：14KB（8 文件 64KB → gzip）· 比例合理
- index tarball：988KB（Round 42 真 ONNX 索引 + Tantivy bm25/）
- 模拟用户解压：tar -xzf 两份 tarball 到独立目录 · query 应可用（受 CoreML bug 限制本地 build · 用 Round 42 产物代替）
- CoreML + external data bug 实测复现 + 文档化

## 残留 / 下一轮

<!-- 用 - [ ] 标记未解决项，- [x] 标记已解决项；若无残留写"无" -->
- [x] PoC seed corpus（8 mapping doc）放 corpus/official/
- [x] PoC index 打包（用 Round 42 真 ONNX 索引）
- [x] maintainer 发布流程文档 docs/RELEASE-CORPUS.md
- [x] 双轨归档（仅 feature log · 无 meta · 业务变更）
- [ ] **Round 49.5 关键**：修 CoreML + BGE-M3 external data 加载 bug（推荐方案 B · single-file BGE-M3 ONNX）· 否则 maintainer 无法本地 build index
- [ ] **Round 49.5**：用户给 ArkUI-X / OpenHarmony 真仓库 GitHub URL + 确认 Apache 2.0 可重分发 · agent 写 collect-corpus.sh + 实际收集
- [ ] **Round 49.5 后期**：build 含真 ArkUI-X / OpenHarmony 的 corpus + index · 推 GitHub Release `corpus-v1.0.0`
- [ ] **Round 50**：加 `arkui-rag index-pull` 命令 · 共用 corpus-pull 基础设施
- [ ] **Round 51**：maintainer CI 自动 re-build + 推 release（master 改 corpus/ 触发）
- [ ] **Round 52**：加 `arkui-rag init` wizard
- [ ] **Round 53**：终端用户视角文档（USER-VERIFICATION + README）
- [ ] **PoC 局限说明**：本轮 seed 是 mapping doc · 不是真 ArkUI-X 文档 · 终端用户当前 `corpus pull` 默认 URL 仍 404（未推 release）
