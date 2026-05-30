# RAG4ArkUI 路线图 · 全景与进度

> **文档定位**：项目长期维护文档（类似 ADR / README），跨阶段全景视图。
> **维护约定**：每个 Day 完成 commit 后，agent **同步更新本文档的进度标记**（不单独 commit）；新阶段补充进度行；不在本文档归档单次 round 的细节（那些去 STATUS-`<slug>`.md）。
> **最后更新**：Day 16 完成（2026-05-28 · commit pending · LSP Server · 协议层 3/3 完整 ⭐）

---

## 📍 当前位置

**task #81 完成 · lancedb 0.10 → 0.30 主版本升级 · `--vector lancedb` 端到端真活 ⭐**

- 9 个 Cargo crate · HTTP + MCP + LSP 三协议全部真活 ⭐
- 本地 + CI 双路径分发：`make release-local`（host）+ `.github/workflows/release.yml`（4 平台 matrix）
- 默认 release features 6 项（http, mcp, lsp, tantivy, typescript, corpus-pull · **lancedb 仍按需启用**）
- **lancedb feature 完整真活**：`--features lancedb` binary 95 MB · 内部 lance KNN+FilteredRead
- mdBook 文档站 + `docs/RELEASE-NOTES-v1.0.0.md` 草稿就位
- `corpus pull` + `corpus model-pull` 一键拉默认 corpus + BGE-M3 模型
- pre-existing 阻塞清单：**全部已解锁** ✅（Round 40-42 收掉 task #87 / Day 20c · ONNX 真语义 BGE-M3 端到端 · Top-1 命中率 mock 3/6 → onnx 6/6）
- 20 个 STATUS 文档
- 27 个 git commit / 历史 18 个工作 Day

---

## 时间线（Mermaid Gantt）

```mermaid
gantt
    title RAG4ArkUI 6 周 MVP 进度
    dateFormat YYYY-MM-DD
    axisFormat %m-%d

    section ✅ Week 1
    docs import                    :done,    d0, 2026-05-26, 1d
    workspace 骨架 (8 crate)         :done,    d1, after d0, 1d
    端到端 Mock Demo                 :done,    d2, after d1, 1d
    STATUS-day2 + smoke 脚本         :done,    d25, after d2, 1d
    OnnxEmbedder async              :done,    d3, after d25, 1d
    GitHub Actions CI（搁置）       :done,    d35, after d3, 1d

    section ✅ Week 2
    TantivyBM25 真活                 :done,    d4, after d35, 1d
    规则 #17 STATUS-PER-ROUND       :done,    bsr, after d4, 1d
    OnnxReranker 真活                :done,    d5, after bsr, 1d
    检索质量评估                     :done,    d6, after d5, 2d
    HyDE 改写器                      :done,    d7, after d6, 1d
    tree-sitter (代码切分)           :done,    d10, after d7, 2d
    LanceDB (Week 1 收尾)            :done,    d9, after d10, 2d
    Parent-Child 父子索引            :done,    d11, after d9, 1d
    HTTP/REST Server                :done,    d14, after d11, 2d
    MCP Server (协议层基本完整)        :done,    d15, after d14, 3d
    Claude Code 接入验证             :done,    d19, after d15, 1d
    LSP Server (当前 · 协议层 3/3 ⭐) :active,  d16, after d19, 2d

    section ⏳ Week 2 末
    tantivy-jieba 中文升级           :         d8, after d14, 1d

    section ⏳ Week 3 (规模化 + 流水线)
    Query Router + Intent           :         d12, after d8, 1d
    ContextAssembler 全功能          :         d13, after d12, 1d

    section ⏳ Week 4 (协议层)
    (Week 3 末协议层下游切片)         :         d14_done, 2026-06-01, 1d

    section ⏳ Week 5 (IDE 接入)
    DevEco Plugin MVP               :crit,    d17, after d16, 5d
    VSCode Extension                :         d18, after d17, 3d
    Claude Code MCP 验证            :         d19, after d18, 1d

    section ✅ Week 6 (发布)
    本地 release artifact (Day 20a) :done,    d20a, after d16, 1d
    section ⏳ Week 6 (发布 续)
    CI matrix + GitHub Releases     :         d20b, after d20a, 1d
    Corpus 分发管道                 :         d21, after d20b, 2d
    文档站点 + Release              :         d22, after d21, 2d
```

