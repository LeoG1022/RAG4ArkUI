#!/usr/bin/env bash
# check-consistency.sh — 跨文档元数据一致性校验
#
# 检查 README / CLAUDE.md / 子目录文件结构等多处声明的"数量 / 编号"是否一致，
# 防止文档与现实漂移。
#
# 用法: bash scripts/check-consistency.sh
# 退出码: 0 PASS / 1 FAIL / 2 WARN

set -u

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

FAIL=0
WARN=0

red()    { echo -e "\033[31m[FAIL]\033[0m $*"; }
yellow() { echo -e "\033[33m[WARN]\033[0m $*"; }
green()  { echo -e "\033[32m[PASS]\033[0m $*"; }

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  Consistency Check"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# ─────────────────────────────────────────────
# M-SKILL-01: skill 文件数 == CLAUDE.md "Skill 速查" 表行数
# ─────────────────────────────────────────────
SKILL_FILES=$(find .claude/skills -maxdepth 1 -name "*.md" -not -name "AGENTS.md" -not -name "SKILL_SCHEMA.md" | wc -l | tr -d '[:space:]')
# 提取 CLAUDE.md 中以 | `/<name>` 开头的表格行(skill 表格的标识)
SKILL_TABLE=$(grep -cE "^\| \`/" CLAUDE.md | tr -d '[:space:]')
if [[ "$SKILL_FILES" -ne "$SKILL_TABLE" ]]; then
  red "M-SKILL-01 skill 文件数($SKILL_FILES) 与 CLAUDE.md 速查表行数($SKILL_TABLE) 不一致"
  FAIL=1
else
  green "M-SKILL-01 skill 数量一致:$SKILL_FILES"
fi

# ─────────────────────────────────────────────
# M-MAP-01: mapping-*.md 数量 == CLAUDE.md "按领域拆分的 Mapping" 列表项数
# ─────────────────────────────────────────────
MAP_FILES=$(find .claude/references -maxdepth 1 -name "mapping-*.md" | wc -l | tr -d '[:space:]')
MAP_LIST=$(awk '/按领域拆分的 Mapping/,/固定加载的参考表/' CLAUDE.md | grep -cE "^> - \`\.claude/references/mapping-" | tr -d '[:space:]')
if [[ "$MAP_FILES" -ne "$MAP_LIST" ]]; then
  red "M-MAP-01 mapping 文件数($MAP_FILES) 与 CLAUDE.md 列表项数($MAP_LIST) 不一致"
  FAIL=1
else
  green "M-MAP-01 mapping 数量一致:$MAP_FILES"
fi

# ─────────────────────────────────────────────
# M-RULE-01: check-api-parity.sh 规则数 == CLAUDE.md 校验规则表行数
# ─────────────────────────────────────────────
PARITY_RULES=$(grep -cE "^  # [PRSC]-[A-Z]+-[0-9]+:" scripts/check-api-parity.sh | tr -d '[:space:]')
CLAUDE_RULES=$(grep -cE "^\| [PRSC]-[A-Z]+-[0-9]+" CLAUDE.md | tr -d '[:space:]')
if [[ "$PARITY_RULES" -ne "$CLAUDE_RULES" ]]; then
  yellow "M-RULE-01 parity 脚本规则($PARITY_RULES) 与 CLAUDE.md 表行数($CLAUDE_RULES) 不一致(启发式计数，确认无误后可忽略)"
  WARN=1
else
  green "M-RULE-01 校验规则数量一致:$PARITY_RULES"
fi

# ─────────────────────────────────────────────
# M-FB-01: feedback 编号连续 1..N，不跳号
# ─────────────────────────────────────────────
FB_NUMS=$(find feedback/meta -maxdepth 1 -name "[0-9]*-*.md" | sed -E 's|.*/([0-9]+)-.*|\1|' | sort -n)
if [[ -z "$FB_NUMS" ]]; then
  yellow "M-FB-01 feedback/ 下没有编号文件"
  WARN=1
