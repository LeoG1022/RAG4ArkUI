# STATUS — ci-lint-cleanup

> 配套 feature log：`feedback/features/rag4arkui-core/54-2026-06-02-ci-lint-cleanup.md`
> 日期：2026-06-02

---

## 当前状态

Round 49.6 push 后 GitHub Actions ci.yml 报 fmt + clippy `-D warnings` 失败 · Rust 1.95 新加多个 lint 戳到存量代码。本轮统一修 · 不放宽 ci 门禁。

`cargo fmt --all` + `cargo clippy --fix` 自动修 13+ 个 · 加 5 类手工修。最终 `cargo clippy -- -D warnings` 0 warning。

## 输入契约

无 CLI / API 变化（仅 lint cleanup）。

唯一外露增量：`trait VectorStore` 加默认 `is_empty()` 方法：

```rust
async fn is_empty(&self) -> Result<bool> {
    Ok(self.len().await? == 0)
}
```

向后兼容（既有 impl 不需要重写 · 默认 impl 接管）。

## 输出契约

无行为变更 · 仅消除编译/lint 噪声。CI ci.yml 后续应全绿。

## 验证手段

```bash
cd crates
cargo fmt --all -- --check          # ✓ PASS
cargo clippy --workspace --all-targets -- -D warnings   # ✓ 0 warning
cargo check --workspace             # ✓ PASS
```

CI 真活：push 后 `https://github.com/LeoG1022/RAG4ArkUI/actions` 的 ci 应全绿。

## 与上一阶段的关联性

| Round | 主题 | 关系 |
|---|---|---|
| 49.6 | maintainer CI corpus-build | 本轮触发场景：49.6 push 后撞 ci.yml lint 门禁 |
| 49.8 | cli index-pull | 本轮修两处 `from_file.unwrap()` 是 49.8 新写 + 抄 model-pull 既有模式 |
| **54（本轮）** | fmt + clippy 1.95 cleanup | 解锁 Round 49.6 真正能 push 通过 ci |

## 完成度 / 下一阶段

| 项 | 状态 |
|---|---|
| cargo fmt --all 全过 | ✅ |
| cargo clippy -D warnings 全过 | ✅ |
| ci.yml 门禁不放宽 | ✅ |
| 业务行为不变 | ✅ |
| 用户 push 后 CI 真绿 | ⏭ 等 push |

### 下一阶段建议

1. **立即（用户操作）**：
   ```bash
   git push origin master
   ```
   等 ci.yml 跑完看绿 → 然后才能触发 corpus-build workflow（GitHub Actions 不会跑 lint 挂掉的 commit 的下游 workflow）。

2. **长期**：每次 Rust toolchain 升级（1.96 / 2026 edition 等）时跑一次 `cargo clippy --fix --allow-dirty + cargo fmt` 一把过 · 不积压。
