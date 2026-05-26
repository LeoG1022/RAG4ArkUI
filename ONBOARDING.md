# ONBOARDING.md — 新 Agent 入门指南

> **不要盲读全部**。按下面"必读 → 启动 → 按需"三层加载，把 bootstrap 成本控制在 ~5K tokens 以内。
>
> **不使用 Claude Code？** 见 [`RULES_FOR_AGENTS.md`](RULES_FOR_AGENTS.md)（适配 Cursor / Aider / Cline / Windsurf / Continue / opencode / hermes / 自建 API agent + DeepSeek / Qwen / GLM 等 LLM）。

---

## 第一步：必读 3 个文件（按顺序，~5K tokens）

| # | 文件 | 大小 | 内容 |
|---|---|---|---|
| 1 | [`README.md`](README.md) | ~5 KB | 用户视角能力速览、推荐操作流程、IDE 入口 |
| 2 | [`AGENTS.md`](AGENTS.md) | ~4 KB | Agent 入口 + 子目录索引 + 11 条全局规则（含 Git 前置 / 不可逆操作禁令）|
| 3 | [`CLAUDE.md`](CLAUDE.md) | ~10 KB | 运行时 SOP + Skill 速查表 + 机械化校验规则索引 + 工作流 |

读完这 3 份，你已经掌握工程**整体面貌 + 行为约束 + 工具入口**。剩下文档**不要主动读**，触发条件出现时再按需加载。

---

## 第二步：启动 ritual（强制）

任何 skill 调用 / 写操作前：

```bash
bash scripts/preflight.sh
```

输出：git 状态 + 改动分类（meta/business）+ 一致性检查 + 残留追踪 + 业务文件 vs feature log 同步提示。

---

## 第三步：按需读（触发条件出现才读）

| 文件 / 目录 | 触发条件 |
|---|---|
| `feedback/[N]-*.md` | 用户问历史 / 想知道某规则起源 |
| `feedback/DESIGN.md` | 需理解某硬性规则**为什么**（含 benchmark 依据） |
| `feedback/refactor-rules.md` | 重构起规则、想沉淀新模式 |
| `.claude/skills/<name>.md` | 用户触发对应 skill（先用 `parse-skill-meta.sh --field description <file>` 看 80 字摘要再决定是否读正文） |
| `.claude/references/mapping-*.md` | skill 主体提示"按关键词路由加载"时（5 份按 list/state/layout/animation/async 划分） |
| `.claude/references/arkuix-best-practices.md` | `/generate` 或 `/kmp-to-arkuix` 触发时 |
| `.claude/references/arkuix-refactor-checklist.md` | `/arkuix-refactor` 触发时 |
| `.claude/skills/SKILL_SCHEMA.md` | 加新 skill / 改 frontmatter 时 |
| `tests/skills/<skill>/expected/` | 跑 LLM skill 行为回归 review 时 |
| `tests/scripts/<script>/` | 改对应 script 想验证回归时 |
| 各子目录 `AGENTS.md` | 进入该子目录工作时（不要预读全部 9 份） |
| `stats/tokens.jsonl` + `stats/AGENTS.md` | 用户问 token / 改动量统计时 |
| `features/<name>/` | AI 驱动业务改动后必须 feature log（用 `scripts/new-feature.sh` / `new-feature-log.sh`，不需预读） |

---

## 工程演进记录

工程演化历史记录在 `feedback/meta/` 目录中（初始为空，每轮迭代后自动追加）。

查看具体的演化历程：`ls feedback/meta/`，按编号读取对应的 `[N]-*.md` 文件。

---

## 当前关键数据（使用者按实际情况更新）

- **全局规则数**：16 条（见 `AGENTS.md`）
- **自动化规则数**：19（1 parity + 19 一致性，parity 规则由使用者按项目添加）
- **Skill 数**：见 `CLAUDE.md` SKILL-TABLE
- **Git hooks**：4（pre-commit + commit-msg + prepare-commit-msg + post-commit）

---

## 常见任务速查

| 想做什么 | 入口 |
|---|---|
| 跑某 skill | 输入 `/<skill-name>` 触发 |
| 看历史决策依据 | 读 `feedback/DESIGN.md` 找对应决策号 |
| 加新 skill | `.claude/skills/` 新建文件 + 跑 `regenerate-skill-table.sh` |
| 加新规则 | 改 `scripts/check-api-parity.sh` 或 `check-consistency.sh`，同步 DESIGN.md |
| 查 token 用量 | `bash scripts/stats-report.sh` |
| 标 token 真实值 | `bash scripts/log-tokens.sh annotate <round> --input-tokens N --output-tokens N` |

---

## 一句话总览

**先 preflight、按需读、写完跑 classify、commit 跑 hook、留 feedback（meta）或 feature log（AI 驱动 business）。**
