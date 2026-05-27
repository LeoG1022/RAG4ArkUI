# STATUS · ROADMAP 归档

> 日期：2026-05-27
> 对应 commit：[本 commit · ROADMAP 全景图归档]
> 对应 feature log：[`feedback/features/rag4arkui-core/11-2026-05-27-roadmap-doc.md`](../feedback/features/rag4arkui-core/11-2026-05-27-roadmap-doc.md)
> 上一阶段：[`STATUS-day6-eval.md`](STATUS-day6-eval.md)
> 下一阶段：`STATUS-day7-hyde.md`（推荐 · HyDE 改写器）

> 🎯 **本轮性质**：纯文档归档。无代码改动；把对话里展示的"全景图"持久化为 `docs/ROADMAP.md` 长期维护文档。

---

## 当前状态

| 项 | 状态 |
|---|---|
| `docs/ROADMAP.md` | ✅ 新增（200 行 / 1 mermaid gantt / 5 表） |
| `docs/STATUS-roadmap-doc.md` | ✅ 新增（本文件，规则 #17 配套） |
| `feedback/features/rag4arkui-core/README.md` | ✅ 加 Round 11 行 |
| 代码 / Cargo.toml / Makefile | 不变 |
| 测试数 | 不变（31 默认 / 41 全 feature） |
| crate 数 | 不变（9） |

### ROADMAP.md 结构

9 节：定位 + 维护约定 → 当前位置 → mermaid gantt → 已完成表（12 commit） → 剩余切片表（按 Week） → 达成度 → 业界基线 → 里程碑 + 关键路径 → 文档导航。

---

## 输入契约

### 文档读者

| 角色 | 用途 |
|---|---|
| 项目发起人 / 投资人 | 看进度 + 估算剩余工作量 |
| 新加入的 agent | 一文了解整体路线 + 当前位置 |
| 协作开发者 | 找剩余切片 backlog 接手 |

### 文档维护者

- **每个 Day commit 后**：当前 commit 的 agent 同步更新 ROADMAP 中：
  - "当前位置" 区
  - "已完成" 表追加一行
- **路线调整时**：调整"剩余切片"表中的优先级 / 工作量
- **新增维度时**：在"业界基线对照"或"完成度"中追加行

### 不触发 ROADMAP 更新的情况

- 手工提交（用户直接 git commit）
- 纯文档微调（README typo 等）
- meta-only 改动（如规则 #17 立时）

---

## 输出契约

### `docs/ROADMAP.md` 公开 anchors

读者可直接锚定到以下章节：

| 锚 | 用途 |
|---|---|
| `#📍-当前位置` | 看当前 Round 进度 |
| `#时间线mermaid-gantt` | mermaid gantt 图 |
| `#✅-已完成12-commits` | 已完成切片表（含 STATUS 链接） |
| `#⏳-剩余切片按推荐顺序` | 待做 backlog（按 Week 分组） |
| `#📊-6-周-mvp-路线图达成度` | % 完成度估算 |
| `#🎯-业界基线对照§85` | §8.5 五条共识对照 |
| `#🔴-关键路径必走切片不可省` | 必走切片箭头链 |

### 与 STATUS-<slug>.md 的关系

- **ROADMAP**：跨阶段全景图，回答 "还有多少 / 当前在哪"
- **STATUS**：单轮快照（规则 #17 强制），回答 "这一轮做了什么 / 现在状态"
- **互相引用，不重复内容**：ROADMAP 的"已完成"表引用 STATUS 链接；STATUS 不复制 ROADMAP 章节

---

## 验证手段

### 用户手动

```bash
# 1. 看 ROADMAP（任何 markdown 阅读器）
cat docs/ROADMAP.md
# 或在 GitHub / VSCode 看 mermaid 渲染

# 2. 跟踪当前位置一致性
git log --oneline -1               # 应等于 ROADMAP "已完成"最后一行的 commit
ls docs/STATUS-*.md | wc -l        # 应等于 ROADMAP 计数（当前 6 份）

# 3. ROADMAP 内部链接完整性（含相对路径）
bash scripts/check-consistency.sh   # M-LINK-DEAD 校验所有 markdown 相对链接
```

### 自动化

