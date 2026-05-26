#!/usr/bin/env bash
# check-api-parity.sh — 项目代码 API 合规检查
#
# 用法：
#   bash scripts/check-api-parity.sh <path/to/file>
#   bash scripts/check-api-parity.sh <dir>          (递归扫描目录下所有目标文件)
#
# 退出码：0 = 全部通过 ; 1 = 有 FAIL ; 2 = 有 WARN (无 FAIL)
#
# 规则 ID 命名约定（使用者按项目添加具体规则）：
#   P-*   性能类（Performance）
#   R-*   资源泄漏类（Resource）
#   S-*   状态管理类（State）
#   C-*   代码复杂度类（Complexity）
#
# 规则索引：（使用者自定义）
#   示例：
#   P-EXAMPLE-01   FAIL  禁止使用某 API（无性能保障）
#   R-EXAMPLE-01   FAIL  资源必须在生命周期结束时释放

set -u

TARGET="${1:-}"
if [[ -z "$TARGET" ]]; then
  echo "用法: $0 <文件 或 目录>"
  exit 1
fi

FAIL=0
WARN=0

red()    { echo -e "\033[31m[FAIL]\033[0m $*"; }
yellow() { echo -e "\033[33m[WARN]\033[0m $*"; }
green()  { echo -e "\033[32m[PASS]\033[0m $*"; }

FILES=()
if [[ -d "$TARGET" ]]; then
  # 使用者按项目调整文件扩展名过滤（当前示例：扫描所有文本文件，排除构建产物）
  while IFS= read -r f; do FILES+=("$f"); done < <(find "$TARGET" -type f -not -path "*/node_modules/*" -not -path "*/build/*" -not -path "*/.git/*")
else
  FILES=("$TARGET")
fi

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  API Parity Check"
echo "  扫描文件数：${#FILES[@]}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

for FILE in "${FILES[@]}"; do
  [[ ! -f "$FILE" ]] && { yellow "$FILE 不存在，跳过"; continue; }
  echo ""
  echo "── $FILE"

  # ── 使用者在此添加项目特定规则 ──────────────────────────────────────────
  # 格式示例：
  #
  # # P-EXAMPLE-01: 禁止使用 FooBar API
  # if grep -qE "FooBar\(" "$FILE" 2>/dev/null; then
  #   LINES=$(grep -nE "FooBar\(" "$FILE" | awk -F: '{print $1}' | tr '\n' ' ')
  #   red "P-EXAMPLE-01 禁止使用 FooBar，应改用 BazQux  行：$LINES"
  #   FAIL=1
  # else
  #   green "P-EXAMPLE-01 无 FooBar 用法"
  # fi
  #
  # # C-COMP-01: 文件超 300 行
  # LINE_COUNT=$(wc -l < "$FILE" | tr -d '[:space:]')
  # if [[ "$LINE_COUNT" -gt 300 ]]; then
  #   yellow "C-COMP-01 文件超过 300 行（当前 $LINE_COUNT 行），建议拆分"
  #   WARN=1
  # else
  #   green "C-COMP-01 文件行数合理（$LINE_COUNT 行）"
  # fi
  # ────────────────────────────────────────────────────────────────────────

  # 暂无规则：跳过（使用者添加规则后删除此行）
  green "（暂无规则，使用者按项目添加）"

done

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if [[ "$FAIL" -eq 1 ]]; then
  echo -e "\033[31m结果：FAIL — 存在必须修复的问题\033[0m"
  exit 1
elif [[ "$WARN" -eq 1 ]]; then
  echo -e "\033[33m结果：WARN — 存在建议修改的问题\033[0m"
  exit 2
else
  echo -e "\033[32m结果：全部通过\033[0m"
  exit 0
fi
