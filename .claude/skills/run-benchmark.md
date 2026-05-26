---
name: run-benchmark
version: 1.1.0
trigger: /run-benchmark
description: 自动编译/安装/运行 benchmark + 产出 comparison.md（只读豁免 feature log）
feature_log_required: false
classify_required: true
preflight_required: true
calls:
  - scripts/preflight.sh
  - scripts/classify-change.sh
references:
  - .claude/references/arkuix-best-practices.md
---

# Skill: /run-benchmark

Automate compile → install → run → collect → report for all benchmark targets.

> **执行前必须**遵守 [`AGENTS.md`](../../AGENTS.md) 的"通用前置协议（Git 状态检查）"。
> （本 skill 不写源码但写 `reports/`，仍属写操作。）

## Trigger
User types `/run-benchmark` optionally followed by:
- `--platform <android|ios|harmony|native-android|native-ios|all>` (default: `all`)
- `--benchmarks <LazyList,AnimatedVisibility,...>` (default: all registered scenes)
- `--out <path>` — report output directory (default: `reports/<YYYY-MM-DD_HH-MM>/`)

## Execution Plan

Execute the following steps in order. On any failure, report the error and ask the user whether to continue with remaining platforms.

---

### Step 0 — Preflight（强制）

```bash
bash scripts/preflight.sh
```

git 非干净 → 按 [`AGENTS.md`](../../AGENTS.md) 规则 13 询问用户。

---

### Step 1 — Setup

```bash
DATE=$(date +%Y-%m-%d_%H-%M)
REPORT_DIR="reports/$DATE"
mkdir -p "$REPORT_DIR"
FILTER=""  # set to "--es args benchmarks=LazyList,..." if --benchmarks was specified
```

---

### Step 2 — KMP Android

```bash
cd benchmarks/KMP
./gradlew :androidApp:assembleRelease

adb install -r androidApp/build/outputs/apk/release/androidApp-release.apk

# Clear previous results
adb shell rm -rf /sdcard/benchmarks/kmp/

# Launch benchmark
adb shell am start \
  -n org.jetbrains.benchmarks/.BenchmarkActivity \
  --es args "saveStatsToJSON=true $FILTER"

# Wait for completion (poll for result file)
echo "Waiting for KMP Android benchmark to finish..."
until adb shell ls /sdcard/benchmarks/kmp/ 2>/dev/null | grep -q "\.json"; do sleep 5; done

# Pull results
adb pull /sdcard/benchmarks/kmp/ "$REPORT_DIR/kmp-android/"
```

---

### Step 3 — KMP iOS (Simulator)

```bash
cd benchmarks/KMP

# Build framework
./gradlew :benchmarks:linkReleaseFrameworkIosSimulatorArm64

# Build Xcode app
xcodebuild \
  -project iosApp/iosApp.xcodeproj \
  -scheme iosApp \
  -configuration Release \
  -destination 'platform=iOS Simulator,name=iPhone 15' \
  build

# Install + launch on booted simulator
xcrun simctl install booted \
  $(find ~/Library/Developer/Xcode/DerivedData -name "iosApp.app" -path "*/Release-iphonesimulator/*" | head -1)

xcrun simctl launch --console booted org.example.kmpbenchmarks \
  2>&1 | tee "$REPORT_DIR/kmp-ios.json"
```

---

### Step 4 — ArkUI-X HarmonyOS

```bash
cd benchmarks/ArkUI-X

# Build HAP
hvigor assembleHap --mode release

# Install
hdc install entry/build/outputs/default/entry-default.hap

# Clear previous results
hdc shell rm -rf /data/app/el2/100/base/com.example.benchmarks/files/benchmarks/

# Launch
hdc shell aa start \
  -a EntryAbility \
  -b com.example.benchmarks \
  -p "saveStatsToJSON=true $FILTER"

echo "Waiting for ArkUI-X HarmonyOS benchmark..."
until hdc shell ls /data/app/el2/100/base/com.example.benchmarks/files/benchmarks/ 2>/dev/null | grep -q "\.json"; do sleep 5; done

hdc file recv \
  /data/app/el2/100/base/com.example.benchmarks/files/benchmarks/ \
  "$REPORT_DIR/arkuix-harmony/"
```

---

### Step 5 — ArkUI-X Android

```bash
cd benchmarks/ArkUI-X/.arkui-x/android
./gradlew app:assembleRelease

adb install -r app/build/outputs/apk/release/app-release.apk
adb shell rm -rf /sdcard/benchmarks/arkuix-android/
adb shell am start \
  -n com.example.benchmarks/.BenchmarkActivity \
  --es args "saveStatsToJSON=true $FILTER"

until adb shell ls /sdcard/benchmarks/arkuix-android/ 2>/dev/null | grep -q "\.json"; do sleep 5; done
adb pull /sdcard/benchmarks/arkuix-android/ "$REPORT_DIR/arkuix-android/"
```

