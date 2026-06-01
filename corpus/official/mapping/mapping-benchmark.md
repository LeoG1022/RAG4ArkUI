# Benchmark API 对标映射

> 适用关键词：`benchmark` / `performance` / `fps` / `frame time` / `measurement` / `statistics`

---

## 一、数据结构对标

### 1.1 BenchmarkStats 对标

| KMP (Kotlin) | ArkUI-X (ArkTS) | 说明 |
|--------------|-----------------|------|
| `BenchmarkStats` | `BenchmarkStats` | 统计结果容器，字段对齐 |
| `BenchmarkConditions` | `BenchmarkConditions` | 测量条件（帧数/预热帧数） |
| `FrameInfo` | `FrameInfo` | CPU/GPU时间分离（ArkUI-X GPU暂为0） |
| `FPSInfo` | `FPSInfo` | FPS信息 |
| `BenchmarkPercentileAverage` | `BenchmarkPercentileAverage` | 百分位统计 |
| `MissedFrames` | `MissedFrames` | 掉帧统计 |
| `StartupTimeInfo` | `StartupTimeInfo` | 启动时间分析 |

### 1.2 测量方法对标

| KMP API | ArkUI-X API | 精度 | 对标方案 |
|---------|------------|------|----------|
| `withFrameNanos` | `setTimeout(16)` | 纳秒 vs 毫秒 | ArkUI-X用setTimeout，但需记录帧间隔 |
| `TimeSource.Monotonic.markNow()` | `Date.now()` | 纳秒 vs 毫秒 | ArkUI-X用Date.now()，精度足够 |
| `GraphicsContext.awaitGPUCompletion()` | ❌ 缺失 | GPU时间测量 | 暂不测量，gpuTimeMs=0 |
| `measureTime { }` | 手动计算 | - | ArkUI-X手动计算帧间隔 |

---

## 二、测量逻辑对标

### 2.1 基础测量流程

**KMP流程：**
```kotlin
// 1. VSYNC同步
withFrameNanos { frameTimeNanos -> 
    // 在VSYNC回调中精确测量
}

// 2. 预热阶段
repeat(warmupCount) {
    withFrameNanos { }
}

// 3. 测量阶段
repeat(frameCount) {
    val frameStart = TimeSource.Monotonic.markNow()
    withFrameNanos { }
    val frameTime = frameStart.elapsedNow()
    frames[it] = BenchmarkFrame(frameTime, Duration.ZERO)
}
```

**ArkUI-X流程：**
```typescript
// 1. 定时器模拟帧回调
setTimeout(() => {
    const now = Date.now()
    const frameTime = now - lastFrameTime
    lastFrameTime = now
    
    // 2. 预热阶段
    if (tickCount < warmupCount) {
        onFrame()
        tickCount++
        scheduleNextFrame()
    } else {
        // 3. 测量阶段
        frameTimes.push(frameTime)
        onFrame()
        tickCount++
        
        if (frameTimes.length < measureCount) {
            scheduleNextFrame()
        } else {
            finish()
        }
    }
}, 16)
```

### 2.2 统计计算对标

**KMP计算：**
```kotlin
// 百分位
val sorted = frames.sortedBy { it.cpuDuration }
val p50 = sorted[sorted.size * 0.50]
val p99 = sorted[sorted.size * 0.99]

// 掉帧
val missed = frames.count { it.cpuDuration > frameBudget }

// FPS
val fps = frameCount / (totalTime.inWholeMilliseconds / 1000.0)
```

**ArkUI-X计算（BenchmarkCalculator.ets）：**
```typescript
// 百分位（线性插值）
static percentile(sorted: number[], p: number): number {
    const index = p * (sorted.length - 1)
    const lower = Math.floor(index)
    const upper = Math.ceil(index)
    const weight = index - lower
    return sorted[lower] * (1 - weight) + sorted[upper] * weight
}

// 掉帧
static countMissedFrames(sorted: number[], budgetMs: number): MissedFrames {
    const count = sorted.filter(t => t > budgetMs).length
    return { count, ratio: count / sorted.length }
}

// FPS
const fps = len / (sum / 1000)
```

---

## 三、输出格式对标

### 3.1 JSON结构对标

**KMP JSON：**
```json
{
  "name": "LazyGrid",
  "frameBudget": "8.33ms",
  "conditions": {
    "frameCount": 300,
    "warmupCount": 60
  },
  "averageFrameInfo": {
    "cpuTime": "12.5ms",
    "gpuTime": "3.2ms",
    "totalTime": "15.7ms"
  },
  "averageFPSInfo": {
    "fps": 63.5
  },
  "percentileCPUAverage": [
    { "percentile": 0.01, "average": "25.3ms" },
    { "percentile": 0.50, "average": "12.0ms" }
  ],
  "noBufferingMissedFrames": {
    "count": 45,
    "ratio": 0.15
  }
}
```

