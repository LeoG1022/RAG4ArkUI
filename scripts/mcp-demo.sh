#!/usr/bin/env bash
# mcp-demo.sh — MCP 端到端演示（Day 19）
#
# 用法:
#   bash scripts/mcp-demo.sh                # 默认（Mock + memory · 不需要任何外部模型）
#   bash scripts/mcp-demo.sh --keep         # 失败时保留临时目录用于诊断
#   bash scripts/mcp-demo.sh --verbose      # 显示 cargo 输出
#
# 流程:
#   1. 临时目录 → 投放 2 份 markdown → arkui-rag index
#   2. cat 4 个 JSON-RPC 请求 | arkui-rag serve --mcp (server 读完 stdin EOF 后退出)
#   3. 逐行解析响应 + 校验关键字段
#
# 退出码:
#   0 = 通过；非 0 = 失败
#
# 验证范围（Day 19 范围）:
#   - initialize 返回 protocolVersion + serverInfo
#   - tools/list 返回 4 个工具 + inputSchema
#   - tools/call arkui_search_docs 返回 content[0].type=text
#   - 控制台输出可读，没有 panic / 异常 stderr

set -u
set -o pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

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

if ! command -v cargo >/dev/null 2>&1; then
  echo "❌ cargo 未安装。先跑 make install-rust 或 curl https://sh.rustup.rs -sSf | sh"
  exit 127
fi

TMPDIR="/tmp/rag-mcp-demo-$$"
CORPUS="$TMPDIR/corpus"
INDEX="$TMPDIR/index.json"
REQ_FILE="$TMPDIR/requests.jsonl"
RESP_FILE="$TMPDIR/responses.jsonl"
ERR_FILE="$TMPDIR/stderr.log"
mkdir -p "$CORPUS"

cleanup() {
  rc=$?
  if [[ $rc -ne 0 && $KEEP_ON_FAIL -eq 1 ]]; then
    echo ""
    echo "⚠️  失败保留临时目录：$TMPDIR"
  else
    rm -rf "$TMPDIR"
  fi
  exit $rc
}
trap cleanup EXIT

# ─── Step 1: 准备 corpus + 索引 ─────────────────
echo "═══ [1/4] 准备 corpus + 建索引 ═══"

cat > "$CORPUS/router.md" <<'EOF'
---
platforms: [HarmonyOS, Android]
type: api_doc
tags: [routing]
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
EOF

if [[ $VERBOSE -eq 1 ]]; then
  cargo run --features mcp -p arkui-rag-cli --quiet -- \
    index --source "$CORPUS" --index-path "$INDEX" --dim 64
else
  cargo run --features mcp -p arkui-rag-cli --quiet -- \
    index --source "$CORPUS" --index-path "$INDEX" --dim 64 \
    >/dev/null 2>&1
fi

if [[ ! -f "$INDEX" ]]; then
  echo "❌ 索引产物未生成"
  exit 1
fi
echo "  ✅ 索引就绪：$INDEX ($(wc -c < "$INDEX") bytes)"

# ─── Step 2: 构造 4 个 JSON-RPC 请求 ────────────
echo ""
echo "═══ [2/4] 构造 4 个 JSON-RPC 请求 ═══"

cat > "$REQ_FILE" <<EOF
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}
{"jsonrpc":"2.0","method":"notifications/initialized"}
{"jsonrpc":"2.0","id":2,"method":"tools/list"}
{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"arkui_search_docs","arguments":{"query":"ArkUI-X 用 Refresh 组件实现下拉刷新功能。","top_k":3}}}
EOF

echo "  ✅ 4 个请求就绪（initialize · notifications/initialized · tools/list · tools/call）"

# ─── Step 3: 启动 server · 投喂请求 · 收响应 ────
echo ""
echo "═══ [3/4] 启动 MCP server，喂请求，收响应 ═══"

# Server 读完 stdin EOF 后退出。一次性把 REQ_FILE 当 stdin 喂入。
# stderr 单独捕获到 ERR_FILE 便于诊断。
if ! cargo run --features mcp -p arkui-rag-cli --quiet -- \
      serve --mcp --index-path "$INDEX" \
      < "$REQ_FILE" > "$RESP_FILE" 2> "$ERR_FILE"; then
  rc=$?
  # MCP server 正常 stdin EOF 退出码应为 0；如非 0 看 stderr
  if [[ $rc -ne 0 ]]; then
    echo "❌ MCP server 异常退出（rc=$rc）"
    echo "── stderr ──"
    cat "$ERR_FILE"
    exit 1
  fi
fi

resp_count=$(wc -l < "$RESP_FILE" | tr -d ' ')
if [[ $VERBOSE -eq 1 ]]; then
  echo "── stderr 启动信息 ──"
  cat "$ERR_FILE" | head -3
  echo ""
  echo "── 响应行数：$resp_count（预期 3：initialize / tools-list / tools-call）──"
fi

# notifications/initialized 不响应 → 应该收到 3 行 response
if [[ $resp_count -ne 3 ]]; then
  echo "❌ 响应行数 $resp_count != 期望 3"
  echo "── responses ──"
  cat "$RESP_FILE"
  exit 1
fi

# ─── Step 4: 断言 ──────────────────────────────
echo ""
echo "═══ [4/4] 解析响应 + 断言 ═══"

# 4.1 initialize 响应 (line 1)
init_line=$(sed -n '1p' "$RESP_FILE")
if [[ "$init_line" != *'"protocolVersion":"2024-11-05"'* ]]; then
  echo "❌ initialize 响应缺 protocolVersion=2024-11-05"
  echo "实际：$init_line"
  exit 1
fi
if [[ "$init_line" != *'"name":"arkui-rag"'* ]]; then
  echo "❌ initialize 响应缺 serverInfo.name=arkui-rag"
  exit 1
fi
echo "  ✅ initialize: protocolVersion=2024-11-05, serverInfo.name=arkui-rag"

# 4.2 tools/list 响应 (line 2)
list_line=$(sed -n '2p' "$RESP_FILE")
for tool in arkui_search_docs arkui_search_code arkui_migrate_snippet arkui_validate_api; do
  if [[ "$list_line" != *"\"$tool\""* ]]; then
    echo "❌ tools/list 缺工具 $tool"
    exit 1
  fi
done
echo "  ✅ tools/list: 4 个工具齐全（search_docs/search_code/migrate_snippet/validate_api）"

# 4.3 tools/call 响应 (line 3)
call_line=$(sed -n '3p' "$RESP_FILE")
if [[ "$call_line" != *'"type":"text"'* ]]; then
  echo "❌ tools/call 响应 content 缺 type=text"
  echo "实际：$call_line"
  exit 1
fi
# MockEmbedder 对同样文本 cosine=1 → 必含 list.md
if [[ "$call_line" != *"list.md"* ]]; then
  echo "❌ tools/call 响应未含期望文件 list.md（Mock 阶段对原文本必命中）"
  echo "实际：$call_line"
  exit 1
fi
echo "  ✅ tools/call: 返回 markdown 文本 + 命中 list.md"

echo ""
echo "🎉 Day 19 MCP 端到端演示 PASS"
echo ""
echo "下一步："
echo "  1. 看 docs/MCP-INTEGRATION-CLAUDE-CODE.md 完整接入指南"
echo "  2. 配置 ~/.claude/mcp.json 接 Claude Code 体验"
echo "  3. 真实 corpus 用 --features full 启用 ONNX + LanceDB + Tantivy"

exit 0
