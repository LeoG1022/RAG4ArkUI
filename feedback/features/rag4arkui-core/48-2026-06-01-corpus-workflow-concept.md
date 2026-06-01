# 48 — corpus-workflow-concept

> 日期：2026-06-01
> 涉及代码：`docs/concepts/corpus-workflow.md`（新）· `docs/concepts/README.md` · `docs/GLOSSARY.md` · `mdbook/src/reference/concepts/corpus-workflow.md` · `mdbook/src/SUMMARY.md`
> 类型：知识归档（概念问答驱动 · AGENTS.md #18 第 3 次实战）+ 新需求开启信号

## 本轮目标

承接 Round 47 v1.0.0 上线后用户两连问：

1. 「release 的包中主要是什么？」
2. 「投喂大量文档做索引时 · 这些文档需要本地投喂还是上传到 release？」
3. 「我希望通过该 rag 在本地整理训练好的数据索引等 · 能够直接分发给别人使用 · 而不需要使用者自己搜集 / 下载各种文档和 arkui / openharmony 的语料材料 · 该怎么做」

Agent 答完 1+2 后按 #18 询问归档 · 用户回「归档」+ 提新需求 3 · 本轮：
- 5 步流程归档 corpus-workflow 概念（含 3 类场景对比）
- 在概念文档「场景 C · maintainer 分发整套」节描绘 Round 48+ 实施方向

## Plan

### 5 步归档

1. ✅ `docs/concepts/corpus-workflow.md` 4 节模板 · 200 行
   - 一句话：corpus / index / query 三段拆 · 都可本地也可预打包
   - 业界用法：LlamaIndex / LangChain / Mem0 / OpenWebUI / Cursor 横向对比
   - 本项目里：3 类场景实操（私有本地 / 公共 + 私有 / maintainer 整套分发）+ 数据规模 vs backend 选型
   - 类比：Docker Hub / 浏览器历史 + 同步服务
2. ✅ `docs/concepts/README.md` 现有条目表 +1
3. ✅ `docs/GLOSSARY.md` 链接 +1
4. ✅ `mdbook/src/reference/concepts/corpus-workflow.md` include
5. ✅ `mdbook/src/SUMMARY.md` 「概念解释」节 +1

### 新需求设计（Round 48+ candidate · 本轮不实施）

「整套分发」目标：

```
用户体验：
  arkui-rag init                    # 一键完成 5 件事
    → corpus pull   (~5MB)
    → model-pull bge-m3 (~2GB)
    → index-pull    (~50MB · 新)
    → install-binary 配三端 MCP
    → 完成 · 直接 chat 调用
```

需要的工程项（Round 48-52 估算）：

| Round | 工作 | 工作量 |
|---|---|---|
| 48 (本轮) | 概念归档 corpus-workflow | ✅ |
| 49 | 收集 ArkUI-X 官方文档（git submodule / curl 脚本）放 `corpus/official/` | 1 round · 看官方文档量 |
| 50 | 加 `arkui-rag index-pull` 命令 · 共用 corpus pull 基础设施 | 1 round |
| 51 | maintainer CI：master 改 `corpus/` → 自动 re-index + 打包 + 推 GitHub Release `corpus-v1.x.x` / `index-v1.x.x` | 1-2 round（写 workflow + ARM cross-compile build index）|
| 52 | 加 `arkui-rag init` wizard | 1 round |
| 53 | docs/USER-VERIFICATION 加「我是终端用户 · 想一键体验」段 + README 推广 | 1 round |

Round 49 是关键卡点 · 因为收集 ArkUI-X / OpenHarmony 官方文档涉及：
- 文档来源（GitHub gitcode / openharmony-sig / arkui-x 多个仓库）
- 文档格式（中文 markdown / yaml / API .ets 定义）
- 切分（已有 markdown / typescript chunker · 大体够用）
- 版本绑定（ArkUI-X 5.0 vs 6.0 文档可能各 build 一个 corpus）
- 法务（OpenHarmony 是 Apache 2.0 · markdown 文档应该可分发 · 需 maintainer 确认）

