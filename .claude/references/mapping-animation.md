# Mapping — 动画与过渡

适用关键词：`animation` / `animate` / `transition` / `AnimatedVisibility` / `Crossfade` / `animateTo` / `animateFloatAsState` / `tween`。

## 核心映射

| KMP | ArkUI-X |
|---|---|
| `animateFloatAsState(target)` | `animateTo({ duration: 300, curve: Curve.EaseInOut }, () => { this.x = target })` |
| `animateDpAsState(target.dp, tween(durationMs))` | `animateTo({ duration: durationMs, curve: Curve.EaseInOut }, () => { this.x = target })` |
| `animateColorAsState(target)` | 同上，状态字段为颜色 |
| `AnimatedVisibility(visible) { Child() }` | `if (this.visible) { Child().transition({ type: TransitionType.All, opacity: 1 }) }` |
| `Crossfade(target) { state -> Content(state) }` | `if/else` 分支 + `.transition(TransitionEffect.OPACITY)` |
| `rememberInfiniteTransition().animateFloat(...)` | `animateTo` + `aboutToAppear` 启动循环（或 `DisplaySync`） |
| `Modifier.graphicsLayer { rotationZ = r }` | `.rotate({ angle: r })` |
| `Modifier.scale(s)` | `.scale({ x: s, y: s })` |
| `Modifier.alpha(a)` | `.opacity(a)` |
| `tween(durationMs, easing=FastOutSlowInEasing)` | `{ duration: durationMs, curve: Curve.FastOutSlowIn }` |

## 动画曲线（Curve）对照

| KMP Easing | ArkUI-X Curve |
|---|---|
| `LinearEasing` | `Curve.Linear` |
| `FastOutSlowInEasing` | `Curve.FastOutSlowIn` |
| `LinearOutSlowInEasing` | `Curve.LinearOutSlowIn` |
| `FastOutLinearInEasing` | `Curve.FastOutLinearIn` |
| `CubicBezierEasing(a,b,c,d)` | `cubicBezierCurve(a, b, c, d)`（API 11+） |

## 必要的最佳实践（生成时强制）

- 任何动画都必须用 `animateTo` 或 `transition`，**禁用 `setTimeout` 模拟动画**
- `animateTo` 必须带 `curve` 参数（默认 Linear 视觉效果差）
- 循环动画需在 `aboutToDisappear` 中停止（`animateTo` 异步任务、`DisplaySync.stop()` 等）
- 过渡进入/退出动画用 `transition` 装饰器，不在 `build()` 中条件渲染时手动 toggle 多个状态

## Anti-Patterns

| Pattern | Problem | Fix | Check |
|---|---|---|---|
| `setTimeout` 用作动画驱动 | 不与帧同步，掉帧严重 | 换 `animateTo` | LLM 扫描 |
| `animateTo` 不带 `curve` | 默认线性，视觉粗糙 | 加 `curve: Curve.EaseInOut` | LLM 扫描 |
| 循环动画无 `aboutToDisappear` 取消 | 组件卸载后继续触发回调 | `aboutToDisappear` 中 stop | R-RES-02 / LLM 扫描 |
| `build()` 中手动 toggle 多个状态做过渡 | 渲染抖动，无插值 | 用 `.transition(...)` 装饰器 | LLM 扫描 |
| 用 `DisplaySync` 做帧驱动但未 stop | 内存泄漏 | `aboutToDisappear` 中 `.stop()` | LLM 扫描（特殊场景） |
