#!/usr/bin/env bash
# stats-report.sh — 从 stats/tokens.jsonl 生成统计报告
#
# 用法：
#   bash scripts/stats-report.sh                  # 全量摘要
#   bash scripts/stats-report.sh --by-round       # 按 round 列表
#   bash scripts/stats-report.sh --by-feature     # 按 feature 列表
#   bash scripts/stats-report.sh --trend          # 累计趋势
#   bash scripts/stats-report.sh --top <N>        # 行数最多的 N 轮
#
# 输出：markdown 到 stdout（可重定向到 stats/summaries/）

set -u

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

JSONL="stats/tokens.jsonl"

if [[ ! -f "$JSONL" ]]; then
  echo "ERROR: $JSONL 不存在" >&2
  exit 1
fi

# JSON 字段抽取（行级正则，不用 jq 依赖）
extract() {
  local line="$1"
  local field="$2"
  echo "$line" | grep -oE "\"$field\":[^,}]+" | sed -E "s/\"$field\"://" | sed -E 's/^"|"$//g'
}

# 数字字段（null 返回 0）
extract_num() {
  local val
  val=$(extract "$1" "$2")
  if [[ "$val" == "null" || -z "$val" ]]; then echo 0; else echo "$val"; fi
}

MODE="${1:-default}"

