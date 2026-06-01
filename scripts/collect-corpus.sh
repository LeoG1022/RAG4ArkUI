#!/usr/bin/env bash
# collect-corpus.sh — maintainer 收集 ArkUI-X / OpenHarmony 官方文档 · 放 corpus/official/
#
# 设计:
#   - shallow clone (--depth=1) 不要 Git history · 省磁盘
#   - rsync 只复制 .md 文件 + LICENSE · 不要 images / .git / 二进制资源
#   - 输出到 corpus/official/<source>/ · 子目录隔离
#
# 用法:
#   bash scripts/collect-corpus.sh                  # 全收 (默认: arkui-x + openharmony)
#   bash scripts/collect-corpus.sh --src arkui-x    # 单收 ArkUI-X
#   bash scripts/collect-corpus.sh --src openharmony   # 单收 OpenHarmony
#   bash scripts/collect-corpus.sh --lang zh-cn     # 只要中文 (默认 zh-cn+en)
#   bash scripts/collect-corpus.sh --clean          # 清掉 corpus/official/{arkui-x,openharmony}
#
# 注意:
#   - OpenHarmony docs 仓库 600MB+ · clone 可能 10+ 分钟 · 看网络
#   - 收完只跑 .md 文件 size 一般 < 50MB · gzip 后 ~5-30MB
#   - LICENSE 必保留 (Apache 2.0 重分发要求)

set -euo pipefail

# ── 配置 ──
ARKUIX_REPO="https://gitcode.com/arkui-x/docs.git"
OPENHARMONY_REPO="https://gitcode.com/openharmony/docs.git"

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CORPUS_DIR="$REPO_ROOT/corpus/official"
TMP_DIR="${TMPDIR:-/tmp}/corpus-collect"

# ── 参数 ──
SRC=""           # 默认全收
LANGS="zh-cn en" # 默认中英双语
CLEAN=0
while [[ $# -gt 0 ]]; do
    case "$1" in
        --src) SRC="$2"; shift 2 ;;
        --lang) LANGS="$2"; shift 2 ;;
        --clean) CLEAN=1; shift ;;
        -h|--help) sed -n '2,20p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
        *) echo "未知参数: $1" >&2; exit 2 ;;
    esac
done

# ── Clean ──
if [[ "$CLEAN" == "1" ]]; then
    echo "🧹 清理 corpus/official/{arkui-x,openharmony}/"
    rm -rf "$CORPUS_DIR/arkui-x" "$CORPUS_DIR/openharmony"
    exit 0
fi

# ── shallow clone + rsync .md 函数 ──
collect_one() {
    local name="$1"      # arkui-x | openharmony
    local repo="$2"      # 仓库 URL
    local clone_dir="$TMP_DIR/$name"
    local dst="$CORPUS_DIR/$name"

    mkdir -p "$TMP_DIR"
    if [[ -d "$clone_dir/.git" ]]; then
        echo "📂 $name: 复用既有 clone (rm -rf $clone_dir 强制重拉)"
    else
        echo "🌐 $name: shallow clone（不带 history · 几分钟）"
        git clone --depth=1 "$repo" "$clone_dir" || {
            echo "❌ $name clone 失败 · 跳过" >&2
            return 1
        }
    fi

    mkdir -p "$dst"
    # LICENSE 必保留
    [[ -f "$clone_dir/LICENSE" ]] && cp "$clone_dir/LICENSE" "$dst/"
    [[ -f "$clone_dir/README.md" ]] && cp "$clone_dir/README.md" "$dst/"

    # 按 lang 收 .md
    for lang in $LANGS; do
        if [[ -d "$clone_dir/$lang" ]]; then
            echo "  📄 $name/$lang"
            rsync -a \
                --include='*/' \
                --include='*.md' \
                --exclude='*' \
                "$clone_dir/$lang/" \
                "$dst/$lang/"
        fi
    done

    local cnt=$(find "$dst" -name '*.md' 2>/dev/null | wc -l | tr -d ' ')
    local size=$(du -sh "$dst" 2>/dev/null | awk '{print $1}')
    echo "  ✅ $name: $cnt files · $size"
}

# ── 执行 ──
echo "═══ Collect Corpus ($LANGS) ═══"
if [[ -z "$SRC" ]] || [[ "$SRC" == "arkui-x" ]]; then
    collect_one "arkui-x" "$ARKUIX_REPO"
fi
if [[ -z "$SRC" ]] || [[ "$SRC" == "openharmony" ]]; then
    collect_one "openharmony" "$OPENHARMONY_REPO"
fi

echo ""
echo "═══ 总览 ═══"
find "$CORPUS_DIR" -name '*.md' 2>/dev/null | wc -l | xargs echo "  总 .md 文件数:"
du -sh "$CORPUS_DIR" 2>/dev/null | awk '{print "  corpus/official/ 总大小: " $1}'

echo ""
echo "下一步:"
echo "  bash scripts/build-corpus-index.sh   # build index (CoreML env bypass)"
echo "  或手动:"
echo "    arkui-rag index --source corpus/official \\"
echo "        --embedder onnx --model-path ~/.arkui-rag/models/bge-m3 \\"
echo "        --index-path /tmp/corpus-vX.Y.Z/index.json \\"
echo "        --bm25 tantivy"
