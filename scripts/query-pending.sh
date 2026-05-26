#!/usr/bin/env bash
# query-pending.sh — 查询归档中未解决的残留项
#
# 用法：
#   bash scripts/query-pending.sh                    # 扫全部归档
#   bash scripts/query-pending.sh --meta             # 只看 feedback/meta
#   bash scripts/query-pending.sh --features         # 只看 feedback/features
#   bash scripts/query-pending.sh --feature <name>   # 指定特性
#   bash scripts/query-pending.sh --last N           # 只看最近 N 轮 meta（默认全部）
#
# 输出：含 - [ ] 的归档条目，分组显示
# 退出码：0 无残留 / 1 有残留

set -u

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

bold()   { echo -e "\033[1m$*\033[0m"; }
yellow() { echo -e "\033[33m$*\033[0m"; }
green()  { echo -e "\033[32m$*\033[0m"; }
dim()    { echo -e "\033[2m$*\033[0m"; }

SCAN_META=1
SCAN_FEATURES=1
FEATURE_FILTER=""
LAST_N=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --meta)     SCAN_FEATURES=0; shift ;;
    --features) SCAN_META=0; shift ;;
    --feature)  FEATURE_FILTER="$2"; shift 2 ;;
    --last)     LAST_N="$2"; shift 2 ;;
    *) echo "未知参数: $1"; exit 1 ;;
  esac
done

TOTAL_OPEN=0

print_pending_in_file() {
  local f="$1"
  local label="$2"
  # 提取 - [ ] 行并带行号
  local items
  items=$(grep -nE "^- \[ \]" "$f" 2>/dev/null)
  if [[ -z "$items" ]]; then return; fi
  local count
  count=$(echo "$items" | wc -l | tr -d '[:space:]')
  TOTAL_OPEN=$((TOTAL_OPEN + count))
  bold "[$label] $(basename "$f" .md)  (${count} 项未解决)"
  echo "$items" | while IFS= read -r line; do
    echo "  $line"
  done
  echo ""
}

echo "━━━━━━ 残留追踪 ━━━━━━"

# ── feedback/meta ──
if [[ "$SCAN_META" -eq 1 ]]; then
  META_FILES=$(find feedback/meta -maxdepth 1 -name "[0-9]*-*.md" 2>/dev/null | sort -V)
  if [[ "$LAST_N" -gt 0 ]]; then
    META_FILES=$(echo "$META_FILES" | tail -"$LAST_N")
  fi
  META_HAS=0
  for f in $META_FILES; do
    OPEN=$(grep -cE "^- \[ \]" "$f" 2>/dev/null | tr -d '[:space:]')
    if [[ "${OPEN:-0}" -gt 0 ]]; then
      META_HAS=1
      print_pending_in_file "$f" "meta"
    fi
  done
  if [[ "$META_HAS" -eq 0 ]]; then
    dim "(feedback/meta：无未解决残留)"
    echo ""
  fi
fi

# ── feedback/features ──
if [[ "$SCAN_FEATURES" -eq 1 ]] && [[ -d feedback/features ]]; then
  FEAT_HAS=0
  for d in feedback/features/*/; do
    [[ -d "$d" ]] || continue
    FEAT=$(basename "$d")
    if [[ -n "$FEATURE_FILTER" && "$FEAT" != "$FEATURE_FILTER" ]]; then continue; fi
    for f in $(find "$d" -maxdepth 1 -name "[0-9]*-*.md" 2>/dev/null | sort -V); do
      OPEN=$(grep -cE "^- \[ \]" "$f" 2>/dev/null | tr -d '[:space:]')
      if [[ "${OPEN:-0}" -gt 0 ]]; then
        FEAT_HAS=1
        print_pending_in_file "$f" "feature/$FEAT"
      fi
    done
  done
  if [[ "$FEAT_HAS" -eq 0 ]]; then
    dim "(feedback/features：无未解决残留)"
    echo ""
  fi
fi

echo "━━━━━━━━━━━━━━━━━━━━━━━"
if [[ "$TOTAL_OPEN" -gt 0 ]]; then
  yellow "共 $TOTAL_OPEN 项未解决残留"
  echo ""
  echo "处理方式："
  echo "  - 已解决：将对应归档文件中 '- [ ]' 改为 '- [x]'"
  echo "  - 转移执行：新建 feature log 或 feedback，在本条末尾注明 '→ 见 <新文件>'"
  exit 1
else
  green "无未解决残留 ✓"
  exit 0
fi
