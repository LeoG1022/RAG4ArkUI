# Mapping — 异步、网络、资源、格式化、边角 case

适用关键词：`coroutine` / `suspend` / `await` / `Dispatchers` / `Flow` / `http` / `network` / `taskpool` / `resource` / `math` / `format`。

## Coroutines → taskpool / async

| KMP | ArkUI-X |
|---|---|
| `viewModelScope.launch { suspend block }` | `taskpool.execute(() => { block })` 或 `async/await` |
| `withContext(Dispatchers.IO) { ... }` | `taskpool.execute(...)`（默认在 worker 线程） |
| `withContext(Dispatchers.Default) { ... }` | `taskpool.execute(new taskpool.Task(fn))` |
| `delay(ms)` | `await new Promise(r => setTimeout(r, ms))` |
| `coroutineScope { ... }` | `Promise.all([...])` |
| `Flow.collect { v -> ... }` | 自实现 observer / 用 EventBus |
| `MutableStateFlow.update {}` | 直接修改 `@State` 字段 |

### taskpool 标准模式（必须 cancel）

```typescript
private task: taskpool.Task | null = null

aboutToAppear(): void {
  this.task = new taskpool.Task(heavyWork)
  taskpool.execute(this.task).then(result => { /* ... */ })
}

aboutToDisappear(): void {
  if (this.task) { taskpool.cancel(this.task); this.task = null }
}
```

## 网络

| KMP | ArkUI-X |
|---|---|
| `Ktor HttpClient { get { } }` | `@ohos.net.http`：`http.createHttp()` + `.request()` + 必 `.destroy()` |
| `httpClient.close()` | `client.destroy()` |
| JSON 序列化（kotlinx.serialization） | `JSON.parse()` / `JSON.stringify()` 或 `@kit.ArkTS` 内置 |

### http 标准模式（必须 destroy）

```typescript
async aboutToAppear() {
  const client = http.createHttp()
  try {
    const resp = await client.request(url, { method: http.RequestMethod.GET })
    // ...
  } finally {
    client.destroy()
  }
}
```

## 资源访问

| KMP | ArkUI-X |
|---|---|
| `painterResource("drawable/foo.png")` | `$r('app.media.foo')` |
| `stringResource(R.string.hello)` | `$r('app.string.hello')` |
| 原始 assets 文件 | `$rawfile('foo.json')` 或 `resourceManager.getRawFileContent('foo.json')` |
| `LocalContext.current.assets.open(...)` | `getContext(this).resourceManager.getRawFileContent(...)` |

## 本地存储

| KMP | ArkUI-X |
|---|---|
| `DataStore<Preferences>` | `@ohos.data.preferences` |
| Room 数据库 | `@ohos.data.relationalStore` |
| `runBlocking { dataStore.edit { ... } }` | `await preferences.put('key', v); await preferences.flush()` |

## 数字与字符串格式

| KMP | ArkUI-X |
|---|---|
| `"%.2f".format(x)` | `x.toFixed(2)` |
| `String.format("%03d", n)` | `n.toString().padStart(3, '0')` |
| `kotlin.math.PI` | `Math.PI` |
| `kotlin.math.abs(x)` | `Math.abs(x)` |
| `kotlin.math.sin(x)` / `cos` / `sqrt` | `Math.sin(x)` / `cos` / `sqrt` |
| `Random.nextInt(n)` | `Math.floor(Math.random() * n)` |

## 日期时间

| KMP | ArkUI-X |
|---|---|
| `Clock.System.now()` | `new Date()` |
| `Instant.parse(s)` | `new Date(s)` |
| `kotlinx.datetime` | 用 JS `Date` 或第三方库 |

## 必要的最佳实践（生成时强制）

- **任何异步资源都必须显式释放** —— 这是 KMP→ArkUI-X 最常见的泄漏来源
- `setInterval` 必须存 handle 并配 `clearInterval`
- `taskpool.execute` 创建的 Task 必须在 `aboutToDisappear` 中 `cancel`
- `http.createHttp()` 必须在 `finally` 中 `destroy()`
- `DisplaySync.start()` 必须配 `stop()`

## Anti-Patterns

| Pattern | Problem | Fix | Check |
|---|---|---|---|
| `setInterval` 未存 handle / 未 clearInterval | Timer 泄漏，累计 OOM | 存 handle，`aboutToDisappear` 调 `clearInterval` | `check-api-parity.sh` → R-RES-01 |
| `taskpool.execute` 未保留 task 引用 / 未 cancel | 组件卸载后任务继续 | 存 Task，`aboutToDisappear` 调 `taskpool.cancel(task)` | R-RES-02 |
| `http.createHttp()` 无 `.destroy()` | 连接句柄泄漏 → 连接池耗尽 | `try / finally { client.destroy() }` | LLM 扫描 |
| `build()` 中含 `await` | 阻塞渲染或编译报错 | 移到 `aboutToAppear` / 事件处理 | P-RENDER-01 |
| `DisplaySync.start()` 无 `.stop()` | 帧回调持续触发，OOM | `aboutToDisappear` 调 `displaySync.stop()` | LLM 扫描 |
| 异步回调中直接修改 `this.@State` 但组件已 disappear | 引用已释放对象，崩溃 | 在 `aboutToDisappear` 设 `isDisposed` 标志位，回调判断 | LLM 扫描 |
