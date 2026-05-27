# STATUS · Bootstrap · 每轮 STATUS 硬性规则

> 日期：2026-05-27
> 对应 commit：[本 commit（含 AGENTS.md #17 立规则 + 校验 + commit.sh 标记）]
> 对应 feature log：[`feedback/features/rag4arkui-core/8-2026-05-27-bootstrap-status-rule.md`](../feedback/features/rag4arkui-core/8-2026-05-27-bootstrap-status-rule.md)
> 上一阶段：[`STATUS-day4-bm25-tantivy.md`](STATUS-day4-bm25-tantivy.md)（Day 4 BM25 实装；规则未立时的最后一份产物）
> 下一阶段：[`STATUS-day5-reranker.md`](STATUS-day5-reranker.md)（计划：Day 5 Reranker 主线，首份按新规则走的 STATUS）

> 📌 本文件是**规则化的自我演示**：立规则的 commit 本身也按新规则走完整 STATUS，验证规则可用。

---

## 当前状态

立 AGENTS.md 规则 #17 为 FAIL 级硬性规则：**agent 通过 `scripts/commit.sh` 提交时，staged 中新增 feature log 必须配套同 slug 的 `docs/STATUS-*.md`**。

### 交付清单

| 模块 | 变化 |
|---|---|
| `AGENTS.md` | 新增规则 #17 完整定义（FAIL 级 · 6 节模板 · 命名约定 · 手工豁免） |
| `CLAUDE.md` | 速查表新增 `M-STATUS-PER-ROUND` 行；规则总数 19 → 20 |
| `scripts/commit.sh` | 加 `touch .git/hooks/.agent-pending` + `trap EXIT rm` 清理 |
| `scripts/check-consistency.sh` | 加 `M-STATUS-PER-ROUND` 规则（46 行新增代码） |
| `docs/STATUS-day4-bm25-tantivy.md` | 追溯 Round 7 STATUS（规则未立时缺位的补回） |
| `docs/STATUS-bootstrap-status-rule.md` | 本文件 · 演示规则生效 |
| `feedback/meta/5-*.md` + `feedback/features/rag4arkui-core/8-*.md` | 双轨归档 |

### 关键工程逻辑

```
agent 调 scripts/commit.sh
  → touch .git/hooks/.agent-pending（trap EXIT 清理）
  → git commit
    → pre-commit hook → check-consistency.sh
      → 检测 .agent-pending 存在
      → 扫 staged 新增 feature log
      → 校验对应 STATUS 是否存在（已在仓库 OR 本 staged 新增）
      → 缺则 FAIL
  → commit 完成 / 失败 → trap 清理 .agent-pending

用户直接 git commit（不走 commit.sh）
  → 无 .agent-pending
  → check-consistency.sh 跳过 M-STATUS-PER-ROUND
  → 不受规则约束（手工豁免）
```

---

## 输入契约

### Agent 输入（强制）

每次 agent 准备 commit 含新增 feature log 时，**必须**：

1. 提取 feature log 的 slug（去掉 `N-YYYY-MM-DD-` 前缀和 `.md` 后缀）
2. 创建 `docs/STATUS-<slug>.md` 文件（与 feature log 同 commit staged）
3. STATUS 文档必含 6 个 H2 节（具体见 AGENTS.md #17）

### 用户输入（豁免）

用户直接 `git commit` 不走 `scripts/commit.sh` → 规则不触发。
适用场景：紧急修复 / pure 文档微调 / 用户自己改的代码。

### 配置层

- `AGENTS.md` 规则 #17 是单一事实源
- `CLAUDE.md` 速查表反映规则总数与简表
- `scripts/check-consistency.sh` 实施机械化校验
- `scripts/commit.sh` 提供 agent vs 手工区分机制

---

## 输出契约

### 强制产物

每轮 agent 提交：
- 1 份 feature log（`feedback/features/<name>/N-DATE-<slug>.md`）
- 1 份 STATUS（`docs/STATUS-<slug>.md`，6 节齐）

### 校验输出

pre-commit hook 在 `check-consistency.sh` 输出：

**通过场景**：
```
[PASS] M-STATUS-PER-ROUND 所有新增 feature log 都配套了 STATUS 文档
```

**失败场景**：
```
[FAIL] M-STATUS-PER-ROUND 以下 STATUS 文档缺失（AGENTS.md 规则 #17，FAIL级）：
     - docs/STATUS-day5-reranker.md  ← 缺，对应 feedback/features/rag4arkui-core/9-2026-05-28-day5-reranker.md
  生成模板：touch docs/STATUS-<slug>.md 后填 6 节（见 AGENTS.md 规则 #17）
```

### STATUS 文档结构（6 节固化）

| H2 节 | 内容 |
|---|---|
| `## 当前状态` | 本阶段交付清单 + 关键变化 |
| `## 输入契约` | 新增/变化的 CLI 参数、API、文件格式 |
| `## 输出契约` | 新增/变化的产物 |
| `## 验证手段` | 用户手动 + 自动化 两个子节 |
| `## 与上一阶段的关联性` | 增量、兼容、破坏性 |
| `## 完成度 / 下一阶段` | 进度 + 下一步建议 |

---

## 验证手段

### 用户手动

```bash
# 1. 本 commit 自身：触发 M-STATUS-PER-ROUND，预期 PASS（Round 8 + STATUS 配套）
bash scripts/commit.sh -m "..."

# 2. 验证规则真能拦：模拟"忘写 STATUS"
bash scripts/new-feature-log.sh some-feature test-without-status
# 不写对应 STATUS-test-without-status.md
git add feedback/features/some-feature/N-*.md
bash scripts/commit.sh -m "test"
# 期望：pre-commit FAIL，打印 'M-STATUS-PER-ROUND 缺失 docs/STATUS-test-without-status.md'

# 3. 验证手工豁免：用户直接 git commit
git commit -m "manual change"
# 期望：M-STATUS-PER-ROUND 静默跳过（.agent-pending 不存在）
```

### 自动化

| 手段 | 范围 | 触发 |
|---|---|---|
| `M-STATUS-PER-ROUND` 规则 | staged 新增 feature log → 必有 STATUS 配套 | 每次 `scripts/commit.sh` 调用的 pre-commit |
| `.agent-pending` 标记机制 | 文件存在 = agent commit；不存在 = 手工 commit | `commit.sh` touch + trap EXIT 清理 |
| AGENTS.md #17 文档 | 规则正文 + 6 节模板 + 例外说明 | 人类 + agent 读 |
| CLAUDE.md 速查表 | 一行 reference + 链接到 AGENTS.md | 速查 |

---

## 与上一阶段（STATUS-day4-bm25-tantivy）的关联性

### 增量

| 维度 | Day 4 | Bootstrap |
|---|---|---|
| 架构快照频率 | 偶发（仅 STATUS-day2） | **每个 agent commit round 强制** |
| 缺位检测 | 无 | pre-commit FAIL 拦截 |
| 规则总数 | 19 | **20** |
| Agent vs 手工识别 | 无 | `.agent-pending` 文件标记 |
| feature log 强制 | #10/#16 已立 | + #17（与之协同） |

### 与既有规则的协同

- 规则 #10：业务变更必须有 feature log
- 规则 #16：feature log 必含 Plan + 对话摘要
- **规则 #17（新）**：feature log 必有 STATUS 配套（架构快照）

三者构成 "为什么这样做（#16）+ 改了什么（#10）+ 现在是什么状态（#17）" 完整闭环。

### 破坏性变更（仅 agent）

| 影响 | 旧 agent 行为 | 新 agent 行为 |
|---|---|---|
| 提交含新增 feature log | 只要 feature log 合规即可 | **必须**同 commit 写 STATUS-<slug>.md，否则 FAIL |
| 提交不含 feature log（如 pure meta） | 无影响 | 无影响 |

用户行为完全不变（手工 commit 豁免）。

---

## 完成度 / 下一阶段

### Bootstrap 完成度

| 项 | 状态 |
|---|---|
| AGENTS.md 规则 #17 | ✅ |
| CLAUDE.md 速查 | ✅ |
| scripts/commit.sh 标记机制 | ✅ |
| check-consistency.sh M-STATUS-PER-ROUND | ✅ |
| STATUS 文档命名约定 | ✅ slug 严格 |
| 6 节内容结构 | ✅ 模板固化 |
| 文件存在性校验 | ✅ |
| 6 节内容存在性校验 | ⏳ 下一轮（仅校验文件存在不够，要校 H2） |
| STATUS-INDEX.md 时间线索引 | ⏳ 下一轮 |

**结论**：规则化机制 100% 就位，可推 Day 5 主线。

### 紧接的下一阶段：Day 5 Reranker

**Commit B** 紧跟本 commit，按新规则走完整 STATUS：
- `feedback/features/rag4arkui-core/9-2026-05-28-day5-reranker.md`（feature log）
- `docs/STATUS-day5-reranker.md`（STATUS）

这将是首份"规则立后" 的 STATUS，正常生效。

### 后续规则演进建议

| 时机 | 加什么 |
|---|---|
| Commit B 后 | 6 节内容存在性校验（grep `^## 当前状态` 等） |
| 跨 5 轮后 | `docs/STATUS-INDEX.md` 时间线索引（手动 OR 自动生成） |
| Week 2 末 | STATUS 模板 generator 脚本 `scripts/new-status-doc.sh <slug>` |
| Week 3 | STATUS 内容质量校验（如要求"完成度"节必有量化指标） |
