#!/usr/bin/env bash
# check-no-verify.sh — 检测是否有 --no-verify 提交痕迹
#
# 机制：pre-commit hook 成功通过后会写入 .git/hooks/.last-verified 文件，
# 记录本次 commit 的 hash。如果 HEAD commit 不在这个文件中，
# 说明该 commit 可能使用了 --no-verify 跳过了 hooks。
#
# 用法:
#   bash scripts/check-no-verify.sh              # 检查 HEAD commit
#   bash scripts/check-no-verify.sh --last N     # 检查最近 N 个 commit
#
# 退出码:
#   0 = PASS（所有被检查的commit都有verified标记）
#   1 = FAIL（发现未经verified的commit，可能用了--no-verify）
#   2 = WARN（首次clone或标记文件不存在，无法判定）

set -u

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

MARKER="$ROOT/.git/hooks/.last-verified"
LAST="${1:-1}"

if [[ "$LAST" == "--last" ]]; then
  LAST="${2:-1}"
fi

# 标记文件不存在（首次clone或全新环境）
if [[ ! -f "$MARKER" ]]; then
  echo "[WARN] check-no-verify: .last-verified 标记文件不存在"
  echo "       可能原因：首次clone / 初始commit / 标记被清理"
  echo "       建议：bash scripts/install-hooks.sh && 正常提交一次以创建标记"
  echo ""
  echo "  退出码:2 (WARN)"
  exit 2
fi

# 读取已验证的 commit hash 列表
VERIFIED_HASHES=$(cat "$MARKER" 2>/dev/null || true)

# 检查最近 N 个 commit
FAIL_COUNT=0
CHECK_COUNT=0

for i in $(seq 0 $((LAST - 1))); do
  HASH=$(git rev-parse HEAD~$i 2>/dev/null || break)
  CHECK_COUNT=$((CHECK_COUNT + 1))

  if echo "$VERIFIED_HASHES" | grep -q "$HASH"; then
    echo "[PASS] check-no-verify: HEAD~$i ($HASH) 经过 pre-commit hook 验证"
  else
    MSG=$(git log -1 --format="%s" "$HASH" 2>/dev/null || echo "unknown")
    echo "[FAIL] check-no-verify: HEAD~$i ($HASH) 未经过 pre-commit hook 验证"
    echo "       Commit message: $MSG"
    echo "       可能使用了 --no-verify 跳过了所有 hooks"
    echo "       AGENTS.md 规则 #14: AI agent 绝对禁止使用 --no-verify"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
done

echo ""
echo "  检查了 $CHECK_COUNT 个 commit，发现 $FAIL_COUNT 个未经验证"

if [[ "$FAIL_COUNT" -gt 0 ]]; then
  echo ""
  echo "❌ 发现未经 pre-commit hook 验证的 commit（可能使用了 --no-verify）"
  echo "   AI agent 绝对禁止使用 --no-verify（AGENTS.md 规则 #14，FAIL级）"
  exit 1
fi

exit 0