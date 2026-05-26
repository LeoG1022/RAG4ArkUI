# RULES_FOR_AGENTS.md — 通用兜底入口

> 你的 AI agent 工具**没自动加载** AGENTS.md / CLAUDE.md / ONBOARDING.md？
> 把本文件全文复制到你的 system prompt 中。覆盖所有未列出的工具 / 自建 API agent / 弱模型。

---

## Required Reading

进入 crossplatform-harness 工程的 AI agent 在执行**任何 Edit / Write / Bash 写操作前**必须读：

1. **[ONBOARDING.md](ONBOARDING.md)** — 3 层加载策略：必读 / 启动 / 按需
2. **[AGENTS.md](AGENTS.md)** — 11 条全局硬性规则
3. **[CLAUDE.md](CLAUDE.md)** — 运行时 SOP + Skill 速查 + 机械化校验

---

## Required Acknowledgment

第一个回复必须显式包含此短句：

> 已阅读 ONBOARDING.md 和 AGENTS.md，遵守规则 #11 和 #13。

省略此 acknowledgment = 用户不应信任后续行为。

---

## 11 条全局硬性规则（速览）

详见 AGENTS.md：

| # | 规则 |
|---|---|
| 1 | Git 前置检查（写操作前 `git status --porcelain`） |
| 2 | Skill 首步必跑 `scripts/preflight.sh` |
| 3 | Mapping 按关键词路由加载 |
| 4 | 生成/转换/重构后必跑 `check-api-parity.sh`，FAIL 必修 |
| 5 | Mapping 缺失 → 停下问用户，不猜 |
| 6 | 重构经验沉淀 → `refactor-rules.md` |
| 7 | 元变更必留 feedback |
| 8 | 元变更必主动复述告知用户（🔔 块）|
| 9 | AI 驱动业务必自动写 feature log |
| 10 | 历史档案不可删 |
| 11 | Agent 禁止主动不可逆操作（reset --hard / push --force / git rm 归档 / --no-verify）|

---

## 支持的 Agent 工具

每个工具有自己的薄入口文件指向 AGENTS.md：

| 工具 | 入口 | 状态 |
|---|---|---|
| Claude Code | `CLAUDE.md` / `AGENTS.md` | 原生支持 |
| opencode | `AGENTS.md` | 原生支持（AGENTS.md 标准早期采纳）|
| Cursor | `.cursorrules` + `.cursor/rules/main.mdc` | 已配 |
| Aider | `.aiderrules` + `CONVENTIONS.md` | 已配 |
| Cline (VS Code) | `.clinerules` | 已配 |
| Windsurf | `.windsurfrules` | 已配 |
| Continue.dev | `.continue/rules.md` + 用户手动复制到 `.continue/config.json` | 半自动 |
| hermes | `.hermes/rules.md`（best-guess）| 未验证，可能需用户手动注入 |
| GitHub Copilot Workspace | `AGENTS.md`（部分支持）| 原生（部分版本）|
| 其他 / API 自建 | 本文件（RULES_FOR_AGENTS.md）| 用户手动复制到 system prompt |

---

## 支持的 LLM Provider

| Provider | 已知可用模型 | 备注 |
|---|---|---|
| Anthropic | Claude 3.5 Sonnet+ / Opus 4.x | CLAUDE.md 原生支持，最佳兼容 |
| OpenAI | GPT-4o / GPT-4-turbo / o1 / o3 | system prompt 支持完整 |
| DeepSeek | deepseek-coder / deepseek-v3 / deepseek-v3.1 | 中文 / 代码双优；用强化 prompt 模板 |
| 阿里百炼 | Qwen2.5-Coder-32B / Qwen-Max / Qwen3 | 中文友好；长上下文支持完整 |
| 智谱 AI | GLM-4-Plus / GLM-4.5 / GLM-4-Long | 中文规则理解好 |
| 开源（Llama / Mistral / Qwen 本地）| 各家 | 建议用强化 prompt 模板 + 必跑 git hooks 兜底 |
| 其他 | 自评估 | 复制本文件到 system prompt 后人工验证规则遵循度 |

**未列出 = 未实测**，不代表不支持；任何遵循指令的 LLM 都可用，effectivenes 取决于具体模型。

---

## 强化 Prompt 模板（中文 / 弱指令遵循模型）

对**非 Claude / 非 GPT-4** 系列模型，建议用户把以下文本作为 system prompt 注入：

```text
你是接入 crossplatform-harness 工程的 AI 助手。

[强制要求]
1. 你必须严格遵循 ./AGENTS.md 中的 11 条全局硬性规则
2. 第一次回复必须包含字符串："已阅读 ONBOARDING.md 和 AGENTS.md，遵守规则 #11 和 #13。"
3. 任何 Edit / Write / Bash 写操作之前，先运行 `git status --porcelain`；输出非空必须先询问用户
4. 不可逆操作（`git reset --hard` / `git push --force` / `git rm` 已提交归档 / `--no-verify` 绕过 hook 等）必须先向用户描述意图 + 影响，得到 yes 才执行
5. 元变更（修改 .claude/、scripts/、AGENTS.md 等）必须先用 `bash scripts/new-feedback.sh <slug>` 生成 feedback 模板再 commit
6. AI 驱动的业务改动必须用 `bash scripts/new-feature-log.sh <name> <slug>` 自动留档

[要读的文件]
- ONBOARDING.md（3 层加载策略）
- AGENTS.md（11 条全局规则）
- CLAUDE.md（运行时 SOP）

[关键 hook 兜底]
- pre-commit / commit-msg / post-commit 已配置，规则违反会被自动拦截
- 紧急绕过 (--no-verify) 留 git log 痕迹，会被审计
```

---

## Hook 兜底保障

无论 agent / LLM 遵守规则与否，git hooks **都会运行**：

- `pre-commit`：拦不合规 ets 代码、元变更无 feedback、跨文档不一致
- `commit-msg`：拦删除/重命名归档
- `post-commit`：自动记录到 `stats/tokens.jsonl`

绕过方式只有 `git commit --no-verify`，会留显眼 git log 痕迹给用户审计。

---

## 多工具协同建议

- 同一仓库可同时给多人 / 多工具用，每人自己的 agent 入口文件独立
- 同时跑多 agent 时注意：`new-feedback.sh` / `new-feature-log.sh` 当前未加文件锁（Round 18 候选）
- 远端 GitHub 启用 branch protection + CI 是绝对兜底（Round 17 候选）

---

## 反馈

发现规则误判 / 工具不兼容 / 模型行为异常？
- 用户：直接在 git log 留 commit
- Agent：跑 `bash scripts/new-feedback.sh <slug>` 记录到 feedback/
