# 28 — rag-smoke-script

> 日期：2026-06-03
> 触发：用户「先不纠结于 release · 先重点验证本地 RAG 效果」
> 类型：工具脚本（新增 scripts/rag-smoke.sh + corpus/_eval/smoke-queries.yaml）

---

## 用户提出的要求

> 「先不纠结于release 。先重点验证本地RAG效果」

潜在需求：跑一批典型 query · 看本地 RAG 命中是否合理 · 出对比报告。不要一次性的命令 · 要可重复跑的脚本。

## Agent 给出的修改建议

新增两件东西：

**A. `corpus/_eval/smoke-queries.yaml`** —— 16 个典型 ArkUI-X query
- 入门类 4（a1-a4）：创建项目 / 概念 / DevEco / ace tools
- 工程类 2（b1-b2）：目录结构 / Stage 模型
- 多端类 3（c1-c3）：Android / iOS / 平台差异
- 进阶类 4（d1-d4）：platform-bridge / 动态化 / FFI / 双向绑定
- 资源类 1（e1）：resource 访问
- 负样本 2（f1-f2）：天气 / 炒鸡蛋（应过滤）
- 每条 query 含 `expect: "..."` 注释（不当 ground truth · 仅人眼对照）

**B. `scripts/rag-smoke.sh`** —— 跑 query 集 + 出 markdown 报告
- 接口：`bash scripts/rag-smoke.sh [--index-path ...] [--queries-file ...] [--top-k 3] [--out reports/X.md] [--min-vector-score 0.7]`
- 解析 yaml（ruby 系统自带）· 跑 cli query · awk 抽 hit 表 · markdown 输出
- 用 BSD awk 兼容（不用 gawk 三参 match）

### 替代方案

- A · 直接用 `arkui-rag eval`：需要 ground truth chunk_id · 现成的 queries.yaml 都是 placeholder
- B · 每次手敲 16 条 query：3 分钟一遍 · 不可重复
- **C · 单独 smoke 脚本 + query-only YAML（本次选）**：可重复 + 看人眼合理性

### 关键决策

| 决策 | 选择 | 理由 |
|---|---|---|
| ground truth | 不要 · 用 `expect: "..."` 注释 | quick-start 子集 chunk_id 未稳定 · 不写死 |
| 报告格式 | markdown · 每 query 表格 | 易读 + 可 diff（两份报告 vs 阈值） |
| 阈值参数 | `--min-vector-score <f32>` 传 cli | 同款 cli 参数 · 不在脚本层做过滤 |
| BSD awk | 不用 gawk 三参 match | macOS 默认 awk · 不要装 gawk |

## 多轮互动

| 轮次 | 动作 |
|---|---|
| 1 | 写脚本 + 跑第一遍 → 16 条 query 报告 hit 表全空（awk 抓不到）+ id 含 tab 字符 |
| 2 | 修：ruby `'\t'` 单引号无效 → 改双引号 / awk 用 BSD 兼容写法 |
| 3 | 跑 v1 报告 16 条全 OK · score 全 0.0164/0.0161/0.0159 暴露 RRF 问题 → 触发 Round 52 加 vector_score 字段 |
| 4 | Round 52 完成后跑 v2 报告 · 加 `--min-vector-score 0.7` 真过滤负样本 |

## 实际改动

- 接口变化：新增 `bash scripts/rag-smoke.sh [...]`
- 规则变化：无
- 文件变化：
  - 新增 `corpus/_eval/smoke-queries.yaml`（72 行 · 16 query）
  - 新增 `scripts/rag-smoke.sh`（~150 行 · ruby + awk + cli 串）
- 配置变化：脚本内默认 `INDEX_PATH=/Users/leo/tmp-index-pull2/index.json`（Round 49.8 测试解压）

## 执行生效后总结

### 实际产出

| 报告 | 用途 |
|---|---|
| `reports/rag-smoke-quick-start.md` | v1 Phase A · 单 score · 暴露 RRF rank-based 问题 |
| `reports/rag-smoke-v2-no-threshold.md` | v2 Round 52 后 · 三 score 透明 |
| `reports/rag-smoke-v2-threshold-0.7.md` | v2 阈值 0.7 · 负样本全过滤 |
| `reports/rag-smoke-v2-summary.md` | 三份汇总 + 阈值决策 |

### 前后对比

| 维度 | Phase A（v1）| v2 |
|---|---|---|
| score 列 | score（RRF 0.0164/0.0161/0.0159）| **rrf + vector + bm25 三列** |
| 负样本判定 | 不能（score 都一样）| **vector 0.62 vs 正样本 0.7+ 一目了然** |
| 阈值过滤 | 无 | `--min-vector-score 0.7` 自动剔 |
| 报告 reproducibility | 手敲 16 条 | `bash scripts/rag-smoke.sh` |

### 实测验证

```bash
bash -n scripts/rag-smoke.sh   # 语法 OK ✓
bash scripts/rag-smoke.sh --out /tmp/test.md   # 16 query 跑通 ✓
bash scripts/rag-smoke.sh --min-vector-score 0.7 --out /tmp/test-thr.md   # 负样本全过滤 ✓
```

数据样本：
- 正样本 a1 "ArkUI-X 怎么创建第一个应用" → vector=0.8161（强相关）
- 负样本 f2 "怎么炒西红柿炒鸡蛋" → vector=0.6181（无关 · 但 cosine 仍 0.6 因 BGE-M3 中文 base）

### 残留 / 下一轮处理

- [x] scripts/rag-smoke.sh + smoke-queries.yaml 真活
- [x] 三 score 输出 + 阈值过滤实测
- [x] 4 份 v1/v2 报告
- [ ] **Phase B 全量 build 完成后跑同款 smoke** · 看 590 files 比 130 chunks 覆盖差距
- [ ] **smoke 跑慢**（16 query 2 分钟 · BGE-M3 重 load 每次 3-7s）· serve mode 常驻可破
- [ ] **BM25 全空 — 问题**：装中文分词器（jieba / lindera）才有用 · Round 53 候选
- [ ] **eval 命令同款升级**：让 arkui-rag eval 也用 vector_score 计算指标
