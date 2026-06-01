# STATUS — corpus-distribution-poc

> 配套 feature log：`feedback/features/rag4arkui-core/49-2026-06-01-corpus-distribution-poc.md`
> 日期：2026-06-01

---

## 当前状态

Round 48 概念归档后 · 用户拍板「决策 1 = A(ArkUI-X)+B(OpenHarmony) · 决策 2 = corpus+index 双发 · 决策 3 = 版本绑定」· Round 49 跑通 PoC 流水线 + 写 maintainer 发布流程文档。

本阶段交付：
- `corpus/official/` seed 8 文件（mapping doc · 项目自身 100% 可重分发）
- `docs/RELEASE-CORPUS.md` 3 步发布流程（含法务 + 版本矩阵 + 大小外推）
- 实测：corpus 14KB · index 988KB 的两份 tarball
- 暴露 ort 2.0.0-rc.12 + CoreML + BGE-M3 external data **加载 bug**（Round 49.5 关键残留）

意义：分发流水线设计可行 · 但 CoreML + external data bug 是 Round 49.5 前的硬阻塞。Round 49.5 修完 bug + 用户给真仓库 URL · 才能 build 含真 ArkUI-X / OpenHarmony 文档的 corpus + index 推 GitHub Release。

## 输入契约

### Maintainer 发布契约（docs/RELEASE-CORPUS.md）

```
Step 1: 收集 corpus → corpus/official/{arkui-x,openharmony,mapping}/
       必保留上游 LICENSE
Step 2: arkui-rag index --source corpus/official --embedder onnx ...
       ⚠️ 当前阻塞: CoreML + external data bug
Step 3: tar -czf 两份 tarball + gh release create corpus-v<VER>
```

### 法务底线

- 必须保留上游 LICENSE 文件
- 只放 Apache 2.0 / MIT / BSD / CC-BY 可重分发的文档
- DevEco 文档之类**不要拿**（华为官网 license 不许重分发）

### 不变项

- 现有 corpus pull / model pull 命令不变
- v1.0.0 binary 完全不变
- 没推任何新 GitHub Release

## 输出契约

### PoC 实测

| 项 | 实测 |
|---|---|
| corpus tarball | 14KB（8 文件 64KB → gzip 压缩比 4×）|
| index tarball | 988KB（Round 42 真 ONNX 索引 · 107 chunks + bm25/）|
| 模拟用户解压 | tar -xzf 两份成功 · 目录结构对 |

### 外推数据规模

| 场景 | corpus 预估 | index 预估 |
|---|---|---|
| ArkUI-X (500 .md) | ~3 MB | ~30 MB |
| OpenHarmony (5000 .md) | ~30 MB | ~300 MB |
| A+B 合并 | ~33 MB | ~330 MB |

均 < GitHub Release 2GB 单文件限制 · 可行。

### CoreML 兼容 bug（Round 49.5 必修）

```
error: 加载 ONNX 模型失败 ...:
       open file ".../model.onnx/model.onnx_data" failed: Not a directory
```

3 个修法：

| 修法 | 工作量 | 选 |
|---|---|---|
| A · CPU-only binary 单独 build index · CoreML binary 仅 query | 30 分钟 | 临时方案 |
| B · BGE-M3 single-file 化（onnx merge external data）| 1 round | **推荐长期** |
| C · 等 ort 上游修 + 提 issue | 不可控 | 备选 |

## 验证手段

### Agent 本轮已做

- corpus tarball 打包 · 14KB · 解压结构对
- index tarball 打包 · 988KB · 解压结构对
- CoreML bug 复现 + 文档化

### 用户验证（Round 49.5 之前不必）

无需立即跑 · 等 Round 49.5 实际收完 ArkUI-X / OpenHarmony 文档后再做端到端跑通。

## 与上一阶段的关联性

| Round | 主题 | 关系 |
|---|---|---|
| 21 (Day 21) | corpus pull 真实下载 | 复用基础设施 |
| 42 | task #87 ONNX 真活 | **本轮 PoC 复用 Round 42 build 的 index** |
| 47 | v1.0.0 + CoreML 21× | **本轮暴露 CoreML 兼容 bug** |
| 48 | 概念归档 corpus-workflow | 本轮 PoC 验证概念可行 |
| **49（本轮）** | corpus-distribution-poc | 跑通流水线 + 写 maintainer 文档 |

层次：48 设计 → 49 PoC 验证 → 49.5 实施 → 50-53 工程化收尾。

兼容性：完全向后兼容。

破坏性变更：无（不动 binary / API / 用户配置）。

## 完成度 / 下一阶段

### 达成度

| 项 | 状态 |
|---|---|
| seed corpus 放 corpus/official/ | ✅ |
| PoC 打包验证 | ✅ |
| docs/RELEASE-CORPUS.md | ✅ |
| 双轨归档（feature log + STATUS）| ✅ |
| CoreML + external data bug 修 | ⏳ Round 49.5 关键 |
| 真 ArkUI-X / OpenHarmony 收集 | ⏳ Round 49.5（等用户提供仓库 URL）|
| 推 GitHub Release corpus-v1.0.0 | ⏳ Round 49.5 后期 |

### 下一阶段建议

立即（用户做）：

1. **决定 CoreML bug 修法**（A 临时 / B 推荐 / C 备选）
2. **提供 ArkUI-X / OpenHarmony 真仓库 URL**：
   - ArkUI-X 推测：`https://gitcode.com/openharmony-sig/arkui_for_aki` 或 GitHub mirror（用户确认）
   - OpenHarmony docs：`https://gitcode.com/openharmony/docs`（中文版本完整）
   - 用户确认仓库 + 分支 + 子目录（`zh-cn/` 等）
3. **确认 Apache 2.0 重分发 OK**（一般 OK · 但用户作为 maintainer 拍板）

短期（agent 做 · 1-3 round）：
- Round 49.5: 修 CoreML bug + 写 scripts/collect-corpus.sh + 实际收 ArkUI-X / OpenHarmony · build 真 corpus + index
- Round 50: 加 `arkui-rag index-pull` 命令
- Round 51: maintainer CI 自动 re-build + 推 release
- Round 52: 加 `arkui-rag init` wizard
- Round 53: 终端用户视角文档

中期：
- 多版本 corpus（v1.0.0 / v1.1.0 跟 ArkUI-X 版本绑定）
- 增量 update（不必每次全 rebuild）
- 路由（国内用户 hf-mirror · 国外 GitHub）
