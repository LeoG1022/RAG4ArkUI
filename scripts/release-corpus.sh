#!/usr/bin/env bash
# release-corpus.sh —— 打包 corpus + index tarball + SHA256SUMS · 本地 + CI 共用
#
# 设计:
#   - 输入：build 已完成 · INDEX_DIR 含 index.json + bm25/（cli build 自然产物）
#   - 输入：CORPUS_DIR 含 corpus/official/... 子目录
#   - 输出：OUTPUT_DIR/{corpus tarball, index tarball, SHA256SUMS}
#
# 用法:
#   bash scripts/release-corpus.sh \
#       --version v1.0.0 \
#       --embedder bge-m3 \
#       --index-dir /tmp/dist/index \
#       --corpus-dir corpus/official \
#       --output-dir /tmp/dist
#
# 命名约定（与 cli default_index_url / DEFAULT_CORPUS_URL 对齐）:
#   arkui-rag-corpus-${VERSION}.tar.gz
#   arkui-rag-index-${EMBEDDER}-${VERSION}.tar.gz
#   SHA256SUMS

set -euo pipefail

VERSION=""
EMBEDDER="bge-m3"
INDEX_DIR=""
CORPUS_DIR=""
OUTPUT_DIR=""

while [[ $# -gt 0 ]]; do
    case "$1" in
        --version) VERSION="$2"; shift 2 ;;
        --embedder) EMBEDDER="$2"; shift 2 ;;
        --index-dir) INDEX_DIR="$2"; shift 2 ;;
        --corpus-dir) CORPUS_DIR="$2"; shift 2 ;;
        --output-dir) OUTPUT_DIR="$2"; shift 2 ;;
        -h|--help) sed -n '2,22p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
        *) echo "未知参数: $1" >&2; exit 2 ;;
    esac
done

# ── 参数校验 ──
for var in VERSION INDEX_DIR CORPUS_DIR OUTPUT_DIR; do
    if [[ -z "${!var}" ]]; then
        echo "❌ 缺参数: --${var,,}" >&2
        echo "  用 -h 看帮助" >&2
        exit 2
    fi
done

if [[ ! -f "$INDEX_DIR/index.json" ]]; then
    echo "❌ INDEX_DIR 缺 index.json: $INDEX_DIR" >&2
    exit 3
fi
if [[ ! -d "$INDEX_DIR/bm25" ]]; then
    echo "❌ INDEX_DIR 缺 bm25/: $INDEX_DIR" >&2
    exit 3
fi
if [[ ! -d "$CORPUS_DIR" ]]; then
    echo "❌ CORPUS_DIR 不存在: $CORPUS_DIR" >&2
    exit 3
fi

mkdir -p "$OUTPUT_DIR"

# ── tar 选项检测：GNU tar 用 --owner/--mtime · BSD tar 不支持 ──
TAR_REPRODUCIBLE=()
if tar --version 2>&1 | grep -qi 'GNU tar'; then
    # 跨机器 reproducible build（hash 稳定）
    TAR_REPRODUCIBLE=(
        --sort=name
        --owner=0
        --group=0
        --numeric-owner
        --mtime='2026-01-01 00:00:00 UTC'
    )
    echo "🔧 GNU tar 检测到 · 启用 reproducible options"
else
    echo "🔧 BSD tar 检测到（macOS）· reproducible options 跳过（hash 跨机器可能不同）"
fi

# ── corpus tarball ──
CORPUS_TGZ="$OUTPUT_DIR/arkui-rag-corpus-${VERSION}.tar.gz"
echo ""
echo "═══ 打包 corpus ═══"
echo "  来源: $CORPUS_DIR"
echo "  目标: $CORPUS_TGZ"
# 从 CORPUS_DIR 的父目录打包 · 保留 official/ 结构
CORPUS_PARENT="$(dirname "$CORPUS_DIR")"
CORPUS_LEAF="$(basename "$CORPUS_DIR")"
tar -czf "$CORPUS_TGZ" \
    ${TAR_REPRODUCIBLE[@]+"${TAR_REPRODUCIBLE[@]}"} \
    -C "$CORPUS_PARENT" \
    "$CORPUS_LEAF"
ls -lh "$CORPUS_TGZ" | awk '{print "  size:", $5}'

# ── index tarball ──
INDEX_TGZ="$OUTPUT_DIR/arkui-rag-index-${EMBEDDER}-${VERSION}.tar.gz"
echo ""
echo "═══ 打包 index ═══"
echo "  来源: $INDEX_DIR (index.json + bm25/)"
echo "  目标: $INDEX_TGZ"
tar -czf "$INDEX_TGZ" \
    ${TAR_REPRODUCIBLE[@]+"${TAR_REPRODUCIBLE[@]}"} \
    -C "$INDEX_DIR" \
    index.json bm25
ls -lh "$INDEX_TGZ" | awk '{print "  size:", $5}'

# ── SHA256SUMS ──
echo ""
echo "═══ SHA256SUMS ═══"
cd "$OUTPUT_DIR"
# shasum on macOS · sha256sum on Linux
if command -v sha256sum >/dev/null; then
    sha256sum "$(basename "$CORPUS_TGZ")" "$(basename "$INDEX_TGZ")" > SHA256SUMS
else
    shasum -a 256 "$(basename "$CORPUS_TGZ")" "$(basename "$INDEX_TGZ")" > SHA256SUMS
fi
cat SHA256SUMS

# ── 总结 ──
echo ""
echo "═══ 总结 ═══"
ls -lh "$CORPUS_TGZ" "$INDEX_TGZ" SHA256SUMS
echo ""
echo "下一步:"
echo "  上传到 GitHub Release tag corpus-${VERSION}:"
echo "    gh release create corpus-${VERSION} \\"
echo "      $CORPUS_TGZ $INDEX_TGZ ${OUTPUT_DIR}/SHA256SUMS \\"
echo "      --title 'Corpus + Index ${VERSION}' \\"
echo "      --notes 'Pre-built corpus + index for arkui-rag corpus index-pull'"