**ArkUI-X JSON（对齐后）：**
```json
{
  "name": "LazyGrid",
  "frameBudgetMs": 16.67,
  "conditions": {
    "frameCount": 300,
    "warmupCount": 60
  },
  "averageFrameInfo": {
    "cpuTimeMs": 12.5,
    "gpuTimeMs": 0,
    "totalTimeMs": 12.5
  },
  "averageFPSInfo": {
    "fps": 80.0
  },
  "percentileCPUAverage": [
    { "percentile": 0.01, "averageMs": 25.3 },
    { "percentile": 0.50, "averageMs": 12.0 }
  ],
  "noBufferingMissedFrames": {
    "count": 45,
    "ratio": 0.15
  },
  "frameTimesMs": [12.5, 13.2, ...]
}
```

### 3.2 输出渠道对标

| KMP | ArkUI-X | 说明 |
|-----|---------|------|
| `println()` | `hilog.info()` | 标准输出 |
| `JSON_START` / `JSON_END` marker | `JSON_START` / `JSON_END` marker | JSON块标记 |
| 文件写入（iOS） | hilog（暂无文件写入） | 持久化 |

---

## 四、Benchmark场景对标

### 4.1 LazyGrid对标

| 维度 | KMP | ArkUI-X | 对标要求 |
|------|-----|---------|----------|
| 数据量 | 12000项 | 12000项 | ✅ 已对齐 |
| 列数 | 4列 | 4列 | ✅ 已对齐 |
| 滚动方式 | smoothScroll=true时每帧55px | 每帧55px | ✅ 已对齐 |
| 测量变体 | 4种（smooth/launchedEffect组合） | 1种 | ⚠️ 需补充3种变体 |
| 组件 | `LazyVerticalGrid` + `GridCells.Fixed` | `Grid` + `columnsTemplate` | ✅ API对标 |

**改造要点：**
- ✅ 数据量已对齐
- ⚠️ 需添加smoothScroll变体（带/不带LaunchedEffect）

### 4.2 LazyList对标

| 维度 | KMP | ArkUI-X | 对标要求 |
|------|-----|---------|----------|
| 数据量 | 动态加载（无限） | 500项 | ⚠️ 需对齐为动态加载 |
| 滚动方式 | scrollToItem（步长50） | scrollTo（步长100） | ⚠️ 需对齐步长 |
| 列表项 | 带图片、文本、图标 | 仅文本 | ⚠️ 需增加复杂度 |
| 搜索功能 | ✅ 有 | ❌ 缺失 | ⚠️ 需补充 |

**改造要点：**
- ⚠️ 需改造成动态加载（IDataSource）
- ⚠️ 需增加列表项复杂度（图片+文本+图标）
- ⚠️ 需对齐滚动步长为50

### 4.3 AnimatedVisibility对标

| 维度 | KMP | ArkUI-X | 对标要求 |
|------|-----|---------|----------|
| 动画API | `AnimatedVisibility` + `animateEnterExit` | 手动opacity动画 | ⚠️ 需改用transition |
| 动画时长 | 300ms | 300ms | ✅ 已对齐 |
| 动画内容 | 单张Compose Multiplatform图 | 25个Stack叠加 | ⚠️ 需简化为单图 |
| 触发方式 | LaunchedEffect定时触发 | setInterval定时触发 | ⚠️ 需对齐触发方式 |

**改造要点：**
- ⚠️ 需使用ArkUI-X的`transition` API对标Compose的`AnimatedVisibility`
- ⚠️ 动画内容需简化为单图（与KMP对齐）
- ⚠️ 触发方式需对齐（interval时长）

---

## 五、最佳实践（强制）

### 5.1 测量前必做

1. ✅ **预热阶段**：至少60帧预热，避免冷启动影响
2. ✅ **测量帧数**：至少300帧测量，确保统计稳定
3. ✅ **帧预算**：基于60FPS（16.67ms）作为掉帧标准

### 5.2 统计必含

1. ✅ **百分位统计**：p50/p90/p95/p99（至少这4个）
2. ✅ **掉帧统计**：掉帧数 + 掉帧率
3. ✅ **FPS**：平均FPS
4. ✅ **平均帧时间**：平均帧耗时

### 5.3 输出必做

1. ✅ **JSON输出**：结构化JSON，机器可读
2. ✅ **标记包裹**：JSON_START / JSON_END marker
3. ✅ **日志输出**：hilog输出关键指标（fps/avg_ms/p50/p99/missed）

