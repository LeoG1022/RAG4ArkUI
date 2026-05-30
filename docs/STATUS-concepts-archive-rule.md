# STATUS — concepts-archive-rule

> 配套 feature log：`feedback/features/rag4arkui-core/33-2026-05-30-concepts-archive-rule.md`
> 配套 meta：`feedback/meta/17-2026-05-30-concepts-archive-rule.md`
> 日期：2026-05-30

---

## 当前状态

新增**全局硬性规则 #18**：概念问答必须询问归档。

本阶段交付：
- `AGENTS.md` 全局规则从 17 条扩展到 18 条
- `docs/concepts/README.md` 加「Agent 自我约束（硬性规则）」节
- 双轨归档 + STATUS 三件套齐全

知识库基础设施（Round 32 / meta 16 建好）的**触发层**补完——用户主动说「归档」之外，agent 也必须主动询问。

## 输入契约

无 CLI / API / 文件格式变化。本规则的"输入"是用户的对话——属于 agent 行为契约层。

触发条件（agent 必须识别）：用户提问意图属以下任一：

| 类型 | 典型问法 |
|---|---|
| 直接询问 | 「X 是什么」「X 有什么用」「X 有什么功能」 |
| 设计 / 对比 | 「为什么用 X」「X 跟 Y 啥区别」 |
| 深度 / 原理 | 「X 怎么工作」「X 的原理」 |

边界（不触发）：
- 用户问操作步骤（「怎么跑 X」「怎么装 X」）→ 走文档而非 concepts/
- 用户问 bug 状态（「X 为什么失败」）→ 走 feedback 而非 concepts/

## 输出契约

Agent 答完概念问题后的输出必须包含一条**主动询问**：

> 「这个解答要不要归档到 `docs/concepts/<term>.md`？」

变种（措辞可调，意图不变）：「需要归档吗 / 这个值得沉淀吗 / 要不要写 concepts/<term>.md」。

**禁止的输出**：
- 沉默掠过（答完直接结束）
- 默认归档（不问就开始写文件）
- 默认不归（不问就当作不需要）

用户答「要」→ 触发 5 步归档流程（见 `docs/concepts/README.md`）。
用户答「不」→ 本轮结束 · 下次同主题问仍需重复询问。

## 验证手段

### 用户手动

下次问一个新概念（如「LSP 是什么」「ONNX 是什么」），观察 agent 输出末尾是否包含归档询问。

- 包含 → 规则生效 ✓
- 不包含 → 规则未生效 · 应记 errata 到 `feedback/meta/18-...-rule-violation.md` 并修正 agent 行为

### 自动化

无机械化校验——属对话行为规则，pre-commit hook 无法静态扫描 agent 是否在对话中说了某句话。

唯一相关的自动化是 pre-commit 对**归档结构**的检查：
- M-FB-01 编号连续
- M-FB-FORMAT meta 5 段
- M-FEATURE-PLAN Plan + 对话摘要
- M-STATUS-PER-ROUND 配套 STATUS

这些只保证「归档触发后产物合规」，不保证「触发本身被遵守」。

## 与上一阶段的关联性

| 阶段 | Round | 产出 | 角色 |
|---|---|---|---|
| 上上轮 | meta 15 / feature 31 | README 精简 + 文档导航表 | 用户视角入口梳理 |
| 上一轮 | meta 16 / feature 32 | `docs/concepts/` 知识库目录 + 5 步流程 + 3 篇 concept | 知识沉淀的**基础设施** |
| **本轮** | meta 17 / feature 33 | AGENTS.md #18 + concepts/README.md「自我约束」节 | 知识沉淀的**触发机制** |

增量关系：Round 32 = 静态资产（写了什么）· Round 33 = 行为约束（agent 何时触发流程）。

兼容性：完全向后兼容。原有「用户说归档 → agent 走 5 步」流程不变，本轮只是**再加一条触发路径**「agent 主动询问 → 用户确认 → 走 5 步」。

破坏性变更：无。

## 完成度 / 下一阶段

### 达成度

| 项 | 状态 |
|---|---|
| AGENTS.md 加规则 #18 | ✅ |
| docs/concepts/README.md 加自我约束节 | ✅ |
| feedback/meta/17 归档 | ✅ |
| feedback/features/rag4arkui-core/33 归档 | ✅ |
| docs/STATUS-concepts-archive-rule.md | ✅（本文件）|
| 规则被实际遵守 | ⏳（需下次概念问答时验证）|

### 下一阶段建议

短期：
- 用户下次问任何「X 是什么」时观察 agent 行为，验证规则生效
- 收集 2-3 个新归档样本（agent 询问 → 用户确认 → 5 步执行），看流程是否顺畅

中期（可选 · 看价值）：
- 做 `/archive-concept <term>` skill 自动化 5 步流程
  - 现状：5 步靠 agent 手动执行（写文件 + 更 README + GLOSSARY 加链 + mdbook include + SUMMARY 加项）· 容易漏一两步
  - skill 化后：agent 询问归档 → 用户答「是」 → 一条 skill 调用搞定 5 步
  - 触发条件：当 5 步流程被走过 ≥3 次且都没漏步 → 模式稳定 → 可 skill 化

长期：
- 观察 `docs/concepts/` 是否实际积累内容（目标：1.0 前 ≥10 篇）
- 看是否需要分类（如 `concepts/protocol/` `concepts/retrieval/` `concepts/tooling/`）· 当 ≥15 篇时考虑
