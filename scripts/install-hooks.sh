#!/usr/bin/env bash
# install-hooks.sh — 一键安装 git pre-commit + commit-msg hook
#
# pre-commit hook 跑：
#   1. check-api-parity.sh 扫描 staged 目标文件（使用者按项目调整文件过滤）
#   2. classify-change.sh 分类 staged 改动；若 meta/mixed 必须关联 feedback；若 business 必须关联 feature log
#   3. check-consistency.sh 跨文档一致性（含 M-SKILL-TABLE-SYNC 验证 CLAUDE.md SKILL-TABLE 与 frontmatter 同步）
# commit-msg hook 跑：
#   check-archive-deletion.sh 拦截删除/重命名归档文件（无 override 通道）
# 任一 FAIL 阻止 commit。

set -e

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
HOOK="$ROOT/.git/hooks/pre-commit"
MSG_HOOK="$ROOT/.git/hooks/commit-msg"

if [[ ! -d "$ROOT/.git" ]]; then
  echo "错误：当前不是 git 仓库（$ROOT 下无 .git/）"
  exit 1
fi

cat > "$HOOK" <<'EOF'
#!/usr/bin/env bash
# Auto-installed by scripts/install-hooks.sh

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

# 记录 pre-commit hook 执行时间戳（供 post-commit hook 对比验证）
date +%s > "$ROOT/.git/hooks/.pre-commit-timestamp"

FAIL=0

# 1. 校验 staged 业务代码文件
# ── 使用者按项目取消注释并调整以下过滤条件 ──────────────────────────────────
# 示例：扫描 .ets 文件（ArkTS 项目）：
#   STAGED_SRC=$(git diff --cached --name-only --diff-filter=ACM | grep '\.ets$' | grep -v '^tests/' || true)
# 示例：扫描 .ts 文件（TypeScript 项目）：
#   STAGED_SRC=$(git diff --cached --name-only --diff-filter=ACM | grep '\.ts$' || true)
# 示例：扫描 src/ 目录下的 .py 文件（Python 项目）：
#   STAGED_SRC=$(git diff --cached --name-only --diff-filter=ACM | grep '^src/.*\.py$' || true)
# ─────────────────────────────────────────────────────────────────────────────
# 默认：无文件过滤（check-api-parity.sh 无规则时自动跳过）
STAGED_SRC=""  # 使用者替换此行为上方示例之一
if [[ -n "$STAGED_SRC" ]]; then
  echo "── check-api-parity.sh on staged files ──"
  for f in $STAGED_SRC; do
    [[ -f "$f" ]] || continue
    bash scripts/check-api-parity.sh "$f"
    rc=$?
    if [[ "$rc" -eq 1 ]]; then FAIL=1; fi
  done
fi

# 2. 元变更必须关联 feedback & 业务变更必须关联 feature log
echo "── classify-change.sh ──"
CLASSIFY_OUTPUT=$(bash scripts/classify-change.sh 2>&1)
classify_rc=$?
echo "$CLASSIFY_OUTPUT"

# 提取 business 文件数
BIZ_COUNT=$(echo "$CLASSIFY_OUTPUT" | grep -cE '^\[business\]' || true)

if [[ "$classify_rc" -ne 0 ]]; then
  HAS_FB=$(git diff --cached --name-only | grep -cE '^feedback/meta/[0-9]+-.*\.md$' || true)
  if [[ "$HAS_FB" -eq 0 ]]; then
    echo ""
    echo "❌ 元变更必须关联 feedback 记录（staged 中没有 feedback/meta/[N]-*.md）"
    echo "   生成模板：bash scripts/new-feedback.sh <slug>"
    echo "   填好后 git add feedback/<生成的文件> 再重试 commit"
    FAIL=1
  fi
fi

if [[ "$BIZ_COUNT" -gt 0 ]]; then
  HAS_FL=$(git diff --cached --name-only | grep -cE '^feedback/features/[^/]+/[0-9]+-.*\.md$' || true)
  if [[ "$HAS_FL" -eq 0 ]]; then
    echo ""
    echo "❌ 业务变更必须关联 feature log（staged 中没有 feedback/features/<name>/<N>-*.md）"
    echo "   生成模板：bash scripts/new-feature.sh <name>"
    echo "   填好后 git add feedback/<生成的文件> 再重试 commit"
    echo "   豁免：纯人工编辑、/run-benchmark 只读"
    FAIL=1
  fi
