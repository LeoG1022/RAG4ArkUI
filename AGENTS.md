# agent-harness-template — Agent 入口

> 任何 AI agent / skill 进入本工程的第一站。
> 用户向说明见 [`README.md`](README.md)；Claude Code 运行时 SOP 见 [`CLAUDE.md`](CLAUDE.md)。
>
> **🆕 第一次接触本工程？** 先读 [`ONBOARDING.md`](ONBOARDING.md) 了解"必读 / 启动 / 按需"三层加载策略，把 bootstrap 成本从 ~48K 降到 ~5K tokens。

---

## ⚠️ AGENT 接入声明（强制）

任何 AI agent / LLM（不论接入方式）在执行**第一个 Edit / Write / Bash 写操作前**，必须在响应中显式声明：

> 已阅读 ONBOARDING.md 与 AGENTS.md，遵守全局规则 #12（不可逆操作禁令）与 #1（Git 前置检查）。

省略此声明 = 用户不应信任后续行为。本声明对**所有 agent 框架与 LLM 模型一视同仁**：

- Agent 工具：Claude Code / opencode / Cursor / Aider / Cline / Windsurf / Continue.dev / hermes / 自建 API agent
- LLM Provider：Anthropic / OpenAI / DeepSeek / 阿里百炼 Qwen / 智谱 GLM / 开源 Llama / Mistral / 其他

如你的工具未自动加载本文件（如 Continue / hermes / 自建 agent），请用户先把 [`RULES_FOR_AGENTS.md`](RULES_FOR_AGENTS.md) 全文粘贴到对话上下文。

每家工具的入口指针文件（薄包装，全部指向本文件）：
- `.cursorrules` + `.cursor/rules/main.mdc`
- `.aiderrules` + `CONVENTIONS.md`
- `.clinerules` / `.windsurfrules` / `.continue/rules.md` / `.hermes/rules.md`
- `RULES_FOR_AGENTS.md`（兜底）

---

## 通用前置协议（所有写操作前必须执行）

### Pre-flight: Git 状态检查

任何 skill / agent 在执行 Edit / Write / 删除 / 重命名 / Bash 写操作前，**必须**：

1. 跑 `git status --porcelain`
2. 如果输出非空（有未提交修改）：
   - **停下来**，向用户列出未提交文件
   - 询问：「检测到有未提交的修改。是否需要先提交后再进行本次修改？(commit / stash / proceed)」
   - 等用户明确回复后才继续：
     - `commit` → 等用户提交完成后回 `done`
     - `stash` → 跑 `git stash -u`，操作完成后提醒用户 `git stash pop`
     - `proceed` → 继续，但在最终 summary 显式标注「用户选择在未提交状态下继续」
3. 输出为空才直接执行写操作

**例外**：纯只读操作（Read / Grep / Bash 只读命令、Plan 模式下编辑 plan 文件）不受此协议约束。

---

## 子目录索引

每个目录都有自己的 `AGENTS.md`，说明本目录"放什么 / 约定 / 下一步去哪里"。

| 目录 | 用途 | 入口 |
|---|---|---|
| [`.claude/skills/`](.claude/skills/AGENTS.md) | 核心 Skill 定义 | skill 命名约定 + 必备节 |
| [`.claude/references/`](.claude/references/AGENTS.md) | 按需加载的参考表 | mapping 拆分约定 + 关键词路由 |
| [`scripts/`](scripts/AGENTS.md) | 机械化校验脚本 | 退出码契约 |
| [`feedback/`](feedback/AGENTS.md) | Agent 交互归档总入口（`meta/`=元变更 `features/`=业务特性） | 命名规范 + feedback 模板 + 残留追踪 |
| [`reports/`](reports/AGENTS.md) | 自动产物输出目录 | 报告结构 |
| [`stats/`](stats/AGENTS.md) | Token / 改动量统计（agent commit 后主动记录，用户手动分析） | 字段定义 + log/report 工具 |

---

## 全局硬性规则

所有 skill / agent 必须遵守：