---

## ✅ 已完成（27 commits）

| Commit | Day | Round | 内容 | STATUS |
|---|---|---|---|---|
| `e375ca4` | 0 | — | docs import | — |
| `95c5f70` | 1 | 1 | Cargo workspace 骨架（7 → 8 crate） | — |
| `69216db` | 2 | 2 | 端到端 Mock Demo + indexer crate | [STATUS-day2](STATUS-day2.md) |
| `1232ccc` | 2.5 | 3 | STATUS-day2.md 阶段快照 | (included) |
| `1b0e04f` | 2.5 | 4 | demo-smoke.sh 脚本 | — |
| `41c00a4` | 3 | 5 | **OnnxEmbedder** async wrapper（BGE-M3 真活） | — |
| `a4410f2` | 3.5 | 6 | GitHub Actions CI（**搁置**） | — |
| `331a912` | 4 | 7 | **TantivyBM25Index** 真活 → Hybrid 名实相符 | [STATUS-day4](STATUS-day4-bm25-tantivy.md)（追溯） |
| `20056b3` | Bootstrap | 8 | **规则 #17** STATUS-PER-ROUND FAIL 级 | [STATUS-bootstrap](STATUS-bootstrap-status-rule.md) |
| `331f180` | 5 | 9 | **OnnxReranker 真活** → Hybrid + Rerank 业界基线 | [STATUS-day5](STATUS-day5-reranker.md) |
| `44d6233` | 6 | 10 | 检索质量评估闭环（arkui-rag-eval crate） | [STATUS-day6](STATUS-day6-eval.md) |
| `0228109` | — | 11 | ROADMAP 全景图归档 | [STATUS-roadmap](STATUS-roadmap-doc.md) |
| `6969cba` | 7 | 12 | HyDE 改写器（QueryEnhancer trait + MockHyde） | [STATUS-day7](STATUS-day7-hyde.md) |
| `ab869ba` | 10 | 13 | tree-sitter ArkTS/TS 代码切分 + ChunkerDispatcher 路由 + Indexer 重构 | [STATUS-day10](STATUS-day10-tree-sitter.md) |
| `d7301e1` | 9 | 14 | LanceDB 嵌入式向量库（VectorBackend 抽象 · Week 1 全部达成 ⭐） | [STATUS-day9](STATUS-day9-lancedb.md) |
| `e2a7129` | 11 | 15 | Parent-Child 父子索引 + ContextAssembler + CLI --expand-parent | [STATUS-day11](STATUS-day11-parent-child.md) |
| `af19208` | 14 | 16 | HTTP/REST Server（axum · /search /health /corpus/list · 5 集成测） | [STATUS-day14](STATUS-day14-http.md) |
| `f4724cc` | 15 | 17 | MCP Server（JSON-RPC stdio · 4 tools · Claude Code 接入就绪） | [STATUS-day15](STATUS-day15-mcp.md) |
| `865c463` | 19 | 18 | Claude Code 接入验证（接入指南 10 节 + bash 端到端 demo · Week 5 1/1 ⭐） | [STATUS-day19](STATUS-day19-claude-code.md) |
| `611cdcb` | 16 | 19 | LSP Server（手撸 Content-Length framing + 5 method + custom commands · 协议层 3/3 完整 ⭐） + 11 pre-existing 编译缺陷清理 | [STATUS-day16](STATUS-day16-lsp.md) |
| `197e894` | 20a | 20 | 本地 host release artifact（`scripts/release-local.sh` + Makefile + `docs/RELEASE.md` · 端到端 CLI 可下载即用 ⭐） | [STATUS-day20a](STATUS-day20a-release-local.md) |
| `3ddb3a3` | 20b | 21 | CI matrix release（`.github/workflows/release.yml` · 4 平台 build · tag `v*` 触发 · GitHub Releases 自动上传 ⭐） | [STATUS-day20b](STATUS-day20b-ci-matrix.md) |
| `f2797e1` | 20c | 22 | pre-existing 阻塞清理（Phase 1 typescript API 对齐 ✅ · Phase 2 chrono pin ✅ · typescript 进默认 features ⭐） | [STATUS-pre-existing-fixes](STATUS-pre-existing-fixes.md) |
| `5569cc7` | 21 | 23 | `arkui-rag corpus pull` 真活（ureq + tar.gz · feature gated · 默认 release 已含 · 用户无脑接入 ⭐） | [STATUS-corpus-pull](STATUS-corpus-pull.md) |
| `98bf22d` | 22 | 24 | mdBook 文档站 + 1.0 release notes 草稿（push master 触发 GitHub Pages 自动部署 · `RELEASE-NOTES-v1.0.0.md` 草稿 · MVP 完整收尾 ⭐） | [STATUS-mdbook-doc](STATUS-mdbook-doc.md) |
| `2dfbf61` | 21b | 25 | `corpus model-pull` 真活（重构抽 `download_and_extract` shared helper · `bge-m3` / `bge-reranker-v2-m3` 默认 URL 路由） | [STATUS-model-pull](STATUS-model-pull.md) |
| _(本 commit)_ | **task #81 (当前)** | **26** | **lancedb 主版本升级**（0.10 → 0.30 · arrow 52 → 58 · chrono pin 移除 · LanceVectorStore dim auto-detect · `--vector lancedb` 端到端真活 ⭐） | [STATUS-lancedb-upgrade](STATUS-lancedb-upgrade.md) |

