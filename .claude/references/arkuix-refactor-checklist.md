# ArkUI-X 重构检查清单

`/arkuix-refactor` 加载本文件作为完整扫描规则。每条 issue 都带稳定 ID（P*/M*/R*/A*），用户可按 ID 引用。

---

## 性能 Performance

| ID | 模式 | 问题 | 修复 |
|---|---|---|---|
| P1 | `ForEach` 用于 >100 项列表 | 任何变化都全量重渲染 | 换 `LazyForEach` + `IDataSource` |
| P2 | `ForEach` / `LazyForEach` 缺 `keyGenerator` | 全量重建 | 加 `(item) => item.id.toString()` 第三参 |
| P3 | `List` 缺 `.cachedCount()` | 滚动掉帧 | 加 `.cachedCount(3)` |
| P4 | `Grid` 缺 `.cachedCount()` | 滚动掉帧 | 加 `.cachedCount(4)` |
| P5 | `Image` 缺 `.objectFit()` | 内存浪费 / 视觉变形 | 加 `.objectFit(ImageFit.Cover)` |
| P6 | 动画用 `setTimeout` | 不与帧同步 | 换 `animateTo` |
| P7 | `build()` 中重建 `ArrayDataSource` | 每帧分配新对象 | 提到 `@State` 或字段 |
| P8 | 昂贵计算在 `build()` 中 | 每次渲染重算 | 提到 `@Computed` 或 `aboutToAppear` |
| P9 | 列表项 `@Component` 缺 `@Reusable` | 没有 cell 池化 | 加 `@Reusable` 装饰器 |

## 内存 Memory

| ID | 模式 | 问题 | 修复 |
|---|---|---|---|
| M1 | `taskpool.Task` 创建后未取消 | 页面退出泄漏 | `aboutToDisappear()` 中取消 |
| M2 | `setInterval` 未保存/未清理 | Timer 泄漏 | 存 handle，`aboutToDisappear` 中 `clearInterval` |
| M3 | `@State` 大对象 —— 全对象触发 diff | 任一字段变更引起全部重渲染 | 拆 `@Observed` class + 子组件 `@ObjectLink` |
| M4 | `@State` 大数组整体更新 | 每次 update 全数组复制 | 用 `@ObservedV2` + `@Trace` 标个别字段 |
| M5 | `Image` 加载未设 size | 按原图全尺寸解码 | `.objectFit()` 前先 `.width()` + `.height()` |

## 可读性 Readability

| ID | 模式 | 问题 | 修复 |
|---|---|---|---|
| R1 | 内联子组件重复 >2 次 | 代码重复 | 抽 `@Builder` 方法 |
| R2 | `build()` 方法 >80 行 | 难读 | 拆多个 `@Builder` 子方法 |
| R3 | 颜色/尺寸魔法数 | 语义不明 | 提到文件顶部 `const` |
| R4 | `@Component struct` 缺描述注释 | 意图不明 | 上方加单行注释 |

## API 升级 API Upgrade

| ID | 模式 | 问题 | 修复 |
|---|---|---|---|
| A1 | `@State` 数组按 index 修改 | 已废弃模式 | 用 `@ObservedV2` 数组方法 |
| A2 | `animateTo` 不带 `curve` | 线性动画 | 加 `curve: Curve.EaseInOut` |
| A3 | `List.onScroll` 旧回调签名 | API 11+ 已变 | 换 `onDidScroll` |
| A4 | `Image(src: string)` 相对路径 | 解析问题 | 换 `$rawfile()` 或 `$r()` |

---

## 输出格式约定

```
Found N issues:

[P1] Line 42 — ForEach on 1000-item list without LazyForEach
     Fix: Replace ForEach+items array with LazyForEach+IDataSource

[M2] Line 78 — setInterval handle not stored, will leak on page exit
     Fix: Store handle in field, clear in aboutToDisappear()

[R1] Lines 55–70 — ListCell built inline, duplicated 3 times
     Fix: Extract to @Builder private listCell(item: Item)

Which issues to fix? (enter numbers, "all", or "skip")
```

按用户选择的 ID 执行 Edit，每次修复后重读文件验证未破坏结构。
