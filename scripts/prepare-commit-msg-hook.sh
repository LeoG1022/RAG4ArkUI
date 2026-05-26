#!/usr/bin/env bash
# prepare-commit-msg-hook.sh — 元变更时自动添加 override 提示
#
# 此 hook 在 commit message 编辑前运行。
# 如果检测到元变更，会在 message 末尾添加提示注释。
#
# 注意：--no-verify 会跳过此 hook，所以提示只在正常流程有效。
# 用户使用 --no-verify 时需自行添加 [MANUAL-OVERRIDE: <理由>] 标记。
#
# 用法：作为 .git/hooks/prepare-commit-msg 安装
#   参数：$1 = commit message 文件路径
#   参数：$2 = commit 来源（message, template, merge, squash, commit）
#
# 退出码：0 = 继续（不阻止 commit）

set -u

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

MSG_FILE="$1"
COMMIT_SOURCE="${2:-}"

# merge / squash / commit 等来源不需要提示（已有 message）
if [[ "$COMMIT_SOURCE" == "merge" || "$COMMIT_SOURCE" == "squash" || "$COMMIT_SOURCE" == "commit" ]]; then
  exit 0
fi

# 检测元变更
bash scripts/classify-change.sh 2>/dev/null
classify_rc=$?

# 退出码 0 = 纯业务 / 无改动，不需要提示
# 退出码 1 = 纯元变更
# 退出码 2 = 混合（meta + business）
if [[ "$classify_rc" -eq 0 ]]; then
  exit 0
fi

# 元变更或混合 → 在 message 文件末尾添加提示
cat >> "$MSG_FILE" <<'EOF'

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# ⚠️ 元变更检测：本次修改涉及 meta 文件
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
#
# 如需使用 --no-verify 绕过 hook，请在 commit message 中添加：
# [MANUAL-OVERRIDE: <理由>]
#
# 示例：
# [MANUAL-OVERRIDE: 紧急修复 CI 阻塞问题]
#
# 此提示由 prepare-commit-msg hook 自动生成，请勿删除或修改。
# 事后可通过 `bash scripts/audit-overrides.sh` 审计 override 使用情况。
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
EOF

exit 0