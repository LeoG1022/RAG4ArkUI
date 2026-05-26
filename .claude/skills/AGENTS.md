# .claude/skills/ — Skill 定义

5 个核心 Skill：`/generate` `/kmp-to-arkuix` `/arkuix-refactor` `/migrate-to-benchmark` `/run-benchmark`。

---

## 文件命名

- 一个 skill 一个文件：`{slug}.md`，slug 即触发名去掉 `/`
- 文件名小写，连字符分隔（如 `kmp-to-arkuix.md`、`run-benchmark.md`）

---

## Skill 文件必备节（顺序）

**前序（自 Round 10 起强制）**：YAML frontmatter。详见 [`SKILL_SCHEMA.md`](SKILL_SCHEMA.md)。

```yaml
---
name: <kebab-case>
version: 1.0.0
trigger: /<name>
description: <一行用途>
feature_log_required: true/false  # run-benchmark 设 false 豁免
classify_required: true
preflight_required: true
calls: [...]
references: [...]
---
```

**正文**：

```markdown
# Skill: /<name>

<一句话用途>

## Trigger
<触发命令 + 参数说明>

## Required References (可选)
<必读 + 按需加载的 reference 文件路径>

## Steps / Phase / Round Protocol
<编号步骤；每步说明"输入 / 输出 / 验证">

## Rules
<必须遵守的硬约束，含 Git 前置检查引用>
```

---

## 全局约束（所有 skill 必须遵守）

1. **Git 前置检查**：每个 skill 在执行任何写操作前，必须先按 [`AGENTS.md`](../../AGENTS.md) 的"通用前置协议"跑 `git status` 并征得用户同意
2. **关键词路由 mapping**：不一次性加载全部 mapping，按 `.claude/references/AGENTS.md` 路由表加载相关领域文件
3. **机械化校验**：生成 / 转换 / 重构 后调用 `scripts/check-api-parity.sh`，FAIL 必须修复
4. **mapping 缺失协议**：遇到 reference 未覆盖的 API → 停下来问用户，禁止猜测
5. **不要在 skill 主体内联大表**：表格 > 20 行的应抽到 `.claude/references/`，skill 只保留高频速查 + 引用指引

---

## 当前 5 个 skill 速查

详见 [`CLAUDE.md`](../../CLAUDE.md) "Skill 速查" 节（权威来源）。新增 skill 时必须同步更新 CLAUDE.md。

---

## 添加新 skill 的流程

1. 跑 Git 前置检查
2. 在本目录新建 `{slug}.md`，遵循上方必备节结构
3. 更新 [`CLAUDE.md`](../../CLAUDE.md) "Skill 速查" 表加一行
4. 如需新规则 → 追加到 `scripts/check-api-parity.sh` + `feedback/DESIGN.md`
5. 在 `feedback/{N}-{date}-{slug}.md` 记录本轮迭代

---

## 下一步

- 想了解每个 skill 怎么用 → 直接看对应文件
- 想知道引用哪些 mapping → 见 [`../references/AGENTS.md`](../references/AGENTS.md)
- 想加新校验规则 → 见 [`../../scripts/AGENTS.md`](../../scripts/AGENTS.md)
