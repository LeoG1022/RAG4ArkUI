# Benchmark 验证报告模板

> 用于生成KMP vs ArkUI-X跨平台对比报告

---

## 报告结构

### 1. 执行摘要

```
## 执行摘要

对比平台：KMP iOS vs ArkUI-X iOS
对比时间：YYYY-MM-DD HH:MM
测试场景：[场景列表]

总体结论：
- FPS差异：±X fps
- 平均帧时间差异：±X ms
- 掉帧率差异：±X%

关键发现：
1. [发现1]
2. [发现2]
```

### 2. 单场景对比（每个场景一节）

```markdown
## 场景：{SceneName}

### 2.1 统计对比表

| 指标 | KMP iOS | ArkUI-X iOS | 差异 | 结论 |
|------|---------|------------|------|------|
| FPS | {fps} | {fps} | ±{diff} | ✅/⚠️/❌ |
| 平均帧时间(ms) | {avg} | {avg} | ±{diff} | ✅/⚠️/❌ |
| p50帧时间(ms) | {p50} | {p50} | ±{diff} | ✅/⚠️/❌ |
| p90帧时间(ms) | {p90} | {p90} | ±{diff} | ✅/⚠️/❌ |
| p99帧时间(ms) | {p99} | {p99} | ±{diff} | ✅/⚠️/❌ |
| 掉帧数 | {missed} | {missed} | ±{diff} | ✅/⚠️/❌ |
| 掉帧率 | {ratio}% | {ratio}% | ±{diff}% | ✅/⚠️/❌ |
| 测试帧数 | {frames} | {frames} | - | - |

### 2.2 帧时间分布对比

**KMP iOS：**
```
min={minMs}ms, avg={avgMs}ms, median={medianMs}ms, max={maxMs}ms
百分位：p1={p1}ms, p5={p5}ms, p10={p10}ms, p25={p25}ms, p50={p50}ms, p75={p75}ms, p90={p90}ms, p95={p95}ms, p99={p99}ms
```

**ArkUI-X iOS：**
```
min={minMs}ms, avg={avgMs}ms, median={medianMs}ms, max={maxMs}ms
百分位：p50={p50}ms, p90={p90}ms, p95={p95}ms, p99={p99}ms
```

### 2.3 性能分析

**达标情况：**
- ✅ FPS >= 55：{达标/不达标}
- ✅ 平均帧时间 <= 16.67ms（60fps预算）：{达标/不达标}
- ⚠️ 掉帧率 <= 10%：{达标/不达标}

**瓶颈分析：**
- {瓶颈点描述}

**差异根因：**
- {差异原因分析}
```

### 3. 总体对比表

```markdown
## 总体对比

| 场景 | KMP FPS | ArkUI-X FPS | FPS差 | KMP AvgMs | ArkUI-X AvgMs | AvgMs差 | 结论 |
|------|---------|------------|-------|-----------|--------------|---------|------|
| AnimatedVisibility | {fps} | {fps} | ±{diff} | {avg} | {avg} | ±{diff} | ✅ |
| LazyGrid | {fps} | {fps} | ±{diff} | {avg} | {avg} | ±{diff} | ✅ |
| LazyList | {fps} | {fps} | ±{diff} | {avg} | {avg} | ±{diff} | ⚠️ |
| TextLayout | {fps} | {fps} | ±{diff} | {avg} | {avg} | ±{diff} | ✅ |
| CanvasDrawing | {fps} | {fps} | ±{diff} | {avg} | {avg} | ±{diff} | ✅ |
| VisualEffects | {fps} | {fps} | ±{diff} | {avg} | {avg} | ±{diff} | ⚠️ |
| HeavyShader | {fps} | {fps} | ±{diff} | {avg} | {avg} | ±{diff} | ❌ |

**结论统计：**
- ✅ 完全对齐（FPS差 < 5%，帧时间差 < 2ms）：{N}个场景
- ⚠️ 接近对齐（FPS差 5-15%，帧时间差 2-5ms）：{N}个场景
- ❌ 需优化（FPS差 > 15%，帧时间差 > 5ms）：{N}个场景
```

### 4. 改造建议

```markdown
## 改造建议

### 已完成的改造

1. ✅ 数据结构对齐（BenchmarkTypes.ets）
2. ✅ 统计指标完善（BenchmarkCalculator.ets）
3. ✅ 测量逻辑统一（BenchmarkRunner.ets）
4. ✅ 输出格式统一（JSON + hilog）
5. ✅ API对标映射文件

### 待完成的改造

1. ⚠️ 场景实现细节对齐：
   - LazyList：数据量/滚动步长
   - AnimatedVisibility：动画触发机制
   - 其他场景：确认实现完全对齐

2. ⚠️ 测量精度提升：
   - 调研ArkUI-X VSYNC API（替代setTimeout(16))
   - GPU时间测量（需API支持）

3. ⚠️ 扩展测量变体：
   - LazyGrid smoothScroll变体
   - 启动时间测量
```

---

## 数据来源

**KMP数据：**
- 文件路径：`benchmarks/KMP/benchmarks/src/commonMain/kotlin/Benchmarks.kt`
- 输出格式：BenchmarkStats JSON
- 输出标记：JSON_START / JSON_END

**ArkUI-X数据：**
- 文件路径：`benchmarks/ArkUIX/entry/src/main/ets/common/BenchmarkTypes.ets`
- 输出格式：BenchmarkStats JSON
- 输出标记：JSON_START / JSON_END（通过hilog）

**解析方式：**
```bash
# KMP iOS
cat reports/YYYY-MM-DD_HH-MM/kmp-ios.json | jq '.[] | {name, fps: .averageFPSInfo.fps, avgMs: .averageFrameInfo.cpuTimeMs}'

# ArkUI-X iOS（从hilog提取）
hilog -t BENCH -x | grep JSON_START | sed 's/JSON_START//' | jq .
```

---

## 验证脚本

```bash
#!/bin/bash
# scripts/validate-benchmark-parity.sh

KMP_JSON="reports/$1/kmp-ios.json"
ARKUIX_JSON="reports/$1/arkuix-ios.json"

if [ ! -f "$KMP_JSON" ] || [ ! -f "$ARKUIX_JSON" ]; then
  echo "❌ 缺少数据文件"
  exit 1
fi

# 对比FPS
echo "=== FPS对比 ==="
jq -r '.[] | "\(.name) \(.averageFPSInfo.fps)"' "$KMP_JSON" | sort
jq -r '.[] | "\(.name) \(.averageFPSInfo.fps)"' "$ARKUIX_JSON" | sort

# 对比平均帧时间
echo "=== 平均帧时间对比 ==="
jq -r '.[] | "\(.name) \(.averageFrameInfo.totalTimeMs)"' "$KMP_JSON" | sort
jq -r '.[] | "\(.name) \(.averageFrameInfo.totalTimeMs)"' "$ARKUIX_JSON" | sort

# 统计达标率
echo "=== 达标统计 ==="
# ... （待实现）
```

---

**维护者：** Claude Code Agent  
**版本：** v1.0  
**创建时间：** 2026-05-19