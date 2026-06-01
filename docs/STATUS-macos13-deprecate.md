# STATUS — macos13-deprecate

> 配套 feature log：`feedback/features/rag4arkui-core/45-2026-06-01-macos13-deprecate.md`
> 配套 meta：`feedback/meta/22-2026-06-01-macos13-deprecate.md`
> 日期：2026-06-01

---

## 当前状态

承接 v0.0.2-rc.1 release.yml 实战：3 平台绿（aarch64-darwin / linux / windows）+ 1 平台卡（x86_64-darwin macos-13 永远 waiting）。

修法：改 `macos-14 + cross-compile` · 1 行修改 + 注释。

本阶段交付：
- `.github/workflows/release.yml` matrix 节 1 行修改
- 双轨归档 + STATUS（本文件）
- meta/8 残留预测的 macos-13 deprecate 真实触发 + 修复

意义：release.yml 4 平台 matrix 完整修复 · 用户重推 v0.0.2-rc.2 后期望 4/4 全绿 · task #76 完整收尾。

## 输入契约

### Workflow matrix 变化

| Target | OS Before | OS After |
|---|---|---|
| aarch64-apple-darwin | macos-14 | macos-14 |
| **x86_64-apple-darwin** | **macos-13** ⚠️ deprecate | **macos-14** ✓ 跨编 |
| x86_64-unknown-linux-gnu | ubuntu-latest | ubuntu-latest |
| x86_64-pc-windows-msvc | windows-latest | windows-latest |

### 不变项

- features `http,mcp,lsp,tantivy,typescript,corpus-pull`
- artifact 命名 `arkui-rag-v<TAG>-<TARGET>.tar.gz`
- Releases 上传方式（softprops/action-gh-release@v2）
- ci.yml / book.yml 完全不动

## 输出契约

### 期望 4 平台 build 产物

```
artifact 上传到 GitHub Releases v0.0.2-rc.2:
- arkui-rag-v0.0.2-rc.2-aarch64-apple-darwin.tar.gz      ← macos-14 native build
- arkui-rag-v0.0.2-rc.2-x86_64-apple-darwin.tar.gz       ← macos-14 cross-compile
- arkui-rag-v0.0.2-rc.2-x86_64-unknown-linux-gnu.tar.gz  ← ubuntu native build
- arkui-rag-v0.0.2-rc.2-x86_64-pc-windows-msvc.zip       ← windows native build
```

### 跨编 binary 验证

x86_64-apple-darwin binary 应该可在 Intel Mac 上原生跑 · 也可在 Apple Silicon Mac 上经 Rosetta 跑（但 aarch64 binary 原生更快）。

## 验证手段

### 用户操作（验证本轮修复）

```bash
# 1. push 修复（本轮 commit）
git push origin master

# 2. 删除旧的 cancelled v0.0.2-rc.1 那个卡死的 job（可选 · 浏览器去 cancel · 或者过期自动 cleanup）

# 3. 推新 tag v0.0.2-rc.2 触发新一轮 release.yml
git tag v0.0.2-rc.2
git push origin v0.0.2-rc.2

# 4. 看 release.yml 跑结果
# https://github.com/LeoG1022/RAG4ArkUI/actions/workflows/release.yml
# 期望 4/4 全绿
```

### 期望

| Job | 期望状态 |
|---|---|
| Build aarch64-apple-darwin | 🟢 同前 · 不变 |
| **Build x86_64-apple-darwin** | 🟢 **macos-14 跨编成功**（本轮关键验证）|
| Build x86_64-unknown-linux-gnu | 🟢 同前 |
| Build x86_64-pc-windows-msvc | 🟢 同前 |
| Release（上传 artifact）| 🟢 4 个 tarball/zip 全到 v0.0.2-rc.2 |

### 失败 fallback

如果 x86_64-darwin 跨编 fail（虽然不太可能）：
- 看错误：链接器 / Xcode SDK / target 等
- 切 cargo-zigbuild（用 zig cc 当跨编 linker · GitHub Actions 常见）
- 或者直接删 x86_64-darwin matrix · 只保留 3 平台

## 与上一阶段的关联性

| Round | 主题 | 跟本轮关系 |
|---|---|---|
| 75 (Day 20b) | release.yml CI matrix 4 平台建立 | **本轮维护** |
| 8 / meta-8 | release-ci-matrix 残留：「macos-13 deprecate 跟进」预测 | **本轮实战触发 + 修复** |
| 44 | node24-force-flag | 上轮 · 本轮和它配套（合并验证）|
| 76 | 用户首推 v0.0.2-rc.1 | 触发源 |
| **45（本轮）** | macos13-deprecate | 修 1 平台卡 |

层次：本轮纯 release.yml 维护 · 不改既有逻辑 / matrix / features · 只改 1 个 runner image。

兼容性：完全向后兼容 · 跨编出的 x86_64 binary 跟 macos-13 host build 出的 binary 应该字节级一致或近似（Rust 跨编通常 deterministic）。

破坏性变更：无。

## 完成度 / 下一阶段

### 达成度

| 项 | 状态 |
|---|---|
| release.yml matrix 修改 | ✅ |
| 双轨归档 + STATUS | ✅ |
| 用户重推 v0.0.2-rc.2 验证 | ⏳ |
| 4 平台全绿确认 | ⏳ |
| task #76 完整收尾 | ⏳ |

### 下一阶段建议

立即（用户做）：
1. git push origin master（推本轮 commit）
2. （可选）去 Actions UI cancel 那个 macos-13 卡死的 job · 释放并发槽
3. git tag v0.0.2-rc.2 + git push origin v0.0.2-rc.2 · 触发新一轮
4. 等 ~25-30 分钟看 4 平台跑结果
5. **跨编 fail 概率不低 · 可能要 cargo-zigbuild · 别太乐观**

短期：
- 4 平台全绿 → task #76 ✅ → 可决定推 v1.0.0（task #85）
- 4 平台仍有 fail → 看错误 · 决定 fallback
- docs/RELEASE.md 检查 · 是否提到 macos-13 需要更新

中期：
- 看 GitHub Actions 是否推出 macos-15 · 未来可能升级
- universal binary（fat binary）方案考虑：把 aarch64 + x86_64 打成一个 universal binary · 简化分发

长期：
- 看 Apple Intel Mac 用户比例 · 决定是否长期维持 x86_64-darwin build · 还是 1.5+ 弃 Intel Mac 直接 ARM only
