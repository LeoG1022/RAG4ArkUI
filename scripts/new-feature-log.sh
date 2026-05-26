#!/usr/bin/env bash
# new-feature-log.sh — 在已有特性下追加迭代日志
#
# 用法：bash scripts/new-feature-log.sh <feature-name> <slug>
#   feature-name：已存在的 features/<name>/
#   slug：本轮简述，kebab-case
#
# 退出码：0 成功 / 1 失败

set -u

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

NAME="${1:-}"
SLUG="${2:-}"

if [[ -z "$NAME" || -z "$SLUG" ]]; then
  echo "用法：bash scripts/new-feature-log.sh <feature-name> <slug>"
  echo "示例：bash scripts/new-feature-log.sh lazy-list add-pull-refresh"
  exit 1
fi

# 校验 kebab-case
if [[ ! "$NAME" =~ ^[a-z0-9-]+$ ]]; then
  echo "❌ feature 名不合法：'$NAME'（要求 kebab-case）"
  exit 1
fi

if [[ ! "$SLUG" =~ ^[a-z0-9-]+$ ]]; then
  echo "❌ slug 不合法：'$SLUG'（要求 kebab-case）"
  exit 1
fi

FEATURE_DIR="feedback/features/$NAME"
if [[ ! -d "$FEATURE_DIR" ]]; then
  echo "❌ 特性不存在：$FEATURE_DIR"
  echo "   新建特性：bash scripts/new-feature.sh $NAME"
  exit 1
fi

# 计算下一编号
MAX_N=$(find "$FEATURE_DIR" -maxdepth 1 -name "[0-9]*-*.md" 2>/dev/null \
  | sed -E 's|.*/([0-9]+)-.*|\1|' \
  | sort -n \
  | tail -1)
N=$(( ${MAX_N:-0} + 1 ))

DATE=$(date +%Y-%m-%d)
LOG="$FEATURE_DIR/${N}-${DATE}-${SLUG}.md"

if [[ -e "$LOG" ]]; then
  echo "❌ 目标文件已存在：$LOG"
  exit 1
fi

cat > "$LOG" <<EOF
# $N — $SLUG

> 日期：$DATE
> 涉及代码：<填写：相关文件路径>
> 类型：<新建 | 重构 | bug修复 | 性能优化 | 迁入benchmark>

## 本轮目标

<填写：本次解决什么问题 / 实现什么能力>

## 改动要点

<填写：API 选型 / 算法 / 关键决策 / 与上轮的差异>

## 验证结果

- 编译：<填写>
- check-api-parity：<填写>
- benchmark（如有）：<填写或留空>

## 残留 / 下一轮

<!-- 用 - [ ] 标记未解决项，- [x] 标记已解决项；若无残留写"无" -->
- [ ] <填写或"无">
EOF

echo "✅ 已创建 $LOG"
echo ""
echo "下一步："
echo "  1. 编辑该文件填 4 段内容"
echo "  2. 如需更新特性状态：编辑 $FEATURE_DIR/README.md"
