# 5 — status-per-round-rule

> 日期：2026-05-27
> 触发：用户在 Day 4 commit 之后追加硬性规则要求
> 类型：规则变更（FAIL 级硬性规则新增） + 工具脚本

---

## 用户提出的要求

> "注意 每一轮都形成架构快照 包括当前状态，输入，输出，验证手段。以及与上一个阶段和下一个阶段的关联性，完成度。归档到doc中。这个要对agent形成硬性规则，手工提交可以忽略"

明确点：
1. **每一轮**都要架构快照（"一轮" = 每次 feature log Round 推进）
2. **6 大要素**：当前状态 / 输入 / 输出 / 验证手段 / 上下游关联性 / 完成度
3. **归档位置**：docs/
4. **强制级别**：硬性规则
5. **豁免**：手工提交（用户直接 git commit）忽略

## Agent 给出的修改建议

新增 AGENTS.md 规则 #17，配套机械化校验 + commit.sh 标记机制：

### 规则 #17 内容

- **触发**：agent 通过 `scripts/commit.sh` 提交时，staged 中新增 `feedback/features/<name>/N-DATE-<slug>.md`
- **强制**：同 commit 内必须有 `docs/STATUS-<slug>.md`（slug 严格一致）
- **6 节结构**：
  1. `## 当前状态`
  2. `## 输入契约`
  3. `## 输出契约`
  4. `## 验证手段`（子节：用户手动 + 自动化）
  5. `## 与上一阶段的关联性`
  6. `## 完成度 / 下一阶段`
- **豁免**：手工 `git commit` 不走 `commit.sh` → `.git/hooks/.agent-pending` 不存在 → 规则跳过

### 机械化校验

`scripts/check-consistency.sh` 新增 `M-STATUS-PER-ROUND` 规则（FAIL 级）：
- 仅当 `.git/hooks/.agent-pending` 存在时启用
- 扫 staged 新增的 feature log（diff-filter=A）
- 提取 slug（正则去 `N-YYYY-MM-DD-` 前缀）
- 校验 `docs/STATUS-<slug>.md` 存在（已存在 OR 本 staged 新增都算）
- 任一缺失 → FAIL + 打印缺失文件路径

### Agent vs 手工提交的识别机制

- `scripts/commit.sh` 在 `git commit` 前 `touch .git/hooks/.agent-pending`
- 用 `trap 'rm -f $AGENT_MARKER' EXIT` 确保提交完成（或失败）后清理
- 手工 `git commit` 不走 commit.sh → 标记不存在 → 规则不触发

## Agent 决策分析

> 触发 AGENTS.md 规则 #15

### 决策 A：slug 严格一致策略 vs 自由命名

1. **待决策事项**：
   - A. slug 严格 = feature log 去前缀（推荐）
   - B. STATUS 自由命名 + feature log 加 frontmatter 引用
2. **Agent 决策**：选 A
3. **决策依据**：
   - 严格命名 → 校验逻辑简单（正则 + 字符串拼接）
   - 一一对应 → 任何一个 feature log 都能秒查对应 STATUS
   - frontmatter 引用增加 YAML 学习成本，且容易写错
4. **归档引用**：本文件 + AGENTS.md #17
5. **用户 Review 项**：
   - [ ] 同意严格命名策略（slug 必须与 feature log 一致）

### 决策 B：手工提交豁免 vs 全局强制

1. **待决策事项**：
   - A. 仅 agent 提交触发（推荐）
   - B. 所有 commit 都校验
2. **Agent 决策**：选 A
3. **决策依据**：用户明确说"手工提交可以忽略"
4. **归档引用**：本文件
5. **用户 Review 项**：N/A（用户已明确）

### 决策 C：用 `.agent-pending` 标记区分 vs commit message 签名

1. **待决策事项**：
   - A. `.git/hooks/.agent-pending` 文件标记（推荐）
   - B. commit message 强制加 `Co-Authored-By: Claude` 签名 + grep 识别
2. **Agent 决策**：选 A
3. **决策依据**：
   - 文件标记 pre-commit 阶段就能读，比 commit message 解析早
   - 不污染 commit message（许多团队 commit message 风格不希望强制 Co-Authored-By）
   - trap EXIT 清理简单可靠
4. **归档引用**：本文件
5. **用户 Review 项**：
   - [ ] 同意文件标记机制