else
  EXPECTED=1
  GAP=""
  for n in $FB_NUMS; do
    n_int=$((10#$n))
    if [[ "$n_int" -ne "$EXPECTED" ]]; then
      GAP="缺号 $EXPECTED(实际下一个是 $n_int)"
      break
    fi
    EXPECTED=$((EXPECTED+1))
  done
  if [[ -n "$GAP" ]]; then
    red "M-FB-01 feedback 编号不连续:$GAP"
    FAIL=1
  else
    LAST=$((EXPECTED-1))
    green "M-FB-01 feedback 编号连续 1..$LAST"
  fi
fi

# ─────────────────────────────────────────────
# M-AGENTS-01: 所有顶层子目录有 AGENTS.md
# ─────────────────────────────────────────────
MISSING=""
for d in .claude/skills .claude/references benchmarks kmp-workspace arkuix-workspace scripts feedback feedback/features feedback/meta reports stats; do
  if [[ -d "$d" && ! -f "$d/AGENTS.md" ]]; then
    MISSING="$MISSING $d"
  fi
done
if [[ -n "$MISSING" ]]; then
  yellow "M-AGENTS-01 以下目录缺 AGENTS.md:$MISSING"
  WARN=1
else
  green "M-AGENTS-01 所有顶层子目录均有 AGENTS.md"
fi

# ─────────────────────────────────────────────
# M-ROOT-01: 根 AGENTS.md 存在且提到本工程主要子目录
# ─────────────────────────────────────────────
if [[ ! -f AGENTS.md ]]; then
  red "M-ROOT-01 根 AGENTS.md 不存在"
  FAIL=1
else
  UNCOVERED=""
  for d in benchmarks kmp-workspace arkuix-workspace scripts feedback feedback/features feedback/meta reports stats; do
    if ! grep -q "$d" AGENTS.md; then
      UNCOVERED="$UNCOVERED $d"
    fi
  done
  if [[ -n "$UNCOVERED" ]]; then
    yellow "M-ROOT-01 根 AGENTS.md 未提及:$UNCOVERED"
    WARN=1
  else
    green "M-ROOT-01 根 AGENTS.md 覆盖所有主要子目录"
  fi
fi

# ─────────────────────────────────────────────
# M-FB-FORMAT: 每份 feedback/[0-9]*-*.md 必含 5 段必备标题
# 备注:5 段结构由第 4 轮引入，因此 N < 4 的 feedback 视为 legacy 自动跳过
# ─────────────────────────────────────────────
REQUIRED_SECTIONS="用户提出的要求|Agent 给出的修改建议|多轮互动|实际改动|执行生效后总结"
LEGACY_BEFORE=4
FB_BAD=""
for f in $(find feedback/meta -maxdepth 1 -name "[0-9]*-*.md" 2>/dev/null); do
  N=$(basename "$f" | sed -E 's|^([0-9]+)-.*|\1|')
  if [[ "${N:-0}" -lt "$LEGACY_BEFORE" ]]; then
    continue  # legacy 跳过
  fi
  COUNT=$(grep -cE "^## ($REQUIRED_SECTIONS)" "$f" | tr -d '[:space:]')
  if [[ "$COUNT" -ne 5 ]]; then
    FB_BAD="$FB_BAD\n     - $f(实际 $COUNT/5 段)"
  fi
done
if [[ -n "$FB_BAD" ]]; then
  red "M-FB-FORMAT 以下 feedback 缺必备段标题(需 5 段:用户要求/Agent建议/多轮互动/实际改动/总结):"
  echo -e "$FB_BAD"
  FAIL=1
else
  green "M-FB-FORMAT 所有 feedback(N≥$LEGACY_BEFORE)5 段结构完整"
fi

# ─────────────────────────────────────────────
# M-MAP-AP: 每份 mapping-*.md 必含 ## Anti-Patterns 节
# ─────────────────────────────────────────────
MAP_BAD=$(grep -L "^## Anti-Patterns" .claude/references/mapping-*.md 2>/dev/null)
if [[ -n "$MAP_BAD" ]]; then
  red "M-MAP-AP 以下 mapping 缺 ## Anti-Patterns 节:"
  echo "$MAP_BAD" | sed 's/^/     - /'
  FAIL=1
else
  green "M-MAP-AP 所有 mapping 均含 Anti-Patterns 节"
fi

# ─────────────────────────────────────────────
# M-SKILL-PREFLIGHT: skill 必含 AGENTS.md 与 Git 前置协议引用
# ─────────────────────────────────────────────
SKILL_BAD=""
for f in .claude/skills/*.md; do
  base=$(basename "$f")
  [[ "$base" == "AGENTS.md" || "$base" == "SKILL_SCHEMA.md" ]] && continue
  if ! grep -qE "AGENTS\.md" "$f" || ! grep -qE "Git" "$f"; then
    SKILL_BAD="$SKILL_BAD\n     - $f"
  fi
done
if [[ -n "$SKILL_BAD" ]]; then
  red "M-SKILL-PREFLIGHT 以下 skill 缺 AGENTS.md / Git 前置协议引用:"
  echo -e "$SKILL_BAD"
  FAIL=1
else
  green "M-SKILL-PREFLIGHT 所有 skill 均含 AGENTS.md + Git 前置引用"
fi

# ─────────────────────────────────────────────
# M-LINK-DEAD: feedback/*.md 与 *AGENTS.md 中相对路径链接必须存在
# ─────────────────────────────────────────────
DEAD_LINKS=""
for f in $(find feedback/meta -maxdepth 1 -name "*.md" 2>/dev/null) $(find feedback/features -name "*.md" 2>/dev/null) $(find . -name "AGENTS.md" -not -path "*/.git/*" 2>/dev/null); do
  # 抓 [text](relative/path.md) 形式的 markdown 链接，过滤掉 http(s):// 和 #anchor-only
  LINKS=$(grep -oE '\]\([^)#]*\.md[^)]*\)' "$f" 2>/dev/null \
    | sed -E 's/^\]\(//; s/\)$//; s/#.*$//' \
    | grep -vE '^https?://')
  DIR=$(dirname "$f")
  for link in $LINKS; do
    # 跳过空 / 锚点
    [[ -z "$link" ]] && continue
    # 解析相对路径
    if [[ "$link" == /* ]]; then
      TARGET="$link"
    else
      TARGET="$DIR/$link"
    fi
    # 简化路径(处理 ../)
    TARGET=$(cd "$DIR" 2>/dev/null && cd "$(dirname "$link")" 2>/dev/null && pwd)/$(basename "$link")
    if [[ ! -e "$TARGET" ]]; then
      DEAD_LINKS="$DEAD_LINKS\n     - $f → $link"
    fi
  done
done
if [[ -n "$DEAD_LINKS" ]]; then
  red "M-LINK-DEAD 发现死链:"
  echo -e "$DEAD_LINKS"
  FAIL=1
else
  green "M-LINK-DEAD 所有 markdown 相对链接均有效"
fi

# ─────────────────────────────────────────────
# M-SKILL-SUMMARY-CLASSIFY: skill 必含 classify-change.sh 引用
# 目的:强制 skill 在完成后跑 classify 并把醒目块复述给用户
# ─────────────────────────────────────────────
SKILL_NO_CLASSIFY=""
for f in .claude/skills/*.md; do
  base=$(basename "$f")
  [[ "$base" == "AGENTS.md" || "$base" == "SKILL_SCHEMA.md" ]] && continue
  if ! grep -qE "classify-change\.sh" "$f"; then
    SKILL_NO_CLASSIFY="$SKILL_NO_CLASSIFY\n     - $f"
  fi
done
if [[ -n "$SKILL_NO_CLASSIFY" ]]; then
  red "M-SKILL-SUMMARY-CLASSIFY 以下 skill 缺 classify-change.sh 引用(无法触发中段告知):"
  echo -e "$SKILL_NO_CLASSIFY"
  FAIL=1
else
  green "M-SKILL-SUMMARY-CLASSIFY 所有 skill 均引用 classify-change.sh"
fi

# ─────────────────────────────────────────────
# M-FEATURE-NAMING: features/ 目录命名与日志编号规范
# ─────────────────────────────────────────────
NAMING_VIOLATIONS=""
if [[ -d feedback/features ]]; then
  for d in feedback/features/*/; do
    [[ -d "$d" ]] || continue
    NAME=$(basename "$d")
    # 跳过 AGENTS.md 等非目录情况
    [[ "$NAME" == "*" ]] && continue
    # kebab-case 检查
    if [[ ! "$NAME" =~ ^[a-z0-9-]+$ ]]; then
      NAMING_VIOLATIONS="$NAMING_VIOLATIONS\n     - $d 不是 kebab-case"
      continue
    fi
    # 必含 README.md
    if [[ ! -f "$d/README.md" ]]; then
      NAMING_VIOLATIONS="$NAMING_VIOLATIONS\n     - $d 缺 README.md"
    fi
    # 日志编号连续性 1..N
    NUMS=$(find "$d" -maxdepth 1 -name "[0-9]*-*.md" 2>/dev/null \
      | sed -E 's|.*/([0-9]+)-.*|\1|' | sort -n)
    if [[ -n "$NUMS" ]]; then
      EXP=1
      for n in $NUMS; do
        n_int=$((10#$n))
        if [[ "$n_int" -ne "$EXP" ]]; then
          NAMING_VIOLATIONS="$NAMING_VIOLATIONS\n     - $d 日志编号不连续(缺 $EXP)"
          break
        fi
        EXP=$((EXP+1))
      done
    fi
  done
fi
if [[ -n "$NAMING_VIOLATIONS" ]]; then
  yellow "M-FEATURE-NAMING feature 目录/日志命名问题:"
  echo -e "$NAMING_VIOLATIONS"
  WARN=1
else
  green "M-FEATURE-NAMING feedback/features/ 命名与编号规范"
fi

# ─────────────────────────────────────────────
# M-FEATURE-NO-META: features/*/[N]-*.md 不应含元术语
# ─────────────────────────────────────────────
META_TERMS="RULE-[A-Z]|check-api-parity|check-consistency|mapping-[a-z]+\.md|\.claude/skills"
FEATURE_META_HITS=""
if [[ -d feedback/features ]]; then
  for f in $(find feedback/features -maxdepth 2 -name "[0-9]*-*.md" 2>/dev/null); do
    if grep -qE "$META_TERMS" "$f" 2>/dev/null; then
      FEATURE_META_HITS="$FEATURE_META_HITS\n     - $f 含元术语(请检查是否应去 feedback/meta/)"
    fi
  done
fi
if [[ -n "$FEATURE_META_HITS" ]]; then
  yellow "M-FEATURE-NO-META 以下 feature 日志疑似含元术语:"
  echo -e "$FEATURE_META_HITS"
  WARN=1
else
  green "M-FEATURE-NO-META feedback/features 日志无元术语污染"
fi

# ─────────────────────────────────────────────
# M-FEATURE-PLAN: feature log 必须含 ## Plan 与 ## 对话摘要 节(FAIL)
# 规则 #17 生效次日 2026-05-22 起；早于此日期的文件视为 legacy 自动跳过
# ─────────────────────────────────────────────
PLAN_CUTOFF="2026-05-22"
FEATURE_PLAN_BAD=""
if [[ -d feedback/features ]]; then
  for f in $(find feedback/features -maxdepth 2 -name "[0-9]*-*.md" 2>/dev/null); do
    FILE_DATE=$(basename "$f" | grep -oE '[0-9]{4}-[0-9]{2}-[0-9]{2}' | head -1)
    if [[ -z "$FILE_DATE" || "$FILE_DATE" < "$PLAN_CUTOFF" ]]; then
      continue  # legacy 跳过
    fi
    HAS_PLAN=$(grep -cE "^## Plan" "$f" 2>/dev/null | tr -d '[:space:]')
    HAS_CONV=$(grep -cE "^## 对话摘要" "$f" 2>/dev/null | tr -d '[:space:]')
    if [[ "${HAS_PLAN:-0}" -eq 0 ]]; then
      FEATURE_PLAN_BAD="$FEATURE_PLAN_BAD\n     - $f 缺 ## Plan 节"
    fi
    if [[ "${HAS_CONV:-0}" -eq 0 ]]; then
      FEATURE_PLAN_BAD="$FEATURE_PLAN_BAD\n     - $f 缺 ## 对话摘要 节"
    fi
  done
fi
if [[ -n "$FEATURE_PLAN_BAD" ]]; then
  red "M-FEATURE-PLAN 以下 feature log 缺必要节(规则 #17，生效自 $PLAN_CUTOFF):"
  echo -e "$FEATURE_PLAN_BAD"
  FAIL=1
else
  green "M-FEATURE-PLAN 所有 feature log(≥$PLAN_CUTOFF) 含 Plan + 对话摘要节"
fi

# ─────────────────────────────────────────────
# M-SKILL-FEATURE-LOG: skill 必含 new-feature-log 引用(除 frontmatter feature_log_required=false 的)
# 自 Round 10 起:基于 frontmatter 而非 HTML 注释
# ─────────────────────────────────────────────
SKILL_NO_FL=""
for f in .claude/skills/*.md; do
  base=$(basename "$f")
  [[ "$base" == "AGENTS.md" || "$base" == "SKILL_SCHEMA.md" ]] && continue
  # 读 frontmatter feature_log_required 字段
  FLR=$(bash scripts/parse-skill-meta.sh --field feature_log_required "$f" 2>/dev/null)
  if [[ "$FLR" == "false" ]]; then
    continue  # 显式豁免
  fi
  if ! grep -qE "new-feature(-log)?\.sh" "$f"; then
    SKILL_NO_FL="$SKILL_NO_FL\n     - $f"
  fi
done
if [[ -n "$SKILL_NO_FL" ]]; then
  red "M-SKILL-FEATURE-LOG 以下 skill 缺 new-feature(-log).sh 引用(business 改动必须留 feature log):"
  echo -e "$SKILL_NO_FL"
  FAIL=1
else
  green "M-SKILL-FEATURE-LOG 所有非豁免 skill 均引用 feature-log 工具"
fi

# ─────────────────────────────────────────────
# M-SKILL-FRONTMATTER: skill 必含合法 frontmatter(必填字段齐全)
# 自 Round 10 起
# ─────────────────────────────────────────────
SKILL_BAD_FM=""
REQUIRED_FIELDS=(name version trigger description feature_log_required classify_required preflight_required)
for f in .claude/skills/*.md; do
  base=$(basename "$f")
  [[ "$base" == "AGENTS.md" || "$base" == "SKILL_SCHEMA.md" ]] && continue
  MISSING=""
  for field in "${REQUIRED_FIELDS[@]}"; do
    VAL=$(bash scripts/parse-skill-meta.sh --field "$field" "$f" 2>/dev/null)
    if [[ -z "$VAL" ]]; then
      MISSING="$MISSING $field"
    fi
  done
  if [[ -n "$MISSING" ]]; then
    SKILL_BAD_FM="$SKILL_BAD_FM\n     - $f 缺:$MISSING"
    continue
  fi
  # name <-> trigger 一致性
  NAME=$(bash scripts/parse-skill-meta.sh --field name "$f")
  TRIGGER=$(bash scripts/parse-skill-meta.sh --field trigger "$f")
  if [[ "/$NAME" != "$TRIGGER" ]]; then
    SKILL_BAD_FM="$SKILL_BAD_FM\n     - $f:name(/$NAME) 与 trigger($TRIGGER) 不一致"
    continue
  fi
  # version semver 检查
  VER=$(bash scripts/parse-skill-meta.sh --field version "$f")
  if [[ ! "$VER" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    SKILL_BAD_FM="$SKILL_BAD_FM\n     - $f:version($VER) 不是合法 semver"
  fi
done
if [[ -n "$SKILL_BAD_FM" ]]; then
  red "M-SKILL-FRONTMATTER 以下 skill frontmatter 不合法:"
  echo -e "$SKILL_BAD_FM"
  FAIL=1
else
  green "M-SKILL-FRONTMATTER 所有 skill frontmatter 合法"
fi

# ─────────────────────────────────────────────
# M-SKILL-REF-VALID: frontmatter 中 calls / references 声明的路径必须存在
# 自 Round 10 起
# ─────────────────────────────────────────────
SKILL_BAD_REF=""
for f in .claude/skills/*.md; do
  base=$(basename "$f")
  [[ "$base" == "AGENTS.md" || "$base" == "SKILL_SCHEMA.md" ]] && continue
  # calls:路径必须存在 + 可执行
  CALLS=$(bash scripts/parse-skill-meta.sh --field calls "$f" 2>/dev/null)
  if [[ -n "$CALLS" ]]; then
    IFS=',' read -ra CALL_ARR <<< "$CALLS"
    for c in "${CALL_ARR[@]}"; do
      if [[ ! -e "$c" ]]; then
        SKILL_BAD_REF="$SKILL_BAD_REF\n     - $f:calls 中 $c 不存在"
      elif [[ ! -x "$c" ]]; then
        SKILL_BAD_REF="$SKILL_BAD_REF\n     - $f:calls 中 $c 不可执行"
      fi
    done
  fi
  # references:路径必须存在
  REFS=$(bash scripts/parse-skill-meta.sh --field references "$f" 2>/dev/null)
  if [[ -n "$REFS" ]]; then
    IFS=',' read -ra REF_ARR <<< "$REFS"
    for r in "${REF_ARR[@]}"; do
      if [[ ! -e "$r" ]]; then
        SKILL_BAD_REF="$SKILL_BAD_REF\n     - $f:references 中 $r 不存在"
      fi
    done
  fi
done
if [[ -n "$SKILL_BAD_REF" ]]; then
  red "M-SKILL-REF-VALID 以下 skill 的 calls/references 路径无效:"
  echo -e "$SKILL_BAD_REF"
  FAIL=1
else
  green "M-SKILL-REF-VALID 所有 skill 的 calls/references 路径有效"
fi

# ─────────────────────────────────────────────
# M-SKILL-TABLE-SYNC: CLAUDE.md SKILL-TABLE 与 frontmatter 同步
# 自 Round 10 起
# ─────────────────────────────────────────────
if bash scripts/regenerate-skill-table.sh --check >/dev/null 2>&1; then
  green "M-SKILL-TABLE-SYNC CLAUDE.md SKILL-TABLE 与 frontmatter 同步"
else
  red "M-SKILL-TABLE-SYNC CLAUDE.md SKILL-TABLE 与 frontmatter 不同步"
  echo "     - 修复:bash scripts/regenerate-skill-table.sh && git add CLAUDE.md"
  FAIL=1
fi

# ─────────────────────────────────────────────
# M-AGENT-ENTRY: 9 个跨工具 agent 入口文件存在且引用 AGENTS.md(WARN)
# 自 Round 16 起
# ─────────────────────────────────────────────
EXPECTED_ENTRIES=(
  .cursorrules
  .cursor/rules/main.mdc
  .aiderrules
  CONVENTIONS.md
  .clinerules
  .windsurfrules
  .continue/rules.md
  .hermes/rules.md
  RULES_FOR_AGENTS.md
)
ENTRY_MISSING=""
ENTRY_NO_REF=""
for f in "${EXPECTED_ENTRIES[@]}"; do
  if [[ ! -f "$f" ]]; then
    ENTRY_MISSING="$ENTRY_MISSING\n     - $f"
  elif ! grep -q "AGENTS\.md" "$f" 2>/dev/null; then
    ENTRY_NO_REF="$ENTRY_NO_REF\n     - $f"
  fi
done
if [[ -n "$ENTRY_MISSING" || -n "$ENTRY_NO_REF" ]]; then
  yellow "M-AGENT-ENTRY 跨工具入口文件问题:"
  if [[ -n "$ENTRY_MISSING" ]]; then
    echo -e "   缺失文件:$ENTRY_MISSING"
  fi
  if [[ -n "$ENTRY_NO_REF" ]]; then
    echo -e "   存在但未引用 AGENTS.md:$ENTRY_NO_REF"
  fi
  WARN=1
else
  green "M-AGENT-ENTRY 9 个跨工具入口文件均存在且引用 AGENTS.md"
fi

# ─────────────────────────────────────────────
# M-SKILL-FIXTURE-EXISTS: 非豁免 skill 必有 tests/skills/<name>/ fixture(WARN)
# 自 Round 11 起
# ─────────────────────────────────────────────
SKILL_NO_FIX=""
for f in .claude/skills/*.md; do
  base=$(basename "$f")
  [[ "$base" == "AGENTS.md" || "$base" == "SKILL_SCHEMA.md" ]] && continue
  FLR=$(bash scripts/parse-skill-meta.sh --field feature_log_required "$f" 2>/dev/null)
  [[ "$FLR" == "false" ]] && continue  # 豁免(如 run-benchmark)
  NAME=$(bash scripts/parse-skill-meta.sh --field name "$f" 2>/dev/null)
  if [[ -z "$NAME" ]]; then continue; fi
  FIX_DIR="tests/skills/$NAME"
  if [[ ! -d "$FIX_DIR" ]]; then
    SKILL_NO_FIX="$SKILL_NO_FIX\n     - $f → 缺 $FIX_DIR/"
  else
    # 至少一个子目录
    SUBDIR_COUNT=$(find "$FIX_DIR" -maxdepth 1 -mindepth 1 -type d | wc -l | tr -d '[:space:]')
    if [[ "$SUBDIR_COUNT" -eq 0 ]]; then
      SKILL_NO_FIX="$SKILL_NO_FIX\n     - $f → $FIX_DIR/ 为空"
    fi
  fi
done
if [[ -n "$SKILL_NO_FIX" ]]; then
  yellow "M-SKILL-FIXTURE-EXISTS 以下 skill 缺 tier 2 fixture:"
  echo -e "$SKILL_NO_FIX"
  WARN=1
else
  green "M-SKILL-FIXTURE-EXISTS 所有非豁免 skill 均有 tier 2 fixture"
fi

# ─────────────────────────────────────────────
# M-PENDING-01: 最近 5 轮 feedback/meta 与 feedback/features 中 - [ ] 未解决项扫描(WARN)
# ─────────────────────────────────────────────
PENDING_HITS=""
# 扫 feedback/meta 最近 5 份
for f in $(find feedback/meta -maxdepth 1 -name "[0-9]*-*.md" 2>/dev/null | sort -V | tail -5); do
  OPEN=$(grep -cE "^- \[ \]" "$f" 2>/dev/null | tr -d '[:space:]')
  if [[ "${OPEN:-0}" -gt 0 ]]; then
    BASE=$(basename "$f" .md)
    PENDING_HITS="$PENDING_HITS\n     - [meta] $BASE ($OPEN 项未解决)"
  fi
done
# 扫 feedback/features 全部特性最近 1 份日志
if [[ -d feedback/features ]]; then
  for d in feedback/features/*/; do
    [[ -d "$d" ]] || continue
    LATEST=$(find "$d" -maxdepth 1 -name "[0-9]*-*.md" 2>/dev/null | sort -V | tail -1)
    [[ -z "$LATEST" ]] && continue
    OPEN=$(grep -cE "^- \[ \]" "$LATEST" 2>/dev/null | tr -d '[:space:]')
    if [[ "${OPEN:-0}" -gt 0 ]]; then
      BASE=$(basename "$LATEST" .md)
      FEAT=$(basename "$d")
      PENDING_HITS="$PENDING_HITS\n     - [feature/$FEAT] $BASE ($OPEN 项未解决)"
    fi
  done
fi
if [[ -n "$PENDING_HITS" ]]; then
  yellow "M-PENDING-01 存在未解决的 - [ ] 残留项(详见 scripts/query-pending.sh):"
  echo -e "$PENDING_HITS"
  WARN=1
else
  green "M-PENDING-01 近期归档无未解决残留项"
fi

# ─────────────────────────────────────────────
# M-README-PURE: README.md 主体不含过多 agent 专属术语(WARN)
# ─────────────────────────────────────────────
# 阈值:3 次(允许末尾 "我是 AI / Agent 进来的" 段保留少量提及)
THRESHOLD=5
README_AGENT_HITS=$(grep -ciE "(agent|LLM|prompt)" README.md | tr -d '[:space:]')
if [[ "$README_AGENT_HITS" -gt "$THRESHOLD" ]]; then
  yellow "M-README-PURE README 中 agent/LLM/prompt 出现 $README_AGENT_HITS 次(建议 ≤ $THRESHOLD，agent 细节应放 AGENTS.md / CLAUDE.md)"
  WARN=1
else
  green "M-README-PURE README agent 专属术语 $README_AGENT_HITS 次(阈值 ≤ $THRESHOLD)"
fi

# ─────────────────────────────────────────────
# M-NO-VERIFY-BAN: 最近 N 个 commit 是否经过 pre-commit hook 验证(FAIL)
# ─────────────────────────────────────────────
MARKER="$ROOT/.git/hooks/.last-verified"
if [[ -f "$MARKER" ]]; then
  UNVERIFIED=0
  CHECKED=0
  for i in 0 1 2 3 4; do
    # 用 --verify --quiet 让 git 在 ref 不存在时返回非零且不打印字面量；
    # `|| break` 放在赋值外面才能真正跳出 for 循环（在 $() 内 break 仅退出子 shell）。
    HASH=$(git rev-parse --verify --quiet "HEAD~$i" 2>/dev/null) || break
    CHECKED=$((CHECKED + 1))
    if ! grep -q "$HASH" "$MARKER" 2>/dev/null; then
      UNVERIFIED=$((UNVERIFIED + 1))
      MSG=$(git log -1 --format="%s" "$HASH" 2>/dev/null || echo "unknown")
      yellow "M-NO-VERIFY-BAN HEAD~$i ($HASH) 未经过 hook 验证: $MSG"
    fi
  done
  if [[ "$UNVERIFIED" -eq 0 ]]; then
    green "M-NO-VERIFY-BAN 最近 $CHECKED 个 commit 均经过 pre-commit hook 验证"
  else
    FAIL=1
  fi
else
  yellow "M-NO-VERIFY-BAN .last-verified 标记文件不存在（首次clone / 初始环境）"
fi

# ─────────────────────────────────────────────
# M-STATUS-PER-ROUND: agent 每轮 feature log 必须配套 docs/STATUS-<slug>.md(FAIL)
# 仅当 .git/hooks/.agent-pending 存在时启用（即 scripts/commit.sh 发起的提交）；
# 手工 git commit 不触发此校验。
# 规则 #17 详见 AGENTS.md。
# ─────────────────────────────────────────────
AGENT_PENDING="$ROOT/.git/hooks/.agent-pending"
if [[ -f "$AGENT_PENDING" ]]; then
  # 仅扫 staged 新增的 feature log（Added，不含 Modified）
  NEW_FEATURE_LOGS=$(git diff --cached --name-only --diff-filter=A 2>/dev/null \
                     | grep -E '^feedback/features/[^/]+/[0-9]+-[0-9]{4}-[0-9]{2}-[0-9]{2}-[^/]+\.md$' || true)
  if [[ -n "$NEW_FEATURE_LOGS" ]]; then
    MISSING_STATUS=()
    for fl in $NEW_FEATURE_LOGS; do
      # 从 N-YYYY-MM-DD-<slug>.md 提取 slug
      basename=$(basename "$fl" .md)
      slug=$(echo "$basename" | sed -E 's/^[0-9]+-[0-9]{4}-[0-9]{2}-[0-9]{2}-//')
      status_path="docs/STATUS-${slug}.md"
      # STATUS 文件可以在本 staged 新增，或已存在
      if [[ ! -f "$status_path" ]]; then
        if ! git diff --cached --name-only 2>/dev/null | grep -q "^${status_path}$"; then
          MISSING_STATUS+=("$status_path  ← 缺，对应 $fl")
        fi
      fi
    done
    if [[ ${#MISSING_STATUS[@]} -eq 0 ]]; then
      green "M-STATUS-PER-ROUND 所有新增 feature log 都配套了 STATUS 文档"
    else
      red "M-STATUS-PER-ROUND 以下 STATUS 文档缺失（AGENTS.md 规则 #17，FAIL级）："
      for m in "${MISSING_STATUS[@]}"; do
        echo -e "     - $m"
      done
      echo -e "  生成模板：\033[1mtouch docs/STATUS-<slug>.md\033[0m 后填 6 节（见 AGENTS.md 规则 #17）"
      FAIL=1
    fi
  else
    green "M-STATUS-PER-ROUND 本次提交无新增 feature log，跳过检查"
  fi
else
  # 手工 git commit（未走 scripts/commit.sh）→ 规则不生效，静默
  :
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if [[ "$FAIL" -eq 1 ]]; then
  echo -e "\033[31m结果:FAIL — 存在必须修复的不一致\033[0m"
  exit 1
elif [[ "$WARN" -eq 1 ]]; then
  echo -e "\033[33m结果:WARN — 存在建议核对的项\033[0m"
  exit 2
else
  echo -e "\033[32m结果:全部一致\033[0m"
  exit 0
fi
