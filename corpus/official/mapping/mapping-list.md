# Mapping — 列表与 Grid

适用关键词：`list` / `scroll` / `grid` / `items` / `LazyColumn` / `LazyRow` / `LazyForEach` / `ForEach`。

## 核心映射

| KMP | ArkUI-X |
|---|---|
| `LazyColumn { items(list, key={it.id}) { ... } }` | `List() { LazyForEach(ds, (item) => { ListItem() { ... } }, (item) => item.id.toString()) }.cachedCount(3)` |
| `LazyRow { items(list, key={it.id}) { ... } }` | `List({ space: 8 }) { LazyForEach(...) }.listDirection(Axis.Horizontal).cachedCount(3)` |
| `LazyVerticalGrid(GridCells.Fixed(n))` | `Grid() { LazyForEach(ds, ...) }.columnsTemplate('1fr '.repeat(n).trim()).cachedCount(4)` |
| `stickyHeader { Header() }` | `ListItemGroup({ header: () => this.HeaderBuilder() }) { ListItem()... }` |
| `LaunchedEffect(Unit) { state.scrollToItem(i) }` | `this.scroller.scrollToIndex(i, true)`（平滑滚动）|
| `itemsIndexed(items, key={_,i->i.id}, contentType={_,i->i::class})` | `LazyForEach + 类型判断分支 + key 区分类型`（详见下方示例） |

## IDataSource 模板（使用 LazyForEach 必须生成）

```typescript
class ArrayDataSource<T> implements IDataSource {
  private list: T[]
  constructor(list: T[]) { this.list = list }
  totalCount(): number { return this.list.length }
  getData(index: number): T { return this.list[index] }
  registerDataChangeListener(_: DataChangeListener): void {}
  unregisterDataChangeListener(_: DataChangeListener): void {}
}
```

放在使用它的 `@Component` 之上、同文件内。需要增删时实现一个支持监听的子类。

## 多类型列表（对标 `contentType`）

```typescript
LazyForEach(this.ds, (item: BaseModel) => {
  ListItem() {
    if (item.type === 'hot') { HotCell({ item: item as HotItem }) }
    else { FollowCell({ item: item as FollowItem }) }
  }
}, (item: BaseModel) => `${item.type}-${item.id}`)
```

## 必要的最佳实践（生成时强制）

- 列表项 `@Component` 必须加 `@Reusable`（启用 cell 复用池）
- `LazyForEach` 必须带 keyGenerator（第 3 个参数）
- `List` / `Grid` 必须设 `.cachedCount(3)` 或 `.cachedCount(4)`
- 单 ListItem 内的 `Image` 必须 `.width(n).height(n)` 在 `.objectFit()` 之前
- 大列表的 cell 嵌套层级 ≤ 5 层

## Anti-Patterns

| Pattern | Problem | Fix | Check |
|---|---|---|---|
| `ForEach(...)` 用于大列表 | 无虚拟滚动，>100 条全量渲染 | 换 `LazyForEach + keyGenerator` | `scripts/check-api-parity.sh` → P-LIST-01 |
| `LazyForEach` 缺第 3 参 keyGenerator | 按 index 匹配 diff，增删时全量重建 | 加 `(item) => item.id.toString()` | P-LIST-02 |
| `List` / `Grid` 缺 `.cachedCount()` | 快速滚动白屏 | `.cachedCount(3)` (List) / `.cachedCount(4)` (Grid) | P-LIST-03 |
| 列表项 `@Component` 缺 `@Reusable` | 无 cell 复用，GC 压力大 | 加 `@Reusable` 装饰器 | 暂依赖 LLM 扫描 |
| `build()` 内重建 `ArrayDataSource(...)` | 每帧分配新 DataSource 对象 | 提到 `@State private ds`，在 `aboutToAppear` 初始化 | LLM 扫描 |
| ListItem 嵌套深度 > 5 | 测量耗时增加 | 抽 `@Builder` 拆分 | LLM 扫描 |
