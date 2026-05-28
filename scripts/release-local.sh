#!/usr/bin/env bash
# release-local.sh — Day 20 跨平台分发的本地构建版
#
# 在当前 host 上编译 release 二进制并打成 .tar.gz（macOS/linux）或 .zip（windows），
# 输出到 dist/ 目录，可直接上传 GitHub Releases。
#
# 用法:
#   bash scripts/release-local.sh                   # 默认 features 组合（http+mcp+lsp+tantivy）
#   bash scripts/release-local.sh --features XXX    # 自定义 features
#   bash scripts/release-local.sh --skip-build      # 只重新打包已有 binary
#
# 输出:
#   dist/arkui-rag-v<VERSION>-<TARGET_TRIPLE>.tar.gz    （Linux/macOS）
#   dist/arkui-rag-v<VERSION>-<TARGET_TRIPLE>.zip       （Windows，未来）
#   dist/SHA256SUMS                                     （所有产物的 sha256）
#
# 当前 Day 20 范围：
#   - 仅本地 host 平台（CI matrix 留 Day 20 续）
#   - 默认 feature 组合避开两处 pre-existing 编译阻塞：
#       · lancedb (arrow-arith / chrono trait method 歧义)
#       · typescript (tree-sitter-typescript API 漂移)
#     这两项不影响默认 markdown 索引 + 真 BM25 + 三协议的核心闭环。
#
# 退出码:
#   0 = 打包成功；非 0 = 失败

set -uo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
CRATES_DIR="$REPO_ROOT/crates"
DIST_DIR="$REPO_ROOT/dist"
DEFAULT_FEATURES="http,mcp,lsp,tantivy"

# 颜色
if [ -t 1 ]; then
    RED=$'\033[0;31m'; GREEN=$'\033[0;32m'; YELLOW=$'\033[1;33m'; CYAN=$'\033[0;36m'; BOLD=$'\033[1m'; NC=$'\033[0m'
else
    RED=""; GREEN=""; YELLOW=""; CYAN=""; BOLD=""; NC=""
fi

usage() {
    cat <<EOF
Usage: $0 [--features FEATURES] [--skip-build]

Options:
  --features F   cargo features to enable（默认: $DEFAULT_FEATURES）
  --skip-build   不重新跑 cargo build，仅打包 target/release/arkui-rag
  -h, --help     show this help

Example:
  bash scripts/release-local.sh
  bash scripts/release-local.sh --features http,mcp,lsp
EOF
}

# 解析参数
FEATURES="$DEFAULT_FEATURES"
SKIP_BUILD=0
while [ $# -gt 0 ]; do
    case "$1" in
        --features) FEATURES="$2"; shift 2 ;;
        --skip-build) SKIP_BUILD=1; shift ;;
        -h|--help) usage; exit 0 ;;
        *) echo "${RED}未知参数: $1${NC}" >&2; usage; exit 2 ;;
    esac
done

echo "${BOLD}${CYAN}━━━ RAG4ArkUI · 本地 Release 打包 (Day 20) ━━━${NC}"
echo "  仓库      : $REPO_ROOT"
echo "  features  : $FEATURES"
echo "  skip-build: $SKIP_BUILD"
echo ""

# 1. 探测版本与 target triple
VERSION="$(grep -E '^version' "$CRATES_DIR/Cargo.toml" | head -1 | sed -E 's/.*"([^"]+)".*/\1/')"
if [ -z "$VERSION" ]; then
    echo "${RED}✗ 无法从 crates/Cargo.toml 读到 [workspace.package].version${NC}" >&2
    exit 1
fi

# rustc -vV 输出 host: aarch64-apple-darwin 这种
TARGET_TRIPLE="$(rustc -vV 2>/dev/null | awk '/^host:/ {print $2}')"
if [ -z "$TARGET_TRIPLE" ]; then
    echo "${RED}✗ rustc 未安装或不可执行${NC}" >&2
    exit 1
fi

# 平台后缀（统一用 tar.gz · Win10+ 内置 tar.exe 能解 · 简化 CI matrix）
OS_KIND=""
EXT="tar.gz"
BIN_SUFFIX=""
case "$TARGET_TRIPLE" in
    *-apple-darwin)        OS_KIND="macos" ;;
    *-unknown-linux-gnu*)  OS_KIND="linux-gnu" ;;
    *-unknown-linux-musl*) OS_KIND="linux-musl" ;;
    *-pc-windows-*)        OS_KIND="windows"; BIN_SUFFIX=".exe" ;;
    *)                     OS_KIND="unknown" ;;
esac

echo "  version   : $VERSION"
echo "  triple    : $TARGET_TRIPLE  ($OS_KIND)"
echo ""

ARTIFACT_NAME="arkui-rag-v${VERSION}-${TARGET_TRIPLE}"

# 2. 编译（除非 --skip-build）
if [ "$SKIP_BUILD" -eq 0 ]; then
    echo "${BOLD}[1/4] cargo build --release --features $FEATURES${NC}"
    ( cd "$CRATES_DIR" && cargo build --release -p arkui-rag-cli --features "$FEATURES" ) || {
        echo "${RED}✗ cargo build 失败${NC}" >&2
        exit 1
    }
else
    echo "${YELLOW}[1/4] 跳过 cargo build（--skip-build）${NC}"
fi

BIN_PATH="$CRATES_DIR/target/release/arkui-rag${BIN_SUFFIX}"
if [ ! -f "$BIN_PATH" ]; then
    echo "${RED}✗ 找不到产物: $BIN_PATH${NC}" >&2
    exit 1
fi

