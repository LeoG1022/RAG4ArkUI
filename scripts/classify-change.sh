#!/usr/bin/env bash
# classify-change.sh — 按路径分类 staged 改动
#
# 用法:
#   bash scripts/classify-change.sh             # 分类 staged 文件
#   bash scripts/classify-change.sh --worktree  # 分类工作区 + staged
#
# 退出码:
#   0 = 纯业务 / 无改动
#   1 = 纯元变更
#   2 = 混合(meta + business 同时存在)
#
# Meta 路径(需要 feedback):
#   .claude/skills/**  .claude/references/**  scripts/**  .github/workflows/**
#   */AGENTS.md  根 AGENTS.md / CLAUDE.md / README.md
#   feedback/DESIGN.md  feedback/refactor-rules.md  feedback/AGENTS.md
#   feedback/meta/[N]-*.md
#
# Business 路径（使用者按项目实际目录修改）:
#   workspace/**  output/**  reports/**
#   <项目业务代码目录>/** (除 */AGENTS.md)
#   feedback/features/**

set -u

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

MODE="${1:-staged}"
if [[ "$MODE" == "--worktree" ]]; then
  FILES=$(git status --porcelain 2>/dev/null | awk '{print $NF}')
else
  FILES=$(git diff --cached --name-only 2>/dev/null)
fi

META_FILES=()
BIZ_FILES=()
UNK_FILES=()

classify() {
  local f="$1"
  # 优先匹配精细规则:任意层级 AGENTS.md 都算 meta
  case "$f" in
    */AGENTS.md|AGENTS.md) echo meta; return ;;
  esac
# 然后按目录前缀
  case "$f" in
    .claude/skills/*|.claude/references/*|scripts/*|.github/workflows/*|.gitignore|.editorconfig|.eslintrc*) echo meta ;;
    CLAUDE.md|README.md) echo meta ;;
    feedback/features/*) echo business ;;
    feedback/*) echo meta ;;
    tests/*) echo meta ;;
    # ── 使用者按项目结构修改以下业务路径 ──────────────────────────────────
    # 示例：workspace/*|output/*|reports/*|src/*
    reports/*|stats/*) echo business ;;
    # RAG4ArkUI 产品代码与文档
    crates/*|corpus/*|docs/*) echo business ;;
    # ─────────────────────────────────────────────────────────────────────
    *) echo unknown ;;
  esac
}

for f in $FILES; do
  [[ -z "$f" ]] && continue
  c=$(classify "$f")
  case "$c" in
    meta) META_FILES+=("$f") ;;
    business) BIZ_FILES+=("$f") ;;
    *) UNK_FILES+=("$f") ;;
  esac
done

# 输出明细
for f in "${META_FILES[@]:-}"; do [[ -n "$f" ]] && echo "[meta]     $f"; done
for f in "${BIZ_FILES[@]:-}"; do  [[ -n "$f" ]] && echo "[business] $f"; done
for f in "${UNK_FILES[@]:-}"; do  [[ -n "$f" ]] && echo "[unknown]  $f(视作业务，不强制 feedback)"; done

META_N=${#META_FILES[@]}
BIZ_N=${#BIZ_FILES[@]}

# 决定退出码
if [[ "$META_N" -gt 0 && "$BIZ_N" -gt 0 ]]; then
  CLASS="mixed"; RC=2
elif [[ "$META_N" -gt 0 ]]; then
  CLASS="meta"; RC=1
else
  CLASS="business"; RC=0
fi

echo ""
echo "分类:$CLASS(meta=$META_N, business=$BIZ_N)"

# meta / mixed 时输出醒目告知(写到 stdout，agent 应复述给用户)
if [[ "$RC" -ne 0 ]]; then
  cat <<EOF

$(printf '\033[33m')━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━$(printf '\033[0m')
$(printf '\033[33m🔔 元变更检测\033[0m'):本次改动含 $META_N 个 meta 文件
   提交前必须用:$(printf '\033[1mbash scripts/new-feedback.sh <slug>\033[0m')
   触发原因:修改了 skill / mapping / AGENTS.md / scripts / 等规则相关文件
$(printf '\033[33m')━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━$(printf '\033[0m')
EOF
fi

exit $RC