case "$MODE" in
  --by-round)
    echo "# Token Stats — By Round"
    echo ""
    echo "| Round | Commit | Type | Files | +Lines | -Lines | InputTok | OutputTok | Duration |"
    echo "|---|---|---|---|---|---|---|---|---|"
    while IFS= read -r line; do
      [[ -z "$line" ]] && continue
      r=$(extract "$line" "round")
      [[ "$r" == "null" ]] && continue
      commit=$(extract "$line" "commit")
      type=$(extract "$line" "type")
      files=$(extract_num "$line" "files_changed")
      added=$(extract_num "$line" "lines_added")
      removed=$(extract_num "$line" "lines_removed")
      input_t=$(extract "$line" "input_tokens_actual")
      output_t=$(extract "$line" "output_tokens_actual")
      duration=$(extract "$line" "duration_sec")
      [[ "$input_t" == "null" ]] && input_t="—"
      [[ "$output_t" == "null" ]] && output_t="—"
      [[ "$duration" == "null" ]] && duration="—"
      echo "| R$r | \`$commit\` | $type | $files | $added | $removed | $input_t | $output_t | $duration |"
    done < "$JSONL" | sort -t'|' -k2 -V
    ;;

  --by-feature)
    echo "# Token Stats — By Feature"
    echo ""
    echo "| Feature | Iter | Commit | Files | +Lines | -Lines | InputTok | OutputTok |"
    echo "|---|---|---|---|---|---|---|---|"
    while IFS= read -r line; do
      [[ -z "$line" ]] && continue
      f=$(extract "$line" "feature")
      [[ "$f" == "null" ]] && continue
      iter=$(extract "$line" "iteration")
      [[ "$iter" == "null" ]] && iter="—"
      commit=$(extract "$line" "commit")
      files=$(extract_num "$line" "files_changed")
      added=$(extract_num "$line" "lines_added")
      removed=$(extract_num "$line" "lines_removed")
      input_t=$(extract "$line" "input_tokens_actual")
      output_t=$(extract "$line" "output_tokens_actual")
      [[ "$input_t" == "null" ]] && input_t="—"
      [[ "$output_t" == "null" ]] && output_t="—"
      echo "| $f | $iter | \`$commit\` | $files | $added | $removed | $input_t | $output_t |"
    done < "$JSONL"
    ;;

  --trend)
    echo "# Token Stats — Cumulative Trend"
    echo ""
    echo "| Round | +Lines (本轮) | +Lines (累计) | EstTok (本轮 ~×5) | EstTok (累计) |"
    echo "|---|---|---|---|---|"
    cum_added=0
    cum_est=0
    while IFS= read -r line; do
      [[ -z "$line" ]] && continue
      r=$(extract "$line" "round")
      [[ "$r" == "null" ]] && continue
      added=$(extract_num "$line" "lines_added")
      est=$((added * 5))
      cum_added=$((cum_added + added))
      cum_est=$((cum_est + est))
      printf "| R%s | %d | %d | %d | %d |\n" "$r" "$added" "$cum_added" "$est" "$cum_est"
    done < "$JSONL" | sort -t'|' -k2 -V
    ;;

  --top)
    N="${2:-5}"
    echo "# Top $N Rounds (by lines_added)"
    echo ""
    echo "| Round | Commit | Subject | +Lines |"
    echo "|---|---|---|---|"
    while IFS= read -r line; do
      [[ -z "$line" ]] && continue
      r=$(extract "$line" "round")
      added=$(extract_num "$line" "lines_added")
      commit=$(extract "$line" "commit")
      subject=$(extract "$line" "subject" | head -c 80)
      label="R${r}"
      [[ "$r" == "null" ]] && label="(business)"
      printf "%d\t| %s | \`%s\` | %s | %d |\n" "$added" "$label" "$commit" "$subject" "$added"
    done < "$JSONL" | sort -rn | head -"$N" | cut -f2-
    ;;

  default|*)
    # 全量摘要
    TOTAL=$(grep -c '"commit"' "$JSONL" 2>/dev/null | tr -d '[:space:]'); TOTAL=${TOTAL:-0}
    META=$(grep -c '"type":"meta"' "$JSONL" 2>/dev/null | tr -d '[:space:]'); META=${META:-0}
    BIZ=$(grep -c '"type":"business"' "$JSONL" 2>/dev/null | tr -d '[:space:]'); BIZ=${BIZ:-0}
    MIXED=$(grep -c '"type":"mixed"' "$JSONL" 2>/dev/null | tr -d '[:space:]'); MIXED=${MIXED:-0}

    # 累加 lines / token
    SUMS=$(awk '
      {
        added += match($0, /"lines_added":[0-9]+/) ? substr($0, RSTART+15, RLENGTH-15) : 0
        removed += match($0, /"lines_removed":[0-9]+/) ? substr($0, RSTART+17, RLENGTH-17) : 0
      }
      END { printf "%d %d", added, removed }
    ' "$JSONL")
    read TOTAL_ADDED TOTAL_REMOVED <<< "$SUMS"

    # token actual 求和（忽略 null）
    TOKEN_IN=$(grep -oE '"input_tokens_actual":[0-9]+' "$JSONL" | grep -oE '[0-9]+' | awk '{s+=$1} END {print s+0}')
    TOKEN_OUT=$(grep -oE '"output_tokens_actual":[0-9]+' "$JSONL" | grep -oE '[0-9]+' | awk '{s+=$1} END {print s+0}')
    ANNOTATED=$(grep -cE '"input_tokens_actual":[0-9]+' "$JSONL" 2>/dev/null | tr -d '[:space:]'); ANNOTATED=${ANNOTATED:-0}

    cat <<EOF
# Stats Report Summary

> 生成于：$(date '+%Y-%m-%d %H:%M:%S')
> 数据源：\`stats/tokens.jsonl\`

## 总览

- 总记录数：**$TOTAL** 轮
- 分类：meta=$META / business=$BIZ / mixed=$MIXED
- 累计行变化：**+${TOTAL_ADDED:-0}** / **-${TOTAL_REMOVED:-0}**
- Token 估算（行数 × 5）：~$((${TOTAL_ADDED:-0} * 5)) input-equivalent

## Token 实际记录（用户标注）

- 已标注记录：$ANNOTATED / $TOTAL
- 累计实际 input tokens：${TOKEN_IN:-0}
- 累计实际 output tokens：${TOKEN_OUT:-0}

## 其他视图

\`\`\`bash
bash scripts/stats-report.sh --by-round       # 按 round 详细
bash scripts/stats-report.sh --by-feature     # business 视图
bash scripts/stats-report.sh --trend          # 累计趋势
bash scripts/stats-report.sh --top 5          # 最大 5 轮
\`\`\`

## 待回填提示

用 \`bash scripts/log-tokens.sh annotate <round> --input-tokens N --output-tokens N\` 回填真实 token。
EOF
    ;;
esac