BIN_SIZE="$(du -h "$BIN_PATH" | awk '{print $1}')"
echo "  ✅ binary: $BIN_PATH ($BIN_SIZE)"

# 3. 烟雾测试：--version
echo ""
echo "${BOLD}[2/4] 烟雾测试 --version${NC}"
"$BIN_PATH" --version || {
    echo "${RED}✗ binary 无法跑${NC}" >&2
    exit 1
}

# 4. 暂存 staging 目录
STAGING="$DIST_DIR/.staging/$ARTIFACT_NAME"
rm -rf "$STAGING"
mkdir -p "$STAGING"

echo ""
echo "${BOLD}[3/4] 暂存产物到 $STAGING${NC}"
cp "$BIN_PATH" "$STAGING/arkui-rag${BIN_SUFFIX}"
[ -f "$REPO_ROOT/LICENSE" ]   && cp "$REPO_ROOT/LICENSE"   "$STAGING/"
[ -f "$REPO_ROOT/README.md" ] && cp "$REPO_ROOT/README.md" "$STAGING/"

# 生成 INSTALL.txt 用户拿到包后看的第一份说明
cat > "$STAGING/INSTALL.txt" <<INSTALL_EOF
RAG4ArkUI · arkui-rag v${VERSION}
Target: ${TARGET_TRIPLE}
Features: ${FEATURES}

【安装】
  tar -xzf arkui-rag-v${VERSION}-${TARGET_TRIPLE}.${EXT}
  cd arkui-rag-v${VERSION}-${TARGET_TRIPLE}
  ./arkui-rag --version
  # 可选：把 ./arkui-rag 复制到 PATH 中
  cp ./arkui-rag /usr/local/bin/

【快速试用】
  # 1. 准备一个含 .md 的 corpus 目录
  mkdir -p ~/my-corpus && echo "# Hello" > ~/my-corpus/test.md

  # 2. 建索引（Hybrid: Mock embedder + Tantivy BM25）
  ./arkui-rag index --source ~/my-corpus --index-path ~/my-corpus/index.json --bm25 tantivy

  # 3. 检索
  ./arkui-rag query --text "hello" --index-path ~/my-corpus/index.json --bm25 tantivy -k 3

  # 4. 启 HTTP 服务（IDE 集成）
  ./arkui-rag serve --http --addr 127.0.0.1:7654 --index-path ~/my-corpus/index.json --bm25 tantivy
  # 另一终端：
  curl http://127.0.0.1:7654/health
  curl -X POST http://127.0.0.1:7654/search -d '{"query":"hello","top_k":3}' -H "Content-Type: application/json"

  # 5. 启 MCP stdio 服务（Claude Code / Cursor）
  ./arkui-rag serve --mcp --index-path ~/my-corpus/index.json --bm25 tantivy
  # Claude Code 配置参考: docs/MCP-INTEGRATION-CLAUDE-CODE.md

  # 6. 启 LSP stdio 服务（DevEco / IntelliJ inline 提示）
  ./arkui-rag serve --lsp --index-path ~/my-corpus/index.json --bm25 tantivy

【本包未包含的能力】
  - ONNX 真实语义 embedding（需 --features onnx + 下载 BGE-M3 模型）
  - LanceDB 向量库（pre-existing arrow-arith 编译阻塞，已挂 follow-up）
  - tree-sitter 代码切分（pre-existing tree-sitter-typescript 0.21 API 漂移，已挂 follow-up）

【完整文档】
  https://github.com/keerecles/RAG4ArkUI
INSTALL_EOF

ls -lh "$STAGING/"

# 5. 打包
echo ""
echo "${BOLD}[4/4] 打包为 .${EXT}${NC}"
mkdir -p "$DIST_DIR"
ARCHIVE="$DIST_DIR/${ARTIFACT_NAME}.${EXT}"
rm -f "$ARCHIVE"

# 全平台统一 tar.gz（Win10+ 内置 tar.exe 能解 · 简化 CI matrix）
( cd "$DIST_DIR/.staging" && tar -czf "$ARCHIVE" "$ARTIFACT_NAME" )

if [ ! -f "$ARCHIVE" ]; then
    echo "${RED}✗ 打包失败: $ARCHIVE${NC}" >&2
    exit 1
fi

ARCHIVE_SIZE="$(du -h "$ARCHIVE" | awk '{print $1}')"

# 6. 计算 SHA256
SHA_FILE="$DIST_DIR/SHA256SUMS"
if command -v shasum >/dev/null 2>&1; then
    ( cd "$DIST_DIR" && shasum -a 256 "${ARTIFACT_NAME}.${EXT}" >> "$SHA_FILE" )
elif command -v sha256sum >/dev/null 2>&1; then
    ( cd "$DIST_DIR" && sha256sum "${ARTIFACT_NAME}.${EXT}" >> "$SHA_FILE" )
else
    echo "${YELLOW}⚠ 找不到 shasum/sha256sum，跳过 SHA256 计算${NC}" >&2
fi

# 清理 staging
rm -rf "$DIST_DIR/.staging"

echo ""
echo "${GREEN}${BOLD}✅ Release artifact 完成${NC}"
echo "  产物 : $ARCHIVE ($ARCHIVE_SIZE)"
[ -f "$SHA_FILE" ] && echo "  sha  : $(tail -1 "$SHA_FILE")"
echo ""
echo "${CYAN}下一步：${NC}"
echo "  1. 解压验证: cd /tmp && tar -xzf $ARCHIVE && cd $ARTIFACT_NAME && ./arkui-rag --version"
echo "  2. 推 GitHub Release（待 Day 20 续 · CI matrix 自动化）"
