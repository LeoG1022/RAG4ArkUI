# 22 — macos13-deprecate

> 日期：2026-06-01
> 触发：v0.0.2-rc.1 release.yml 4 平台 matrix · 其中 `x86_64-apple-darwin (macos-13)` job 永远「Waiting for a runner to pick up this job」· 不分配 runner
> 类型：CI 配置（runner image deprecate 应对）

---

## 用户提出的要求

承接 task #76 残留：「**macos-13 deprecate 跟进**：未来切到 `macos-14` + 跨编 `--target=x86_64-apple-darwin`」（meta/8 写过）。

实战触发：v0.0.2-rc.1 推完 release.yml 跑 · 3 平台绿（aarch64-darwin / linux / windows）· 1 平台卡：

```
Evaluating build.if
Evaluating: success()
Result: true
Requested labels: macos-13
Job defined at: LeoG1022/RAG4ArkUI/.github/workflows/release.yml@refs/heads/master
Waiting for a runner to pick up this job...
```

诊断：macos-13 runner pool 已 deprecate · 不再接新 job。

## Agent 给出的修改建议

### 修法对比

| 方案 | 工作量 | 结果 | 选 |
|---|---|---|---|
| **A · 改 macos-14 + 跨编 x86_64-apple-darwin**（本轮）| 1 行 + 注释 | macos-14 (arm64) host 跨编出 x86_64 binary · Xcode SDK 自带 fat 库 · 链接可行 | ✅ |
| B · 删 x86_64-apple-darwin matrix 行 | 5 行 | 不出 Intel Mac 版 · 用户 Intel Mac 跑 Rosetta 模拟 aarch64 | ❌ |
| C · 不动接受 3/4 | 0 | task #76 算 3 平台真活 · 推 v1.0.0 时还是 3 平台 | ❌ |

选 A · 不丢 Intel Mac 用户 · GitHub 官方推荐做法。

### 实施

```yaml
# Before
- target: x86_64-apple-darwin
  os: macos-13

# After
# macos-13 runner pool deprecate · 改 macos-14 (arm64 host) 跨编
- target: x86_64-apple-darwin
  os: macos-14
```

`dtolnay/rust-toolchain@stable` 的 `targets: ${{ matrix.target }}` 已经自动装目标 toolchain · 跨编 setup 已 ready · 不需要额外配置。

### 不动

- 其它 3 个 matrix 平台（aarch64-darwin / linux / windows）已绿 · 不动
- features `http,mcp,lsp,tantivy,typescript,corpus-pull` 不动
- ci.yml / book.yml 不动（macos-13 只用在 release.yml）

## 多轮互动

| 轮次 | 用户 | Agent |
|---|---|---|
| 1 | 「三个平台绿 aarch64，pc-Windows linux」（隐含 x86_64-darwin 没绿）| 推测是 macos-13 deprecate · 给 3 方案（A/B/C）· 推荐 A · 让用户决定 |
| 2 | 贴 macos-13 job 「Waiting for a runner」日志 | 确认推测 · 直接按 A 实施 · 不需用户再决定 |

## 实际改动

- 接口变化：无
- 规则变化：x86_64-apple-darwin build 从 macos-13 host 改为 macos-14 host 跨编
- 文件变化：`.github/workflows/release.yml` matrix 节 1 行 `macos-13` → `macos-14` + 3 行注释
- 配置变化：无（跨编自动 setup · 不动 toolchain）

## 执行生效后总结

### 实际产出

| 文件 | 改动 |
|---|---|
| `.github/workflows/release.yml` | +4 行（注释 + 1 行修改）|
| `feedback/meta/22-...` | 本归档 |
| `feedback/features/.../45-...` | feature log |
| `docs/STATUS-macos13-deprecate.md` | STATUS |

### 前后对比

| 维度 | Before | After |
|---|---|---|
| x86_64-apple-darwin job | macos-13 host · 永远 waiting | macos-14 host 跨编 · 应该 pick up |
| release matrix 完整性 | 3/4 绿 + 1 pending | 4/4 全跑 |
| macos-13 依赖 | 有（仅这 1 处）| 无（全用 macos-14）|

### 实测验证

- 本轮 commit 不影响 v0.0.2-rc.1（tag 不变）
- 用户需重推 v0.0.2-rc.2 tag 验证 4/4 全绿
- 期望：x86_64-apple-darwin build 通过 cross-compile · binary 能在 Intel Mac 上跑

### 残留 / 下一轮处理

- [x] release.yml macos-13 → macos-14
- [x] 双轨归档 + STATUS
- [ ] **用户重推 v0.0.2-rc.2 验证**：4 个平台全绿后 task #76 完成
- [ ] **如果 x86_64 跨编 fail**：fallback 改用 macos-15 host（如果 GitHub 已上线）· 或者改 docker x86_64 容器跑（zig cc · cargo-zigbuild）
- [ ] **同步 docs/RELEASE.md**：把 macos-13 描述更新为 macos-14（如果文档里有提）
