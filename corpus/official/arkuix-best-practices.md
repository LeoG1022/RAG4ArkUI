# KMP & ArkUI-X 最佳实践规则 + 输出模板

Skills (`/generate`, `/kmp-to-arkuix`) 加载本文件以确保生成代码符合各自平台性能/内存最优写法。

---

## KMP 最佳实践（生成代码必须遵守）

| 规则 | 模式 |
|---|---|
| 列表 | 始终 `LazyColumn` / `LazyRow` / `LazyVerticalGrid`，不用 `Column { forEach }` |
| 列表 key | `items()` 必须带 `key = { item -> item.id }` |
| State 作用域 | `remember { mutableStateOf() }` 放最窄作用域；大状态拆多个 `remember` |
| 衍生值 | `remember(dep) { compute(dep) }` 或 `derivedStateOf` —— 不在 composition 中重算 |
| 一次性初始化 | `LaunchedEffect(Unit)` 或 `remember` 持有 |
| 图片 | 缩略图用 `contentScale = ContentScale.Crop` |
| 动画 | `AnimatedVisibility` / `animateFloatAsState` / `Crossfade`，禁用 `Thread.sleep` |
| Modifier | 链式调用；避免在 `items {}` lambda 内创建 `Modifier` |
| 子组件 | 重复出现 ≥2 次的内联块抽成 `@Composable private fun` |

---

## ArkUI-X 最佳实践（生成代码必须遵守）

| 规则 | 强制模式 |
|---|---|
| 列表 > 20 项 | `LazyForEach` + `IDataSource`，禁用 `ForEach` |
| `LazyForEach` | 必须带 `keyGenerator`：`(item) => item.id.toString()` |
| `List` 容器 | 必须 `.cachedCount(3)`（Grid 用 `.cachedCount(4)`） |
| 列表项组件 | 必须加 `@Reusable` 装饰器 |
| `Image` | 链顺序固定：`.width(n).height(n).objectFit(ImageFit.Cover)` |
| 动画 | `animateTo({ duration: 300, curve: Curve.EaseInOut }, ...)`，禁用 `setTimeout` |
| 一次性计算 | 放在 `aboutToAppear()`，结果存到私有字段；禁止放在 `build()` |
| `@State` 大对象 | 字段 > 3 个时用 `@ObservedV2` class + `@Trace` 标记变化字段 |
| 资源清理 | `taskpool` / `setInterval` 必须在 `aboutToDisappear()` 中取消 |
| 入口装饰器 | `@Entry` 只加在页面根组件，不加在子组件 |

---

## KMP 输出代码模板

```kotlin
package com.example.kmpworkspace

// imports

// 数据模型（新建时）
data class <Item>(val id: Int, /* fields */)

// 主 composable —— 入口
@Composable
fun <FeatureName>() {
    // state
    val items = remember { /* init list */ }
    // 布局
    LazyColumn {
        items(items, key = { it.id }) { item ->
            <ItemCard>(item)
        }
    }
}

// 子组件（抽出便于复用 + 性能）
@Composable
private fun <ItemCard>(item: <Item>) {
    Row {
        Image(/* ... */, contentScale = ContentScale.Crop)
        Column { Text(item.name); Text(item.subtitle) }
    }
}
```

---

## ArkUI-X 输出代码模板

```typescript
// 数据模型
class <Item> {
  id: number = 0
  // fields...
}

// IDataSource 实现（使用 LazyForEach 时必须）
// 完整实现见 .claude/references/kmp-arkuix-mapping.md
class <Item>DataSource implements IDataSource {
  private list: <Item>[]
  constructor(list: <Item>[]) { this.list = list }
  totalCount(): number { return this.list.length }
  getData(index: number): <Item> { return this.list[index] }
  registerDataChangeListener(_: DataChangeListener): void {}
  unregisterDataChangeListener(_: DataChangeListener): void {}
}

// 列表项组件 —— @Reusable 复用池
@Reusable
@Component
struct <ItemCard> {
  @Prop item: <Item> = new <Item>()

  build() {
    Row() {
      Image(this.item.image).width(60).height(60).objectFit(ImageFit.Cover)
      Column() {
        Text(this.item.title).fontSize(16)
        Text(this.item.subtitle).fontSize(14).fontColor('#666')
      }
    }
  }
}

// 主页面组件 —— @Entry 只加在这里
@Entry
@Component
struct <FeatureName> {
  @State private ds: <Item>DataSource = new <Item>DataSource([])

  aboutToAppear(): void {
    const data: <Item>[] = Array.from({ length: 1000 }, (_, i) => {
      const item = new <Item>()
      item.id = i
      return item
    })
    this.ds = new <Item>DataSource(data)
  }

  aboutToDisappear(): void {
    // 清理 taskpool / clearInterval（如有）
  }

  build() {
    List() {
      LazyForEach(this.ds, (item: <Item>) => {
        ListItem() {
          <ItemCard>({ item })
        }
      }, (item: <Item>) => item.id.toString())
    }
    .cachedCount(3)
    .width('100%')
    .height('100%')
  }
}
```
