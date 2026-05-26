---
# ── Skill Frontmatter ─────────────────────────────────────────────────────────
# All fields are REQUIRED. See SKILL_SCHEMA.md for spec.
#
name: example-skill                          # kebab-case, must match filename (without .md)
version: 1.0.0                               # semver; bump on body changes
trigger: /example-skill                      # user-typed command, must start with /
description: Example skill template — delete and replace with your own workflow  # ≤80 chars
feature_log_required: true                   # true = AI-driven business changes must leave a feature log
                                             # false = read-only or meta-only skill (exempt)
classify_required: true                      # true = must run classify-change.sh in summary step
preflight_required: true                     # true = must run preflight.sh as Step 0
calls:                                       # scripts this skill invokes (paths must exist + be executable)
  - scripts/preflight.sh
  - scripts/check-api-parity.sh
  - scripts/classify-change.sh
  - scripts/new-feature-log.sh
references:                                  # reference files this skill reads (paths must exist)
  # - .claude/references/example-mapping.md  # uncomment when the file exists
# ─────────────────────────────────────────────────────────────────────────────
---

# /example-skill

> **Delete this file** and replace it with your own skills.
> This file serves as a complete frontmatter + structure reference.

---

## Step 0 — Preflight（强制）

> 已阅读 ONBOARDING.md 与 AGENTS.md，遵守全局规则 #12（不可逆操作禁令）与 #1（Git 前置检查）。

```bash
bash scripts/preflight.sh
```

If there are unresolved pending items from the previous round, report them to the user before proceeding.

---

## Step 1 — [Your first step]

Describe what the agent should do in this step. Be specific:
- What files to read
- What questions to ask the user (if any)
- What decisions to make

---

## Step 2 — [Your second step]

Continue the workflow. Common patterns:
- Load relevant reference files from `.claude/references/`
- Generate or modify code
- Run validation: `bash scripts/check-api-parity.sh <target>`

---

## Step N — Summary & Archive

After all changes are made:

```bash
# 1. Classify changes (MUST report the 🔔 block to user if meta/mixed)
bash scripts/classify-change.sh

# 2. If business code was changed, create/update feature log:
bash scripts/new-feature-log.sh <feature-name> <slug>
# Fill in the ## Plan and ## 对话摘要 sections — do NOT leave them empty.

# 3. If meta files were changed, create feedback:
bash scripts/new-feedback.sh <slug>

# 4. Stage and commit via wrapper (never use --no-verify):
bash scripts/commit.sh -m "feat: <description>"

# 5. Record token statistics:
bash scripts/log-tokens.sh --from-commit HEAD
```

**If `classify-change.sh` outputs a `🔔 元变更检测` block, paste it verbatim in your reply to the user.**