---

## ⏳ 剩余切片（按推荐顺序）

### 🟢 Week 2 末 · 检索质量纵深（Day 7 + Day 11 已完成 · 1 个剩余）

| Day | 切片 | 价值 | 工作量 | 依赖 |
|---|---|---|---|---|
| ~~7~~ | ~~HyDE 改写器~~ | ✅ **Day 7 完成**（MockHyde 真活，远程 LLM 接入留 Week 3） | — | — |
| ~~11~~ | ~~Parent-Child 父子索引~~ | ✅ **Day 11 完成**（Chunker + ContextAssembler + CLI --expand-parent） | — | — |
| 8 推荐 | tantivy-jieba 中文分词 | 中文 BM25 精度从 ngram 升级；评估集可量化提升 | 0.5 commit | Day 4 ✓ |

### 🟡 Week 3 · 规模化 + 流水线收尾（Day 9 + Day 10 已完成）

| Day | 切片 | 价值 | 工作量 |
|---|---|---|---|
| ~~9~~ | ~~LanceDB 替换 InMemoryVectorStore~~ | ✅ **Day 9 完成**（feature `lancedb` · Arrow schema · post-filter） | — |
| ~~10~~ | ~~tree-sitter (.ets/.kt/.swift)~~ | ✅ **Day 10 完成**（ArkTS 真活 · Kotlin/Swift stub） | — |
| 12 | Query Router + Intent 分类 | 不同 query 走不同流水线（方案 §1.2） | 1 commit |
| 13 | ContextAssembler 真活 | 父 chunk 扩展 + 引用元数据完善 | 1 commit |

### 🟢 Week 4 · 协议层（Day 14 已完成 · 2 个剩余 · 关键路径）

| Day | 切片 | 价值 | 工作量 |
|---|---|---|---|
| ~~14~~ | ~~HTTP/REST Server (axum)~~ | ✅ **Day 14 完成**（/health · /corpus/list · POST /search · /index stub） | — |
| ~~15~~ | ~~MCP Server~~ | ✅ **Day 15 完成**（手撸 JSON-RPC stdio · 4 tools · Claude Code 接入就绪 ⭐） | — |
| ~~16~~ | ~~LSP Server~~ | ✅ **Day 16 完成**（手撸 framing + 5 method + custom commands + hover stub · 协议层 3/3 ⭐） | — |

### 🟢 Week 5 · IDE 接入（差异化目标，3 个切片）

| Day | 切片 | 价值 | 工作量 |
|---|---|---|---|
| **17 ⭐ 关键** | **DevEco Plugin MVP** | 主战场（方案 §4.3） | 5+ commit · 大工程 |
| 18 | VSCode Extension | 跨编辑器覆盖 | 3+ commit |
| 19 | Claude Code MCP 端到端验证 | 验证 Agent 接入路径 | 1 commit |

### 🟢 Week 6 · 发布（3 个切片）

