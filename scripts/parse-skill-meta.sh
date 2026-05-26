#!/usr/bin/env bash
# parse-skill-meta.sh — 解析单个 skill 文件的 YAML frontmatter
#
# 用法：
#   bash scripts/parse-skill-meta.sh <skill-file>          # key=value 形式输出
#   bash scripts/parse-skill-meta.sh --field <key> <file>  # 输出单个字段值
#
# 支持的字段：
#   标量：name / version / trigger / description / feature_log_required /
#         classify_required / preflight_required
#   列表：calls / references （每条以 "- " 前缀的项，输出为逗号分隔）
#
# 退出码：
#   0 = 成功解析
#   1 = 文件缺 frontmatter / 字段缺失 / 格式错误

set -u

FIELD=""
FILE=""

# 简易参数解析
while [[ $# -gt 0 ]]; do
  case "$1" in
    --field)
      FIELD="$2"
      shift 2
      ;;
    *)
      FILE="$1"
      shift
      ;;
  esac
done

if [[ -z "$FILE" || ! -f "$FILE" ]]; then
  echo "用法：bash scripts/parse-skill-meta.sh [--field <key>] <skill-file>" >&2
  exit 1
fi

# 提取 frontmatter（首个 --- 到第二个 --- 之间）
FRONTMATTER=$(awk '
  /^---$/ { count++; if (count == 1) { capture = 1; next } else { exit } }
  capture { print }
' "$FILE")

if [[ -z "$FRONTMATTER" ]]; then
  echo "ERROR: 文件 $FILE 缺 frontmatter（找不到 --- 分隔块）" >&2
  exit 1
fi

# 标量字段
extract_scalar() {
  local key="$1"
  echo "$FRONTMATTER" | awk -v k="$key" '
    BEGIN { found = 0 }
    /^[a-zA-Z_]+:/ {
      if ($1 == k":") {
        val = $0
        sub(/^[^:]+:[ \t]*/, "", val)
        # 去掉行尾注释
        sub(/[ \t]*#.*$/, "", val)
        # trim
        gsub(/^[ \t]+|[ \t]+$/, "", val)
        print val
        found = 1
        exit
      }
    }
  '
}

# 列表字段（顶层 key 后跟 - item 行）
extract_list() {
  local key="$1"
  echo "$FRONTMATTER" | awk -v k="$key" '
    BEGIN { in_list = 0 }
    /^[a-zA-Z_]+:/ {
      if (in_list) { in_list = 0 }
      if ($1 == k":") {
        in_list = 1
        next
      }
    }
    in_list && /^[ \t]+-[ \t]+/ {
      val = $0
      sub(/^[ \t]+-[ \t]+/, "", val)
      sub(/[ \t]*#.*$/, "", val)
      gsub(/^[ \t]+|[ \t]+$/, "", val)
      items[length(items) + 1] = val
    }
    END {
      for (i = 1; i <= length(items); i++) {
        if (i > 1) printf ","
        printf "%s", items[i]
      }
      if (length(items) > 0) print ""
    }
  '
}

# 提取所有字段到关联（用普通变量模拟）
get_all() {
  for key in name version trigger description feature_log_required classify_required preflight_required; do
    val=$(extract_scalar "$key")
    echo "${key}=${val}"
  done
  for key in calls references; do
    val=$(extract_list "$key")
    echo "${key}=${val}"
  done
}

if [[ -n "$FIELD" ]]; then
  # 单字段输出
  case "$FIELD" in
    calls|references)
      extract_list "$FIELD"
      ;;
    *)
      extract_scalar "$FIELD"
      ;;
  esac
else
  get_all
fi
