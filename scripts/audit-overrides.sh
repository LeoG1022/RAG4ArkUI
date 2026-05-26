#!/usr/bin/env bash
# audit-overrides.sh — 扫描历史提交中的 MANUAL-OVERRIDE 标记
#
# 用法：
#   bash scripts/audit-overrides.sh              # 扫描最近 100 条提交
#   bash scripts/audit-overrides.sh --all        # 扫描所有提交
#   bash scripts/audit-overrides.sh -N 50        # 扫描最近 50 条
#   bash scripts/audit-overrides.sh --since "2025-01-01"  # 指定起始日期
#   bash scripts/audit-overrides.sh --author "leo"        # 指定作者
#
# 输出：
#   带 [MANUAL-OVERRIDE: ...] 标记的提交列表
#   每条包含：commit hash、作者、日期、override 理由
#
# 退出码：0 = 有结果或无结果都成功

set -u

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

# 默认扫描最近 100 条
N=100
SINCE=""
AUTHOR=""
ALL=false

while [[ $# -gt 0 ]]; do
  case "$1" in
    --all) ALL=true; shift ;;
    -N) N="$2"; shift 2 ;;
    --since) SINCE="$2"; shift 2 ;;
    --author) AUTHOR="$2"; shift 2 ;;
    *) shift ;;
  esac
done

# 构建 git log 命令
if [[ "$ALL" == "true" ]]; then
  LOG_CMD="git log --format=%H%n%an%n%ad%n%B%n---COMMIT_END---"
elif [[ -n "$SINCE" ]]; then
  LOG_CMD="git log --since=\"$SINCE\" --format=%H%n%an%n%ad%n%B%n---COMMIT_END---"
elif [[ -n "$AUTHOR" ]]; then
  LOG_CMD="git log --author=\"$AUTHOR\" -n $N --format=%H%n%an%n%ad%n%B%n---COMMIT_END---"
else
  LOG_CMD="git log -n $N --format=%H%n%an%n%ad%n%B%n---COMMIT_END---"
fi

# 执行并筛选含 MANUAL-OVERRIDE 的提交
OVERIDES=()
CURRENT_COMMIT=""
CURRENT_AUTHOR=""
CURRENT_DATE=""
CURRENT_BODY=""
IN_BODY=false

while IFS= read -r line; do
  if [[ "$line" == "---COMMIT_END---" ]]; then
    # 检查 body 是否含 MANUAL-OVERRIDE
    if echo "$CURRENT_BODY" | grep -qE '\[MANUAL-OVERRIDE:'; then
      REASON=$(echo "$CURRENT_BODY" | grep -oE '\[MANUAL-OVERRIDE: [^\]]+\]' | head -1)
      OVERIDES+=("$CURRENT_COMMIT|$CURRENT_AUTHOR|$CURRENT_DATE|$REASON")
    fi
    CURRENT_COMMIT=""
    CURRENT_AUTHOR=""
    CURRENT_DATE=""
    CURRENT_BODY=""
    IN_BODY=false
  elif [[ -z "$CURRENT_COMMIT" ]]; then
    CURRENT_COMMIT="$line"
  elif [[ -z "$CURRENT_AUTHOR" ]]; then
    CURRENT_AUTHOR="$line"
  elif [[ -z "$CURRENT_DATE" ]]; then
    CURRENT_DATE="$line"
    IN_BODY=true
  elif [[ "$IN_BODY" == "true" ]]; then
    CURRENT_BODY="${CURRENT_BODY}${line}\n"
  fi
done < <(eval "$LOG_CMD")

# 输出结果
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📊 MANUAL-OVERRIDE 审计报告"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if [[ ${#OVERIDES[@]} -eq 0 ]]; then
  echo ""
  echo "✅ 无 MANUAL-OVERRIDE 标记的提交"
  echo ""
else
  echo ""
  echo "⚠️ 发现 ${#OVERIDES[@]} 条带 override 标记的提交："
  echo ""
  for entry in "${OVERIDES[@]}"; do
    HASH=$(echo "$entry" | cut -d'|' -f1)
    AUTHOR=$(echo "$entry" | cut -d'|' -f2)
    DATE=$(echo "$entry" | cut -d'|' -f3)
    REASON=$(echo "$entry" | cut -d'|' -f4)
    echo "  Commit: $HASH"
    echo "  Author: $AUTHOR"
    echo "  Date:   $DATE"
    echo "  Reason: $REASON"
    echo ""
  done
fi

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "💡 提示："
echo "   - AI agent 应绝对禁止使用 --no-verify（AGENTS.md 规则 #11、#12）"
echo "   - 用户使用 --no-verify 时必须添加 [MANUAL-OVERRIDE: <理由>] 标记"
echo "   - 无标记的 --no-verify 使用需人工核查 git log"
echo ""

exit 0