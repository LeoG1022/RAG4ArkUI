#!/usr/bin/env bash
# preflight.sh — skill 入口仪式
#
# 一站式输出当前仓库健康状况，供 skill Step 0 调用。
#
# 用法：bash scripts/preflight.sh
# 退出码：0 = 信息性输出（不阻止后续）
#
# 注意：本脚本只**报告**状态，不阻止写操作。
# 写操作的拦截由 AGENTS.md 规则 13（Git 前置检查）+ pre-commit hook 负责。

set -u

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

bold()   { echo -e "\033[1m$*\033[0m"; }
dim()    { echo -e "\033[2m$*\033[0m"; }
green()  { echo -e "\033[32m$*\033[0m"; }
yellow() { echo -e "\033[33m$*\033[0m"; }
red()    { echo -e "\033[31m$*\033[0m"; }

echo "━━━━━━ Preflight ━━━━━━"

# 1. Git 状态（忽略 stats/tokens.jsonl，该文件由 agent 持续追加，未提交属正常状态）
DIRTY=$(git status --porcelain 2>/dev/null | grep -v '^\s*M\s*stats/tokens\.jsonl$' | head -10)
if [[ -z "$DIRTY" ]]; then
  echo "[INFO] Git 状态：$(green clean ✓)"
else
  COUNT=$(echo "$DIRTY" | wc -l | tr -d '[:space:]')
  echo "[INFO] Git 状态：$(yellow "$COUNT 个文件未提交"，按 AGENTS.md 规则 13 处理)"
  echo "$DIRTY" | head -5 | sed 's/^/         /'
  if [[ "$COUNT" -gt 5 ]]; then echo "         ... 共 $COUNT 项"; fi
fi

# 1.5 改动分类（worktree 范围）
if [[ -n "$DIRTY" ]]; then
  CLASSIFY_OUT=$(bash scripts/classify-change.sh --worktree 2>/dev/null)
  CLASS_LINE=$(echo "$CLASSIFY_OUT" | grep "^分类：")
  if [[ -n "$CLASS_LINE" ]]; then
    case "$CLASS_LINE" in
      *meta*|*mixed*) echo "[INFO] 改动分类：$(yellow "$CLASS_LINE") → 提交前需要 feedback" ;;
      *) echo "[INFO] 改动分类：$(green "$CLASS_LINE") → 不需 feedback" ;;
    esac
  fi
fi

# 2. check-consistency
CC_OUT=$(bash scripts/check-consistency.sh 2>&1)
CC_RC=$?
PASS_COUNT=$(echo "$CC_OUT" | grep -c "\[PASS\]" | tr -d '[:space:]')
FAIL_COUNT=$(echo "$CC_OUT" | grep -c "\[FAIL\]" | tr -d '[:space:]')
WARN_COUNT=$(echo "$CC_OUT" | grep -c "\[WARN\]" | tr -d '[:space:]')
TOTAL=$((PASS_COUNT + FAIL_COUNT + WARN_COUNT))
if [[ "$CC_RC" -eq 0 ]]; then
  echo "[INFO] check-consistency：$(green "$PASS_COUNT/$TOTAL PASS ✓")"
elif [[ "$CC_RC" -eq 2 ]]; then
  echo "[INFO] check-consistency：$(yellow "$PASS_COUNT PASS / $WARN_COUNT WARN")"
else
  echo "[INFO] check-consistency：$(red "$FAIL_COUNT FAIL / $WARN_COUNT WARN")"
  echo "$CC_OUT" | grep "\[FAIL\]" | sed 's/^/         /'
fi

# 3. feedback 统计
FB_COUNT=$(find feedback/meta -maxdepth 1 -name "[0-9]*-*.md" 2>/dev/null | wc -l | tr -d '[:space:]')
LAST_FB=$(find feedback/meta -maxdepth 1 -name "[0-9]*-*.md" 2>/dev/null | sort -V | tail -1 | xargs -I{} basename {} .md)
echo "[INFO] feedback/meta 总数：${FB_COUNT} 轮，最近：${LAST_FB:-（无）}"

# 3.5 业务文件变动 vs feature log 同步检查
if [[ -n "$DIRTY" ]]; then
  BIZ_FILES=$(echo "$DIRTY" | awk '{print $NF}' \
    | grep -E '^(kmp-workspace|arkuix-workspace|benchmarks)/' \
    | grep -v '/AGENTS\.md$' || true)
  if [[ -n "$BIZ_FILES" ]]; then
    echo "[INFO] 业务文件变动追踪："
    for f in $BIZ_FILES; do
      # 推断 feature 名：取主文件名 PascalCase → kebab-case
      BASE=$(basename "$f" | sed -E 's/\.(kt|ets|swift|java)$//')
      # PascalCase → kebab-case（启发式）
      KEBAB=$(echo "$BASE" | sed -E 's/([a-z])([A-Z])/\1-\2/g; s/^([A-Z])/\1/g' | tr '[:upper:]' '[:lower:]')
      if [[ -d "feedback/features/$KEBAB" ]]; then
        LATEST=$(find "feedback/features/$KEBAB" -maxdepth 1 -name "[0-9]*-*.md" 2>/dev/null | sort -V | tail -1 | xargs -I{} basename {} .md)
        echo "         - $f → feedback/features/$KEBAB/ 最近：$LATEST"
      else
        echo "         - $f → feedback/features/$KEBAB/ $(yellow "（不存在，建议 bash scripts/new-feature.sh $KEBAB）")"
      fi
    done
  fi
fi

# 4. 残留追踪：扫各 feedback 文件 "残留 / 下一轮处理" 节后的条目
echo "[INFO] 残留追踪："
HAS_RESIDUAL=0
for f in $(find feedback/meta -maxdepth 1 -name "[0-9]*-*.md" 2>/dev/null | sort -V); do
  # 抓 "残留" 标题之后的连续 bullet/列表行（最多扫 15 行）
  RESIDUAL=$(awk '
    /^### / && /残留/ { capture = 1; next }
    capture && /^### / { exit }
    capture && /^## / { exit }
    capture && /^- / { count++; if (count <= 3) print }
  ' "$f")
  if [[ -n "$RESIDUAL" ]]; then
    HAS_RESIDUAL=1
    BASE=$(basename "$f" .md)
    echo "   $(dim "[$BASE]")"
    echo "$RESIDUAL" | sed 's/^/     /'
  fi
done
if [[ "$HAS_RESIDUAL" -eq 0 ]]; then
  echo "   $(dim "（无残留）")"
fi

echo "━━━━━━━━━━━━━━━━━━━━━━━"
echo "$(dim "[hint] 新 agent？读 ONBOARDING.md 了解分层加载策略（必读 3 文件 ≈ 5K tokens）")"
exit 0
