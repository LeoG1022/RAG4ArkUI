# .claude/references/ — 参考资料库

按领域拆分的 API 映射 + 最佳实践模板 + 重构检查表。
Skill 按关键词路由**按需 Read**，不一次性全部加载。

---

## 文件分类

### 按领域拆分的 Mapping（5 份，按需加载）

| 文件 | 适用关键词 |
|---|---|
| [`mapping-list.md`](mapping-list.md) | `list` / `scroll` / `grid` / `items` / `LazyColumn` / `LazyRow` / `LazyForEach` / `ForEach` |
| [`mapping-state.md`](mapping-state.md) | `state` / `remember` / `mutableStateOf` / `LaunchedEffect` / `DisposableEffect` / `@State` / `@Prop` / `@Link` |
| [`mapping-layout.md`](mapping-layout.md) | `Row` / `Column` / `Box` / `Modifier` / `padding` / `Text` / `Image` / `Spacer` / `Divider` |
| [`mapping-animation.md`](mapping-animation.md) | `animation` / `animate` / `transition` / `AnimatedVisibility` / `Crossfade` / `tween` |
| [`mapping-async.md`](mapping-async.md) | `coroutine` / `suspend` / `await` / `Dispatchers` / `Flow` / `http` / `taskpool` / `resource` / `format` |

### 固定加载的参考表（2 份）

| 文件 | 用途 |
|---|---|
| [`arkuix-best-practices.md`](arkuix-best-practices.md) | KMP 与 ArkUI-X 最优写法清单 + 双份输出代码模板 |
| [`arkuix-refactor-checklist.md`](arkuix-refactor-checklist.md) | 重构 4 张 checklist（性能 P* / 内存 M* / 可读性 R* / API 升级 A*） |

---

## 每份 mapping 的必备节

```markdown
# Mapping — {领域名}

适用关键词：{关键词列表}

## 核心映射
{KMP → ArkUI-X 表格}

## (可选) 模板 / 标准模式
{IDataSource / taskpool / http 等标准代码骨架}

## 必要的最佳实践（生成时强制）
{条目列表，对应 check-api-parity.sh 规则}

## Anti-Patterns
| Pattern | Problem | Fix | Check |
|---|---|---|---|
{反模式 → 问题 → 修复 → 检测命令}
```

**Anti-Patterns 节是强制的**——这是 skill 在生成代码时直接可参照的"红线表"。

---

## 添加新映射条目的流程

1. 跑 Git 前置检查（见 [`../../AGENTS.md`](../../AGENTS.md)）
2. 确定属于哪个领域 → 编辑对应 `mapping-*.md`
3. 必须同时填写 Anti-Pattern（即使 `check: LLM 扫描` 也要列出）
4. 如新增 case 揭示了机械化检测的可能 → 同步在 `scripts/check-api-parity.sh` 增规则 + `feedback/DESIGN.md` 记录决策
5. 在 `feedback/{N}-{date}-*.md` 中记录本次补充

---

## 缺失 API 处理协议

skill 遇到 mapping 未覆盖的 API 时：

1. **停止生成**
2. 向用户提问：「mapping-`<领域>`.md 中没有找到 [API名] 的 ArkUI-X 等价写法，请确认。确认后追加到该 mapping 文件再继续。」
3. 用户回复后，把映射补到对应 `mapping-*.md`
4. 再继续生成

**禁止 agent 自行猜测对标关系。**详见 [`../../feedback/DESIGN.md`](../../feedback/DESIGN.md) 决策八。

---

## 下一步

- 想知道 skill 何时加载哪份 mapping → 见 `.claude/skills/{generate,kmp-to-arkuix}.md` 中的"按需加载"表
- 想加新校验规则 → 见 [`../../scripts/AGENTS.md`](../../scripts/AGENTS.md)
- 重构检查表使用方式 → 见 [`../skills/arkuix-refactor.md`](../skills/arkuix-refactor.md)
