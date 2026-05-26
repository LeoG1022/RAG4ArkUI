#!/usr/bin/env bash
# commit.sh — Agent commit wrapper
#
# 所有 AI agent 必须通过此脚本提交，禁止直接调用 git commit。
# 内置拦截：
#   1. 拒绝 --no-verify 参数（规则 #14，FAIL级硬性规则）
#   2. Git 前置检查（规则 #1）
#   3. Agent commit 后自动记录统计（规则 #3）
#
# 用法:
#   bash scripts/commit.sh -m "commit message"
#   bash scripts/commit.sh -m "commit message" [其他 git commit 参数，不含 --no-verify]
#
# 退出码: 与 git commit 一致（0=成功，1=失败）

set -eu

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

# 1. 拒绝 --no-verify
for arg in "$@"; do
  if [[ "$arg" == "--no-verify" ]]; then
    echo ""
    echo "❌ 禁止使用 --no-verify（AGENTS.md 全局规则 #14，FAIL级硬性规则）"
    echo "   AI agent 绝对不允许绕过 pre-commit / commit-msg hook。"
    echo "   如确需绕过（极端情况），只能由用户手动执行 git commit --no-verify"
    echo "   并在 commit message 中添加 [MANUAL-OVERRIDE: <理由>] 标记。"
    exit 1
  fi
done

# 2. Git 前置检查
bash scripts/preflight.sh 2>&1 | grep -q '未提交' && {
  echo ""
  echo "⚠️  检测到未提交修改，请先处理后再 commit。"
  echo "   选项：commit（先提交当前修改） / stash（暂存）/ proceed（继续）"
}

# 3. 执行 git commit
git commit "$@"
commit_rc=$?

# 4. Agent commit 后自动记录统计（规则 #3）
if [[ "$commit_rc" -eq 0 ]]; then
  bash scripts/log-tokens.sh --from-commit HEAD 2>&1 || true
fi

exit $commit_rc