1. **Git 前置检查**（见上文）—— 写操作前必跑；**例外**：`stats/tokens.jsonl` 由 agent 持续追加，未提交属正常状态，检查时忽略该文件的 dirty 状态
2. **每次 skill 调用首步必跑** `bash scripts/preflight.sh` —— 一站式查 git 状态 + 一致性 + 残留追踪
3. **Agent commit 后主动记录统计**：Agent 每次执行 `git commit` 成功后，必须紧跟调用 `bash scripts/log-tokens.sh --from-commit HEAD`。用户手动提交不需要此步骤（无 hook，不自动触发）。
4. **mapping 按关键词路由加载**—— 不要一次性 Read 全部 mapping，按 `.claude/references/AGENTS.md` 的路由表选择
5. **生成 / 转换 / 重构 后必跑** `scripts/check-api-parity.sh`，FAIL 必须修复
6. **遇到 mapping 缺失** → 停下来问用户，禁止自行猜测（详见 [`feedback/DESIGN.md`](feedback/DESIGN.md)）
7. **重构经验沉淀** → 重构类 skill Round 4 把验证规则追加到 [`feedback/refactor-rules.md`](feedback/refactor-rules.md)，须用户确认
8. **元变更必须留 feedback**：修改 `.claude/`、`scripts/`、任意 `AGENTS.md`、根 `CLAUDE.md`/`README.md`、`feedback/DESIGN.md` 等规则池、`.github/workflows/` 等"元文件"时，必须用 `bash scripts/new-feedback.sh <slug>` 生成模板并随同提交。**业务迭代**（项目业务代码目录下的改动）不强制要求 feedback。分类由 `scripts/classify-change.sh` 自动判定，由 pre-commit hook 拦截"元变更无 feedback"的提交。
9. **元变更必须主动复述告知用户**：Agent 完成任何一组写操作后，必须跑 `bash scripts/classify-change.sh`。若退出码 ≠ 0（meta / mixed），**必须**把脚本输出的 `🔔 元变更检测` 醒目块**原样**嵌入对用户的回复。禁止"我改完了"沉默交差。
10. **AI 驱动的业务迭代必须自动 feature log**：任何 skill（除只读豁免 skill 外）完成业务代码改动后，**必须**调用 `bash scripts/new-feature.sh <name>`（新特性）或 `bash scripts/new-feature-log.sh <name> <slug>`（已有特性新一轮），并把日志内容由 agent 直接填好——不依赖用户额外输入。`<name>` 推断：业务代码主文件 PascalCase → kebab-case。若无法确定，agent 必须**问用户**而不是静默跳过。纯人工编辑（用户在 IDE 直接改）不强制此规则。
11. **残留项强制追踪**：每轮 feedback（无论 `feedback/meta/` 还是 `feedback/features/` 中的 feature log）必须在"残留/下一轮"节用 `- [ ]` / `- [x]` 复选框明确标记。Agent 每轮 skill 调用开始时，`preflight.sh` 会自动列出未解决残留项；Agent 也可手动跑 `bash scripts/query-pending.sh` 查询。有未解决项时，必须向用户汇报，并在本轮新建的归档文件顶部注明"来自上轮残留"。`check-consistency.sh` 的 M-PENDING-01 规则在 CI 中 WARN。
12. **历史档案绝对不可删除**：已提交的 `feedback/meta/[N]-*.md`、`feedback/DESIGN.md`、`feedback/refactor-rules.md`、`feedback/features/*/[N]-*.md`、`feedback/features/*/README.md` 禁止删除或重命名（含编号前缀变更）。Pre-commit（M-FB-01）+ commit-msg（check-archive-deletion）双重拦截。**无 override 通道**——确需清理（极端情况）只能 `git commit --no-verify` 显式破坏，留显眼痕迹给审计。建议：旧记录有错 → **新增一条修正日志**或在原档案末尾补 errata 段，而非删除/重写历史。
13. **Agent 禁止主动执行不可逆操作**：以下操作必须先向用户描述意图并得到显式授权（明确"yes / proceed / 做"等），agent **不得**自主发起：

    **明确禁止**：
    - `git reset --hard <任意 ref>`（含 HEAD、HEAD~N、commit hash）
    - `git push --force` / `git push --force-with-lease`
    - `git rm` 已提交的归档文件（feedback / features 中的 N-*.md / README.md / DESIGN.md / refactor-rules.md / 任意 AGENTS.md）
    - `git branch -D`
    - `rm -rf` 跨 git 受控目录
    - `git commit --no-verify` 绕过 hook（**AI agent 绝对禁止，用户可用但需标记**）
    - 任何修改远端历史的强制操作

    **允许（可逆，agent 自由使用）**：
    - `git add` / `git restore --staged`（仅 unstage）
    - `git checkout -b` / `git switch -c`（新建分支）
    - 写新文件、`Edit` / `Write` 工具修改文件
    - 当前轮次内自己刚创建的、未提交的文件可自由删除
    - `rm /tmp/...` 自己刚创建的临时文件

    **执行规则**：agent 决定要做不可逆操作时必须**暂停并提议**：

    > 「我打算 `<命令>`，目的是 `<目的>`，影响范围 `<X>`。是否继续 (yes / no)」

    得到 `yes` 才能执行。**禁止以"测试规则"为由自主跑破坏性命令**。

