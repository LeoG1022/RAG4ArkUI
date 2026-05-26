# CONVENTIONS.md — Project Conventions for AI Agents

> Aider 主入口 + 通用入口（任何不识别 .cursorrules/.clinerules 等的工具）。
> 真正的规则细节在 AGENTS.md / ONBOARDING.md / CLAUDE.md。本文件是"无歧义入口"。

---

## Must-Read（按顺序，~5K tokens）

1. **[ONBOARDING.md](ONBOARDING.md)** — 3 层加载策略：必读 / 启动 / 按需
2. **[AGENTS.md](AGENTS.md)** — 11 条全局硬性规则
3. **[CLAUDE.md](CLAUDE.md)** — 运行时 SOP + Skill 速查 + 机械化校验

---

## Must-Acknowledge

在第一个回复中显式包含：

> 已阅读 ONBOARDING.md 和 AGENTS.md，遵守规则 #11 和 #13。

省略此 acknowledgment = 用户不应信任后续行为。

---

## 11 条全局硬性规则（速览）

详见 AGENTS.md，简要：

1. Git 前置检查（写操作前 `git status --porcelain`）
2. Skill 首步必跑 `scripts/preflight.sh`
3. Mapping 按关键词路由加载
4. 生成/转换/重构后必跑 `check-api-parity.sh`
5. 遇 mapping 缺失停下问用户
6. 重构经验沉淀（Round 4 写 refactor-rules.md）
7. 元变更必留 feedback
8. 元变更必主动复述告知用户（`classify-change.sh` 输出 🔔 块）
9. AI 驱动业务必自动写 feature log
10. 历史档案不可删除
11. Agent 禁止主动不可逆操作（reset --hard / push --force / git rm 归档 / --no-verify）

---

## 启动 Ritual

```bash
bash scripts/preflight.sh
```

输出 git 状态 + 改动分类 + 一致性 + 残留追踪。

---

## 触发后按需读

| 触发 | 读 |
|---|---|
| 用户问历史 | `feedback/[N]-*.md` |
| 需理解规则"为什么" | `feedback/DESIGN.md` |
| 用户触发某 skill | `.claude/skills/<name>.md`（先用 `parse-skill-meta.sh --field description` 看摘要）|
| Skill 提示"按关键词加载 mapping" | `.claude/references/mapping-*.md` |
| 加新 skill | `.claude/skills/SKILL_SCHEMA.md` |

---

## Git Hooks（不可绕的兜底）

- `pre-commit`：跑 check-api-parity（ets 合规）+ classify-change（meta 须含 feedback）+ check-consistency
- `commit-msg`：跑 check-archive-deletion（归档不可删，无 override）
- `post-commit`：跑 log-tokens（自动记录到 stats/tokens.jsonl）

`git commit --no-verify` 可绕，但留显眼 git log 痕迹。

---

## 多工具适配

本工程同时支持以下 agent 工具，各自有薄入口文件指向本仓库 AGENTS.md：

- Claude Code（原生 CLAUDE.md / AGENTS.md）
- opencode（原生 AGENTS.md）
- Cursor（.cursorrules + .cursor/rules/main.mdc）
- Aider（.aiderrules + 本文件）
- Cline（.clinerules）
- Windsurf（.windsurfrules）
- Continue.dev（.continue/rules.md）
- hermes（.hermes/rules.md，best-guess）
- 其他 / 自建 API agent → 见 [RULES_FOR_AGENTS.md](RULES_FOR_AGENTS.md)