---

### Step 6 — ArkUI-X iOS

ArkUI-X iOS benchmark 有两种模式：**真机**（推荐）和**模拟器**（无真机时备用）。

#### 6A — 真机模式（逐用例编译 + 运行）

ArkUI-X iOS 真机 benchmark 使用专用脚本 `bench-all-ios.sh`，逐用例编译并运行。因为 ArkUI-X iOS app 的 `BENCHMARK_NAMES` 需要编译时确定，脚本会在每次运行前 patch 该常量为单个用例。

**前置条件**：
- 真机 USB 连接 + 信任开发者
- ArkUI-X SDK + OpenHarmony SDK 已安装（`DEVECO_SDK_HOME` 或 `/tmp/arkuix-sdk`）
- Xcode 已安装

**JSON 输出通道**：ArkUI-X `hilog`/`console.error()` 在 iOS 真机 `devicectl --console` 下不可见，因此使用 **Bridge API + fprintf(stderr)** 通道：
- `BenchmarkRunner.ets` 通过 `@arkui-x.bridge` 的 `Bridge.sendMessage` 发送 JSON_START/jsonStr/JSON_END
- `EntryEntryAbilityViewController.m` 的 `IMessageListener.onMessage` 接收并 `fprintf(stderr)` + 写文件到 Documents 目录
- Shell 脚本从 devicectl console 输出中提取 JSON，或在 Documents 取回

**运行全部 5 用例（1 轮验证）**：
```bash
bash scripts/bench-all-ios.sh --runs 1 --device <UDID>
```

**运行全部 5 用例（5 轮正式）**：
```bash
bash scripts/bench-all-ios.sh --runs 5 --device <UDID>
```

**运行指定子集**：
```bash
bash scripts/bench-all-ios.sh --runs 3 --cases LazyGrid,LazyList --device <UDID>
```

**5 个标准用例**：LazyGrid / LazyGrid-ItemLaunchedEffect / LazyGrid-SmoothScroll / LazyGrid-SmoothScroll-ItemLaunchedEffect / LazyList

**脚本内部流程（每个用例）**：
1. `patch_benchmark_names()` → 将 `BenchmarkTypes.ets` 的 `BENCHMARK_NAMES` 改为 `['<Case>']`
2. `hvigor assembleApp` → 编译 .ets → .abc + 生成 iOS framework
3. `benchmark-ios-arkuix.sh run -b <Case>` → xcodebuild + install + devicectl console capture
4. JSON 保存到 `reports/ios/<Case>/run{N}.json`

**特殊情况**：
- `LazyGrid-ItemLaunchedEffect` ITEM_COUNT=100（其他 12000），避免 `setInterval` per cell 导致 iOS OOM（signal 9）
- Bridge double-delivery：`sendMessage` 触发 `onMessage` 两次，脚本用 `JSON_SAVED_THIS_RUN` flag 防止重复保存

**查看设备 UDID**：
```bash
bash scripts/benchmark-ios-arkuix.sh devices
```

#### 6B — 模拟器模式（备用）

```bash
cd benchmarks/ArkUI-X
hvigor assembleApp --target ios --mode release
xcodebuild \
  -project .arkui-x/ios/app.xcodeproj \
  -scheme app \
  -configuration Release \
  -destination 'platform=iOS Simulator,name=iPhone 15' \
  build
xcrun simctl install booted <path-to-app.app>
xcrun simctl launch --console booted com.arkuix.benchmarks 2>&1 | tee "$REPORT_DIR/arkuix-ios.json"
```

> **注意**：模拟器模式下 Bridge fprintf(stderr) 通道同样可用，但性能数据不具备真机代表性。

---

### Step 7 — Native Android

```bash
cd benchmarks/nativeAndroidApp
./gradlew app:assembleRelease
adb install -r app/build/outputs/apk/release/app-release.apk
adb shell rm -rf /sdcard/benchmarks/native-android/
adb shell am start \
  -n com.example.nativebenchmarks/.BenchmarkActivity \
  --es args "saveStatsToJSON=true $FILTER"
until adb shell ls /sdcard/benchmarks/native-android/ 2>/dev/null | grep -q "\.json"; do sleep 5; done
adb pull /sdcard/benchmarks/native-android/ "$REPORT_DIR/native-android/"
```

---

### Step 8 — Native iOS

