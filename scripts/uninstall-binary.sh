#!/usr/bin/env bash
# uninstall-binary.sh — 反向 install-binary.sh · 删 binary + 移除三端 MCP 配置
#
# 用法：
#   bash scripts/uninstall-binary.sh                # dry-run · 只显示要做什么 · 不真删
#   bash scripts/uninstall-binary.sh --yes          # 真执行
#   bash scripts/uninstall-binary.sh --bin-dir DIR  # 自定义 binary 目录
#
# 设计要点：
#   - 默认 dry-run · 防误操作（用户必须显式 --yes）
#   - binary 用 mv 到 .uninstalled.* 时间戳 · 不 rm · 可恢复
#   - 配置文件备份后 Python del · 保留其它 mcpServers / mcp 节
#   - **不动索引** `~/.arkui-rag/` · 用户决定是否清理（数据可能很重要）
#
# 副作用（--yes 时）：
#   - mv ~/.local/bin/arkui-rag → arkui-rag.uninstalled.YYYYMMDD-HHMMSS
#   - claude mcp remove arkui-rag --scope user
#   - 改 ~/Library/Application Support/Claude/claude_desktop_config.json（del mcpServers.arkui-rag）
#   - 改 ~/.config/opencode/opencode.json（del mcp.arkui-rag）

set -euo pipefail

# ─── 参数解析 ───
DRY_RUN=1
BIN_DIR="${HOME}/.local/bin"
while [[ $# -gt 0 ]]; do
    case "$1" in
        --yes) DRY_RUN=0; shift ;;
        --bin-dir) BIN_DIR="$2"; shift 2 ;;
        -h|--help)
            sed -n '2,22p' "$0" | sed 's/^# \{0,1\}//'
            exit 0 ;;
        *) echo "未知参数: $1" >&2; exit 2 ;;
    esac
done

DST_BINARY="$BIN_DIR/arkui-rag"
DESKTOP_CONFIG="$HOME/Library/Application Support/Claude/claude_desktop_config.json"
OPENCODE_CONFIG="$HOME/.config/opencode/opencode.json"
INDEX_DIR="$HOME/.arkui-rag"
TS="$(date +%Y%m%d-%H%M%S)"

if [[ "$DRY_RUN" == "1" ]]; then
    cat <<EOF
⚠️  DRY RUN — 只显示要做什么 · 不真删
    真执行：bash scripts/uninstall-binary.sh --yes

EOF
fi

# ─── Step 1 · 移除 binary ───
echo "═══ Step 1 · binary（$DST_BINARY）═══"
if [[ -f "$DST_BINARY" ]]; then
    if [[ "$DRY_RUN" == "1" ]]; then
        echo "  [dry-run] mv $DST_BINARY → $DST_BINARY.uninstalled.$TS"
    else
        mv "$DST_BINARY" "$DST_BINARY.uninstalled.$TS"
        echo "  ✅ binary 重命名 → arkui-rag.uninstalled.$TS（可 mv 回恢复）"
    fi
else
    echo "  ⏭  binary 不存在 · 跳过"
fi

# ─── Step 2 · Claude Code CLI MCP ───
echo ""
echo "═══ Step 2 · Claude Code CLI MCP ═══"
if command -v claude >/dev/null 2>&1; then
    if claude mcp list 2>&1 | grep -q "^arkui-rag:"; then
        if [[ "$DRY_RUN" == "1" ]]; then
            echo "  [dry-run] claude mcp remove arkui-rag --scope user"
        else
            claude mcp remove arkui-rag --scope user 2>&1 | head -3
            echo "  ✅ 移除"
        fi
    else
        echo "  ⏭  claude mcp list 中未找到 arkui-rag · 跳过"
    fi
else
    echo "  ⏭  claude CLI 未装 · 跳过"
fi

# ─── Step 3 · Claude Desktop GUI MCP ───
echo ""
echo "═══ Step 3 · Claude Desktop GUI MCP ═══"
if [[ -f "$DESKTOP_CONFIG" ]]; then
    HAS_ARKUI="$(python3 -c "
