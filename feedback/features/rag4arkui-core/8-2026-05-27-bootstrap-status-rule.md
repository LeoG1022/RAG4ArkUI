# 8 — bootstrap-status-rule

> 日期：2026-05-27
> 涉及代码：
> - `AGENTS.md`、`CLAUDE.md`、`scripts/commit.sh`、`scripts/check-consistency.sh`（meta，详 `feedback/meta/5-*.md`）
> - `docs/STATUS-day4-bm25-tantivy.md`（追溯 Round 7 的 STATUS）
> - `docs/STATUS-bootstrap-status-rule.md`（本 Round 8 自身的 STATUS）
> 类型：新建（规则化 bootstrap）

## 本轮目标

把"每轮 agent 提交都要写架构快照 STATUS"立为 FAIL 级硬性规则（AGENTS.md #17）。

业务侧产物（在 `docs/`）：
1. **追溯**：补 Day 4 BM25/Tantivy 的 STATUS（之前规则未立，缺位）
2. **本轮**：写本规则化自身的 STATUS（也是规则示例）

## Plan

> 必备节（FAIL 级 M-FEATURE-PLAN）

### 规则化整体设计（meta 详 feedback/meta/5-*.md）

本 feature log 只覆盖业务侧（docs/STATUS-*.md 两份）。meta 侧（AGENTS.md/CLAUDE.md/scripts）由 `feedback/meta/5-2026-05-27-status-per-round-rule.md` 归档。

### 业务侧 STATUS 文档结构（统一 6 节）

| H2 节 | 内容 |
|---|---|
| `## 当前状态` | 本阶段交付清单 + 关键变化 |
| `## 输入契约` | 新增/变化的 CLI 参数、API、文件格式、frontmatter |
| `## 输出契约` | 新增/变化的产物（文件、命令输出、JSON schema） |
| `## 验证手段` | 子节：用户手动（命令行步骤）+ 自动化（测试 / hook / smoke / CI） |
| `## 与上一阶段的关联性` | 增量、兼容性、4 状态矩阵之类 |
| `## 完成度 / 下一阶段` | 对照 6 周路线图的进度评估 + 候选建议 |

### Day 4 STATUS（追溯）关键内容

复盘已 commit 的 Round 7 改动：
- 当前状态：TantivyBM25Index + 7 单测 + CLI --bm25
- 输入：`--bm25 memory|tantivy`、`--features tantivy`
- 输出：`<index-path-dir>/bm25/` 目录 + query 输出多 `bm25=X` 行
- 验证：12 个 Tantivy 单测 + ngram 中文分词
- 关联：兼容 Day 2/3 默认行为；HybridRetriever 终于"hybrid"
- 完成度：BM25 真活 100%；ngram 精度有限，jieba 备选

### 本轮 STATUS（自身）关键内容

见 `docs/STATUS-bootstrap-status-rule.md` 详细 6 节。要点：
- 当前状态：FAIL 级规则 + 校验 + commit.sh 标记机制就位
- 输入：agent 写 feature log 时自觉同 commit 内写 STATUS
- 输出：规则 #17 + M-STATUS-PER-ROUND + .agent-pending 文件标记
- 验证：本 commit 自身经过规则验证；后续 Day 5 主线再次演示
- 关联：建立在已有 AGENTS.md 16 条规则之上，与 #10/#16（feature log 强制）协同
- 完成度：规则化机制 100%；STATUS 内容深度校验留下一轮

### 替代方案权衡

- 备选 commit message 签名识别 agent：被否，污染 message 风格
- 备选 commit hash 后置标记（git note）：被否，pre-commit 阶段还没 hash
- 备选规则只校验文件存在不校验内容：暂选，下一轮加 6 节存在性校验
- 备选 STATUS 命名自由 + frontmatter 引用：被否，严格命名更简单可机械

## 改动要点

> API 选型 / 算法 / 关键决策

业务侧（本 feature log 范围）：
- **两份 STATUS 同时入库**：追溯 + 自身演示
- **6 节模板固化**：所有未来 STATUS 都按此结构
- **slug 一致性**：feature log `8-2026-05-27-bootstrap-status-rule.md` ↔ STATUS `docs/STATUS-bootstrap-status-rule.md`

meta 侧由 `feedback/meta/5-*.md` 归档不重复。

## 对话摘要

> 必备节（FAIL 级 M-FEATURE-PLAN）

按时序：

1. **用户在 Day 4 commit 后**先说"按推荐来"（Day 5 Reranker），随后 **[Request interrupted by user]**，重新指令："每一轮都形成架构快照... 这个要对 agent 形成硬性规则，手工提交可以忽略"
2. **Agent 判定**：先做规则化，再做 Reranker —— 因为后者要按新规则走完整 STATUS
3. **Agent 拆分** Commit A（规则化） + Commit B（Reranker 主线）
4. **Commit A 内**：4 项自主决策（slug 严格命名 / 手工豁免 / 文件标记 / bootstrap 自身满足规则）
5. **Agent 直接执行 Commit A**，未再回问；Commit B 在本 commit 完成后启动

## 验证结果

- 编译：N/A（纯文档 + 脚本 + 规则）
- check-api-parity：N/A
- pre-commit：本 commit 期望 PASS（Round 8 feature log + STATUS-bootstrap-status-rule.md 配套，触发新规则但满足）
- 规则首次拦截测试：将在 Commit B（Day 5 Reranker）演示——若我忘了写 STATUS，新规则应当 FAIL

## 残留 / 下一轮

- [ ] **Commit B（紧接本 commit）**：Day 5 Reranker 主线 —— 按新规则走完整 STATUS
- [ ] 下一轮加 STATUS 内容深度校验（6 个 H2 节存在性）
- [ ] 写 STATUS-INDEX.md 时间线索引
- [ ] 把 STATUS-day2.md 改名为 `STATUS-day2-mock-demo.md` 对齐 slug 规则？（暂搁，保持历史）
- [x] Day 4 STATUS 追溯
- [x] AGENTS.md #17 + 机械化校验 + commit.sh 标记机制
- [x] 本 commit 自身演示规则