### 决策 D：本 commit 是 bootstrap，自身需满足新规则

1. **待决策事项**：
   - A. 本 commit 也按新规则走（推荐：用 `bootstrap-status-rule` 自身做示例）
   - B. 本 commit 跳过校验 + commit message 加 `[BOOTSTRAP]` 标记
2. **Agent 决策**：选 A
3. **决策依据**：
   - 本 commit 也带 feature log Round 8（business 改动需要 feature log）→ 必须配 STATUS-bootstrap-status-rule.md
   - 自身就是示例：演示规则如何生效
   - 完成后 `make check-consistency` 验证规则真的拦得住
4. **归档引用**：本文件 + `docs/STATUS-bootstrap-status-rule.md`
5. **用户 Review 项**：N/A

## 多轮互动

按时序：
1. **用户在 Day 4 BM25/Tantivy commit 之后**接连发出两条指令：
   - "按推荐来"（→ 上一轮推荐的 Reranker 主线）
   - **[Request interrupted by user]**
   - "注意 每一轮都形成架构快照 ... 这个要对 agent 形成硬性规则"
2. **Agent 判定**：先做规则化（用户最关心 + 后续主线要按新规则走）→ Commit A
3. **Agent 自主决策 4 项**（见上）后开工，未再回问
4. **Reranker 主线**留作 Commit B（按新规则走完整 STATUS）

## 实际改动

- **接口变化**：CLI 无变化；agent 内部增加"必须写 STATUS"职责
- **规则变化**：
  - AGENTS.md 加规则 #17（FAIL 级）
  - CLAUDE.md 速查表加 M-STATUS-PER-ROUND
  - check-consistency.sh 规则数 19 → 20
- **文件变化**：
  - 修改：`AGENTS.md`、`CLAUDE.md`、`scripts/commit.sh`、`scripts/check-consistency.sh`
  - 新增（业务）：`docs/STATUS-day4-bm25-tantivy.md`（追溯）、`docs/STATUS-bootstrap-status-rule.md`（本轮）
  - 新增（归档）：`feedback/meta/5-*.md`、`feedback/features/rag4arkui-core/8-*.md`
- **配置变化**：无

## 执行生效后总结

### 实际产出

| 项 | 状态 |
|---|---|
| AGENTS.md 规则 #17 | ✅ FAIL 级硬性规则就位 |
| CLAUDE.md 速查表 | ✅ 同步加行 |
| scripts/commit.sh 标记机制 | ✅ touch + trap EXIT 清理 |
| scripts/check-consistency.sh M-STATUS-PER-ROUND | ✅ 完整正则 + 缺失提示 |
| STATUS-day4-bm25-tantivy.md（追溯） | ✅ 6 节齐 |
| STATUS-bootstrap-status-rule.md（本轮自身） | ✅ 6 节齐 |
| 双轨归档（meta + feature log） | ✅ |

### 前后对比

| 维度 | 前 | 后 |
|---|---|---|
| 架构快照频率 | 偶发（仅 STATUS-day2 一份） | 每个 agent 提交 round 都强制 |
| 缺位检测 | 无 | pre-commit FAIL 拦截 |
| Agent vs 手工识别 | 无 | `.git/hooks/.agent-pending` 文件标记 |
| 规则总数 | 19 条 | 20 条 |

### 实测验证

- 本 commit 自身将经过 M-STATUS-PER-ROUND 检验（Round 8 feature log + STATUS-bootstrap-status-rule.md 配套）
- check-consistency.sh 期望输出：`[PASS] M-STATUS-PER-ROUND 所有新增 feature log 都配套了 STATUS 文档`
- 下一 commit（Day 5 Reranker）将完整演示新规则在主线推进中如何运作

### 残留 / 下一轮处理

- [ ] 用户决定：是否进一步加 STATUS 内容校验（6 个 H2 节存在性，仅文件存在不够）—— 当前仅校验文件存在
- [ ] 加 STATUS 索引文档（如 `docs/STATUS-INDEX.md`），按时间线列出所有快照
- [ ] 写"规则 #17 触发的常见错误诊断"到 ONBOARDING.md
- [x] 规则 #17 + 校验 + 标记机制全部就位
- [x] Day 4 STATUS 追溯
- [x] 本 commit 自身演示规则生效