---

## 六、Anti-Patterns

| Pattern | Problem | Fix | Check |
|---------|---------|-----|-------|
| `setInterval(() => onFrame(), 16)` | 无法精确测量帧间隔，误差±4ms | 改为记录实际帧间隔：`const frameTime = now - lastFrameTime` | LLM扫描 |
| 缺少百分位统计 | 无法评估尾部延迟 | 使用`BenchmarkCalculator.percentile()` | grep `p50Ms/p90Ms/p99Ms` |
| 缺少掉帧检测 | 无法量化流畅度 | 使用`BenchmarkCalculator.countMissedFrames()` | grep `missedFrames` |
| 输出文本而非JSON | 无法自动化对比 | 使用`JSON.stringify()` + JSON_START/END marker | grep `JSON_START` |
| GPU时间硬编码为0 | 与KMP数据结构不对齐 | 在FrameInfo中设gpuTimeMs=0，备注说明API限制 | grep `gpuTimeMs: 0` |
| 场景不对齐（数据量/复杂度） | 跨平台对比无意义 | 按本mapping第四节对标要求调整 | LLM扫描 + 手动对比 |

---

## 七、扩展机制

### 7.1 新增Benchmark场景

1. 继承`BenchmarkRunner`
2. 实现`name`、`onComplete`、`onFrame()`
3. 在`Index.ets`中注册到`BenchmarkScene()`
4. 在`BENCHMARK_NAMES`数组中添加场景名

### 7.2 新增统计指标

1. 在`BenchmarkTypes.ets`中扩展接口
2. 在`BenchmarkCalculator.ets`中实现计算逻辑
3. 在`BenchmarkRunner.ets`中输出新指标

### 7.3 新增测量方法

若ArkUI-X未来提供VSYNC API：
1. 替换`setTimeout(16)`为`onFrameReady(callback)`
2. 在`BenchmarkRunner.ets`中修改`scheduleNextFrame()`
3. 更新本mapping文件

---

## 八、验证清单

改造完成后必须验证：

1. ✅ **数据结构对齐**：BenchmarkStats字段与KMP一致
2. ✅ **统计指标齐全**：百分位、掉帧、FPS、帧时间
3. ✅ **输出格式统一**：JSON结构化输出
4. ✅ **场景对齐**：数据量、复杂度、滚动方式与KMP一致
5. ✅ **测量精度**：帧间隔记录正确（非硬编码16ms）
6. ✅ **预热阶段**：至少60帧预热
7. ✅ **测量帧数**：至少300帧测量

验证命令：
```bash
bash scripts/check-api-parity.sh
```

---

## Anti-Patterns

| Pattern | Problem | Fix | Check |
|---------|---------|-----|-------|
| `setInterval(() => onFrame(), 16)` | 无法精确测量帧间隔，误差±4ms | 改为记录实际帧间隔：`const frameTime = now - lastFrameTime` | LLM扫描 |
| 缺少百分位统计 | 无法评估尾部延迟 | 使用`BenchmarkCalculator.percentile()` | grep `p50Ms/p90Ms/p99Ms` |
| 缺少掉帧检测 | 无法量化流畅度 | 使用`BenchmarkCalculator.countMissedFrames()` | grep `missedFrames` |
| 输出文本而非JSON | 无法自动化对比 | 使用`JSON.stringify()` + JSON_START/END marker | grep `JSON_START` |
| GPU时间硬编码为0 | 与KMP数据结构不对齐 | 在FrameInfo中设gpuTimeMs=0，备注说明API限制 | grep `gpuTimeMs: 0` |
| 场景不对齐（数据量/复杂度） | 跨平台对比无意义 | 按本mapping第四节对标要求调整 | LLM扫描 + 手动对比 |
| setInterval未配对clearInterval | Timer泄漏导致内存泄漏 | 在aboutToDisappear中调用clearInterval | R-RES-01 (check-api-parity.sh) |
| BenchmarkRunner未继承 | 测量逻辑不统一 | 继承BenchmarkRunner抽象类，实现name/onComplete/onFrame | grep `extends BenchmarkRunner` |
| 缺少预热阶段 | 冷启动影响测量精度 | 使用BenchmarkRunner.start()（内置60帧预热） | grep `WARMUP_FRAMES` |
| 测量帧数不足 | 统计不稳定 | 使用BenchmarkRunner（内置300帧测量） | grep `MEASURE_FRAMES` |

---

**维护者：** Claude Code Agent  
**最后更新：** 2026-05-19  
**版本：** v1.0