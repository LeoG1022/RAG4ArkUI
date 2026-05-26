#!/usr/bin/env bash
# check-archive-deletion.sh — 拦截删除/重命名归档文件（**无 override**）
#
# 受保护的归档文件：
#   feedback/meta/[N]-*.md
#   feedback/DESIGN.md
#   feedback/refactor-rules.md
#   feedback/features/*/[N]-*.md
#   feedback/features/*/README.md
#
# 用法：
#   作为 commit-msg hook：bash scripts/check-archive-deletion.sh <commit-msg-file>
#   独立调用：bash scripts/check-archive-deletion.sh
#
# **设计**：自 Round 9 起取消 [archive-purge:] override 通道。
# Round 8 实测发现 pre-commit M-FB-01 在归档删除时抢先拦截，
# commit-msg hook 永远到不了，override 是死路径。
# 确需清理归档（极端情况）→ `git commit --no-verify` 显式破坏（留显眼痕迹）。
#
# 退出码：0 PASS / 1 FAIL

set -u

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

red()    { echo -e "\033[31m[FAIL]\033[0m $*"; }
yellow() { echo -e "\033[33m[INFO]\033[0m $*"; }
green()  { echo -e "\033[32m[PASS]\033[0m $*"; }

# 抓 staged 中的 D（删除）和 R（重命名）
# diff-filter=DR 给出删除和重命名条目
STAGED=$(git diff --cached --name-status --diff-filter=DR 2>/dev/null)

if [[ -z "$STAGED" ]]; then
  green "归档保护：无删除/重命名操作"
  exit 0
fi

# 判断一个路径是否归档文件
is_archive() {
  local p="$1"
  case "$p" in
    feedback/meta/[0-9]*-*.md) return 0 ;;
    feedback/DESIGN.md|feedback/refactor-rules.md) return 0 ;;
    feedback/features/*/[0-9]*-*.md) return 0 ;;
    feedback/features/*/README.md) return 0 ;;
    *) return 1 ;;
  esac
}

VIOLATIONS=()
while IFS= read -r line; do
  STATUS=$(echo "$line" | awk '{print $1}')
  case "$STATUS" in
    D)
      F=$(echo "$line" | awk '{print $2}')
      if is_archive "$F"; then
        VIOLATIONS+=("D  $F")
      fi
      ;;
    R*)
      OLD=$(echo "$line" | awk '{print $2}')
      NEW=$(echo "$line" | awk '{print $3}')
      if is_archive "$OLD"; then
        VIOLATIONS+=("R  $OLD → $NEW")
      fi
      ;;
  esac
done <<< "$STAGED"

if [[ ${#VIOLATIONS[@]} -eq 0 ]]; then
  green "归档保护：staged 删除/重命名均非归档文件"
  exit 0
fi

# 检测到归档删除/重命名 → 直接 FAIL（无 override 通道）
echo ""
red "归档文件不可删除或重命名！"
echo "   以下归档文件被删除/重命名："
for v in "${VIOLATIONS[@]}"; do
  echo "     - $v"
done
echo ""
echo "   归档是不可变历史。建议："
echo "     - 旧记录有错误 → 新增一条修正日志，或原文件末尾补 errata 段"
echo "     - 确需删除（极端情况）→ git commit --no-verify 显式破坏，留痕迹给审计"
echo ""
echo "   详见 AGENTS.md 全局规则 #11。"
exit 1
