#!/usr/bin/env bash
# log-tokens.sh — append / annotate / backfill 模式记录每轮 token 与改动量
#
# 用法：
#   bash scripts/log-tokens.sh --from-commit <ref>                  # 自动记录（post-commit hook）
#   bash scripts/log-tokens.sh annotate <commit-or-round> [--input-tokens N] [--output-tokens N] [--duration N] [--notes "text"]
#   bash scripts/log-tokens.sh backfill                              # 回填所有历史 commit
#
# 输出：append/update 到 stats/tokens.jsonl
# 退出码：0 成功 / 1 失败（best-effort，hook 中应当忽略错误）

set -u

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

JSONL="stats/tokens.jsonl"

if [[ ! -d "stats" ]]; then
  echo "ERROR: stats/ 目录不存在" >&2
  exit 1
fi
touch "$JSONL"

# 从 commit subject 提取 round 编号（"Round N:" 或 "Round N "）
extract_round() {
  local subject="$1"
  echo "$subject" | grep -oE '^Round [0-9]+' | grep -oE '[0-9]+' | head -1
}

# 检测 commit 中是否含 feedback/features/<name>/ 路径，返回 feature 名
extract_feature() {
  local commit="$1"
  git show --name-only --format= "$commit" 2>/dev/null \
    | grep -E '^feedback/features/[^/]+/' \
    | sed -E 's|^feedback/features/([^/]+)/.*|\1|' \
    | sort -u | head -1
}