### 替代方案权衡（分发设计）

| 选项 | 工作量 | 用户体验 | 选 |
|---|---|---|---|
| A · 同时分发 corpus + index（本轮规划）| Round 49-52 | 最佳（一键 · 几分钟搞定）| ✅ 长期目标 |
| B · 只分发 corpus · 用户自己 build index | Round 49-50 | 较好（拉 corpus 后 build 5 分钟）| 中间过渡可考虑 |
| C · 只分发预 built index（不含 corpus 源）| Round 50-51 | 不能补充 / 验证 chunks · 黑盒 | ❌（透明度差）|
| D · 不分发 · 文档教用户自己收集 | 0 | 差（用户得自己爬 ArkUI-X 文档）| ❌ |

选 A · 但分阶段：Round 49 起从 B 开始（corpus 先有 · index 用户自己 build）· Round 50+ 加 index-pull 优化体验。

## 对话摘要

| 轮次 | 用户 | Agent |
|---|---|---|
| 1 | 「release 包中主要是什么？投喂文档要本地还是传 release？」 | 答 1+2：release 只 binary 4MB · 私有文档全本地 · 公共 corpus 才上 GitHub Release · 末尾按 #18 询问归档 |
| 2 | 「归档。新一轮需求：希望本地整理好 corpus + index 分发给别人 · 别人不用自己收集 ArkUI-X / OpenHarmony 语料」 | 本轮 5 步归档 + 在概念文档「场景 C」节描绘 Round 48+ 实施方向 |

## 改动要点

- `docs/concepts/corpus-workflow.md` 200 行 · 4 节模板严格
- `docs/concepts/README.md` 现有条目表 +1
- `docs/GLOSSARY.md` 链接区 +1
- `mdbook/src/reference/concepts/corpus-workflow.md` include
- `mdbook/src/SUMMARY.md` 「概念解释」节 +1

与之前关系：
- Round 33 立 #18 规则
- Round 40 归档 onnx-chain（#18 第 1 次）
- Round 43 归档 github-pages-deploy（#18 第 2 次）
- **本轮 Round 48 归档 corpus-workflow（#18 第 3 次）**

每次都跑通 5 步 · 规则有 3 个实证样本。

## 验证结果

- 编译：N/A（纯文档）
- 5 步流程齐 ✓
- 4 节模板齐 ✓
- 「场景 C」节用作 Round 48+ roadmap 锚点 ✓

## 残留 / 下一轮

<!-- 用 - [ ] 标记未解决项，- [x] 标记已解决项；若无残留写"无" -->
- [x] corpus-workflow.md 5 步归档
- [x] feature log + STATUS（本轮）
- [ ] **Round 49 候选**：收集 ArkUI-X / OpenHarmony 官方文档 · 放 `corpus/official/`
  - 需用户决策：哪些仓库 / 哪些版本（5.0 / 6.0）/ 法务（Apache 2.0 是否可重分发）
- [ ] **Round 50 候选**：加 `arkui-rag index-pull` 命令 · 共用 corpus pull tar.gz 基础设施
- [ ] **Round 51 候选**：maintainer CI · master 改 `corpus/` → 自动 re-build index + 推 release
- [ ] **Round 52 候选**：加 `arkui-rag init` wizard · 一键串起 corpus pull + model pull + index pull + install
- [ ] **Round 53 候选**：用户端文档更新（USER-VERIFICATION + README 加「我是终端用户」视角）
- [ ] 长期：跨平台 model-pull URL 国内 mirror 路由（用户在国内时优先 hf-mirror · 自动判断）
- [ ] 长期：版本绑定 / multi-corpus（用户可以同时索引 ArkUI-X 5.0 + 6.0 · query 时指定）
