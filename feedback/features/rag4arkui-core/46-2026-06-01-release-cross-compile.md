# 46 — release-cross-compile

> 日期：2026-06-01
> 涉及代码：`scripts/release-local.sh` · `.github/workflows/release.yml`
> 类型：bug 修复（CI 跨编 · 配 Round 45 macos-13 → macos-14）

## 本轮目标

Round 45 把 macos-13 → macos-14 后 · v0.0.2-rc.1 跑出 upload-artifact `No files were found ... x86_64-apple-darwin.tar.gz` · 根因 = `release-local.sh` 用 host triple 命名 · 不接受 `--target` · 跨编时 host(aarch64) ≠ target(x86_64) · 文件名错配。

修法 = 让 release-local.sh 真正支持 `--target` · CI 把 matrix.target 传进来。

## Plan / 改动 / 详见 meta/23

- `scripts/release-local.sh` 加 `--target` 参数 · CROSS_COMPILE flag · BIN_PATH 切换 · 跨编烟雾跳过
- `.github/workflows/release.yml` build step 传 `--target "${{ matrix.target }}"`

## 对话摘要

| 轮次 | 用户 | Agent |
|---|---|---|
| 1 | 贴 `Error: No files were found ... x86_64-apple-darwin.tar.gz` | 诊断 release-local.sh host-triple bug · 给 4 选项 + 选 A 实施 |

## 验证结果

- 本地 yaml + bash 语法 OK
- CI 验证等用户重推 v0.0.2-rc.3 后看 4 平台全绿（rc.2 tag 不会自动重跑同 workflow）

## 残留 / 下一轮

- [x] release-local.sh --target + 跨编路径
- [x] release.yml 传 matrix.target
- [x] 双轨归档 + STATUS
- [ ] **用户重推 v0.0.2-rc.3 · 看 4 平台全绿 + 4 个 artifact 全到 Releases**
- [ ] 长期：烟雾测试改 host==target 时跑 · 不等时跳过
- [ ] Makefile `release-local-verify` 本地路径不受影响（用户本地默认 host build · 不传 --target）· 但可加 README 提示「CI matrix 用 --target」
