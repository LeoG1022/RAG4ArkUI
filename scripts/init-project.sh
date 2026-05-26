#!/usr/bin/env bash
# init-project.sh — 一键初始化 agent-harness-template
#
# 用法：
#   bash scripts/init-project.sh
#
# 执行步骤：
#   1. 检查 git 仓库
#   2. 安装 git hooks
#   3. 运行 check-consistency.sh 验证结构
#   4. 运行 regenerate-skill-table.sh 同步 CLAUDE.md SKILL-TABLE
#   5. 打印"下一步"引导

set -e

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

green()  { echo -e "\033[32m✅ $*\033[0m"; }
yellow() { echo -e "\033[33m⚠  $*\033[0m"; }
red()    { echo -e "\033[31m❌ $*\033[0m"; }
bold()   { echo -e "\033[1m$*\033[0m"; }

echo ""
bold "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
bold "  Agent Harness Template — 初始化"
bold "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# ── Step 1: 检查 git 仓库 ─────────────────────────────────────────────────────
echo "步骤 1/4：检查 git 仓库..."
if [[ ! -d "$ROOT/.git" ]]; then
  red "当前目录不是 git 仓库（$ROOT 下无 .git/）"
  echo "   请先运行：git init"
  exit 1
fi
green "git 仓库确认"

# ── Step 2: 安装 git hooks ────────────────────────────────────────────────────
echo ""
echo "步骤 2/4：安装 git hooks..."
bash scripts/install-hooks.sh
green "git hooks 已安装（pre-commit + commit-msg + prepare-commit-msg + post-commit）"

# ── Step 3: 运行结构一致性检查 ────────────────────────────────────────────────
echo ""
echo "步骤 3/4：运行结构一致性检查..."
CONSISTENCY_OUT=$(bash scripts/check-consistency.sh 2>&1) || true
CONSISTENCY_RC=$?
echo "$CONSISTENCY_OUT"

if [[ "$CONSISTENCY_RC" -eq 1 ]]; then
  yellow "check-consistency.sh 有 FAIL 项（见上方输出）"
  yellow "初始化完成后请逐项修复，或查阅 CLAUDE.md 的规则说明"
else
  green "结构一致性检查通过"
fi

# ── Step 4: 同步 SKILL-TABLE ──────────────────────────────────────────────────
echo ""
echo "步骤 4/4：同步 CLAUDE.md SKILL-TABLE..."
bash scripts/regenerate-skill-table.sh
green "SKILL-TABLE 已同步"

# ── 完成：打印下一步引导 ──────────────────────────────────────────────────────
echo ""
bold "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
bold "  初始化完成！下一步："
bold "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "  1. 按项目定制（必做）："
echo "     • scripts/check-api-parity.sh  — 添加项目代码规则"
echo "     • scripts/classify-change.sh   — 添加业务目录路径"
echo "     • CLAUDE.md                    — 更新目录结构、IDE 入口"
echo "     • AGENTS.md                    — 更新子目录索引"
echo ""
echo "  2. 添加第一个 Skill（按需）："
echo "     cp .claude/skills/example-skill.md .claude/skills/<your-skill>.md"
echo "     # 编辑后执行："
echo "     bash scripts/regenerate-skill-table.sh"
echo ""
echo "  3. 添加参考表（按需）："
echo "     cp .claude/references/example-mapping.md .claude/references/mapping-<domain>.md"
echo ""
echo "  4. 第一次提交："
echo "     git add . && bash scripts/commit.sh -m 'Initial: add agent harness'"
echo ""
echo "  5. 删除示例文件（完成定制后）："
echo "     rm .claude/skills/example-skill.md"
echo "     rm .claude/references/example-mapping.md"
echo "     bash scripts/regenerate-skill-table.sh"
echo ""
echo "  更多说明见 README.md 和 ONBOARDING.md"
echo ""
