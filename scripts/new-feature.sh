#!/usr/bin/env bash
# new-feature.sh — 创建业务特性目录 + 第 1 条迭代日志
#
# 用法：bash scripts/new-feature.sh <feature-name> [initial-slug]
#   feature-name：kebab-case，小写 + 连字符
#   initial-slug：可选，第 1 条日志的 slug（默认 "initial"）
#
# 退出码：0 成功 / 1 失败

set -u

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

NAME="${1:-}"
INITIAL_SLUG="${2:-initial}"

if [[ -z "$NAME" ]]; then
  echo "用法：bash scripts/new-feature.sh <feature-name> [initial-slug]"
  echo "示例：bash scripts/new-feature.sh lazy-list"
  exit 1
fi

# 校验 kebab-case
if [[ ! "$NAME" =~ ^[a-z0-9-]+$ ]]; then
  echo "❌ feature 名不合法：'$NAME'"
  echo "   要求 kebab-case：小写字母 / 数字 / 连字符，不允许大写 / 空格 / 中文"
  exit 1
fi

if [[ ! "$INITIAL_SLUG" =~ ^[a-z0-9-]+$ ]]; then
  echo "❌ initial slug 不合法：'$INITIAL_SLUG'"
  exit 1
fi

FEATURE_DIR="feedback/features/$NAME"
if [[ -e "$FEATURE_DIR" ]]; then
  echo "❌ 特性已存在：$FEATURE_DIR"
  echo "   如需追加日志：bash scripts/new-feature-log.sh $NAME <slug>"
  exit 1
fi

mkdir -p "$FEATURE_DIR"

DATE=$(date +%Y-%m-%d)

# README.md
cat > "$FEATURE_DIR/README.md" <<EOF
# $NAME

> 状态：in-progress
> 创建：$DATE

## 用途

<填写：一句话描述本特性解决什么问题>

## 涉及代码

- KMP：<填写：kmp-workspace/.../FILE.kt>
- ArkUI-X：<填写：arkuix-workspace/.../FILE.ets>
- Benchmark（如已迁入）：<填写或留空>

## 迭代日志

- 1-${DATE}-${INITIAL_SLUG}.md
EOF

# 初始日志
LOG="$FEATURE_DIR/1-${DATE}-${INITIAL_SLUG}.md"
cat > "$LOG" <<EOF
# 1 — $INITIAL_SLUG

> 日期：$DATE
> 涉及代码：<填写：相关文件路径>
> 类型：新建

## 本轮目标

<填写：本次解决什么问题 / 实现什么能力>

## 改动要点

<填写：API 选型 / 算法 / 关键决策>

## 验证结果

- 编译：<填写>
- check-api-parity：<填写>
- benchmark（如有）：<填写或留空>

## 残留 / 下一轮

<!-- 用 - [ ] 标记未解决项，- [x] 标记已解决项；若无残留写"无" -->
- [ ] <填写或"无">
EOF

echo "✅ 已创建 $FEATURE_DIR/"
echo "   - README.md"
echo "   - 1-${DATE}-${INITIAL_SLUG}.md"
echo ""
echo "下一步："
echo "  1. 编辑 $FEATURE_DIR/README.md 填用途和涉及代码"
echo "  2. 编辑 $LOG 填本轮目标 / 改动要点 / 验证结果"
echo "  3. 后续迭代：bash scripts/new-feature-log.sh $NAME <slug>"