14. **`--no-verify` 绝对禁令（FAIL级硬性规则）**：
    - **AI agent**: 绝对禁止使用 `--no-verify`，任何情况下都不得绕过 pre-commit / commit-msg / post-commit hook
    - **硬性校验**：
      - `scripts/commit.sh` wrapper：拒绝 `--no-verify` 参数，所有 agent commit 必须通过此脚本
      - `check-no-verify.sh` + `check-consistency.sh M-NO-VERIFY-BAN`：审计最近 N 个 commit 是否经过 hook 验证，未经验证 → FAIL
      - `post-commit` hook：记录已验证 commit hash 到 `.git/hooks/.last-verified`
    - **用户**: 紧急情况可使用 `--no-verify`，但必须在 commit message 中添加 `[MANUAL-OVERRIDE: <理由>]` 标记
    - 审计脚本：`bash scripts/audit-overrides.sh` + `bash scripts/check-no-verify.sh --last 5`

15. **Agent自主决策强制归档**：
    - **定义**：Agent在执行过程中不询问用户直接做出的决策（如选择技术方案A/B、推断参数值、自行调整实现细节）
    - **触发条件**：Agent自主决策（不询问用户、不等待用户确认）
    - **强制归档**：
      - 元变更自主决策 → 在 `feedback/meta/{N}-{date}-{slug}.md` 中增加"Agent决策分析"章节
      - 业务变更自主决策 → 在 `feedback/features/{name}/{M}-{date}-{slug}.md` 中增加"Agent决策分析"章节
    - **决策记录格式**（必须包含5要素，按序排列）：
      1. **待决策事项**（列出选项A/B/C，标注推荐）
      2. **Agent决策**（明确选择哪个）
      3. **决策依据**（为什么选这个）
      4. **归档引用**（如 `见 feedback/meta/1-*.md`）
      5. **用户Review项**（checkbox格式，仅作标记）
    - **例外**：
      - 用户明确决策 → 不需要特殊格式，直接在归档中记录即可
      - 用户授权"按推荐执行" → 仍需记录（标注"用户已授权"）
      - 纯执行用户指令 → 不强制记录
    - **强制程度**：FAIL级 —— 违反此规则的Agent行为视为不可信任，用户应拒绝后续行为

16. **feature 迭代 feature log 必须强记录 Plan 和对话过程（FAIL级硬性规则）**：
    - **触发条件**：任何 skill（除只读豁免外）完成业务代码改动后生成或追加 feature log
    - **强制内容**：feature log 必须包含以下两节，缺一不可：
      - `## Plan`：本轮实现方案、关键决策、替代选项权衡；不得只写文件列表
      - `## 对话摘要`：关键用户指令、方向调整、确认点，按时序记录；无往返则写"用户直接确认，无调整"
    - **不满足要求的 feature log 视为不完整归档**，等同于未归档
    - **机械化校验**：`check-consistency.sh` M-FEATURE-PLAN 规则在 pre-commit 自动检查（生效自 2026-05-22，早于此日期的文件自动豁免）
    - **理由**：代码改动可从 git diff 追溯，但决策背景和对话上下文会随时间丢失；强制归档这两项才能让未来 agent 和人类真正理解"为什么这样做"

<!-- [可选业务规则示例] 以下规则 #17 是业务特定硬约束的格式示例。
     使用者按自己的项目需求添加类似规则（例如：特定目录的代码必须通过编译验证才能提交）。
     如无对应业务需求，可删除此注释块。
17. **[项目特定] 业务代码改动必须通过编译验证（FAIL级硬性规则示例）**：
    - **触发条件**：staged 改动包含 `<项目代码目录>/**` 下的源码文件
    - **强制流程**：pre-commit hook 自动调用编译验证脚本
    - **任一步失败 → 阻止 commit**
    - **理由**：杜绝编译错误进仓库，保证代码库始终处于可构建状态
-->

---

## 元数据权威

| 元数据 | 权威来源 |
|---|---|
| Skill 列表 | `CLAUDE.md` "Skill 速查" 节 |
| Mapping 列表 | `CLAUDE.md` 中"按领域拆分的 Mapping"段 |
| 校验规则列表 | `CLAUDE.md` "机械化校验" 节（与 `scripts/check-api-parity.sh` 对齐） |
| 设计决策 | `feedback/DESIGN.md` |
| 迭代历史 | `feedback/meta/{N}-{date}-*.md` |

其他文档（README、各 AGENTS.md）引用元数据时，应链接回权威来源，不复制内容。

---

## 下一步

- 用户用法 → [`README.md`](README.md)
- Claude Code 运行时 SOP → [`CLAUDE.md`](CLAUDE.md)
- 想加新 skill → [`.claude/skills/AGENTS.md`](.claude/skills/AGENTS.md)
- 想加新 mapping → [`.claude/references/AGENTS.md`](.claude/references/AGENTS.md)
- 提交本轮工作记录 → [`feedback/AGENTS.md`](feedback/AGENTS.md)
