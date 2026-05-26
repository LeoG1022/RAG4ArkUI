---
name: migrate-to-benchmark
version: 1.0.0
trigger: /migrate-to-benchmark
description: 把稳定特性从 workspace 迁入 benchmarks/ 基线并注册到运行列表
feature_log_required: true
classify_required: true
preflight_required: true
calls:
  - scripts/preflight.sh
  - scripts/check-api-parity.sh
  - scripts/classify-change.sh
  - scripts/new-feature-log.sh
references:
  - .claude/references/arkuix-refactor-checklist.md
---

# Skill: /migrate-to-benchmark

Migrate a finished, well-tested component from the daily workspace into the stable `benchmarks/` baseline.

> **执行前必须**遵守 [`AGENTS.md`](../../AGENTS.md) 的"通用前置协议（Git 状态检查）"。

## Trigger
User types `/migrate-to-benchmark` followed by:
- `--scene <SceneName>` — e.g. `LazyList`, `AnimatedVisibility` (PascalCase, matches file name)
- `--from <kmp|arkuix|both>` — which workspace to migrate from (default: `both`)
- Optional `--kmp-src <path>` — override default KMP source path
- Optional `--arkuix-src <path>` — override default ArkUI-X source path

## Default Source Paths

| Target | Default source |
|---|---|
| KMP | `kmp-workspace/app/src/commonMain/kotlin/<SceneName>.kt` |
| ArkUI-X | `arkuix-workspace/entry/src/main/ets/<SceneName>.ets` |

## Steps

### Step 0 — Preflight（强制）

```bash
bash scripts/preflight.sh
```

git 非干净 → 按 [`AGENTS.md`](../../AGENTS.md) 规则 13 询问用户。

### Step 1 — Validate

Read both source files. Verify:
- KMP: file contains at least one `@Composable` function named `<SceneName>` or `<SceneName>Benchmark`
- ArkUI-X: file contains at least one `@Component struct` named `<SceneName>` or `<SceneName>Benchmark`
- Both files have been through at least one `/arkuix-refactor` round (check for `LazyForEach`, `cachedCount`, `keyGenerator`)

If validation fails, report what's missing and stop.

### Step 2 — Copy KMP (if --from kmp or both)

1. Copy source to `benchmarks/KMP/benchmarks/src/commonMain/kotlin/benchmarks/<sceneName>/<SceneName>.kt`
2. Update package declaration to `package benchmarks.<sceneName>`
3. Open `benchmarks/KMP/benchmarks/src/commonMain/kotlin/Benchmarks.kt`
4. Add import and register in `getBenchmarks()`:
   ```kotlin
   import benchmarks.<sceneName>.<SceneName>Benchmark
   // ...
   BenchmarkItem("<SceneName>") { <SceneName>Benchmark() }
   ```

### Step 3 — Copy ArkUI-X (if --from arkuix or both)

1. Copy source to `benchmarks/ArkUI-X/entry/src/main/ets/benchmarks/<sceneName>/<SceneName>.ets`
2. Open `benchmarks/ArkUI-X/entry/src/main/ets/Benchmarks.ets`
3. Add import and register in the benchmarks array:
   ```typescript
   import { <SceneName>Benchmark } from './benchmarks/<sceneName>/<SceneName>'
   // ...
   { name: '<SceneName>', builder: () => { <SceneName>Benchmark() } }
   ```

### Step 4 — Report

Output a migration summary:
```
Migrated: <SceneName>

KMP:
  src  → benchmarks/KMP/benchmarks/src/commonMain/kotlin/benchmarks/<sceneName>/<SceneName>.kt
  registered in Benchmarks.kt ✓

ArkUI-X:
  src  → benchmarks/ArkUI-X/entry/src/main/ets/benchmarks/<sceneName>/<SceneName>.ets
  registered in Benchmarks.ets ✓

Next steps:
  - Android Studio: sync and run :androidApp to verify KMP side
  - DevEco Studio: hvigor assembleHap to verify ArkUI-X side
  - Run /run-benchmark --benchmarks=<SceneName> to collect baseline data
```

## Step 4.5 — Feature Log（强制）

迁移完成后**必须**追加 feature log + 更新 README 状态：

```bash
# scene 转 kebab-case
bash scripts/new-feature-log.sh <kebab-scene> migrated-to-benchmark
```

同步把 `features/<name>/README.md` 中"状态"字段改为 `migrated-to-benchmark`。

日志 4 段：
- 本轮目标：把 X 从 workspace 固化到 benchmark
- 改动要点：注册到 Benchmarks.kt + Benchmarks.ets，迁入路径
- 验证结果：编译通过 / hvigor 通过
- 残留：待跑 `/run-benchmark` 验证

## Step 5 — Classify & Notify（强制）

```bash
bash scripts/classify-change.sh
```

迁入通常含 `benchmarks/*/AGENTS.md` 或 `Benchmarks.kt`/`Benchmarks.ets`（注册表）—— 后者属于 business 但若同时改了 AGENTS.md 即变成 meta/mixed。任何 ≠ 0 必须把 `🔔 元变更检测` 块复述给用户。

## Rules

- Never overwrite existing benchmark files — confirm with user if target path already exists.
- Keep the benchmark file self-contained (no imports from workspace paths).
- Do not add `@Entry` to migrated components — the benchmark runner hosts them.
- Update only the `getBenchmarks()` / benchmarks array registration, not the runner logic.