# 调 classify-change.sh 判定 commit 改动类型
classify_commit() {
  local commit="$1"
  # 用 git diff-tree 替换 --cached 检查（针对已 commit 的）
  local meta=0 biz=0
  while IFS= read -r f; do
    [[ -z "$f" ]] && continue
    case "$f" in
      */AGENTS.md|AGENTS.md) meta=1 ;;
      .claude/skills/*|.claude/references/*|scripts/*|.github/workflows/*) meta=1 ;;
      CLAUDE.md|README.md) meta=1 ;;
      feedback/features/*) biz=1 ;;
      feedback/*) meta=1 ;;
      tests/*) meta=1 ;;
      kmp-workspace/*|arkuix-workspace/*|reports/*|stats/*) biz=1 ;;
      benchmarks/*) biz=1 ;;
    esac
  done < <(git show --name-only --format= "$commit" 2>/dev/null)

  if [[ "$meta" -eq 1 && "$biz" -eq 1 ]]; then echo "mixed"
  elif [[ "$meta" -eq 1 ]]; then echo "meta"
  elif [[ "$biz" -eq 1 ]]; then echo "business"
  else echo "unknown"
  fi
}

# 从 commit 生成 JSON 记录行
build_record() {
  local commit="$1"
  local short
  short=$(git rev-parse --short "$commit" 2>/dev/null)
  [[ -z "$short" ]] && return 1

  local timestamp subject
  timestamp=$(git show -s --format='%cI' "$commit" 2>/dev/null)
  subject=$(git show -s --format='%s' "$commit" 2>/dev/null | sed 's/"/\\"/g')

  local round feature type
  round=$(extract_round "$subject")
  feature=$(extract_feature "$commit")
  type=$(classify_commit "$commit")

  # diff stat：与父提交对比
  local stats files added removed
  if git rev-parse "${commit}^" >/dev/null 2>&1; then
    stats=$(git diff --shortstat "${commit}^..$commit" 2>/dev/null)
  else
    # 首个 commit，与空 tree 对比
    stats=$(git show --shortstat --format= "$commit" 2>/dev/null | tail -1)
  fi
  files=$(echo "$stats" | grep -oE '[0-9]+ files? changed' | grep -oE '^[0-9]+')
  added=$(echo "$stats" | grep -oE '[0-9]+ insertions?' | grep -oE '^[0-9]+')
  removed=$(echo "$stats" | grep -oE '[0-9]+ deletions?' | grep -oE '^[0-9]+')

  # JSON 字段（round/feature 为空时输出 null）
  local round_field feature_field
  if [[ -n "$round" ]]; then round_field="$round"; else round_field="null"; fi
  if [[ -n "$feature" ]]; then feature_field="\"$feature\""; else feature_field="null"; fi

  printf '{"timestamp":"%s","round":%s,"feature":%s,"iteration":null,"type":"%s","commit":"%s","subject":"%s","files_changed":%s,"lines_added":%s,"lines_removed":%s,"input_tokens_actual":null,"output_tokens_actual":null,"duration_sec":null,"notes":null}\n' \
    "${timestamp:-unknown}" "$round_field" "$feature_field" "$type" "$short" "$subject" \
    "${files:-0}" "${added:-0}" "${removed:-0}"
}

# 检查 commit 是否已记录
is_logged() {
  local short="$1"
  grep -q "\"commit\":\"$short\"" "$JSONL" 2>/dev/null
}

# ─── 分发 ───
MODE="${1:-}"
case "$MODE" in
  --from-commit)
    REF="${2:-HEAD}"
    SHORT=$(git rev-parse --short "$REF" 2>/dev/null)
    if [[ -z "$SHORT" ]]; then
      echo "ERROR: 无法解析 commit ref: $REF" >&2
      exit 1
    fi
    if is_logged "$SHORT"; then
      echo "[log-tokens] commit $SHORT 已记录，跳过" >&2
      exit 0
    fi
    RECORD=$(build_record "$REF")
    if [[ -z "$RECORD" ]]; then
      echo "ERROR: 无法生成 record for $REF" >&2
      exit 1
    fi
    echo "$RECORD" >> "$JSONL"
    echo "[log-tokens] 已记录 $SHORT" >&2
    ;;

  backfill)
    COUNT=0
    SKIPPED=0
    # 按时间正序遍历所有 commit
    for c in $(git log --reverse --format='%H'); do
      SHORT=$(git rev-parse --short "$c")
      if is_logged "$SHORT"; then
        SKIPPED=$((SKIPPED + 1))
        continue
      fi
      RECORD=$(build_record "$c")
      if [[ -n "$RECORD" ]]; then
        echo "$RECORD" >> "$JSONL"
        COUNT=$((COUNT + 1))
      fi
    done
    echo "[log-tokens backfill] 已写入 $COUNT 条新记录，跳过已存在 $SKIPPED 条"
    ;;

  annotate)
    TARGET="${2:-}"
    if [[ -z "$TARGET" ]]; then
      echo "用法: bash scripts/log-tokens.sh annotate <commit-or-round> [--input-tokens N] [--output-tokens N] [--duration N] [--notes \"text\"]" >&2
      exit 1
    fi
    shift 2
    INPUT_T=""
    OUTPUT_T=""
    DURATION=""
    NOTES=""
    while [[ $# -gt 0 ]]; do
      case "$1" in
        --input-tokens) INPUT_T="$2"; shift 2 ;;
        --output-tokens) OUTPUT_T="$2"; shift 2 ;;
        --duration) DURATION="$2"; shift 2 ;;
        --notes) NOTES="$2"; shift 2 ;;
        *) echo "未知参数: $1" >&2; exit 1 ;;
      esac
    done

    # 判断 TARGET 是 round（纯数字）还是 commit
    if [[ "$TARGET" =~ ^[0-9]+$ ]]; then
      MATCH_PATTERN="\"round\":$TARGET"
    else
      MATCH_PATTERN="\"commit\":\"$TARGET\""
    fi

    if ! grep -q "$MATCH_PATTERN" "$JSONL"; then
      echo "ERROR: 没找到匹配的记录：$TARGET" >&2
      exit 1
    fi

    TMP=$(mktemp)
    while IFS= read -r line; do
      if echo "$line" | grep -q "$MATCH_PATTERN"; then
        # 更新字段
        [[ -n "$INPUT_T"  ]] && line=$(echo "$line" | sed -E "s/\"input_tokens_actual\":[^,}]+/\"input_tokens_actual\":$INPUT_T/")
        [[ -n "$OUTPUT_T" ]] && line=$(echo "$line" | sed -E "s/\"output_tokens_actual\":[^,}]+/\"output_tokens_actual\":$OUTPUT_T/")
        [[ -n "$DURATION" ]] && line=$(echo "$line" | sed -E "s/\"duration_sec\":[^,}]+/\"duration_sec\":$DURATION/")
        if [[ -n "$NOTES" ]]; then
          NOTES_ESC=$(echo "$NOTES" | sed 's/"/\\"/g')
          line=$(echo "$line" | sed -E "s/\"notes\":[^}]+}$/\"notes\":\"$NOTES_ESC\"}/")
        fi
      fi
      echo "$line" >> "$TMP"
    done < "$JSONL"
    mv "$TMP" "$JSONL"
    echo "[log-tokens annotate] 已更新匹配记录"
    ;;

  *)
    cat <<EOF
用法：
  bash scripts/log-tokens.sh --from-commit <ref>              # 自动记录单个 commit
  bash scripts/log-tokens.sh backfill                          # 回填全部历史
  bash scripts/log-tokens.sh annotate <commit-or-round> ...    # 用户回填 token

annotate 参数：
  --input-tokens N      input token 总数
  --output-tokens N     output token 总数
  --duration N          耗时（秒）
  --notes "text"        备注

示例：
  bash scripts/log-tokens.sh annotate 14 --input-tokens 50000 --output-tokens 15000 --duration 1800
  bash scripts/log-tokens.sh annotate 4621ce2 --input-tokens 30000
EOF
    exit 1
    ;;
esac
