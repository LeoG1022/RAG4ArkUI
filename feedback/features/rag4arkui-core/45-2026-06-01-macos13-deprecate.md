# 45 — macos13-deprecate

> 日期：2026-06-01
> 涉及代码：`.github/workflows/release.yml`
> 类型：CI bug 修复（runner deprecate · matrix 1 平台卡）

## 本轮目标

v0.0.2-rc.1 release.yml 4 平台 matrix 跑出 3 绿 + 1 pending。pending 那个是 `x86_64-apple-darwin (macos-13)` · 永远「Waiting for a runner」· macos-13 runner pool deprecate 了。

改 `macos-14 + cross-compile` 收回 4/4 完整。

## Plan

### 1 行修改 + 注释（详见 meta/22）

```yaml
# Before
- target: x86_64-apple-darwin
  os: macos-13              # 已 deprecate · 永远不分配 runner

# After
- target: x86_64-apple-darwin
  os: macos-14              # arm64 host · 跨编 x86_64-apple-darwin
```

### 跨编为什么可行

- macos-14 是 arm64 (Apple Silicon) runner
- `dtolnay/rust-toolchain@stable + targets: ${{ matrix.target }}` 自动装 x86_64 target
- Xcode SDK on macos-14 自带 fat 库（universal SDK）· 可跨编到 x86_64
- 链接器（ld / Apple cctools）自动选 x86_64 lib
- 这是 GitHub 官方推荐做法（macos-13 deprecate announcement）

### 不动

- 其它 3 个 matrix 平台
- features
- ci.yml / book.yml

### 替代方案权衡

| 方案 | 工作量 | 选 |
|---|---|---|
| A · macos-14 + 跨编（本轮）| 1 行 | ✅ |
| B · 删 x86_64-darwin matrix | 5 行 | ❌（丢 Intel Mac 用户）|
| C · docker x86_64 容器（cargo-zigbuild）| 复杂 | ❌（A 失败再考虑）|

## 对话摘要

| 轮次 | 用户 | Agent |
|---|---|---|
| 1 | 「三个平台绿 aarch64，pc-Windows linux」 | 推测剩余 x86_64-darwin 卡 macos-13 deprecate · 给 A/B/C 3 方案 |
| 2 | 贴 macos-13 「Waiting for a runner」日志 · 实锤 deprecate | 按 A 直接实施 · 改 release.yml · 本归档 |

无方向调整 · 用户贴日志后 agent 自主决定 A。

## 改动要点

- `.github/workflows/release.yml` matrix 节 1 行 `macos-13` → `macos-14` + 3 行注释
- 不动其它 matrix 项
- 跨编 setup 已 ready（dtolnay/rust-toolchain 自动装 target）· 不动 toolchain step
- 与 Round 75 (Day 20b CI matrix) 关系：本轮维护 · 不改既有结构

## 验证结果

- YAML 语法：grep / sed 可 parse · 不破触发语义
- 编译验证：N/A（CI 上验证）
- 用户需重推 v0.0.2-rc.2 tag · 看 release.yml 4 平台全绿

## 残留 / 下一轮

- [x] release.yml macos-13 → macos-14
- [x] 双轨归档 + STATUS
- [ ] **用户重推 v0.0.2-rc.2 验证**：4 平台全绿后 task #76 完成
- [ ] **如果跨编 fail**（不太可能 · 但留 fallback）：改 cargo-zigbuild · 或者干脆删 x86_64-darwin
- [ ] **docs/RELEASE.md 检查**：是否需要更新 macos-13 → macos-14 描述
- [ ] **Round 44 残留延续**：跑了 v0.0.2-rc.2 验证 Node 24 env 也生效 · 一次性确认