```bash
xcodebuild \
  -project benchmarks/nativeIosApp/nativeIosApp.xcodeproj \
  -scheme nativeIosApp \
  -configuration Release \
  -destination 'platform=iOS Simulator,name=iPhone 15' \
  build

xcrun simctl install booted \
  $(find ~/Library/Developer/Xcode/DerivedData -name "nativeIosApp.app" -path "*/Release-iphonesimulator/*" | head -1)

xcrun simctl launch --console booted com.example.nativeiosbenchmarks \
  2>&1 | tee "$REPORT_DIR/native-ios.json"
```

---

### Step 9 — Classify & Notify（强制）

完成所有报告写入后：

```bash
bash scripts/classify-change.sh
```

`/run-benchmark` 通常只往 `reports/` 写文件（business），但若运行时改了脚本或配置即变成 meta。任何 ≠ 0 必须复述 `🔔 元变更检测` 块。

### Step 9.5 — Multi-iteration Aggregation（自 Round 12 起强制）

为得到统计意义上严肃的数据，每个 (platform, scene) 应跑 **N=5** 轮（可通过 `--iterations N` 覆盖，最小 3）。每轮独立完成 install + run + 收集 JSON。

完成所有轮次后：

```bash
# 把同一 (platform, scene) 的 N 份 JSON 合并聚合
bash scripts/benchmark-aggregate.sh $REPORT_DIR/kmp-android/*.json > $REPORT_DIR/kmp-android-aggregated.json
bash scripts/benchmark-aggregate.sh $REPORT_DIR/arkuix-android/*.json > $REPORT_DIR/arkuix-android-aggregated.json
# ... 每个 platform 一次

# 与上一次 baseline 对比（自动找 reports/ 下倒数第二份）
bash scripts/benchmark-regression.sh --auto-baseline $REPORT_DIR/kmp-android-aggregated.json
```

聚合输出含 `n / mean / stddev / p50 / p90 / p99 / ci95_lo / ci95_hi / flakiness`。
regression 退出码：0 OK / 1 FAIL（>10% 下降）/ 2 WARN（>5% 下降）。

### Step 10 — Generate Comparison Report

Read all aggregated JSON files from `$REPORT_DIR/*-aggregated.json`. For each benchmark scene:
1. Extract `mean`, `stddev`, `p99`, `flakiness` per (scene, metric, platform).
2. 调 `benchmark-regression.sh` 输出 Δ vs baseline 列。
3. Compute delta% vs native baseline: `(kmp.mean - native.mean) / native.mean * 100`.
4. Write `$REPORT_DIR/comparison.md` in this format:

```markdown
# Benchmark Comparison Report — <DATE>

## Environment
- Android device: (from adb shell getprop ro.product.model)
- iOS simulator: iPhone 15
- HarmonyOS device: (from hdc shell param get const.product.model)

## <SceneName>

| Metric | KMP Android | ArkUI-X Android | Native Android | KMP iOS | ArkUI-X iOS | Native iOS | ArkUI-X HarmonyOS |
|---|---|---|---|---|---|---|---|
| Avg frame (ms) ± stddev | x.x ± y | x.x ± y | x.x ± y | x.x ± y | x.x ± y | x.x ± y | x.x ± y |
| FPS | xxx | xxx | xxx | xxx | xxx | xxx | xxx |
| p50 (ms) | x.x | x.x | x.x | x.x | x.x | x.x | x.x |
| p99 (ms) | x.x | x.x | x.x | x.x | x.x | x.x | x.x |
| Missed frames | n | n | n | n | n | n | n |
| vs Native | +x% | +x% | — | +x% | +x% | — | N/A |
| Δ vs last run | +x% 🟡 | OK | OK | +x% 🔴 | OK | OK | OK |
| Flakiness | OK | OK | OK | ⚠️ HIGH_VARIANCE | OK | OK | OK |
```

**评级标记**：
- 🟢 OK：性能下降 < 5%
- 🟡 WARN：性能下降 5–10%（建议关注）
- 🔴 FAIL：性能下降 > 10%（需调查）
- ⚠️ HIGH_VARIANCE：stddev/mean > 0.15（样本不稳定，建议增加 iterations）

---

## Notes

- If a platform is unavailable (device not connected, SDK not installed), skip that step and note it in the report.
- Benchmark apps must have `android.permission.WRITE_EXTERNAL_STORAGE` (Android < API 29) or use app-specific storage.
- On HarmonyOS, results are in app sandbox; `hdc file recv` requires a connected HarmonyOS device with USB debugging enabled.
- iOS simulator results are captured via `--console` stdout; the app must print JSON to stdout or save to Documents directory.
