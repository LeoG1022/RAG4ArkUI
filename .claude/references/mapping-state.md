# Mapping — 状态、Effect 与生命周期

适用关键词：`state` / `remember` / `mutableStateOf` / `LaunchedEffect` / `DisposableEffect` / `lifecycle` / `Effect` / `@State` / `@Prop` / `@Link`。

## 核心映射

| KMP | ArkUI-X |
|---|---|
| `remember { mutableStateOf(v) }` | `@State x: T = v` |
| `remember { v }` | `private readonly x: T = v` |
| `derivedStateOf { expr }` | `@Computed get x(): T { return expr }` |
| 函数参数传值（自动 recompose） | `@Prop x: T` |
| 回调 lambda `(T) -> Unit` | `@Link x: T`（双向）或回调 prop |
| `CompositionLocalProvider + Local.current` | `@Provide('key') v` + `@Consume('key') x` |
| `StateFlow + collectAsState()` | `AppStorage.SetAndLink('key', ...)` 或自定义 observer |
| `LaunchedEffect(key) { block }` | `aboutToAppear() { block }`（一次性）或 watch state 变化 |
| `LaunchedEffect(Unit) { while(isActive) {} }` | `aboutToAppear` + `aboutToDisappear` 配对取消（无自动取消） |
| `DisposableEffect { onDispose { cleanup } }` | `aboutToDisappear() { cleanup }` |
| `SideEffect { block }` | 在 `build()` 渲染前直接调用 `block` |
| `rememberCoroutineScope()` | 私有 `taskpool` 实例 + `aboutToDisappear` 取消 |
| `@Stable` data class | `@ObservedV2 class` + 字段加 `@Trace` |

## 状态选择决策（生成时按场景选）

| 场景 | 装饰器 |
|---|---|
| 组件内私有状态 | `@State` |
| 父→子 单向传值 | `@Prop`（子端） |
| 父↔子 双向绑定 | `@Link`（子端） + 父端用 `$` 前缀 |
| 跨层级传递 | `@Provide` / `@Consume` |
| 全局共享 | `AppStorage.SetAndLink('key', default)` |
| 大对象 + 部分字段变化 | `@ObservedV2 class` + `@Trace` 标变化字段 |
| 大对象嵌套 | 父用 `@State`，子用 `@ObjectLink` |

## 必要的最佳实践（生成时强制）

- `@State` 作用域放最窄；大状态拆多个 `@State` 或用 `@ObservedV2`
- 一次性初始化放在 `aboutToAppear()`，禁止放在 `build()`
- 衍生值用 `@Computed get`，禁止在 `build()` 中实时计算
- `aboutToAppear` 内启动的任何长时任务，必须有 `aboutToDisappear` 中的对应取消
- 不要在子组件内对父传入的 `@Prop` 修改（应改用 `@Link` 或回调）

## Anti-Patterns

| Pattern | Problem | Fix | Check |
|---|---|---|---|
| `@State` 数组按 index 赋值（`this.arr[0] = x`） | 不触发刷新 | 用 `splice` 或整体赋新数组 `this.arr = [...this.arr]` | `check-api-parity.sh` → S-STATE-01 |
| `build()` 中含 `await` | 阻塞渲染或编译报错 | 移到 `aboutToAppear` 或事件处理 | P-RENDER-01 |
| `build()` 中昂贵计算（filter / sort / 大循环） | 每帧重算 | 提到 `@Computed` 或 `aboutToAppear` | LLM 扫描 |
| `build()` 中重建 `ArrayDataSource` / new 对象 | 每帧 GC 压力 | 提到 `@State` 字段 | LLM 扫描 |
| 顶层全局 `mutableStateListOf()` / 全局可变 | 多实例共享，无法 GC | 移入组件 `@State` 或 `AppStorage` | LLM 扫描 |
| `@Prop` 在子组件内修改 | 不会同步回父 | 改用 `@Link` 或回调 | LLM 扫描 |
| `@Observed` 嵌套对象未标 `@Observed` | 嵌套层不响应变化 | 内层 class 也加 `@Observed` | LLM 扫描 |
| `taskpool` 任务未 `aboutToDisappear` 中 cancel | 组件卸载后任务继续 | 存 task 引用，`aboutToDisappear` 调 `.cancel()` | R-RES-02 |
| `setInterval` 未 `clearInterval` | Timer 泄漏 → OOM | 存 handle，`aboutToDisappear` 调 `clearInterval` | R-RES-01 |
