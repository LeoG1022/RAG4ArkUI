#!/usr/bin/env bash
# new-feedback.sh — feedback 模板生成器
#
# 用法：
#   bash scripts/new-feedback.sh <slug>                         # 仅生成模板
#   bash scripts/new-feedback.sh <slug> --plan <plan-file>      # 同时附 plan 快照
#
# 行为：
#   1. 自动计算下一编号 N（扫描 feedback/[0-9]*-*.md 找最大值 +1）
#   2. 取今天日期 YYYY-MM-DD
#   3. 生成 feedback/{N}-{date}-{slug}.md，含 5 段空架子
#   4. 若 --plan 指定，把 plan 文件内容完整附在文件末尾"## 附录：原始 Plan"段
#
# 退出码：0 成功 / 1 失败

set -u

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

SLUG=""
PLAN_FILE=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --plan)
      PLAN_FILE="$2"
      shift 2
      ;;
    *)
      if [[ -z "$SLUG" ]]; then SLUG="$1"; fi
      shift
      ;;
  esac
done

if [[ -z "$SLUG" ]]; then
  echo "用法：bash scripts/new-feedback.sh <slug> [--plan <plan-file>]"
  echo "示例：bash scripts/new-feedback.sh self-enforcing-guardrails"
  echo "      bash scripts/new-feedback.sh round10-xxx --plan ~/.claude/plans/users-leo-claude-plans-compose-multipla-scalable-gadget.md"
  exit 1
fi

# slug 合法性：只允许小写字母 / 数字 / 连字符
if [[ ! "$SLUG" =~ ^[a-z0-9-]+$ ]]; then
  echo "❌ slug 不合法：'$SLUG'"
  echo "   只允许小写字母 / 数字 / 连字符，不允许空格 / 大写 / 中文。"
  exit 1
fi

if [[ -n "$PLAN_FILE" && ! -f "$PLAN_FILE" ]]; then
  echo "❌ plan 文件不存在：$PLAN_FILE"
  exit 1
fi

# 计算下一编号（meta 归档目录）
MAX_N=$(find feedback/meta -maxdepth 1 -name "[0-9]*-*.md" 2>/dev/null \
  | sed -E 's|.*/([0-9]+)-.*|\1|' \
  | sort -n \
  | tail -1)
N=$(( ${MAX_N:-0} + 1 ))

DATE=$(date +%Y-%m-%d)
TARGET="feedback/meta/${N}-${DATE}-${SLUG}.md"

if [[ -e "$TARGET" ]]; then
  echo "❌ 目标文件已存在：$TARGET"
  exit 1
fi

cat > "$TARGET" <<EOF
# ${N} — ${SLUG}

> 日期：${DATE}
> 触发：<填写：本轮起因，如用户提问 / 上轮残留 / 外部参考>
> 类型：<工程结构调整 | 规则变更 | Skill变更 | API映射 | 流程优化 | 工具脚本>

---

## 用户提出的要求

<填写：原文引用或转述>

## Agent 给出的修改建议

<填写：结构化方案，含权衡 / 替代选项 / 推荐理由>

## 多轮互动

<填写：按时序记录每次往返；无互动则写"无 —— 用户直接接受方案">

## 实际改动

- 接口变化：<填写>
- 规则变化：<填写>
- 文件变化：<填写新建 / 删除 / 移动 / 修改>
- 配置变化：<填写>

## 执行生效后总结

### 实际产出
<填写产出表>

### 前后对比
<填写前后对比表>

### 实测验证
<填写：脚本输出 / 实际触发 / 数据>

### 残留 / 下一轮处理

<!-- 用 - [ ] 标记未解决项，- [x] 标记已解决项；若无残留写"无" -->
- [ ] <填写或"无">
EOF

# 若指定 --plan，追加 plan 快照
if [[ -n "$PLAN_FILE" ]]; then
  cat >> "$TARGET" <<EOF

---

## 附录：原始 Plan（快照于 ${DATE}）

> 此段为本轮启动时的 plan 完整快照。Plan 文件路径：\`${PLAN_FILE}\`
> 后续 plan 文件会被下一轮覆盖；本快照永久保留作为档案。

\`\`\`markdown
$(cat "$PLAN_FILE")
\`\`\`
EOF
  echo "✅ 已附加 plan 快照（来源：$PLAN_FILE）"
fi

echo "✅ 已创建 $TARGET"
echo ""
echo "下一步："
echo "  1. 编辑该文件，填充 5 段内容（plan 快照在末尾，无需手填）"
echo "  2. 完成后 git add $TARGET"
echo "  3. commit 时 pre-commit 会校验结构完整性"
