# feedback/ — Agent 交互归档总入口

`feedback/` 是所有 Agent 交互记录的归档入口，分两个子目录：

| 子目录 | 内容 | 生成工具 |
|---|---|---|
| `feedback/meta/` | 元变更归档（规则/接口/流程/Skill 变更），带连续编号 | `bash scripts/new-feedback.sh <slug>` |
| `feedback/features/` | 业务功能演化档案（按特性归档，多轮迭代） | `bash scripts/new-feature.sh <name>` / `new-feature-log.sh` |

**元变更**（修改工程结构、规则、接口、流程……）必须在 `feedback/meta/` 留下一份 feedback 文件。
**业务迭代**（写代码、跑 benchmark、生成报告）的演化档案放在 `feedback/features/<name>/`。

让后续 agent 能从历史中追溯"为什么有这条规则 / 这个接口 / 这个实现选择"。

---

## 元变更 vs 业务迭代

由 [`scripts/classify-change.sh`](../scripts/classify-change.sh) 按文件路径自动判定。

| 元变更（需要 feedback/meta/） | 业务迭代（feedback/features/ 或无需） |
|---|---|
| `.claude/skills/**` | `<项目业务代码目录>/**` |
| `.claude/references/**` | `reports/**` |
| `scripts/**` | `feedback/features/**` |
| `.github/workflows/**` | |
| 任意层级 `AGENTS.md` | |
| 根 `CLAUDE.md` / `README.md` | |
| `feedback/DESIGN.md` / `refactor-rules.md` / `AGENTS.md` | |
| `feedback/meta/[N]-*.md`（迭代日志本身） | |

**边界例子**：

- `src/components/AGENTS.md` → 元变更（任意 AGENTS.md 都算）
- `src/components/MyFeature.ts` → 业务迭代（留 feature log）
- 改了 `mapping-list.md` + 同时改了 `src/Foo.ts` → 混合，仍需 meta feedback（因为有元变更）

pre-commit hook 自动拦截"元变更但未关联 feedback"的提交。

---

## feedback/meta/ 文件命名

```
{序号}-{YYYY-MM-DD}-{简述}.md
```

- **序号**：从 1 开始严格递增，不跳号（M-FB-01 校验）
- **日期**：本轮启动当天
- **简述**：3-6 个英文单词或拼音，连字符分隔；概括本轮主题

示例：
- `1-2026-05-13-harness-building.md`
- `4-2026-05-14-harness-engineering-review-and-feedback-restructure.md`

## feedback/features/ 目录结构

```
feedback/features/<feature-name>/
  README.md           ← 特性概览 + 状态 + 迭代索引
  1-{date}-{slug}.md  ← 第 1 轮日志
  2-{date}-{slug}.md  ← 第 2 轮日志
  ...
```

---

## 残留项追踪

每份归档（meta 或 feature log）的"残留/下一轮"节必须用 `- [ ]` / `- [x]` 复选框标记：

```markdown
## 残留 / 下一轮处理

- [x] 已完成的事项
- [ ] 未解决的遗留问题
```

查询命令：
```bash
bash scripts/query-pending.sh              # 全部归档
bash scripts/query-pending.sh --meta       # 只看 meta
bash scripts/query-pending.sh --features   # 只看 features
bash scripts/query-pending.sh --last 3     # 只看最近 3 轮 meta
```

`preflight.sh` 每轮 skill 开始时自动扫描并汇报。`check-consistency.sh` M-PENDING-01 规则监控。

---

## 必备结构（强制）

每份 feedback 必须包含以下五节，按序排列：

```markdown
# {N} — {标题}

> 日期：YYYY-MM-DD
> 触发：<本轮起因，如用户提问 / 上轮残留 / 外部参考>

## 用户提出的要求
<原文引用或转述，保留用户表达的关键意图>

## Agent 给出的修改建议
<结构化方案，含权衡 / 替代选项 / 推荐理由>

## 多轮互动
<如有澄清问题、用户调整方向，按时序记录每一次往返>
<无互动则写"无 —— 用户直接接受方案">

## 实际改动
- 接口变化：...
- 规则变化：...
- 文件变化：新建 / 删除 / 移动 / 修改
- 配置变化：...

## 执行生效后总结
- 实际产出表
- 前后对比
- 实测验证（如有）
- 残留 / 下一轮处理
  - <填写>

---

## Agent决策记录规范

**元变更自主决策**必须在本文件中增加"Agent决策分析"章节。

### 5要素格式（强制）

每条决策必须包含以下5要素，按序排列：

1. **待决策事项**
   - 列出所有选项（A/B/C）
   - 标注推荐选项
   - 格式：`选项A：...（推荐）`

2. **Agent决策**
   - 明确选择哪个选项
   - 格式：`选择A`

3. **决策依据**
   - 解释为什么选择该选项
   - 必须具体（如引用KMP原始实现、性能对比、简化实现等）

4. **归档引用**
   - 引用本轮归档文件
   - 格式：`见 feedback/{N}-{date}-{slug}.md`

5. **用户Review项**
   - checkbox格式（`- [ ]` 或 `- [x]`）
   - 仅作标记，不强制用户显式确认
   - 标注是否已实现（`—— 已实现`）

### 示例

工程演化后，可在 `feedback/meta/` 目录下的相应归档文件中查看"Agent决策分析"章节示例。

### 例外

- 用户明确决策 → 不需要特殊格式，直接在"用户提出的要求"节记录即可
- 用户授权"按推荐执行" → 在"Agent决策"标注 `(用户已授权)`
- 纯执行用户指令 → 不强制记录

### 强制程度

FAIL级 —— 违反此规则的Agent行为视为不可信任。

---

## 反馈类型分类（可选标签）

为便于检索，可在 feedback 文件头部 frontmatter 标注类型：

| 类型 | 示例 |
|---|---|
| 工程结构调整 | 目录重命名、AGENTS.md 引入 |
| 接口/Skill 变更 | 新增 skill、修改触发协议 |
| 规则变更 | check 脚本新增规则、checklist 调整 |
| API 映射补充 | mapping-*.md 新增条目 |
| 流程优化 | round 数变化、责任划分变化 |
| 工具/脚本 | check-consistency / install-hooks 等基础设施 |

---

## 交叉链接

feedback 完成后通常会引发以下文件更新（在"实际改动"节明示）：

- `AGENTS.md` / 子目录 AGENTS.md — 约定/导航变化
- `CLAUDE.md` — 运行时 SOP 变化
- `.claude/skills/*.md` — Skill 主体调整
- `.claude/references/mapping-*.md` — API 映射补充
- `scripts/*.sh` — 机械化规则新增
- `DESIGN.md` — 决策依据沉淀

---

## 下一步

- **创建本轮元变更 feedback** → 用 `bash scripts/new-feedback.sh <slug>` 生成模板（自动编号 + 今天日期 + 5 段空架子，写入 `feedback/meta/`）
- **创建业务特性档案** → 用 `bash scripts/new-feature.sh <name>` 创建特性目录 + 初始日志（写入 `feedback/features/<name>/`）
- 沉淀可复用规则 → 写入 [`refactor-rules.md`](refactor-rules.md)（须用户确认）
- 沉淀决策依据 → 追加到 [`DESIGN.md`](DESIGN.md)

**注意**：自 N≥4 起，`feedback/meta/` 中的 feedback 必须遵循上方"必备结构"——pre-commit 会校验 5 段标题。漏段会被 `M-FB-FORMAT` 规则阻止 commit。