fi

# 3. 跨文档一致性
echo "── check-consistency.sh ──"
bash scripts/check-consistency.sh
rc=$?
if [[ "$rc" -eq 1 ]]; then FAIL=1; fi

# 4. [使用者可选] 在此添加项目特定的编译验证步骤
#    示例：如果某目录的代码改动必须通过编译验证才能提交：
#
# STAGED_PROJECT=$(git diff --cached --name-only --diff-filter=ACM | grep -E '^src/.*\.(ts|js)$' || true)
# if [[ "$FAIL" -eq 0 && -n "$STAGED_PROJECT" ]]; then
#   echo "── 项目代码编译验证 ──"
#   if npm run build 2>&1; then
#     echo "✅ 编译验证通过"
#   else
#     echo "❌ 编译验证失败，修复后重试"
#     FAIL=1
#   fi
# fi

if [[ "$FAIL" -eq 1 ]]; then
  echo ""
  echo "❌ pre-commit 校验失败，请修复 [FAIL] 项后重试。"
  echo "   AI agent 禁止使用 --no-verify（AGENTS.md 规则 #14）"
  echo "   用户紧急绕过：git commit --no-verify + [MANUAL-OVERRIDE: <理由>] 标记"
  exit 1
fi

exit 0
EOF

chmod +x "$HOOK"

cat > "$MSG_HOOK" <<'EOF'
#!/usr/bin/env bash
# Auto-installed by scripts/install-hooks.sh
# commit-msg hook: 拦截删除/重命名归档文件（除非 message 含 [archive-purge: <理由>]）

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

bash scripts/check-archive-deletion.sh "$1"
rc=$?
if [[ "$rc" -eq 1 ]]; then
  echo ""
  echo "❌ commit-msg 校验失败（归档保护）"
  echo "   紧急绕过：git commit --no-verify （不推荐）"
  exit 1
fi
exit 0
EOF

chmod +x "$MSG_HOOK"

# prepare-commit-msg hook：元变更时自动添加 override 提示
PREP_HOOK="$ROOT/.git/hooks/prepare-commit-msg"
cat > "$PREP_HOOK" <<'EOF'
#!/usr/bin/env bash
# Auto-installed by scripts/install-hooks.sh
# prepare-commit-msg hook: 元变更时自动添加 override 提示

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

bash scripts/prepare-commit-msg-hook.sh "$1" "$2"
EOF
chmod +x "$PREP_HOOK"

# post-commit hook：记录成功验证的 commit hash（供 check-no-verify.sh 审计）
POST_HOOK="$ROOT/.git/hooks/post-commit"
cat > "$POST_HOOK" <<'EOF'
#!/usr/bin/env bash
# Auto-installed by scripts/install-hooks.sh
# post-commit hook: 记录成功通过 pre-commit 验证的 commit hash

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

MARKER="$ROOT/.git/hooks/.last-verified"
HEAD_HASH=$(git rev-parse HEAD 2>/dev/null || true)
PRE_COMMIT_TS=$(cat "$ROOT/.git/hooks/.pre-commit-timestamp" 2>/dev/null || true)
CURRENT_TS=$(date +%s 2>/dev/null || true)

if [[ -n "$HEAD_HASH" ]]; then
  if [[ -n "$PRE_COMMIT_TS" ]] && [[ "$PRE_COMMIT_TS" -le "$CURRENT_TS" ]]; then
    echo "$HEAD_HASH" >> "$MARKER"
  fi
fi

rm -f "$ROOT/.git/hooks/.pre-commit-timestamp" 2>/dev/null || true
exit 0
EOF
chmod +x "$POST_HOOK"

echo "✅ 已安装 $HOOK"
echo "✅ 已安装 $MSG_HOOK"
echo "✅ 已安装 $PREP_HOOK"
echo "✅ 已安装 $POST_HOOK"
echo ""
echo "测试："
echo "  bash $HOOK          # 模拟 pre-commit"
echo "AI agent commit："
echo "  bash scripts/commit.sh -m \"commit message\"  # 通过 wrapper 提交"
echo "跳过 hook（紧急情况）："
echo "  git commit --no-verify （用户需添加 [MANUAL-OVERRIDE: <理由>] 标记）"
echo ""
echo "审计 override 使用："
echo "  bash scripts/audit-overrides.sh"
echo "  bash scripts/check-no-verify.sh --last 5    # 检查最近 5 个 commit"
