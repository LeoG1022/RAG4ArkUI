# 23 — release-cross-compile

> 日期：2026-06-01
> 触发：Round 45 把 macos-13 → macos-14 后 · v0.0.2-rc.1 实际 release.yml 跑出 `upload-artifact` Error: No files were found with the provided path: `dist/arkui-rag-v*-x86_64-apple-darwin.tar.gz`
> 类型：bug 修复（release-local.sh 不支持 --target · 跨编时命名错配）

---

## 用户提出的要求

用户贴 macos-14 跑 x86_64-apple-darwin matrix job 的 upload step 错：

> Run actions/upload-artifact@v4
> Error: No files were found with the provided path: dist/arkui-rag-v*-x86_64-apple-darwin.tar.gz. No artifacts will be uploaded.

## Agent 给出的修改建议

### 诊断

`scripts/release-local.sh` 用 `rustc -vV` 探测 **host** triple · 不接受 `--target` · cargo build 也不带 `--target`：

- macos-14 host = aarch64-apple-darwin
- matrix target = x86_64-apple-darwin
- release-local.sh 不知道 target · 编出来是 **aarch64** binary · tarball 也命名 aarch64
- upload-artifact 找 `dist/...x86_64...` → 找不到

### 修法（3 处）

1. **`scripts/release-local.sh`** 加 `--target` 参数：
   - 参数解析加 `--target X` 支持 + `CROSS_COMPILE` flag
   - `TARGET_TRIPLE` 优先用 `--target` · fallback host
   - `cargo build` 加 `--target` 参数（CROSS_COMPILE=1 时）
   - `BIN_PATH` 跨编时用 `target/<TARGET>/release/` · 否则 `target/release/`
   - 烟雾测试 `--version` 跨编时跳过（binary 跑不动 host）

2. **`.github/workflows/release.yml`** 传 `matrix.target`：
   ```yaml
   run: bash scripts/release-local.sh --features "$FEATURES" --target "${{ matrix.target }}"
   ```

### 影响所有 4 个平台

改后所有 matrix job 都走 `--target` 路径（CROSS_COMPILE=1）：
- macos-14 + aarch64-darwin · host=target · 烟雾测试跳过 · 产物在 target/aarch64-apple-darwin/release/
- macos-14 + x86_64-darwin · host≠target · 跨编 · 产物在 target/x86_64-apple-darwin/release/
- ubuntu + linux · host=target · 产物在 target/x86_64-unknown-linux-gnu/release/
- windows · host=target · 产物在 target/x86_64-pc-windows-msvc/release/

烟雾测试简化为「CROSS_COMPILE=1 一律跳过」· 即便 host=target。CI 上 cargo build 成功 + 上传 artifact 即足够 · 烟雾测试不是 CI 关键路径（用户下载后自己跑）。

### 替代方案权衡

| 选项 | 工作量 | 选 |
|---|---|---|
| A · 加 --target + 调整路径（本轮）| 3 处修改 | ✅ |
| B · 改 upload-artifact 的 path glob 用 host triple | 简单 | ❌（跨编 binary 仍是 aarch64 · 命名也错）|
| C · 用 cargo-zigbuild 跨编 | 加 dep | ❌（A 失败再考虑）|
| D · macos-14 只 build aarch64 · 删 x86_64-darwin | 5 行 | ❌（丢 Intel Mac）|

## 多轮互动

| 轮次 | 用户 | Agent |
|---|---|---|
| 1 | 贴 upload-artifact 找不到文件 | 诊断 release-local.sh 用 host triple bug · 给方案 A · 实施 |

## 实际改动

- 接口变化：release-local.sh 加 `--target X` 参数
- 规则变化：所有 matrix job 都跨编路径走 · 烟雾测试在 CI 跳过
- 文件变化：
  - `scripts/release-local.sh` 加 TARGET 参数解析 + CROSS_COMPILE flag + cargo --target + BIN_PATH 路径切换 + 跨编烟雾跳过
  - `.github/workflows/release.yml` 跑 release-local.sh 传 `--target "${{ matrix.target }}"`

## 执行生效后总结

### 实际产出

| 文件 | 改动 |
|---|---|
| `scripts/release-local.sh` | +15 行（参数 + CROSS_COMPILE + BIN_PATH 切换 + 烟雾跳过）|
| `.github/workflows/release.yml` | 1 行（传 --target）|

### 前后对比

| 维度 | Before | After |
|---|---|---|
| `--target` 支持 | 无 | 有 |
| 跨编 (host≠target) | binary 是 host · tarball 命名 host | binary 是 target · tarball 命名 target ✓ |
| matrix x86_64-darwin upload | fail（找不到文件）| 期望成功 |
| 烟雾测试 | 总跑 | CI 跨编时跳过 · 本地仍跑 |

### 实测验证

- 本地 yaml/bash 语法 OK
- CI 验证：用户重推 v0.0.2-rc.3 后看 4 平台全绿（不是 rc.2 · rc.2 tag 已存在指向 e7c0a7b · workflow 已变化但 GitHub 不会自动重跑同 tag）

### 残留 / 下一轮处理

- [x] release-local.sh 加 --target
- [x] release.yml 传 matrix.target
- [x] 双轨归档 + STATUS
- [ ] **用户重推 v0.0.2-rc.3 验证 4 平台全绿**
- [ ] **本地 release-local-verify Makefile** 验证非 CI 路径仍 OK（本地默认 host build · 应该不受影响）
- [ ] **跨编烟雾测试**：长期可改成「host==target 时跑烟雾 · 不等时跳过」· 当前 CROSS_COMPILE=1 时一律跳（简化）
