# 33 — concepts-archive-rule

> 日期：2026-05-30
> 涉及代码：`docs/concepts/README.md`（业务侧）· `AGENTS.md`（meta 侧 · 见 `feedback/meta/17-*.md`）
> 类型：规则补充（紧跟 Round 32 concepts-kb 建立的知识库 · 加 agent 自我约束层）

## 本轮目标

Round 32 建好了 `docs/concepts/` 知识库目录 + 5 步归档流程，但**触发**还是"用户主动说归档"。本轮补一条 agent 自我约束规则：

> 用户问「X 是什么 / X 有什么用 / X 有什么功能」类问题时，agent 答完**必须**主动询问是否归档，不能默认归 / 默认不归 / 沉默掠过。

把决策权强制交还用户。

## Plan

本轮设计：

1. **触发场景定义**：用户提问意图属于「概念解释」类。列举 6 种典型问法：
   - 「X 是什么」「X 有什么用」「X 有什么功能」（直接询问）
   - 「为什么用 X」「X 跟 Y 啥区别」（设计/对比）
   - 「X 怎么工作」「X 的原理」（深度）

2. **强制动作**：agent 答完后用一句固定模板询问：
   > 「这个解答要不要归档到 `docs/concepts/<term>.md`？」
   不要替用户做决定。

3. **落地位置（两处）**：
   - `docs/concepts/README.md`：加「Agent 自我约束（硬性规则）」节，含触发列表 + 强制动作 + 三条理由
   - `AGENTS.md`：加全局规则 #18（FAIL 级），13 行 · 含触发条件、强制动作、用户回答两种分支、理由 · 指针式链到 concepts/README.md 看详细

4. **不做**：
   - 不动 `check-consistency.sh`——属对话行为规则，无法静态扫描（在 #18 里明确写"无机械化校验"）
   - 不做 `/archive-concept` skill 自动化——本轮先把规则建起来 · 用过几次再看是否值得做工具

5. **与 Round 32 的关系**：Round 32 = 建知识库（文件结构 + 5 步流程）· Round 33 = 加触发机制（agent 主动询问）· 互补不重复

## 对话摘要

| 轮次 | 用户 | Agent 决策 |
|---|---|---|
| 1 | 提原文「当我问任何。x是什么 x有什么功能 x有什么用 等类似含义问题时。在你回回答后 都要询问用户是否归档。这个是个硬性规则」 | 承认规则 + 提出"规则只活会话里没意义，必须落文件" + 用 AskUserQuestion 给 4 个候选位置 |
| 2 | 选「前两个都记」（docs/concepts/README.md + AGENTS.md） | 双处落地 · concepts/README.md 做详细说明 · AGENTS.md 做契约（指针式）|

无方向调整，用户直接选定方案。

## 改动要点

- **docs/concepts/README.md**：「现有条目」表之后、「加新条目的步骤」之前，插入「Agent 自我约束（硬性规则）」节（21 行）。包含 6 种触发问法 + 强制询问模板 + 3 条理由 + 链回 AGENTS.md #18
- **AGENTS.md（meta · 见 feedback/meta/17）**：#17 STATUS-PER-ROUND 之后插入 #18「概念问答必须询问归档」（FAIL 级）· 注释块业务规则示例编号 #18 → #19 顺移
- 与 Round 32 的差异：Round 32 = 静态资产（写了几篇 concept 文档 + 流程文档）· Round 33 = 行为约束（agent 何时主动触发流程）
- API 选型：无（纯文档 + 规则）

## 验证结果

- 编译：N/A（纯文档）
- check-api-parity：N/A（不动代码）
- pre-commit hook 预期通过：
  - M-FB-01 编号连续 ✓（meta=17，feature=33，紧接 16/32）
  - M-FB-FORMAT meta 5 段齐全 ✓
  - M-FEATURE-PLAN 本文件含 `## Plan` + `## 对话摘要` ✓
  - M-STATUS-PER-ROUND 配套 `docs/STATUS-concepts-archive-rule.md` ✓
  - M-FEATURE-NO-META 本文件不写"meta 变更"等元术语 ✓

事后验证（无法自动化 · 由用户监督）：下次概念问答时观察 agent 是否主动询问归档。违反 = 规则未生效。

## 残留 / 下一轮

- [x] AGENTS.md 加规则 #18 + 注释块编号顺移
- [x] docs/concepts/README.md 加「Agent 自我约束」节
- [x] 双轨归档（meta/17 + feature/33）
- [x] docs/STATUS-concepts-archive-rule.md
- [ ] （行为观察）下次概念问答触发时验证规则被遵守 · 违反则归档为 feedback/meta/18 errata
- [ ] （可选未来）考虑做 `/archive-concept <term>` skill 自动化 5 步归档 · 用户「是」即可触发