| 手段 | 范围 | 状态 |
|---|---|---|
| `M-LINK-DEAD` | 校验 ROADMAP 引用的所有相对路径（STATUS-*.md、ADR-*.md、corpus/、AGENTS.md 等）真实存在 | ✅ 已有 |
| `M-FB-FORMAT` | 校验 feature log Round 11 5 段结构 | ✅ |
| `M-FEATURE-PLAN` | 校验 feature log 含 `## Plan` + `## 对话摘要` | ✅ |
| `M-STATUS-PER-ROUND` | 校验本 commit 同时含 feature log + STATUS | ✅ |

### 暂未自动化（明确缺口）

- ❌ ROADMAP "当前位置" 与最新 commit hash 一致性（**未来 backlog**：`scripts/check-roadmap-sync.sh`）
- ❌ ROADMAP "已完成"表行数 == STATUS 文件数（**未来 backlog**）
- ❌ ROADMAP "剩余切片"表与某权威 backlog 同步（无单一事实源，目前用 ROADMAP 自身做 source of truth）

---

## 与上一阶段（STATUS-day6-eval）的关联性

### 增量

| 维度 | Day 6 (Eval) 完成时 | 本轮（ROADMAP 归档）后 |
|---|---|---|
| 代码 | 9 crate · 31 测试 | 不变 |
| docs/ 文档数 | 4 STATUS + 3 ADR + 2 项目文档 = 9 | + ROADMAP.md + STATUS-roadmap-doc = **11** |
| 全景视图 | 仅对话里 + 各 STATUS 链式追溯 | ✅ 单点入口 `docs/ROADMAP.md` |
| 新 agent 上手成本 | 需读 ONBOARDING + STATUS-day6 才知当前进度 | 直接读 `docs/ROADMAP.md` 即可 |

### 兼容性

- ✅ 无破坏性变更（纯新增文档）
- ✅ 不动 AGENTS.md / CLAUDE.md / scripts（不立强制规则）
- ✅ 与规则 #17 完全兼容（本轮按规则走 feature log + STATUS）

### 与未来规则的设计空间

ROADMAP 维护**先建立惯例**（顶部"维护约定"段），3-5 round 后评估：
- 如果 agent 都遵守 → 不必立规则
- 如果出现漏更新 → 加 `scripts/check-roadmap-sync.sh` + AGENTS.md 规则 #18

避免规则膨胀（已有规则 #17 后增量评估）。

---

## 完成度 / 下一阶段

### 本轮完成度

| 项 | 状态 |
|---|---|
| ROADMAP.md 完整全景图 | ✅ |
| 时间线 gantt + 已完成表 + 剩余切片表 | ✅ |
| 维护约定文档化 | ✅（顶部声明） |
| 配套 feature log + STATUS（规则 #17） | ✅ |

### 维护约定生效时机

**从 Day 7 起**：每个 Day commit 中 agent 同步更新 ROADMAP（不单独 commit）。

具体动作：
1. ROADMAP "已完成" 表追加新 commit 行（含 STATUS 链接）
2. ROADMAP "当前位置" 段更新到新 Day
3. ROADMAP "剩余切片" 中已完成项移除
4. ROADMAP "完成度" / "里程碑" / "业界基线"按需更新

### 下一阶段建议（按优先级 + 价值）

**Agent 推荐**：**Day 7 HyDE 改写器**。理由：
1. 有评估集了，HyDE 接入后**第一时间能量化效果**（mock retrieval 阶段就能看出对自然语言 query 命中率的影响）
2. 与 §1.2 Advanced RAG 范式对齐
3. 工作量较小（1-2 commit），快速迭代
4. ROADMAP 维护约定的**首次实战**：Day 7 commit 时同步更新 ROADMAP

**替代选择**：
- Day 10 tree-sitter（如果计划投放代码 corpus）
- Day 15 MCP Server（如果优先验证 Claude Code 接入）

### 重要的"非完成"项

- ❌ ROADMAP 维护规则尚未强制（先观察）
- ❌ 自动校验 ROADMAP "当前位置"与最新 commit 一致（未来 backlog）
- ❌ ROADMAP "剩余切片"无外部 source of truth（自我维护，依赖 agent 自觉）