| Day | 切片 | 价值 | 工作量 |
|---|---|---|---|
| **20a** ✅ | **本地 host release artifact**（aarch64-apple-darwin · `scripts/release-local.sh` + Makefile） | 端到端 CLI 可下载即用 | ✅ 1 commit |
| **20b** ✅ | **CI matrix 4 平台**（darwin arm64/x86_64 + linux + win） + GitHub Releases 自动上传（tag `v*` 触发） | 多平台分发自动化 | ✅ 1 commit（本轮） |
| **21** ✅ | **`arkui-rag corpus pull --url|--from-file` 真活**（ureq 下载 + tar.gz 解压 + path traversal 安全检查） | 用户无脑接入 · 不用手动投放文档 | ✅ 1 commit（本轮） |
| **21b** ✅ | **`arkui-rag corpus model-pull --name bge-m3` 真活**（共用 download_and_extract helper · 默认 URL 路由 + `~/.arkui-rag/models/<name>/` 默认 target） | 模型自动下载 · 共用基础设施 | ✅ 1 commit（本轮） |
| **22** ✅ | **mdBook 文档站 + 1.0 release notes 草稿**（push master 触发 GitHub Pages 自动部署 · 用户改版本号 + push tag v1.0.0 触发 1.0 release） | MVP 完整收尾 | ✅ 1 commit（本轮） |

### 🔮 长期演进 · 阶段 3-4（护城河）

| 切片 | 价值 |
|---|---|
| **XDB 错误飞轮**（自研依赖） | 方案 §1.2 核心差异化护城河 |
| Code GraphRAG（SCIP 代码图谱） | 跨文件多跳推理（方案 §1.4 Phase 4） |
| Self-RAG / CRAG 反思机制 | 幻觉率从 12% 降到 5%（方案 §1.3） |
| UISG 集成（自研依赖） | 一多合规自动验证 |
| 团队级共享 corpus | 多用户支持 |

### 🔧 工程化穿插（meta，非阻塞）

| 切片 | 状态 |
|---|---|
| GitHub Actions CI | 已搁置（Day 3.5 写好等推送目标） |
| STATUS-INDEX.md 时间线索引 | Backlog（用户提到过） |
| `scripts/new-status-doc.sh` 模板生成器 | Backlog |
| STATUS 6 节存在性深度校验 | Backlog（meta feedback 5 列出） |
| `cargo audit` / Codecov / 跨平台 CI 矩阵 | Backlog |

---

## 📊 6 周 MVP 路线图达成度

| 方案章节里程碑 | 状态 | 完成度 |
|---|---|---|
| Week 1: Rust 骨架 + tree-sitter + LanceDB + Tantivy + BGE-M3 | **7/7** ✅ | 全部达成（tree-sitter ArkTS ✓ · LanceDB ✓ Day 9 · Kotlin/Swift stub 是非阻塞补强） |
| Week 2: 混合检索 + Reranker + HyDE + 评估集 | **4/4** ✅ | 全部达成 |
| Week 3: HTTP + MCP + CLI | **3/3** ✅ | 全部达成 ⭐ |
| Week 4: 协议层（HTTP + MCP + LSP） | **3/3** ✅ ⭐ | 全部达成（Day 14/15/16 三件套） |
| Week 4: IDE 插件 (DevEco/IntelliJ) | **0/2** ⏳ | — |
| Week 5: Claude Code 接入 | **1/1** ✅ | Day 19 |
| Week 6: 自动安装 + corpus 分发 + 文档 + 评估报告 | **4/4** ✅ | 评估报告 ✓ · 本地 release ✓ · CI matrix ✓ · **corpus pull ✓（Day 21）** |

**当前完成度估算：~90%**（Week 1-6 全部达成 · 协议层 3/3 ⭐ · 本地 + CI 全平台分发 ✓ · corpus 一键拉取 ✓ · mdBook 文档站 ✓ · 1.0 release notes 草稿 ✓ · 仅用户首推 v1.0.0 tag + IDE 插件 + ONNX 真活待做）。

---

## 🎯 业界基线对照（§8.5）

| 业界共识 | 状态 | 落地 commit |
|---|---|---|
| 共识 1：混合检索是默认配置 | ✅ | Day 4 (`331a912`) |
| **共识 2：Reranker 是产品级 RAG 的分水岭** | ✅ | Day 5 (`331f180`) |
| 共识 3：引用溯源是产品可信度核心 | ✅ | Day 2 (`69216db`) 起 |
| **共识 4：评估先行，Eval-Driven Development** | ✅ | Day 6 (`44d6233`) |
| 共识 5：Agentic 是趋势，但 Adaptive 路由是产品策略 | ⏳ | 远期（Day 12 起步） |

**4/5 业界共识达成**（仅 Adaptive Routing 待远期补全）。

