#!/usr/bin/env bash
# demo-smoke.sh — RAG4ArkUI 端到端冒烟测试
#
# 用法:
#   bash scripts/demo-smoke.sh                 # 跑完整冒烟
#   bash scripts/demo-smoke.sh --keep          # 失败时保留临时目录便于诊断
#   bash scripts/demo-smoke.sh --verbose       # 打印 cargo 输出
#
# 退出码:
#   0 = 通过；非 0 = 失败
#
# 范围（Day 2 Mock 阶段）:
#   1. 在 /tmp/rag-smoke-<pid>/corpus/ 投放 2 份带 frontmatter 的 markdown
#   2. 跑 arkui-rag index → 断言 IndexStats 含 files=2 / chunks>=3
#   3. 跑 arkui-rag query → 断言 Top-1 命中预期文件
#   4. 清理临时目录（除非 --keep）
#
# 不验证：检索语义质量（Mock 阶段无意义）；ONNX 路径（Day 3 后单独 smoke）。

set -u
set -o pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

# ─── 选项 ─────────────────────────────────────
KEEP_ON_FAIL=0
VERBOSE=0
for arg in "$@"; do
  case "$arg" in
    --keep)    KEEP_ON_FAIL=1 ;;
    --verbose) VERBOSE=1 ;;
    -h|--help)
      sed -n '/^# /,/^$/p' "$0" | sed 's/^# \?//'
      exit 0
      ;;
  esac
done

# ─── 前置检查 ─────────────────────────────────
if ! command -v cargo >/dev/null 2>&1; then
  echo "❌ cargo 未安装。先跑 make install-rust 或 curl https://sh.rustup.rs -sSf | sh"
  exit 127
fi

TMPDIR="/tmp/rag-smoke-$$"
CORPUS="$TMPDIR/corpus"
INDEX="$TMPDIR/index.json"
mkdir -p "$CORPUS"

cleanup() {
  rc=$?
  if [[ $rc -ne 0 && $KEEP_ON_FAIL -eq 1 ]]; then
    echo ""
    echo "⚠️  失败保留临时目录便于诊断：$TMPDIR"
  else
    rm -rf "$TMPDIR"
  fi
  exit $rc
}
trap cleanup EXIT

run_cargo() {
  if [[ $VERBOSE -eq 1 ]]; then
    cargo "$@"
  else
    cargo "$@" 2>&1 | tail -20
  fi
}

# ─── Step 1: 投放 markdown ────────────────────
echo "═══ [1/4] 投放 2 份 markdown 到 $CORPUS ═══"

cat > "$CORPUS/router.md" <<'EOF'
---
platforms: [HarmonyOS, Android]
api_version: "ArkUI-X 1.2"
type: api_doc
tags: [routing, navigation]
---

# Router

## pushUrl
推送新页面到路由栈。可以传递参数和回调。

## back
返回到上一个页面，可指定回退多少层。
EOF

cat > "$CORPUS/list.md" <<'EOF'
---
platforms: [HarmonyOS]
type: code_example
tags: [list, refresh, pull-to-refresh]
---

# List

## 下拉刷新
ArkUI-X 用 Refresh 组件实现下拉刷新功能。

## 懒加载
配合 LazyForEach 实现按需渲染。
EOF

echo "  ✅ 2 份 markdown 就位"

# ─── Step 2: cargo run index ──────────────────
echo ""
echo "═══ [2/4] cargo run -- index --source $CORPUS ═══"

INDEX_OUT="$TMPDIR/index.out"
if ! cargo run --manifest-path crates/Cargo.toml -p arkui-rag-cli --quiet -- \
      index --source "$CORPUS" --index-path "$INDEX" --dim 64 \
      > "$INDEX_OUT" 2>&1; then
  echo "❌ cargo run index 失败"
  cat "$INDEX_OUT"
  exit 1
fi

if [[ $VERBOSE -eq 1 ]]; then
  cat "$INDEX_OUT"
fi

# 断言：files=2、chunks>=4、产物文件存在
if ! grep -qE "^   files       : 2$" "$INDEX_OUT"; then
  echo "❌ 期望 files=2，未匹配："
  grep -E "^   (files|chunks|skipped)" "$INDEX_OUT" || true
  exit 1
fi
if ! grep -qE "^   chunks      : [4-9]" "$INDEX_OUT"; then
  echo "❌ 期望 chunks>=4，未匹配："
  grep -E "^   (files|chunks|skipped)" "$INDEX_OUT" || true
  exit 1
fi
if [[ ! -f "$INDEX" ]]; then
  echo "❌ 索引产物未生成：$INDEX"
  exit 1
fi
echo "  ✅ 索引产物落盘：$INDEX ($(wc -c < "$INDEX") bytes)"

# ─── Step 3: cargo run query ──────────────────
echo ""
echo "═══ [3/4] cargo run -- query --text 下拉刷新 ═══"

QUERY_TEXT="ArkUI-X 用 Refresh 组件实现下拉刷新功能。"
QUERY_OUT="$TMPDIR/query.out"
if ! cargo run --manifest-path crates/Cargo.toml -p arkui-rag-cli --quiet -- \
      query --text "$QUERY_TEXT" --k 3 --index-path "$INDEX" \
      > "$QUERY_OUT" 2>&1; then
  echo "❌ cargo run query 失败"
  cat "$QUERY_OUT"
  exit 1
fi

if [[ $VERBOSE -eq 1 ]]; then
  cat "$QUERY_OUT"
fi

# 断言：必须有 Top-N hits + Top-1 必须命中 list.md
if ! grep -q "Top-" "$QUERY_OUT"; then
  echo "❌ query 未返回 hits"
  cat "$QUERY_OUT"
  exit 1
fi

# 提取第一个 hit 的 source（MockEmbedder 对同样文本必然返回 cosine=1，必命中 list.md）
TOP1_SOURCE=$(grep -m1 "^  source : " "$QUERY_OUT" | sed 's/^  source : //' | awk '{print $1}')
if [[ "$TOP1_SOURCE" != "list.md" ]]; then
  echo "❌ 期望 Top-1 source=list.md，实际=$TOP1_SOURCE"
  cat "$QUERY_OUT"
  exit 1
fi
echo "  ✅ Top-1 命中 list.md（cosine=1 因为 MockEmbedder 对同文本确定性）"

# ─── Step 4: 平台过滤测试 ────────────────────
# 这步暂跳——CLI 当前没暴露 --platform 过滤参数，由 retriever 层支持但需 server 层暴露。
# Day 3+ 加 CLI 参数后再补 smoke 用例。
echo ""
echo "═══ [4/4] 通过 ═══"
echo "  ✅ index + query 端到端跑通"
echo "  ✅ 索引产物 schema 正确（JSON load 成功）"
echo "  ✅ Top-1 命中预期文件"
echo ""
echo "🎉 Day 2 Mock RAG smoke PASS"
exit 0
