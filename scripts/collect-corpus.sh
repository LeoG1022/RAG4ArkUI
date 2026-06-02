#!/usr/bin/env bash
# collect-corpus.sh — maintainer 收集 ArkUI-X / OpenHarmony 官方文档 · 放 corpus/official/
#
# 设计:
#   - ArkUI-X: shallow clone (--depth=1) · 380MB · 1-2 分钟
#   - OpenHarmony: partial clone (--filter=blob:none) + sparse checkout · 600MB → ~30MB meta + 指定子目录 blob
#   - rsync 只复制 .md 文件 + LICENSE · 不要 images / .git / 二进制资源
#   - 输出到 corpus/official/<source>/ · 子目录隔离
#
# 用法:
#   bash scripts/collect-corpus.sh                          # 全收 (默认 arkui-x + openharmony)
#   bash scripts/collect-corpus.sh --src arkui-x            # 单收 ArkUI-X
#   bash scripts/collect-corpus.sh --src openharmony        # 单收 OpenHarmony (partial+sparse)
#   bash scripts/collect-corpus.sh --lang zh-cn             # 只要中文 (默认 zh-cn en)
#   bash scripts/collect-corpus.sh --oh-paths "zh-cn/application-dev"   # 覆盖 OpenHarmony sparse 路径
#   bash scripts/collect-corpus.sh --clean                  # 清掉 corpus/official/{arkui-x,openharmony}
#
# 注意:
#   - OpenHarmony 默认 sparse: zh-cn/application-dev + device-dev · en/* 同款
#   - 复用既有 clone：$TMPDIR/corpus-collect/<name>/ 有 .git 就跳过 · rm -rf 强制重拉
#   - LICENSE 必保留 (Apache 2.0 重分发要求)

set -euo pipefail

# ── 配置 ──
ARKUIX_REPO="https://gitcode.com/arkui-x/docs.git"
OPENHARMONY_REPO="https://gitcode.com/openharmony/docs.git"

# Round 49.7: OpenHarmony 仓库 600MB+ · shallow clone 5 分钟超时仍坏（HEAD 损坏）
# 改 partial clone（--filter=blob:none · 跳 blob 下载）+ sparse checkout（限定子目录）
# 默认子目录 · 用户可 --oh-paths 覆盖
OPENHARMONY_SPARSE_PATHS="zh-cn/application-dev zh-cn/device-dev en/application-dev en/device-dev"

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CORPUS_DIR="$REPO_ROOT/corpus/official"
TMP_DIR="${TMPDIR:-/tmp}/corpus-collect"

# ── 参数 ──
SRC=""           # 默认全收
LANGS="zh-cn en" # 默认中英双语
CLEAN=0
OH_PATHS=""      # OpenHarmony 自定义 sparse paths（空 = 用 OPENHARMONY_SPARSE_PATHS 默认）
while [[ $# -gt 0 ]]; do
    case "$1" in
        --src) SRC="$2"; shift 2 ;;
        --lang) LANGS="$2"; shift 2 ;;
        --oh-paths) OH_PATHS="$2"; shift 2 ;;
        --clean) CLEAN=1; shift ;;
        -h|--help) sed -n '2,20p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
        *) echo "未知参数: $1" >&2; exit 2 ;;
    esac
done

# OpenHarmony sparse paths：用户指定优先 · 否则用默认
OH_SPARSE_LIST="${OH_PATHS:-$OPENHARMONY_SPARSE_PATHS}"

# ── Clean ──
if [[ "$CLEAN" == "1" ]]; then
    echo "🧹 清理 corpus/official/{arkui-x,openharmony}/"
    rm -rf "$CORPUS_DIR/arkui-x" "$CORPUS_DIR/openharmony"
    exit 0
fi

# ── shallow clone（适合 <500MB 仓库）──
clone_shallow() {
    local repo="$1"
    local clone_dir="$2"
    echo "🌐 shallow clone（不带 history · 几分钟）"
    git clone --depth=1 "$repo" "$clone_dir"
}

# ── partial clone + sparse checkout（适合 600MB+ 大仓库 · OpenHarmony 专用）──
# 原理：
#   --filter=blob:none  跳 blob 下载（meta 拉完按需 fetch · 600MB → ~30MB meta）
#   --no-checkout       不展开工作树（避免下全部文件）
#   sparse-checkout set 只检出指定子目录 · checkout 时仅 fetch 这些 blob
clone_partial_sparse() {
    local repo="$1"
    local clone_dir="$2"
    local sparse_paths="$3"   # 空格分隔的子目录路径

    echo "🌐 partial clone --filter=blob:none --no-checkout（meta only · ~30MB）"
    git clone --filter=blob:none --no-checkout --depth=1 "$repo" "$clone_dir" || return 1

    echo "🎯 sparse-checkout 限定子目录：$sparse_paths"
    (
        cd "$clone_dir" || exit 1
        git sparse-checkout init --cone
        # shellcheck disable=SC2086
        git sparse-checkout set $sparse_paths
        # 自动用默认分支 · 不假设是 master/main
        local default_branch
        default_branch=$(git symbolic-ref --short HEAD 2>/dev/null || echo "")
        if [[ -z "$default_branch" ]]; then
            default_branch=$(git remote show origin 2>/dev/null | grep 'HEAD branch' | awk '{print $NF}')
        fi
        if [[ -z "$default_branch" ]]; then
            default_branch="master"   # fallback
        fi
        echo "  📦 checkout $default_branch（拉指定子目录 blob）"
        git checkout "$default_branch"
    )
}

# ── 收集函数 · 按 name 路由 clone 策略 ──
collect_one() {
    local name="$1"      # arkui-x | openharmony
    local repo="$2"      # 仓库 URL
    local clone_dir="$TMP_DIR/$name"
    local dst="$CORPUS_DIR/$name"

    mkdir -p "$TMP_DIR"
    if [[ -d "$clone_dir/.git" ]]; then
        echo "📂 $name: 复用既有 clone (rm -rf $clone_dir 强制重拉)"
    else
        case "$name" in
            openharmony)
                # 大仓库 · partial + sparse
                clone_partial_sparse "$repo" "$clone_dir" "$OH_SPARSE_LIST" || {
                    echo "❌ $name clone 失败 · 跳过" >&2
                    return 1
                }
                ;;
            *)
                # 小仓库 · shallow（ArkUI-X 380MB OK）
                clone_shallow "$repo" "$clone_dir" || {
                    echo "❌ $name clone 失败 · 跳过" >&2
                    return 1
                }
                ;;
        esac
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

    local cnt
    cnt=$(find "$dst" -name '*.md' 2>/dev/null | wc -l | tr -d ' ')
    local size
    size=$(du -sh "$dst" 2>/dev/null | awk '{print $1}')
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