import json, sys
cfg = json.load(open('$DESKTOP_CONFIG'))
print('yes' if cfg.get('mcpServers', {}).get('arkui-rag') else 'no')
")"
    if [[ "$HAS_ARKUI" == "yes" ]]; then
        if [[ "$DRY_RUN" == "1" ]]; then
            echo "  [dry-run] del mcpServers.arkui-rag from $DESKTOP_CONFIG"
        else
            cp "$DESKTOP_CONFIG" "$DESKTOP_CONFIG.bak.$TS"
            python3 - "$DESKTOP_CONFIG" <<'PY'
import json, sys
p = sys.argv[1]
cfg = json.load(open(p))
cfg.get('mcpServers', {}).pop('arkui-rag', None)
# 如果 mcpServers 节空了 · 也删（保持 JSON 干净）
if 'mcpServers' in cfg and not cfg['mcpServers']:
    del cfg['mcpServers']
json.dump(cfg, open(p, 'w'), indent=2, ensure_ascii=False)
print(f"  ✅ 移除 mcpServers.arkui-rag from {p}")
PY
            echo "  备份: $DESKTOP_CONFIG.bak.$TS"
        fi
    else
        echo "  ⏭  $DESKTOP_CONFIG 中未含 arkui-rag · 跳过"
    fi
else
    echo "  ⏭  Claude Desktop config 不存在 · 跳过"
fi

# ─── Step 4 · opencode MCP ───
echo ""
echo "═══ Step 4 · opencode MCP ═══"
if [[ -f "$OPENCODE_CONFIG" ]]; then
    HAS_ARKUI="$(python3 -c "
import json
cfg = json.load(open('$OPENCODE_CONFIG'))
print('yes' if cfg.get('mcp', {}).get('arkui-rag') else 'no')
")"
    if [[ "$HAS_ARKUI" == "yes" ]]; then
        if [[ "$DRY_RUN" == "1" ]]; then
            echo "  [dry-run] del mcp.arkui-rag from $OPENCODE_CONFIG"
        else
            cp "$OPENCODE_CONFIG" "$OPENCODE_CONFIG.bak.$TS"
            python3 - "$OPENCODE_CONFIG" <<'PY'
import json, sys
p = sys.argv[1]
cfg = json.load(open(p))
cfg.get('mcp', {}).pop('arkui-rag', None)
if 'mcp' in cfg and not cfg['mcp']:
    del cfg['mcp']
json.dump(cfg, open(p, 'w'), indent=2, ensure_ascii=False)
print(f"  ✅ 移除 mcp.arkui-rag from {p}")
PY
            echo "  备份: $OPENCODE_CONFIG.bak.$TS"
        fi
    else
        echo "  ⏭  $OPENCODE_CONFIG 中未含 arkui-rag · 跳过"
    fi
else
    echo "  ⏭  opencode config 不存在 · 跳过"
fi

# ─── Step 5 · 索引保留 ───
echo ""
echo "═══ Step 5 · 索引（保留 · 不动）═══"
if [[ -d "$INDEX_DIR" ]]; then
    SIZE="$(du -sh "$INDEX_DIR" 2>/dev/null | cut -f1)"
    echo "  📁 索引保留: $INDEX_DIR ($SIZE)"
    echo "  想彻底清理 · 手动跑: rm -rf $INDEX_DIR"
else
    echo "  ⏭  索引目录不存在"
fi

# ─── 完成 ───
echo ""
if [[ "$DRY_RUN" == "1" ]]; then
    cat <<EOF
🔍 DRY RUN 完成
   真执行: bash scripts/uninstall-binary.sh --yes
   或:     make uninstall
EOF
else
    cat <<EOF
🎉 卸载完成

下一步（可选）：
  - 重启 Claude Code CLI 让配置生效（退当前 · 重跑 claude）
  - 重启 Claude Desktop（pkill -i Claude && open -a Claude）
  - 重启 opencode（退当前 tui · 重跑 opencode）
  - 清索引: rm -rf $INDEX_DIR
  - 恢复 binary: mv $DST_BINARY.uninstalled.$TS $DST_BINARY
EOF
fi
