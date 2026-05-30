#!/usr/bin/env bash
# install-binary.sh — 一键把新 build 的 arkui-rag 装到用户目录 + 自动配 Claude CLI / Desktop MCP
#
# 用法：
#   bash scripts/install-binary.sh                          # 默认装到 ~/.local/bin
#   bash scripts/install-binary.sh --bin-dir ~/bin          # 自定义安装目录
#   bash scripts/install-binary.sh --skip-mcp               # 跳过 MCP 自动配置
#   ARKUI_INDEX_PATH=/foo/idx.json bash scripts/install-binary.sh   # 自定义索引路径
#
# 设计要点（为什么不用 /usr/local/bin）：
#   - macOS Sequoia 对 root-owned + 非 Apple-signed binary 做 provenance 检查
#   - sudo cp 到 /usr/local/bin/ 会触发 SIGKILL（exit 137）· 静默死
#   - 装到 user-owned 目录（~/.local/bin）则不触发 · 长期稳
#
# 副作用：
#   - cp binary 到 BIN_DIR
#   - 改 ~/.claude.json（通过 claude mcp add 命令）
#   - 改 ~/Library/Application Support/Claude/claude_desktop_config.json（合并 mcpServers）
#   - 不动 PATH（脚本只检查 · 不写 shell 启动文件）

set -euo pipefail

# ─── 参数解析 ───
BIN_DIR="${HOME}/.local/bin"
SKIP_MCP=0
while [[ $# -gt 0 ]]; do
    case "$1" in
        --bin-dir) BIN_DIR="$2"; shift 2 ;;
        --skip-mcp) SKIP_MCP=1; shift ;;
        -h|--help)
            sed -n '2,20p' "$0" | sed 's/^# \{0,1\}//'
            exit 0 ;;
        *) echo "未知参数: $1" >&2; exit 2 ;;
    esac
done

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SRC_BINARY="$REPO_ROOT/crates/target/release/arkui-rag"
INDEX_PATH="${ARKUI_INDEX_PATH:-$HOME/.arkui-rag/index.json}"

# ─── Step 1 · 校验 source binary ───
echo "═══ Step 1 · 校验 source binary ═══"
if [[ ! -x "$SRC_BINARY" ]]; then
    cat <<EOF >&2
❌ Source binary 不存在或没执行权限: $SRC_BINARY

请先 build:
    make release-local                                    # 推荐 · 自动选 features
    cd crates && cargo build --release -p arkui-rag-cli   # 或者裸 cargo
EOF
    exit 1
fi
SRC_VERSION="$("$SRC_BINARY" --version)"
echo "  source : $SRC_BINARY"
echo "  version: $SRC_VERSION"

# ─── Step 2 · 安装到 BIN_DIR ───
echo ""
echo "═══ Step 2 · 装到 $BIN_DIR ═══"
mkdir -p "$BIN_DIR"

DST_BINARY="$BIN_DIR/arkui-rag"
# 如果存在旧版 · 备份（防回滚 + 防 macOS in-use 占用）
if [[ -f "$DST_BINARY" ]]; then
    cp "$DST_BINARY" "$DST_BINARY.bak.$(date +%Y%m%d-%H%M%S)"
    echo "  备份旧版本: $DST_BINARY.bak.*"
fi
cp "$SRC_BINARY" "$DST_BINARY"
chmod +x "$DST_BINARY"

# macOS 专属：ad-hoc self-sign · 抑制 provenance 检查 SIGKILL
# 原因：cp 覆盖既有文件后 · macOS 可能缓存旧 provenance 状态 → 新 binary 第一次跑 exit 137
# self-sign 后 codesign 状态明确 · 系统不再 kill。不需要 sudo（user-owned 文件）
if [[ "$(uname -s)" == "Darwin" ]] && command -v codesign >/dev/null 2>&1; then
    codesign --force --sign - "$DST_BINARY" 2>&1 | grep -v "replacing existing signature" || true
fi

# ─── Step 3 · 跑 --version 验证装好的 binary 能正常启动 ───
echo ""
echo "═══ Step 3 · 验证装好的 binary 能跑 ═══"
if ! INSTALLED_VERSION="$("$DST_BINARY" --version 2>&1)"; then
    cat <<EOF >&2
