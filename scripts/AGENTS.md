# scripts/ — 机械化校验与基础设施

所有自动化检查脚本。退出码契约统一为 **0 = PASS / 1 = FAIL / 2 = WARN**。

---

## 框架脚本（直接可用）

| 脚本 | 用途 | 触发时机 |
|------|------|----------|
| [`check-api-parity.sh`](check-api-parity.sh) | 项目代码合规检查（规则由使用者按项目添加） | skill 完成业务代码改动后；pre-commit hook |
| [`check-consistency.sh`](check-consistency.sh) | 跨文档元数据 + 结构一致性（19 条规则） | pre-commit hook；CI；preflight |
| [`check-archive-deletion.sh`](check-archive-deletion.sh) | 拦截删除/重命名归档文件（feedback、features 中的 N-*.md / README.md） | commit-msg hook |
| [`classify-change.sh`](classify-change.sh) | 按路径分类 staged 改动（meta / business / mixed） | pre-commit hook；skill summary；preflight |
| [`preflight.sh`](preflight.sh) | skill 入口仪式：git 状态 + 改动分类 + 业务文件 vs feature 同步 + 一致性 + 残留 | 每个 skill 的 Step 0 强制调用 |
| [`new-feedback.sh`](new-feedback.sh) | 生成下一轮 feedback 模板（自动编号 + 5 段空架子） | 写新 feedback 前；元变更必须关联 |
| [`parse-skill-meta.sh`](parse-skill-meta.sh) | 解析 skill frontmatter（输出 key=value 或 JSON） | check-consistency；regenerate-skill-table 内部使用 |
| [`regenerate-skill-table.sh`](regenerate-skill-table.sh) | 基于 skill frontmatter 重写 CLAUDE.md SKILL-TABLE 段 | 加新 skill 后；pre-commit `--check` 模式校验 |
| [`new-feature.sh`](new-feature.sh) | 创建业务特性目录 + 第 1 条迭代日志 | AI 驱动业务改动后强制调用 |
| [`new-feature-log.sh`](new-feature-log.sh) | 在已有特性下追加迭代日志 | AI 驱动业务改动后强制调用 |
| [`log-tokens.sh`](log-tokens.sh) | 记录每 commit 到 `stats/tokens.jsonl`（auto + annotate + backfill） | post-commit hook 自动；用户手动 annotate |
| [`stats-report.sh`](stats-report.sh) | 从 jsonl 生成 token/改动量统计报告（5 mode） | 用户触发 |
| [`install-hooks.sh`](install-hooks.sh) | 一键安装 `.git/hooks/` 下所有 hook | 工程初始化 / clone 后 |
| [`prepare-commit-msg-hook.sh`](prepare-commit-msg-hook.sh) | 元变更时在 commit message 添加 `[MANUAL-OVERRIDE: <理由>]` 提示 | prepare-commit-msg hook |
| [`check-no-verify.sh`](check-no-verify.sh) | 检测 `--no-verify` 提交痕迹（对比 post-commit 标记） | check-consistency M-NO-VERIFY-BAN；审计 |
| [`commit.sh`](commit.sh) | Agent commit wrapper，拒绝 `--no-verify` 参数 + 自动记录统计 | AI agent 提交必须通过此脚本 |
| [`audit-overrides.sh`](audit-overrides.sh) | 扫描历史提交中的 `[MANUAL-OVERRIDE]` 标记 | 审计 |
| [`query-pending.sh`](query-pending.sh) | 查询归档中未解决的 `- [ ]` 残留项 | preflight；每轮开始时 |

---

## 退出码契约

```
0  →  全部通过，可提交
1  →  FAIL，必须修复才能继续（pre-commit 阻止提交）
2  →  WARN，建议修复但不阻止
```

任何新增脚本必须遵守这个契约。

---

## check-api-parity.sh 规则索引

| 规则 ID | 检查 | 等级 |
|---|---|---|
| （使用者自定义） | — | — |

规则背后的"为什么"见 [`../feedback/DESIGN.md`](../feedback/DESIGN.md)。

---

## check-consistency.sh 规则索引（19 条）

