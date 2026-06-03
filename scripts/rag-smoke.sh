#!/usr/bin/env bash
# rag-smoke.sh — 跑一批典型 query 看本地 RAG 质量 · 输出 markdown 报告
#
# 用法:
#   bash scripts/rag-smoke.sh                                     # 默认 index + queries
#   bash scripts/rag-smoke.sh --index-path /path/to/index.json   # 自定义索引
#   bash scripts/rag-smoke.sh --queries-file path.yaml           # 自定义 query 集
#   bash scripts/rag-smoke.sh --top-k 5                          # top-K 数（默认 3）
#   bash scripts/rag-smoke.sh --out reports/rag-smoke.md         # 输出 markdown
#
# 设计:
#   - 跑 corpus/_eval/smoke-queries.yaml 里所有 query
#   - 用 BGE-M3 embedder + Tantivy BM25
#   - 每 query 取 top-K hits · 打 markdown 表
#   - 同时输出 latency + summary

set -euo pipefail

# ── 默认 ──
INDEX_PATH="${INDEX_PATH:-/Users/leo/tmp-index-pull2/index.json}"
MODEL_PATH="${MODEL_PATH:-$HOME/.arkui-rag/models/bge-m3}"
QUERIES_FILE="${QUERIES_FILE:-corpus/_eval/smoke-queries.yaml}"
TOP_K="${TOP_K:-3}"
OUT_FILE="${OUT_FILE:-}"
BINARY="${BINARY:-$HOME/.local/bin/arkui-rag}"
MIN_VEC_SCORE=""

while [[ $# -gt 0 ]]; do
    case "$1" in
        --index-path) INDEX_PATH="$2"; shift 2 ;;
        --queries-file) QUERIES_FILE="$2"; shift 2 ;;
        --top-k) TOP_K="$2"; shift 2 ;;
        --out) OUT_FILE="$2"; shift 2 ;;
        --model-path) MODEL_PATH="$2"; shift 2 ;;
        --binary) BINARY="$2"; shift 2 ;;
        --min-vector-score) MIN_VEC_SCORE="$2"; shift 2 ;;
        -h|--help) sed -n '2,15p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
        *) echo "未知参数: $1" >&2; exit 2 ;;
    esac
done

# ── 校验 ──
[[ -x "$BINARY" ]] || { echo "❌ binary 不存在: $BINARY" >&2; exit 3; }
[[ -f "$INDEX_PATH" ]] || { echo "❌ index 不存在: $INDEX_PATH" >&2; exit 3; }
[[ -d "$MODEL_PATH" ]] || { echo "❌ model 不存在: $MODEL_PATH" >&2; exit 3; }
[[ -f "$QUERIES_FILE" ]] || { echo "❌ queries 不存在: $QUERIES_FILE" >&2; exit 3; }

# ── 解析 queries.yaml（用 ruby · 系统自带）──
# 注意：ruby 字符串里 "\t" 双引号才是 tab · 单引号 '\t' 是字面 \ + t
extract_queries() {
    ruby -ryaml -e '
        data = YAML.load_file(ARGV[0])
        queries = data["queries"] || []
        queries.each do |q|
            puts [q["id"], q["query"], q["expect"] || ""].join("\t")
        end
    ' "$QUERIES_FILE"
}

# ── 报告头 ──
write_header() {
    cat <<EOF
# 本地 RAG smoke 报告

- 索引: \`$INDEX_PATH\`
- 模型: \`$MODEL_PATH\`
- queries: \`$QUERIES_FILE\`
- top-K: $TOP_K
- 时间: $(date '+%Y-%m-%d %H:%M:%S')
- binary: \`$BINARY\` ($($BINARY --version 2>&1))

---

EOF
}

# ── 跑 1 个 query · 返回 top-K markdown 块 ──
run_one() {
    local id="$1"
    local q="$2"
    local expect="$3"

    local start_ns
    start_ns=$(date +%s%N 2>/dev/null || gdate +%s%N 2>/dev/null || echo 0)

    # cli 输出包含 ANSI escape + tracing log · 全部丢 stderr
    local out
    local extra_args=()
    if [[ -n "$MIN_VEC_SCORE" ]]; then
        extra_args+=("--min-vector-score" "$MIN_VEC_SCORE")
    fi
    out=$("$BINARY" query \
        --text "$q" \
        --embedder onnx \
        --model-path "$MODEL_PATH" \
        --index-path "$INDEX_PATH" \
        --bm25 tantivy \
        -k "$TOP_K" \
        ${extra_args[@]+"${extra_args[@]}"} 2>/dev/null) || {
        echo "⚠️ query 失败: $id"
        return 1
    }

    local end_ns
    end_ns=$(date +%s%N 2>/dev/null || gdate +%s%N 2>/dev/null || echo 0)
    local latency_ms="-"
    if [[ "$start_ns" != "0" && "$end_ns" != "0" ]]; then
        latency_ms=$(( (end_ns - start_ns) / 1000000 ))
    fi

    echo "## \`$id\` · $q"
    echo ""
    [[ -n "$expect" ]] && echo "**期望**: $expect"
    [[ -n "$expect" ]] && echo ""
    echo "**延迟**: ${latency_ms}ms"
    echo ""

    # 兼容 "⚠️ 无命中"（阈值过滤了所有结果）
    if echo "$out" | grep -q '无命中'; then
        echo "⚠️ **无命中**（阈值过滤了所有结果 · 这通常是好事 = 负样本被剔）"
        echo ""
        echo "---"
        echo ""
        return 0
    fi

    # 解析 cli 输出 · BSD awk 兼容
    echo "| # | rrf | vector | bm25 | source | heading |"
    echo "|---|---|---|---|---|---|"
    echo "$out" | awk '
        /^─── \[/ {
            line = $0
            sub(/^.*\[/, "", line); sub(/\].*$/, "", line); idx = line
            sline = $0; sub(/^.*rrf=/, "", sline); sub(/  .*$/, "", sline); rrf = sline
            vline = $0; sub(/^.*vector=/, "", vline); sub(/  .*$/, "", vline); vec = vline
            bline = $0; sub(/^.*bm25=/, "", bline); sub(/ .*$/, "", bline); bm = bline
            in_hit = 1; next
        }
        in_hit && /^  source/ { sub(/^  source *: */, ""); source = $0; next }
        in_hit && /^  heading/ {
            sub(/^  heading *: */, ""); heading = $0
            printf "| %s | %s | %s | %s | %s | %s |\n", idx, rrf, vec, bm, source, heading
            in_hit = 0
        }
    '
    echo ""
    echo "---"
    echo ""
}

# ── 主流程 ──
output() {
    write_header

    local total=0
    local hits=0
    local total_lat=0

    while IFS=$'\t' read -r id q expect; do
        total=$((total + 1))
        if run_one "$id" "$q" "$expect"; then
            hits=$((hits + 1))
        fi
    done < <(extract_queries)

    echo "## 总览"
    echo ""
    echo "- 总 queries: $total"
    echo "- 成功执行: $hits"
    echo ""
}

if [[ -n "$OUT_FILE" ]]; then
    mkdir -p "$(dirname "$OUT_FILE")"
    output > "$OUT_FILE"
    echo "✅ 报告写入 $OUT_FILE"
    echo ""
    echo "─ 摘要 ─"
    grep '^## ' "$OUT_FILE" | head -5
else
    output
fi
