# 17 — concepts-archive-rule

> 日期：2026-05-30
> 触发：用户在上一轮（meta/16 建立 docs/concepts/ 知识库）之后，明确提出新的行为约束
> 类型：规则变更（AGENTS.md 新增全局规则 #18）

---

## 用户提出的要求

原文：

> 当我问任何。x是什么 x有什么功能 x有什么用 等类似含义问题时。在你回回答后 都要询问用户是否归档。这个是个硬性规则

需求拆解：
- 触发条件：用户问「X 是什么 / X 有什么功能 / X 有什么用」类问题
- 强制动作：agent 答完之后必须主动询问是否归档
- 强制等级：硬性规则（FAIL 级语义）
- 隐含问题：规则只活在当前会话里没意义，必须落地到文件，下次新会话也生效

## Agent 给出的修改建议

只活在对话上下文里的"硬性规则"等同于没有，必须写文件。候选位置：

| 位置 | 优点 | 缺点 |
|---|---|---|
| `docs/concepts/README.md` 加节 | 离场景近 · 跟现有 5 步归档流程同位置 | 只有读到这个文件才生效 · 其它 agent 不一定 grep 到 |
| `AGENTS.md` 加规则 #18 | 跟现有 17 条全局规则同级 · 所有 agent 必读 | 文件偏长 · 详细描述放此处会膨胀 |
| `CLAUDE.md` 加节 | Claude Code 专属 | Cursor / Aider 等不生效 |

推荐方案：**前两个都记**——AGENTS.md #18 一条精炼 FAIL 级规则（指针式），`docs/concepts/README.md` 写"Agent 自我约束"节做详细说明 + 理由。这样：

1. 所有 agent 从 AGENTS.md 入口必读到规则
2. 真要展开看为什么、怎么做，跳到 concepts/README.md
3. 不复制内容，单一权威源（concepts/README.md），AGENTS.md 只放契约

## 多轮互动

| 轮次 | 用户 | Agent |
|---|---|---|
| 1 | 「当我问任何 X 是什么...都要询问用户是否归档。这是硬性规则」 | 承认规则 + 用 AskUserQuestion 问记到哪里（推荐 concepts/README.md）|
| 2 | 「前两个都记」 | 双处落地：concepts/README.md 加「Agent 自我约束」节 + AGENTS.md 加规则 #18 |

无方向调整，用户直接选定方案。

## 实际改动

- 接口变化：无
- 规则变化：AGENTS.md 全局硬性规则从 #17 扩展到 #18；注释块内的业务规则示例编号顺移 #18 → #19
- 文件变化：
  - 修改 `AGENTS.md`：#17 之后插入 #18「概念问答必须询问归档」（FAIL 级）+ 注释块 #18→#19 顺移
  - 修改 `docs/concepts/README.md`：「现有条目」与「加新条目的步骤」之间插入「Agent 自我约束（硬性规则）」节
- 配置变化：无（不动 check-consistency.sh — 此规则属对话行为规则无法静态扫描）

## 执行生效后总结

### 实际产出

| 文件 | 改动 | 行数 |
|---|---|---|
| `AGENTS.md` | 新增规则 #18（13 行）+ 注释块编号顺移 | +14 / -1 |
| `docs/concepts/README.md` | 新增「Agent 自我约束」节 | +21 / 0 |
| `feedback/meta/17-2026-05-30-concepts-archive-rule.md` | 本归档 | +N / 0 |
| `feedback/features/rag4arkui-core/33-2026-05-30-concepts-archive-rule.md` | feature log（business 配套） | +N / 0 |
| `docs/STATUS-concepts-archive-rule.md` | STATUS-PER-ROUND 强制产物 | +N / 0 |

### 前后对比

| 维度 | Before | After |
|---|---|---|
| 「X 是什么」类问题处理 | Agent 答完即结束 · 知识不沉淀 · 下次新读者重问 | Agent 答完**必问**「要不要归档」· 决策权交还用户 · 知识增量被强制评估 |
| 全局硬性规则数 | 17 条 | 18 条 |
| AGENTS.md 注释块业务规则示例 | 编号 #18 | 编号 #19（顺移） |
| 概念归档触发 | 用户主动说「归档」才走 5 步流程 | Agent 主动询问 + 用户确认 → 5 步流程 |

### 实测验证

- `bash scripts/classify-change.sh`（add 后）：输出 `mixed(meta=1, business=1)` ✓（AGENTS.md=meta, docs/concepts/README.md=business）
- pre-commit hook 会触发 M-FB-FORMAT / M-FB-01 / M-FEATURE-PLAN / M-STATUS-PER-ROUND 检查 → 本归档 + feature log + STATUS 三件套齐全应可通过

事后验证（用户监督）：下次用户问「Y 是什么」时观察 agent 是否主动询问归档。违反 = 规则未生效。

### 残留 / 下一轮处理

- [x] AGENTS.md 加规则 #18
- [x] docs/concepts/README.md 加 Agent 自我约束节
- [x] 双轨归档 + STATUS
- [ ] （观察）下次概念问答触发时验证规则被遵守
- [ ] （可选未来）考虑做一个 `/archive-concept <term>` skill 自动化 5 步归档流程，让用户「是」一声即可