| 检查项 ID | 等级 | 逻辑 |
|---|---|---|
| M-SKILL-01 | FAIL | `.claude/skills/*.md` 文件数与 CLAUDE.md "Skill 速查" 表行数一致 |
| M-MAP-01 | FAIL | `.claude/references/mapping-*.md` 文件数与 CLAUDE.md 列表一致 |
| M-RULE-01 | WARN | `check-api-parity.sh` 规则数与 CLAUDE.md "机械化校验" 表行数一致 |
| M-FB-01 | FAIL | `feedback/meta/[0-9]*-*.md` 编号 1..N 无跳号 |
| M-AGENTS-01 | WARN | 所有顶层子目录均存在 `AGENTS.md` |
| M-ROOT-01 | WARN | 根 AGENTS.md 覆盖所有主要子目录 |
| M-FB-FORMAT | FAIL | feedback（N≥4）必含 5 段标题 |
| M-MAP-AP | FAIL | 每份 `mapping-*.md` 必含 `## Anti-Patterns` 节 |
| M-SKILL-PREFLIGHT | FAIL | 每份 skill 必含 AGENTS.md + Git 前置引用 |
| M-LINK-DEAD | FAIL | feedback/*.md 与 */AGENTS.md 中相对路径链接必须存在 |
| M-SKILL-SUMMARY-CLASSIFY | FAIL | 每份 skill 必含 `classify-change.sh` 引用 |
| M-FEATURE-NAMING | WARN | feature 目录 kebab-case + 必含 README.md + 日志编号连续 |
| M-FEATURE-NO-META | WARN | features/*/[N]-*.md 不应含元术语 |
| M-FEATURE-PLAN | FAIL | feature log（≥2026-05-22）必含 `## Plan` 和 `## 对话摘要` 节 |
| M-SKILL-FEATURE-LOG | FAIL | 非豁免 skill 必含 new-feature(-log).sh 引用 |
| M-SKILL-FRONTMATTER | FAIL | skill 必含合法 frontmatter（见 `.claude/skills/SKILL_SCHEMA.md`）|
| M-SKILL-REF-VALID | FAIL | frontmatter `calls` 路径存在+可执行；`references` 路径存在 |
| M-SKILL-TABLE-SYNC | FAIL | `regenerate-skill-table.sh --check` 与 CLAUDE.md SKILL-TABLE 一致 |
| M-README-PURE | WARN | README 中 agent/LLM/prompt 出现次数 ≤ 阈值 |

---

## classify-change.sh 退出码契约

| 退出码 | 含义 |
|---|---|
| 0 | 纯业务 / 无 staged 改动 |
| 1 | 纯元变更（meta） |
| 2 | 混合（meta + business） |

≠ 0 时，stdout 末尾输出 `🔔 元变更检测` 醒目块——agent 必须把该块原样复述给用户（[`AGENTS.md`](../AGENTS.md) 全局规则 #9）。

---

## 新增规则的流程

1. 跑 Git 前置检查（见 [`../AGENTS.md`](../AGENTS.md)）
2. 在 [`../feedback/DESIGN.md`](../feedback/DESIGN.md) 记录决策（为什么这条规则必要）
3. 在脚本中加 `grep` / `awk` 检测块，分配 ID（`P-` 性能 / `R-` 资源 / `S-` 状态 / `C-` 复杂度 / `M-` 元数据）
4. 同步更新 [`../CLAUDE.md`](../CLAUDE.md) 的规则表 + 本文件的 ID 索引
5. 在 `../feedback/{N}-{date}-*.md` 记录本轮迭代

---

## pre-commit hook

`install-hooks.sh` 安装的 hook 在 commit 前自动跑：

1. `check-api-parity.sh` 扫描 staged 文件 → 任一 FAIL 阻止 commit
2. `classify-change.sh` 分类 staged 改动 → 元变更必须有 `feedback/meta/[N]-*.md`；业务变更必须有 `feedback/features/<name>/<N>-*.md` → 缺则阻止 commit
3. `check-consistency.sh` 跨文档校验 → FAIL 阻止 commit

跳过 hook（不推荐，仅紧急情况）：`git commit --no-verify`。

---

## 下一步

- 想加新规则 → 流程见上方"新增规则"节
- 想知道规则的设计依据 → [`../feedback/DESIGN.md`](../feedback/DESIGN.md)
- 想加新检查脚本 → 沿用退出码契约 + 在本文件登记 + 加 hook