❌ 装到 $BIN_DIR 后跑不起来（exit code 非 0）。

可能原因：
  1. macOS provenance / quarantine 检查（少见 · 用户目录通常 OK）
     诊断: xattr -l "$DST_BINARY"
     如有 com.apple.quarantine: xattr -d com.apple.quarantine "$DST_BINARY"
  2. 路径在某个特殊保护目录（如 SIP 保护下）
     换 --bin-dir 试 ~/bin 或 /tmp

输出详情:
    $DST_BINARY --version
EOF
    exit 3
fi
echo "  ✅ $INSTALLED_VERSION"

# ─── Step 4 · 检查 BIN_DIR 是否在 PATH ───
echo ""
echo "═══ Step 4 · PATH 检查 ═══"
if [[ ":$PATH:" == *":$BIN_DIR:"* ]]; then
    echo "  ✅ $BIN_DIR 已在 PATH"
    if [[ "$(command -v arkui-rag 2>/dev/null)" == "$DST_BINARY" ]]; then
        echo "  ✅ which arkui-rag → $DST_BINARY（解析对了）"
    else
        echo "  ⚠️  which arkui-rag → $(command -v arkui-rag 2>/dev/null || echo '(找不到)')"
        echo "     可能 PATH 里有别的 arkui-rag 优先 · MCP 配的是绝对路径 · 不影响"
    fi
else
    echo "  ⚠️  $BIN_DIR 不在 PATH"
    echo "     MCP 配置用绝对路径 · 不依赖 PATH · 仍能用"
    echo "     想 shell 直接跑 arkui-rag：echo 'export PATH=\"$BIN_DIR:\$PATH\"' >> ~/.zshrc"
fi

if [[ "$SKIP_MCP" == "1" ]]; then
    echo ""
    echo "⏭  跳过 MCP 配置（--skip-mcp）"
    exit 0
fi

# ─── Step 5 · 检查索引存在 ───
echo ""
echo "═══ Step 5 · 索引检查 ═══"
if [[ ! -f "$INDEX_PATH" ]]; then
    cat <<EOF
  ⚠️  索引不存在: $INDEX_PATH
     MCP 配好但 server 启动会失败。先建索引：
         $DST_BINARY index --source <你的 corpus> --index-path $INDEX_PATH --bm25 tantivy
     或自定义路径：ARKUI_INDEX_PATH=... bash scripts/install-binary.sh
EOF
else
    echo "  ✅ 索引: $INDEX_PATH"
fi

# ─── Step 6 · 配 Claude Code CLI MCP ───
echo ""
echo "═══ Step 6 · Claude Code CLI MCP ═══"
if command -v claude >/dev/null 2>&1; then
    # 删旧（如有）· 重 add · 用绝对路径
    claude mcp remove arkui-rag --scope user 2>/dev/null || true
    claude mcp add --scope user arkui-rag "$DST_BINARY" \
        -- serve --mcp \
           --index-path "$INDEX_PATH" \
           --bm25 tantivy
    echo "  验证:"
    if claude mcp list 2>&1 | grep -q "^arkui-rag:.*✓ Connected"; then
        echo "  ✅ arkui-rag ✓ Connected"
    elif claude mcp list 2>&1 | grep -q "^arkui-rag:"; then
        echo "  ⚠️  已配置但未 Connected · 检查："
        claude mcp list 2>&1 | grep '^arkui-rag:' | sed 's/^/         /'
    else
        echo "  ❌ claude mcp list 未列出 arkui-rag"
    fi
else
    echo "  ⏭  claude CLI 未装 · 跳过 Claude Code 配置"
fi

# ─── Step 7 · 配 Claude Desktop GUI MCP ───
echo ""
echo "═══ Step 7 · Claude Desktop GUI MCP ═══"
DESKTOP_CONFIG="$HOME/Library/Application Support/Claude/claude_desktop_config.json"
if [[ -f "$DESKTOP_CONFIG" ]]; then
    # 备份 + 合并
    BACKUP="$DESKTOP_CONFIG.bak.$(date +%Y%m%d-%H%M%S)"
    cp "$DESKTOP_CONFIG" "$BACKUP"

    python3 - "$DESKTOP_CONFIG" "$DST_BINARY" "$INDEX_PATH" <<'PY'
