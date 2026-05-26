# Mapping — 布局、Modifier、Text、Image

适用关键词：`Row` / `Column` / `Box` / `Modifier` / `padding` / `Text` / `Image` / `Spacer` / `Divider`。

## 容器

| KMP | ArkUI-X |
|---|---|
| `Row(modifier, horizontalArrangement, verticalAlignment)` | `Row({ space: gap }) { ... }.alignItems(VerticalAlign.Center)` |
| `Column(modifier, verticalArrangement, horizontalAlignment)` | `Column({ space: gap }) { ... }.alignItems(HorizontalAlign.Start)` |
| `Box(contentAlignment, modifier)` | `Stack({ alignContent: Alignment.Center }) { ... }` |

## Modifier 链

| KMP | ArkUI-X |
|---|---|
| `Modifier.fillMaxSize()` | `.width('100%').height('100%')` |
| `Modifier.fillMaxWidth()` | `.width('100%')` |
| `Modifier.fillMaxHeight()` | `.height('100%')` |
| `Modifier.size(n.dp)` | `.width(n).height(n)` |
| `Modifier.padding(all.dp)` | `.padding(all)` |
| `Modifier.padding(h.dp, v.dp)` | `.padding({ left: h, right: h, top: v, bottom: v })` |
| `Modifier.background(color)` | `.backgroundColor(color)` |
| `Modifier.clip(RoundedCornerShape(r.dp))` | `.borderRadius(r)` |
| `Modifier.border(w.dp, color)` | `.border({ width: w, color: color })` |
| `Modifier.clickable { }` | `.onClick(() => { })` |
| `Modifier.weight(f)` | `.layoutWeight(f)` |
| `Modifier.aspectRatio(r)` | `.aspectRatio(r)` |
| `Modifier.offset(x.dp, y.dp)` | `.offset({ x: x, y: y })` |
| `Modifier.zIndex(z)` | `.zIndex(z)` |
| `Modifier.alpha(a)` | `.opacity(a)` |

## 文本

| KMP | ArkUI-X |
|---|---|
| `Text("hi")` | `Text('hi')` |
| `Text(s, fontSize=16.sp)` | `Text(s).fontSize(16)` |
| `Text(s, color=Color.Red)` | `Text(s).fontColor(Color.Red)` 或 `.fontColor('#FF0000')` |
| `Text(s, fontWeight=FontWeight.Bold)` | `Text(s).fontWeight(FontWeight.Bold)` |
| `Text(s, textAlign=TextAlign.Center)` | `Text(s).textAlign(TextAlign.Center)` |
| `Text(s, maxLines=1, overflow=Ellipsis)` | `Text(s).maxLines(1).textOverflow({ overflow: TextOverflow.Ellipsis })` |

## 图片

| KMP | ArkUI-X |
|---|---|
| `Image(painter, contentScale=ContentScale.Crop)` | `Image(src).width(n).height(n).objectFit(ImageFit.Cover)` |
| `Image(painter, contentScale=ContentScale.Fit)` | `Image(src).objectFit(ImageFit.Contain)` |
| `painterResource("drawable/foo.png")` | `$r('app.media.foo')` |
| `Image(painter=rememberAsyncImagePainter(url))` | `Image(url)`（系统自动异步加载） |

## 颜色

| KMP | ArkUI-X |
|---|---|
| `Color(0xFFRRGGBB)` | `'#RRGGBB'` |
| `Color.Red` / `Color.Blue` | `Color.Red` / `Color.Blue`（ArkUI-X 同名枚举） |
| `MaterialTheme.colorScheme.primary` | `'#6200EE'` 或自定义 design token |

## 其他

| KMP | ArkUI-X |
|---|---|
| `Spacer(Modifier.width(n.dp))` | `Blank().width(n)` |
| `Spacer(Modifier.height(n.dp))` | `Blank().height(n)` |
| `HorizontalDivider()` | `Divider().width('100%')` |
| `VerticalDivider()` | `Divider().vertical(true).height('100%')` |

## 必要的最佳实践（生成时强制）

- `Image` 的链顺序固定：先 `.width(n).height(n)` 再 `.objectFit(...)`，再其他
- 长度单位：`dp` / `sp` → ArkUI-X 数字（vp 单位，与 dp 近似）
- 颜色：`Color(0xFFRRGGBB)` → 字符串 `'#RRGGBB'`，注意去 `0xFF` 前缀
- 不要在 `items {}` lambda 内创建 Modifier 对象（KMP）/ 不要在 build() 内创建对象（ArkUI-X）

## Anti-Patterns

| Pattern | Problem | Fix | Check |
|---|---|---|---|
| `Image` 缺 `.objectFit()` | 默认拉伸变形 / 全分辨率解码 | 加 `.objectFit(ImageFit.Cover)` | LLM 扫描 |
| `Image` 缺 `.width()` / `.height()` | 按原图分辨率解码 → 内存浪费 | `.width(n).height(n)` 必须在 `.objectFit` 前 | LLM 扫描 |
| `Modifier` 在 `items` 内每帧 new | GC 压力 | 抽到外部 `Modifier.then(...)` 或常量 | LLM 扫描（KMP） |
| 颜色硬编码遍布全文件 | 主题难维护 | 提到文件顶部 `const PRIMARY = '#6200EE'` | LLM 扫描 |
| `Image` 用相对路径字符串 | 路径解析不稳定 | 用 `$rawfile()` 或 `$r('app.media.x')` | LLM 扫描 |