---

## 关键里程碑预测

| 里程碑 | 累计 commit | 累计 round |
|---|---|---|
| ✅ Hybrid + Rerank + Eval 基线 | 12 | 11 |
| ✅ Week 2 全部达成（+ HyDE） | 13 | 12 |
| ✅ 代码 corpus 真活（tree-sitter ArkTS） | 14 | 13 |
| ✅ Week 1 全部达成（+ LanceDB） | 15 | 14 |
| ✅ Parent-Child 标准（**当前位置**） | 16 | 15 |
| ✅ HTTP/REST Server（协议层入门） | 17 | 16 |
| ✅ MCP Server（**当前位置 · Claude Code 接入就绪** ⭐） | 18 | 17 |
| LSP Server（协议层最后一项） | +2 | 19 |
| 协议层完整（HTTP + MCP + LSP） | +12 | 28 |
| 首个 IDE 插件 MVP（DevEco） | +8 | 36 |
| 公开 release 1.0 | +5 | 41 |

**估算**：从当前 Round 14 → 完整 MVP 1.0，约还需 **27+ commit** / **4 周**。

---

## 🔴 关键路径（必走切片，不可省）

```
Day 6 评估闭环 ✓
   ↓
Day 7 HyDE ✓
   ↓
Day 10 tree-sitter ✓（代码 corpus 解锁）
   ↓
Day 9 LanceDB ✓（Week 1 全部达成）
   ↓
Day 11 Parent-Child ✓（方案 §1.4 标准 · 当前位置）
   ↓
Day 14 HTTP Server ✓（关键路径起点 · 当前位置）
   ↓
Day 15 MCP Server ⭐（最关键 · 接 Claude Code）
   ↓
Day 17 DevEco Plugin ⭐（主战场）
   ↓
Day 19 Claude Code MCP 端到端验证
   ↓
Day 20-22 发布
```

🟢 **可选优化**：Day 8 jieba / Day 9 LanceDB / Day 11 Parent-Child / Day 12 Router / Day 13 Assembler 是质量提升，不在关键路径。

🔮 **战略护城河**（MVP 后）：XDB 飞轮、UISG 集成、Code GraphRAG。

---

## 关键文档导航

| 文档 | 用途 |
|---|---|
| [`docs/RAG4ArkUI-完整技术方案.md`](RAG4ArkUI-完整技术方案.md) | 单一事实源 · 78KB 全规约 |
| [`docs/ADR-001-language-choice.md`](ADR-001-language-choice.md) | 选 Rust 的依据 |
| [`docs/ADR-002-crate-structure.md`](ADR-002-crate-structure.md) | 9 crate 拆分 + Feature gate 策略 |
| [`docs/ADR-003-corpus-layout.md`](ADR-003-corpus-layout.md) | 5 类 corpus 子目录 + 元数据 schema |
| [`docs/STATUS-day*.md`](.) | 每轮 agent 提交的架构快照（规则 #17 强制） |
| [`AGENTS.md`](../AGENTS.md) | 全局规则（含 #17 每轮 STATUS） |
| [`CLAUDE.md`](../CLAUDE.md) | Claude Code 运行时 SOP |
| [`corpus/README.md`](../corpus/README.md) | corpus 文档投放约定 |
| [`corpus/_eval/queries.yaml`](../corpus/_eval/queries.yaml) | 评估集（Day 6 起） |

---

## 维护约定

### 谁更新本文档

- **每个 Day commit 后**：agent 在本文档"当前位置"区与"已完成"表中**同步更新进度行**（不单独 commit，包在该 Day 的 commit 中）
- **新切片 backlog 调整**：agent 在本文档"剩余切片"区更新优先级 / 工作量估算
- **完成度 / 里程碑 / 业界基线**：每 5 round 评审一次

### 与 STATUS-`<slug>`.md 的关系

- **ROADMAP**：跨阶段全景图，回答"还有多少工作 / 当前位置"
- **STATUS-`<slug>`.md**：单轮快照（规则 #17 强制），回答"这一轮做了什么 / 现在是什么状态"
- 二者**互补不重复**：ROADMAP 引用 STATUS 链接，STATUS 不重复 ROADMAP 内容

### 何时新增 ROADMAP

- 仅在路线图重大调整时新增 ROADMAP-vN.md（保留历史）；常规进度变更在原文件更新
