#!/usr/bin/env bash
# check-corpus-utf8.sh — 扫 corpus/official/ 下所有 .md 看是否有非 UTF-8 编码文件
#
# 设计:
#   - Round 54: Phase B 3.2h build 死于 1 个 GBK 文件 · 之后所有 build 之前先 check
#   - 用 python3 (系统自带 · macOS / Linux 通用)
#   - 输出非 UTF-8 文件列表 + 位置 + 上下文（给人看修哪个文件）
#   - 退出码: 0 = 全 UTF-8 / 1 = 有非 UTF-8 / 2 = 参数错
#
# 用法:
#   bash scripts/check-corpus-utf8.sh                        # 默认扫 corpus/official/
#   bash scripts/check-corpus-utf8.sh --path corpus/official/arkui-x
#   bash scripts/check-corpus-utf8.sh --fix                  # 探测 GBK 编码 · 自动 iconv 转 UTF-8

set -euo pipefail

ROOT="${ROOT:-corpus/official}"
FIX=0
while [[ $# -gt 0 ]]; do
    case "$1" in
        --path) ROOT="$2"; shift 2 ;;
        --fix) FIX=1; shift ;;
        -h|--help) sed -n '2,15p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
        *) echo "未知参数: $1" >&2; exit 2 ;;
    esac
done

[[ -d "$ROOT" ]] || { echo "❌ 目录不存在: $ROOT" >&2; exit 2; }

echo "═══ 扫 $ROOT 下所有 .md ═══"
echo ""

# python 扫
BAD=$(python3 - "$ROOT" << 'PY'
import os, sys
root = sys.argv[1]
bad = []
for r, _, files in os.walk(root):
    for f in files:
        if not f.endswith('.md'):
            continue
        p = os.path.join(r, f)
        with open(p, 'rb') as fh:
            data = fh.read()
        try:
            data.decode('utf-8')
        except UnicodeDecodeError as e:
            ctx_before = data[max(0, e.start-15):e.start].decode('utf-8', errors='replace')[-10:]
            ctx_after = data[e.end:e.end+10].decode('utf-8', errors='replace')
            bad.append((p, e.start, data[max(0,e.start-3):e.end+3].hex(), f'...{ctx_before}[BAD]{ctx_after}...'))
for p, pos, h, ctx in bad:
    print(f"{p}\t{pos}\t{h}\t{ctx}")
PY
)

if [[ -z "$BAD" ]]; then
    echo "✅ 全部 UTF-8 · 无需修复"
    exit 0
fi

CNT=$(echo "$BAD" | wc -l | tr -d ' ')
echo "❌ $CNT 个非 UTF-8 文件："
echo ""
echo "$BAD" | while IFS=$'\t' read -r path pos hex ctx; do
    echo "  • $path"
    echo "    pos $pos · hex $hex"
    echo "    ctx $ctx"
done
echo ""

if [[ "$FIX" == "1" ]]; then
    echo "═══ --fix · 试 GBK iconv 转 UTF-8 ═══"
    echo "$BAD" | while IFS=$'\t' read -r path _ _ _; do
        if iconv -f GBK -t UTF-8 "$path" > "${path}.utf8" 2>/dev/null && [[ -s "${path}.utf8" ]]; then
            mv "${path}.utf8" "$path"
            echo "  ✓ $path (GBK→UTF-8)"
        elif iconv -f GB18030 -t UTF-8 "$path" > "${path}.utf8" 2>/dev/null && [[ -s "${path}.utf8" ]]; then
            mv "${path}.utf8" "$path"
            echo "  ✓ $path (GB18030→UTF-8)"
        else
            rm -f "${path}.utf8"
            echo "  ✗ $path (无法识别编码 · 需要人工干预)"
        fi
    done
    echo ""
    echo "复查："
    bash "$0" --path "$ROOT"
else
    echo "提示：跑 \`bash scripts/check-corpus-utf8.sh --fix\` 自动 iconv 转码（试 GBK / GB18030）"
    exit 1
fi
