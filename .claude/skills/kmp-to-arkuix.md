---
name: kmp-to-arkuix
version: 1.0.0
trigger: /kmp-to-arkuix
description: 把已有 KMP Compose 代码转为对应 ArkUI-X ArkTS 实现，含 Quality Pass
feature_log_required: true
classify_required: true
preflight_required: true
calls:
  - scripts/preflight.sh
  - scripts/check-api-parity.sh
  - scripts/classify-change.sh
  - scripts/new-feature-log.sh
references:
  - .claude/references/arkuix-best-practices.md
---

# Skill: /kmp-to-arkuix

Convert KMP (Kotlin Multiplatform + Compose Multiplatform) source files into equivalent ArkUI-X ArkTS files.

> **执行前必须**遵守 [`AGENTS.md`](../../AGENTS.md) 的"通用前置协议（Git 状态检查）"。

## Trigger
User types `/kmp-to-arkuix` optionally followed by:
- `<source-file-path>` — KMP `.kt` file to convert
- `--out <target-path>` — output `.ets` path (defaults to `arkuix-workspace/entry/src/main/ets/`)

## Steps

0. **Preflight**（每次调用首步，强制）：
   ```bash
   bash scripts/preflight.sh
   ```
   git 非干净 → 按 [`AGENTS.md`](../../AGENTS.md) 规则 13 询问用户。
1. **Read** the source KMP file.
2. **Identify** all `@Composable` functions, data classes, state declarations.
3. **Route mapping by domain** (见下方"按需加载"表): only Read the mapping files relevant to APIs you see in the source.
4. **Convert** each construct: try inline cheat sheet first; fall back to the loaded mapping files.
5. **Run Post-Conversion Quality Pass** (见下方表) — silently auto-fix.
6. **Write** to target `.ets` (preserve subdirectory structure under `ets/`).
7. **Run mechanical validation**:
   ```bash
   bash scripts/check-api-parity.sh <target.ets>
   ```
   任何 FAIL 必须在交付前修复。WARN 在 summary 中列出。
8. **Output** conversion summary：转换映射 + Quality Pass 自动修复项 + check-api-parity 结果 + 需手动跟进项。

## 按需加载 mapping（关键词路由）

| 源代码出现 | 加载 |
|---|---|
| `LazyColumn` / `LazyRow` / `LazyVerticalGrid` / `items` / `stickyHeader` | `.claude/references/mapping-list.md` |
| `remember` / `mutableStateOf` / `LaunchedEffect` / `DisposableEffect` / `derivedStateOf` | `.claude/references/mapping-state.md` |
| `Row` / `Column` / `Box` / `Modifier` / `Text` / `Image` / `Spacer` / `Divider` | `.claude/references/mapping-layout.md` |
| `animate*AsState` / `AnimatedVisibility` / `Crossfade` / `tween` / `transition` | `.claude/references/mapping-animation.md` |
| `suspend` / `Dispatchers` / `Flow` / `HttpClient` / `painterResource` / `kotlin.math.*` / `format` | `.claude/references/mapping-async.md` |

不要一次性加载全部。一份源文件通常只用 1-3 份 mapping。

## Inline Cheat Sheet（覆盖 80% 简单 case）

| KMP | ArkUI-X |
|---|---|
| `@Composable fun Foo()` | `@Component struct Foo { build() {} }` |
| `remember { mutableStateOf(v) }` | `@State x: T = v` |
| `LaunchedEffect(Unit) { }` | `aboutToAppear() { }` |
| `LazyColumn { items(list, key={it.id}) {} }` | `List() { LazyForEach(ds, item=>{}, item=>item.id.toString()) }.cachedCount(3)` |
| `Row` / `Column` / `Box` | `Row` / `Column` / `Stack` |
| `Modifier.fillMaxSize()` | `.width('100%').height('100%')` |
| `Modifier.clickable { }` | `.onClick(() => { })` |
| `Image(painter, contentScale=Crop)` | `Image(src).width(n).height(n).objectFit(ImageFit.Cover)` |
| `Text(s, fontSize=16.sp)` | `Text(s).fontSize(16)` |

## Post-Conversion Quality Pass

转换后**自动**应用以下修复，不询问用户，在 summary 中列出：

| 检查 | 自动修复 |
|---|---|
| `ForEach` on any list | 换 `LazyForEach` + `ArrayDataSource` |
| `LazyForEach` 缺 keyGenerator | 加 `(item) => item.id.toString()` |
| `List` / `Grid` 缺 `.cachedCount()` | 加 `.cachedCount(3)` / `.cachedCount(4)` |
| 列表项 `@Component` 缺 `@Reusable` | 加 `@Reusable` |
| `Image` 缺 `.objectFit()` | 加 `.objectFit(ImageFit.Cover)` |
| `setTimeout` 用作动画 | 换 `animateTo({ duration: 300, curve: Curve.EaseInOut }, ...)` |
| `build()` 中有计算 | 提到 `aboutToAppear()` 或 `@Computed` |
| `taskpool` / `setInterval` 无清理 | 加 `aboutToDisappear()` 取消逻辑 |

## Step 7.5 — Feature Log（强制）

转换完成后**必须**追加 feature log：

```bash
# 推断：KMP 源文件名 PascalCase → kebab-case
# 例如 LazyList.kt → lazy-list

# 已有特性（通常 KMP 文件存在即特性已存在）
bash scripts/new-feature-log.sh <inferred-name> kmp-converted

# 极少数情况：KMP 文件是新创建的，无 features/<name>/ → new-feature.sh
```

agent 直接填日志，4 段简明扼要：
- 本轮目标：把 X.kt 转 X.ets
- 改动要点：API 映射决策 / 自动 Quality Pass 应用项
- 验证结果：check-api-parity 结果
- 残留：手动跟进项（complex lambdas 等）

## Step 8 — Classify & Notify（强制）

完成 Write 后：

```bash
bash scripts/classify-change.sh
```

如退出码 ≠ 0 → 必须把 `🔔 元变更检测` 醒目块原样复述给用户。

## Rules

- 函数/变量名保留（camelCase → camelCase）
- `dp` / `sp` 单位 → 数字
- 文件扩展名 `.ets`
- `@Entry` 只加在页面根组件
- 无法直接对标的 API → 加 `// TODO: manual port` 注释
- **Mapping 缺失协议**：如果加载完对应领域 mapping 后仍找不到某 API 的对标，**停止生成**，向用户提问：「mapping-`<领域>`.md 中没有找到 [API名] 的 ArkUI-X 等价写法，请确认。确认后追加到该 mapping 文件再继续。」
- Post-Conversion Quality Pass + `check-api-parity.sh` 必须执行，不可跳过
