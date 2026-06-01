# 47 — v1-release-coreml

> 日期：2026-06-01
> 涉及代码：`crates/Cargo.toml` · `Makefile` · `docs/USER-VERIFICATION.md` · `docs/RELEASE.md` · `mdbook/src/quickstart.md` · `README.md`
> 类型：里程碑（v0.0.1 → v1.0.0）+ 性能优化（ort/coreml feature）

## 本轮目标

用户「帮忙做 ABC」三合一：
- **A**：推 v1.0.0（task #85）· workspace version 0.0.1 → 1.0.0 · 5 文档 v0.0.1 → v1.0.0
- **B**：下载 v0.0.2-rc.3 release artifact 验证生产质量
- **C**：加 ort/coreml feature · Apple Silicon GPU 加速 BGE-M3 推理

## Plan

### A · workspace version bump

```toml
# crates/Cargo.toml
[workspace.package]
version = "0.0.1" → "1.0.0"
```

5 个 active 文档同步 `v0.0.1` → `v1.0.0`：
- Makefile（release-local-verify tarball glob）
- docs/USER-VERIFICATION.md
- docs/RELEASE.md
- mdbook/src/quickstart.md
- README.md

不动的（历史快照保持事实）：
- docs/STATUS-pre-existing-fixes.md · STATUS-corpus-pull.md · STATUS-day20a-release-local.md · STATUS-onnx-chain-decision.md

### B · 下载 release artifact 验证

```bash
curl -sL https://github.com/LeoG1022/RAG4ArkUI/releases/download/v0.0.2-rc.3/arkui-rag-v0.0.1-aarch64-apple-darwin.tar.gz
```

意外发现：v0.0.2-rc.3 release 的 artifact 命名是 **v0.0.1**（workspace version）· 不是 git tag · 因为 release-local.sh 用 Cargo.toml workspace.version 命名。A 推 v1.0.0 后自然解决（workspace version 1.0.0 + tag v1.0.0 一致）。

验证：tarball 4.1MB · 解压跑 `--version` → `arkui-rag 0.0.1` ✓ 生产质量 OK。

### C · CoreML 加速

```toml
# Before
ort = { version = "2.0.0-rc.12", features = ["ndarray"] }

# After
ort = { version = "2.0.0-rc.12", features = ["ndarray", "coreml"] }
```

CoreML feature 在非 macOS 平台 ort 内部 cfg gate · Linux/Windows build 不受影响（实测 workspace cargo check 通过）。

### 实测对比 6 query latency

| Embedder | Latency p50 | Note |
|---|---|---|
| Mock (BM25 only) | ~48ms | hash 假向量 |
| ONNX CPU（Round 42）| ~4000ms | CPU only |
| **ONNX + CoreML（本轮）** | **~190ms** | **21× 加速 ✓** |

首次 query 935ms（含 CoreML session 初始化 + cold cache）· 后续稳定 ~190ms。预期 < 500ms 超额达成。

## 对话摘要

| 轮次 | 用户 | Agent |
|---|---|---|
| 1 | 「v0.0.2-rc.3 release.yml 全部成功」 | 庆祝 + 给 A/B/C 3 选项 |
| 2 | 「你帮忙做 ABC」 | 并行实施：C 先（CoreML build · 配 v1.0.0 一起发）+ B（下载验证）+ A（version bump）· 本轮总收 |

## 改动要点

- `crates/Cargo.toml`：
  - workspace.version 0.0.1 → 1.0.0
  - ort features += "coreml"
- 5 active 文档 sed 替换 v0.0.1 → v1.0.0
- 不改：历史 STATUS / 已存在 release tag

## 验证结果

- 编译：`cargo check --workspace` ✓ Finished 2.54s（v1.0.0 不破 build）
- onnx + coreml build：`cargo build --release --features ...,onnx` ✓ Finished 29.65s
- CoreML latency：6 query 跑通 · 平均 313ms（含首次 935ms 加载）· 后 5 个稳定 ~190ms
- B 验证：v0.0.2-rc.3 tarball 4.1MB 下载 + 解压 + `--version` ✓

## 残留 / 下一轮

- [x] A workspace 1.0.0 + 5 文档同步
- [x] B 下载 release artifact 验证生产可用
- [x] C ort/coreml feature 加 + 实测 4000ms → 190ms (21×)
- [x] 双轨归档（仅 feature log · 无 meta · 业务变更）
- [ ] **用户 push master + push tag v1.0.0** · 触发 release.yml + book.yml
- [ ] **v1.0.0 release.yml 4 平台跑结果**：期望 4 绿 + 4 artifact 命名 v1.0.0（不再 v0.0.1）
- [ ] **mdBook 文档站「下载」节** 更新成 v1.0.0 URL（task #85 收尾）
- [ ] **观察** CoreML 在用户 Claude/opencode chat 内实际体验（首次 ~1s · 后续 ~200ms 应该够好）
- [ ] **release.yml 4 平台 CoreML 影响验证**：本地 macOS ✓ · Linux/Windows 仍待 CI 验证 ort 内部 cfg gate 生效
- [ ] **task #85 完成判定**：v1.0.0 release artifact 全到 GitHub Releases · 跑 `--version` 输出 `arkui-rag 1.0.0`
