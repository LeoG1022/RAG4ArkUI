# STATUS · 术语对照表 + 停用「真活」自造词

> 日期：2026-05-29
> 对应 commit：[本 commit · GLOSSARY]
> 对应 feature log：[`feedback/features/rag4arkui-core/27-2026-05-29-glossary.md`](../feedback/features/rag4arkui-core/27-2026-05-29-glossary.md)
> 上一阶段：[`STATUS-lancedb-upgrade.md`](STATUS-lancedb-upgrade.md)

> 🎯 **里程碑**：用户反馈触发的小修补 · 停用 agent 自造词「真活」+ 加术语对照表防漂移

---

## 当前状态

| 模块 | 变化 |
|---|---|
| `docs/GLOSSARY.md` | **新增** 5 类术语对照（agent 自造词 / 项目惯用词 / RAG 业界标准 / 协议层 / pre-existing 状态形容词）+ 末尾防漂移声明 |
| `mdbook/src/reference/glossary.md` | **新增** `{{#include ../../../docs/GLOSSARY.md}}` |
| `mdbook/src/SUMMARY.md` | 参考节加「术语对照表」 |
| `docs/ROADMAP.md` | 一处 `<name>` 反引号转义（mdBook HTML parser 告警 → clean build） |

---

## 输入契约

### 用户阅读

- 直接在 GitHub：`docs/GLOSSARY.md`
- 文档站（部署后）：`https://keerecles.github.io/RAG4ArkUI/reference/glossary.html`

### Agent 自我约束（自此 commit 起）

| 场景 | 行为 |
|---|---|
| 写新 commit message / feature log / STATUS / meta | 不用「真活」· 用「实装 / 真实可用 / 端到端跑通 / 真启动」 |
| 历史 commit（22 个含「真活」） | 不改写 · 由 GLOSSARY 帮新读者解读 |
| 浮出新 agent 自造词 | 先加 GLOSSARY 一行 → 再用 |

---

## 输出契约

`docs/GLOSSARY.md` 结构（5 节）：

```
1. Agent 自造词 → 标准中文      （真活 / 真启动 / 真上线）
2. 项目惯用词                   （meta vs business / 双轨归档 / STATUS-PER-ROUND / M-XXX / 第 N 次实战）
3. RAG/检索领域标准术语          （Hybrid / RRF / Reranker / HyDE / Parent-Child / Citation）
4. 协议层标准术语                （MCP / LSP / JSON-RPC 2.0 / Content-Length framing / stdio server）
5. Pre-existing / 状态形容词     （pre-existing 阻塞 / stub / feature gated / shadow）
末尾：未来防漂移声明
```

---

## 验证手段

| 手段 | 状态 |
|---|---|
| `mdbook build` clean | ✅ 0 error · 0 warning（修了 ROADMAP `<name>` 后） |
| GLOSSARY 在 mdBook 导航出现 | ✅ 参考节末尾 |
| Agent 行为约束 | ⏳ 持续自检 · `git log --grep 真活` 应不再有新 commit |
| **M-STATUS-PER-ROUND** Round 27 + STATUS-glossary 配套 | ✅ |

---

## 与上一阶段（STATUS-lancedb-upgrade）的关联性

| 维度 | task #81 完成时 | 本轮后 |
|---|---|---|
| 文档站术语 | 散落 22 commit | ✅ 单一对照表 |
| Agent 用语规范 | 「真活」遍布 | ✅ 自此停用 + 防漂移机制 |
| mdBook build 警告 | 1（`<name>` HTML parser） | ✅ 0 |
| STATUS 文档数 | 20 | 21 |
| commit 数 | 27 | 28 |

---

## 完成度 / 下一阶段

### 本轮完成度

| 项 | 状态 |
|---|---|
| docs/GLOSSARY.md 5 类对照 | ✅ |
| mdbook include + SUMMARY 链入 | ✅ |
| ROADMAP `<name>` 转义 + clean build | ✅ |
| 防漂移声明 | ✅ |
| 双轨归档（feature log 27 + STATUS-glossary 本档） | ✅ |
| 历史 commit sed 重写 | ⏸️ 决策：不做（无价值） |

### 下一阶段（按推荐）

| 候选 | 价值 | 工作量 |
|---|---|---|
| 🟢 用户操作 task #84 / #85（让 MVP 公开上线） | 文档站 + 1.0 release 真上 | 用户 UI · 0 commit |
| 🟢 Day 17 DevEco Plugin MVP | 关键路径主战场 | 5+ commit |
| 🟡 Day 20c onnx 链路（task #87 阻塞） | 真语义 RAG | 需架构决策（ort 1.16 vs candle vs sherpa-onnx） |
| ⚪️ ArkTS struct custom grammar | ArkTS @Component 方法切分 | 大工程 |

### 重要的"非完成"项

- ❌ task #84 用户首推 master + Settings→Pages → 文档站上线
- ❌ task #85 用户 push tag v1.0.0 → release page 上线
- ❌ Day 17 DevEco Plugin MVP
- ❌ Day 20c onnx 链路（task #87 阻塞）
