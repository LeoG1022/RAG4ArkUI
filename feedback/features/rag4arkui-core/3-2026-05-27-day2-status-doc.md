# 3 — day2-status-doc

> 日期：2026-05-27
> 涉及代码：`docs/STATUS-day2.md`（新增）
> 类型：新建（纯文档）

## 本轮目标

回答用户提出的三个问题：
1. 当前阶段是否需要把 ArkUI-X 文档导入工程？
2. 给当前阶段画架构图 + 输入 / 输出 / 用户验证 / 自动化验证，保存到 `docs/`
3. 现阶段是否需要引入自动化验证手段？

把三问的判断 + 架构快照整合成一份 `docs/STATUS-day2.md`，作为"阶段快照"文档的首份模板。

## Plan

> 必备节（FAIL 级 M-FEATURE-PLAN）

### 文档结构设计（8 节）

| 节 | 内容 | 对应问题 |
|---|---|---|
| §1 架构图 | C4-Level 2 容器图（mermaid，标注 alive/stub/mock 三态）+ Cargo 依赖图 | Q2 |
| §2 流程图 | index 时序图 + query 时序图 | Q2 |
| §3 输入契约 | corpus markdown + frontmatter schema + CLI 参数表 | Q2 |
| §4 输出契约 | IndexStats / index.json schema 完整示例 / Top-K 命中输出格式 | Q2 |
| §5 用户验证手段 | 4 步手动验证（装 rust → cargo check → cargo test → demo） | Q2 |
| §6 自动化验证手段 | 现有 7 项 + 建议新增 8 项按 ROI 排序 | Q2 + Q3 |
| §7 阶段定位 + 问题 1/3 详答 | Day 2 对照 6 周路线图 + 文档导入策略 + 自动化手段三选项 | Q1 + Q3 |
| §8 维护约定 | STATUS-day{N}.md 命名规则 | （元约定） |

### 三态标注方案

- 🟢 **alive**：当前真活，能跑出真实结果（IDX_CMD/QRY_CMD/IND/HYB/MD/VS/BM）
- 🟡 **stub**：接口就位、逻辑占位（OnnxEmbedder/LanceDB/Tantivy/Reranker/Server）
- ⚪ **mock**：算法不真但确定性可测（MockEmbedder 哈希派生向量）

让读者一眼看出"哪些是 demo 用的占位"vs"哪些已经是真活"vs"哪些等下阶段实装"。

### 问题 1 答复方案

给三选项表（不导入 / 少量 2-3 份 / 大批量），明确推荐"少量"作为契约容错测试。完整导入留到 Day 3（OnnxEmbedder 接好后语义检索才有意义）。

### 问题 3 答复方案

8 项自动化手段按 ROI 排序，明确"Day 2.5 三件套" = A(CI) + B(smoke 脚本) + C(clippy/fmt 入 hook)。其余项（D-H）注明何时加。

### 关键决策

- **不写新 ADR**：本文档是阶段快照，不是架构决策。架构决策仍以 `docs/ADR-XXX.md` 为单一事实源；STATUS 引用不复制。
- **命名 `STATUS-day2.md` 而非 `STATUS.md`**：每个 Day 切片独立快照，便于回溯演进。`STATUS-day3.md` 在 Day 3 完成时新增；旧的不删。
- **mermaid 而非 ASCII art**：mermaid 在 GitHub / VSCode 直接渲染；ASCII 在长期维护中容易错位。

## 改动要点

仅新增 `docs/STATUS-day2.md`（约 380 行），无代码改动。

文档亮点：
- 8 个 crate 的 mermaid 容器图，节点带颜色标识三态
- 两张时序图（index pipeline / query pipeline），步骤编号 + 子调用清晰
- 完整的 `index.json` schema 示例 + Top-K 输出预期
- Day 2 完成度对照技术方案 6 周路线图的进度表

## 对话摘要

> 必备节（FAIL 级 M-FEATURE-PLAN）

按时序：

1. **用户提问**：「1. 当前阶段需要我将arkui-x相关文档导入到工程吗。 2. 对当前阶段画出架构图，以及输入，输出，用户验证手段和自动化验证手段，保存到docs目录下。3. 现阶段需要引入一些自动化验证手段吗」
2. **Agent 先给文字判断**（不直接动工）：
   - Q1 → 不必现在导入（MockEmbedder 阶段语义无意义）
   - Q3 → 推荐 A+B+C 三件套（CI / smoke / clippy-fmt）
3. **Agent 直接写 STATUS-day2.md** 作为 Q2 的产物，并把 Q1/Q3 的详细论证也写进去（§7 阶段定位）
4. **本 feature log** 在 commit 阶段补建（pre-commit 强制业务改动必须关联 feature log）

## 验证结果

- 编译：N/A（纯文档）
- check-api-parity：N/A
- 文档链接完整性：M-LINK-DEAD 应 PASS（STATUS-day2.md 内的相对链接指向 ADR-002 等已存在文件）
- mermaid 渲染：GitHub markdown 原生支持，VSCode 装 Markdown Preview Mermaid Support 也可（未实际渲染验证）

## 残留 / 下一轮

- [ ] 等用户决策：Day 3 切片方向（默认推荐 OnnxEmbedder 真实化；备选 LanceDB / Tantivy / tree-sitter）
- [ ] 等用户决策：是否落地 §6.2 推荐的"Day 2.5 三件套"（GitHub Actions CI + demo smoke 脚本 + clippy/fmt 入 hook）
- [ ] 等用户决策：是否现在少量导入 2-3 份 ArkUI-X 文档做契约容错测试
- [x] 当前阶段架构快照文档（本轮完成）
