---
name: generate
version: 1.0.0
trigger: /generate
description: 用一句话描述生成 KMP + ArkUI-X 双份最优实现
feature_log_required: true
classify_required: true
preflight_required: true
calls:
  - scripts/preflight.sh
  - scripts/check-api-parity.sh
  - scripts/classify-change.sh
  - scripts/new-feature.sh
  - scripts/new-feature-log.sh
references:
  - .claude/references/arkuix-best-practices.md
---

# Skill: /generate

Generate a feature implementation in **both** KMP (Compose Multiplatform) and ArkUI-X simultaneously.
Writes idiomatic, best-practice code to both workspaces in a single pass.

> **执行前必须**遵守 [`AGENTS.md`](../../AGENTS.md) 的"通用前置协议（Git 状态检查）"。

## Trigger

```
/generate <feature description>
```

Examples:
```
/generate a lazy scrollable list of 1000 product cards with image, title, and price
/generate an animated visibility toggle button with fade transition
/generate a canvas drawing scene that renders 500 colored circles
```

## Required References

固定加载（一次性，全程复用）：
- `.claude/references/arkuix-best-practices.md` —— KMP 与 ArkUI-X 最优写法清单 + 输出模板

按需加载（关键词路由）：

| 描述 / 源代码出现 | 加载 |
|---|---|
| 列表 / 滚动 / Grid / item / 商品 | `.claude/references/mapping-list.md` |
| 状态 / 响应式 / Effect / 生命周期 | `.claude/references/mapping-state.md` |
| 布局 / Row / Column / 图片 / 文本 / Modifier | `.claude/references/mapping-layout.md` |
| 动画 / 过渡 / animate / transition | `.claude/references/mapping-animation.md` |
| 异步 / 网络 / 协程 / 资源 / format | `.claude/references/mapping-async.md` |

不要一次性加载全部。

## Phase −1 — Preflight（强制，每次调用首步）

```bash
bash scripts/preflight.sh
```

输出 git 状态 / 一致性 / 残留追踪。如 git 非干净 → 按 [`AGENTS.md`](../../AGENTS.md) 规则 13 询问用户。

## Phase 0 — Clarify (仅当必要时)

Ask only if description leaves these ambiguous:
- Data model: 每项有哪些字段？
- Interactions: 点击 / 状态变化？
- Platform scope: Android / iOS / Desktop？

否则跳到 Phase 1。

## Phase 1 — Generate KMP Implementation

按 `arkuix-best-practices.md` 中 "KMP 最佳实践" 与 "KMP 输出代码模板"，写入：
```
kmp-workspace/app/src/commonMain/kotlin/<FeatureName>.kt
```

强制执行规则：列表 key、State 作用域、ContentScale.Crop 等。

## Phase 2 — Generate ArkUI-X Implementation

按 `arkuix-best-practices.md` 中 "ArkUI-X 最佳实践" 与 "ArkUI-X 输出代码模板"，写入：
```
arkuix-workspace/entry/src/main/ets/<FeatureName>.ets
```

强制规则：`LazyForEach` + `keyGenerator` + `.cachedCount(3)` + `@Reusable` cell + `Image` size+`objectFit` + 动画 `animateTo` + 计算放 `aboutToAppear`。

## Phase 3 — 机械化校验

```bash
bash scripts/check-api-parity.sh arkuix-workspace/entry/src/main/ets/<FeatureName>.ets
```

任何 FAIL 必须在 Phase 4 输出前修复。WARN 在 summary 中列出。

## Phase 4 — Summary Output

```
Generated: <FeatureName>

KMP    → kmp-workspace/app/src/commonMain/kotlin/<FeatureName>.kt
ArkUI-X → arkuix-workspace/entry/src/main/ets/<FeatureName>.ets

API correspondence:
  LazyColumn + items(key=) ↔ List + LazyForEach(keyGenerator)
  remember { mutableStateOf() } ↔ @State
  LaunchedEffect(Unit) ↔ aboutToAppear()
  <补充本次实际用到的对照>

Best practices applied:
  KMP:      <实际应用的规则>
  ArkUI-X:  <实际应用的规则>

check-api-parity.sh: PASS / WARN(<N> 项) / FAIL(已修复)

Next steps:
  1. Review 两份文件，调整数据模型
  2. /arkuix-refactor <ets-file> 做质量复扫
  3. 稳定后 /migrate-to-benchmark --scene <FeatureName>
```

## Phase 4.5 — Feature Log（强制，AI 驱动业务必留档）

业务代码生成后**必须**自动创建/追加 feature log（agent 自动填，不向用户提问）：

```bash
# 推断 feature 名：业务代码主文件 PascalCase → kebab-case
# 例如 /generate "懒加载列表" 生成 LazyList.kt → feature 名 lazy-list

# 新特性（features/<name>/ 不存在）
bash scripts/new-feature.sh <inferred-name>

# 已有特性新一轮
bash scripts/new-feature-log.sh <inferred-name> generated-<slug>
```

填日志内容（4 段，agent 直接撰写）：
- **本轮目标**：用户描述的功能
- **改动要点**：生成的关键决策（API 选型、最优写法应用）
- **验证结果**：check-api-parity PASS / WARN / FAIL
- **残留**：待人工 review 项

如无法推断 feature 名（描述模糊），**必须**提问用户："请提供 feature 名（kebab-case），如 lazy-list"。禁止静默跳过。

## Phase 5 — Classify & Notify（强制）

完成所有 Edit / Write 后：

```bash
bash scripts/classify-change.sh
```

如退出码 ≠ 0（meta 或 mixed）→ **必须**把脚本输出的 `🔔 元变更检测` 醒目块**原样**嵌入对用户的 summary。禁止省略。

## Rules

- 始终写真实可编译代码，不留 `// ...` 占位逻辑
- 文件名 PascalCase（`LazyList.kt` / `LazyList.ets`）
- KMP 端不加 `@Entry`；ArkUI-X 端只在页面根组件加
- **Mapping 缺失协议**：加载完对应领域 mapping 后仍找不到某 API 的对标 → **停止生成**，向用户提问：「mapping-`<领域>`.md 中没有找到 [API名] 的 ArkUI-X 等价写法，请确认。确认后追加到该 mapping 文件再继续生成。」**禁止自行猜测**。
- 生成完成后必须运行 Quality Pass + `check-api-parity.sh`，结果写入 summary
