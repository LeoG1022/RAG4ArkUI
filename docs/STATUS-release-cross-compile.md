# STATUS — release-cross-compile

> 配套 feature log：`feedback/features/rag4arkui-core/46-2026-06-01-release-cross-compile.md`
> 配套 meta：`feedback/meta/23-2026-06-01-release-cross-compile.md`
> 日期：2026-06-01

---

## 当前状态

Round 45 把 macos-13 → macos-14 解决 runner deprecate · 但暴露第 2 个 bug：`release-local.sh` 不支持 `--target` · 跨编时 host(aarch64) ≠ target(x86_64) · 编出来是 aarch64 binary · tarball 命名 aarch64 · upload-artifact 找 x86_64 文件不到 → fail。

本轮修：让 release-local.sh 真正接受 `--target` · CI 传 matrix.target。

## 输入契约

### release-local.sh 新接口

```bash
bash scripts/release-local.sh \
    [--features FEATURES] \
    [--skip-build] \
    [--target <triple>]    # Round 46 新加 · CI matrix 用
```

不指定 `--target` 时行为完全不变（用 host triple · 本地用户跑没影响）。

### release.yml build step 变化

```yaml
# Before
run: bash scripts/release-local.sh --features "$FEATURES"

# After
run: bash scripts/release-local.sh --features "$FEATURES" --target "${{ matrix.target }}"
```

### 不变项

- artifact 命名规则 `arkui-rag-v<VERSION>-<TARGET>.<EXT>` 不变（用 TARGET_TRIPLE）· 跟 upload-artifact path glob 一致
- features 不变
- ci.yml / book.yml 不动

## 输出契约

### 路径变化（所有 matrix job · 不只 x86_64-darwin）

| 维度 | Before（用 host）| After（用 --target）|
|---|---|---|
| cargo build target dir | `target/release/` | `target/<TARGET>/release/` |
| BIN_PATH | `target/release/arkui-rag` | `target/<TARGET>/release/arkui-rag` |
| tarball 命名 | host triple | target triple ✓ |
| 烟雾测试 | 跑 `--version` | CI 跨编时跳过（CROSS_COMPILE=1）|

### 期望

| Job | 期望 |
|---|---|
| Build aarch64-apple-darwin | 🟢 macos-14 native build · target/aarch64-apple-darwin/release/ |
| **Build x86_64-apple-darwin** | 🟢 **macos-14 跨编 · target/x86_64-apple-darwin/release/ · 关键验证** |
| Build x86_64-unknown-linux-gnu | 🟢 ubuntu native · target/.../release/ |
| Build x86_64-pc-windows-msvc | 🟢 windows native · target/.../release/ |
| Release（upload artifact）| 🟢 4 个 tarball/zip 命名正确 · 全到 GitHub Releases |

## 验证手段

### 用户操作

```bash
# 1. push 本轮 commit
git push origin master

# 2. 推新 tag v0.0.2-rc.3（不是 rc.2 · rc.2 已存在但跑的是旧 workflow）
git tag v0.0.2-rc.3
git push origin v0.0.2-rc.3

# 3. 看 release.yml 跑
# https://github.com/LeoG1022/RAG4ArkUI/actions/workflows/release.yml
```

### 期望状态

- 4 平台 build 全绿
- upload-artifact 全 4 个成功
- Releases 页 v0.0.2-rc.3 含 4 个 tarball/zip · 命名正确（aarch64 / x86_64 各自对应）

### 本地路径不受影响

```bash
make release-local         # 默认 host build · 不传 --target · 跟之前一样
```

## 与上一阶段的关联性

| Round | 主题 | 关系 |
|---|---|---|
| 75 (Day 20b) | release.yml + release-local.sh 立起来 | 本轮维护 |
| 45 | macos-13 → macos-14 + 跨编 | **本轮 Bug fallout** |
| **46（本轮）** | release-local.sh 加 --target 真支持跨编 | task #76 收尾 |

层次：Round 45 改 OS · Round 46 改脚本支持跨编 · 一起才完整。

## 完成度 / 下一阶段

### 达成度

| 项 | 状态 |
|---|---|
| release-local.sh --target | ✅ |
| release.yml 传 matrix.target | ✅ |
| 跨编 BIN_PATH 切换 | ✅ |
| 跨编烟雾测试跳过 | ✅ |
| 双轨归档 + STATUS | ✅ |
| 用户重推 v0.0.2-rc.3 验证 | ⏳ |

### 下一阶段建议

立即（用户做）：
1. git push origin master
2. git tag v0.0.2-rc.3 + git push origin v0.0.2-rc.3
3. 等约 25-30 分钟 · 看 release.yml 4 平台全绿

后续：
- 4 全绿 → task #76 ✅ → 决定推 v1.0.0
- 跨编 fail → 看具体错（最可能 macOS SDK 链接问题 · 加 SDKROOT env）

中期：
- 长期烟雾测试：精确化「host==target 时跑」（当前简化为 CROSS_COMPILE=1 一律跳）
- universal binary 方案：把 aarch64 + x86_64 打成一个 .app · macOS 用户下载一个文件即可