import json, sys
config_path, binary, index_path = sys.argv[1], sys.argv[2], sys.argv[3]
cfg = json.load(open(config_path))
cfg.setdefault('mcpServers', {})['arkui-rag'] = {
    'command': binary,
    'args': ['serve', '--mcp', '--index-path', index_path, '--bm25', 'tantivy'],
}
json.dump(cfg, open(config_path, 'w'), indent=2, ensure_ascii=False)
print(f"  ✅ 合并到 {config_path}")
print(f"     command: {binary}")
PY
    echo "  备份: $BACKUP"
elif [[ -d "$HOME/Library/Application Support/Claude" ]]; then
    # 目录在 · 文件不在 · 新建
    mkdir -p "$(dirname "$DESKTOP_CONFIG")"
    cat > "$DESKTOP_CONFIG" <<EOF
{
  "mcpServers": {
    "arkui-rag": {
      "command": "$DST_BINARY",
      "args": ["serve", "--mcp", "--index-path", "$INDEX_PATH", "--bm25", "tantivy"]
    }
  }
}
EOF
    echo "  ✅ 新建 $DESKTOP_CONFIG"
else
    echo "  ⏭  Claude Desktop 未装 · 跳过 GUI 配置"
fi

# ─── Step 8 · 配 opencode MCP（如果装了）───
echo ""
echo "═══ Step 8 · opencode MCP ═══"
OPENCODE_CONFIG="$HOME/.config/opencode/opencode.json"
if command -v opencode >/dev/null 2>&1; then
    if [[ -f "$OPENCODE_CONFIG" ]]; then
        BACKUP="$OPENCODE_CONFIG.bak.$(date +%Y%m%d-%H%M%S)"
        cp "$OPENCODE_CONFIG" "$BACKUP"

        python3 - "$OPENCODE_CONFIG" "$DST_BINARY" "$INDEX_PATH" <<'PY'
import json, sys
config_path, binary, index_path = sys.argv[1], sys.argv[2], sys.argv[3]
cfg = json.load(open(config_path))
cfg.setdefault('mcp', {})['arkui-rag'] = {
    'type': 'local',
    'command': [binary, 'serve', '--mcp', '--index-path', index_path, '--bm25', 'tantivy'],
    'enabled': True,
}
json.dump(cfg, open(config_path, 'w'), indent=2, ensure_ascii=False)
print(f"  ✅ 合并到 {config_path}")
PY
        echo "  备份: $BACKUP"
        echo "  验证（opencode mcp list）:"
        if opencode mcp list 2>&1 | grep -q "✓ arkui-rag"; then
            echo "  ✅ arkui-rag connected"
        else
            opencode mcp list 2>&1 | sed 's/^/         /'
        fi
    elif [[ -d "$HOME/.config/opencode" ]]; then
        # 目录在 · 文件不在 · 新建
        cat > "$OPENCODE_CONFIG" <<EOF
{
  "\$schema": "https://opencode.ai/config.json",
  "mcp": {
    "arkui-rag": {
      "type": "local",
      "command": ["$DST_BINARY", "serve", "--mcp", "--index-path", "$INDEX_PATH", "--bm25", "tantivy"],
      "enabled": true
    }
  }
}
EOF
        echo "  ✅ 新建 $OPENCODE_CONFIG"
    else
        echo "  ⏭  opencode config 目录不存在 · 跳过（用户首次启动 opencode 后会自动创建）"
    fi
else
    echo "  ⏭  opencode 未装 · 跳过"
fi

# ─── 完成 ───
cat <<EOF

🎉 安装完成

下一步：
  1. 重启 Claude Code CLI:
       退出当前 claude 进程（Ctrl-D / /exit）· 重新跑 \`claude\`
  2. 重启 Claude Desktop（如果用 GUI）:
       pkill -i "Claude" && sleep 2 && open -a Claude
  3. 重启 opencode（如果用）:
       退当前 opencode tui · 重跑 \`opencode\`
  4. 新 chat 测试：
       「用 arkui_search_docs 检索 @State 双向绑定，top_k=3」
EOF
