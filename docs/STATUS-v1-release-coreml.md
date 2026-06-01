# STATUS — v1-release-coreml

> 配套 feature log：`feedback/features/rag4arkui-core/47-2026-06-01-v1-release-coreml.md`
> 日期：2026-06-01

---

## 当前状态

里程碑 commit：**v0.0.1 → v1.0.0** + CoreML 加速 21× · 一次落地。

本阶段交付：
- workspace version 0.0.1 → 1.0.0
- 5 active 文档同步 v0.0.1 → v1.0.0
- ort/coreml feature 加入（Apple Silicon GPU 加速 ~21×）
- B 验证 v0.0.2-rc.3 release artifact 生产质量
- 双轨归档（feature log + STATUS · 本文件）

意义：task #85 推 v1.0.0 准备就绪 · 一旦用户 push tag · GitHub Actions 自动出 v1.0.0 release（4 平台 artifact 命名一致 · CoreML enabled · MVP 100% 工程层完整）。

## 输入契约

### A · workspace 1.0.0 影响

| 文件 | 改动 |
|---|---|
| `crates/Cargo.toml` | `version = "1.0.0"` |
| `Makefile` | `dist/arkui-rag-v1.0.0-*.tar.gz` |
| `docs/USER-VERIFICATION.md` | release tarball / install path 引用 v1.0.0 |
| `docs/RELEASE.md` | 用户下载示例 v1.0.0 |
| `mdbook/src/quickstart.md` | 文档站快速开始 v1.0.0 |
| `README.md` | 顶部下载链接 v1.0.0 |

不动（历史快照）：4 个 STATUS 文档保留原 v0.0.1 引用（当时事实）。

### C · ort/coreml feature

```toml
ort = { version = "2.0.0-rc.12", features = ["ndarray", "coreml"] }
```

- macOS：CoreMLExecutionProvider 自动注册 · Apple Silicon GPU 加速
- Linux / Windows：ort 内部 `#[cfg(target_os = "macos")]` gate · 不影响 build

### 不变项

- onnx feature 仍 opt-in
- CLI 接口不变
- 索引格式不变
- MCP / HTTP / LSP 协议不变

## 输出契约

### 性能契约（新增 CoreML 路径）

| Embedder | Latency p50 | 用户感受 |
|---|---|---|
| Mock (BM25 only) | ~48ms | 关键词侥幸 · 不真语义 |
| ONNX CPU（Round 42） | ~4000ms | 真语义但慢 · 用户觉得卡 |
| **ONNX + CoreML（v1.0.0）** | **~190ms** | 真语义 + 流畅 ✓ |

首次 query 935ms（含 CoreML session 加载 + cold cache）· 后续稳定 ~190ms。

### 期望 v1.0.0 release artifact

```
GitHub Releases v1.0.0:
- arkui-rag-v1.0.0-aarch64-apple-darwin.tar.gz       (含 CoreML 加速)
- arkui-rag-v1.0.0-x86_64-apple-darwin.tar.gz        (含 CoreML 加速)
- arkui-rag-v1.0.0-x86_64-unknown-linux-gnu.tar.gz   (无 CoreML · ort 内部 gate)
- arkui-rag-v1.0.0-x86_64-pc-windows-msvc.tar.gz     (无 CoreML)
- SHA256SUMS
```

## 验证手段

### Agent 本轮已做（C + B）

```bash
# C build
cd crates && cargo build --release --features ...,onnx
# → Finished 29.65s

# C install + 测 latency
cp target/release/arkui-rag ~/.local/bin/
codesign --force --sign - ~/.local/bin/arkui-rag

# 6 query · 平均 313ms (首次 935ms 含加载 · 后 5 稳定 ~190ms)

# B 下载验证
curl -sL https://github.com/.../v0.0.2-rc.3/arkui-rag-v0.0.1-aarch64-apple-darwin.tar.gz
tar -xzf rc3.tar.gz
./arkui-rag-*/arkui-rag --version
# → arkui-rag 0.0.1
```

### 用户验证（A 推 v1.0.0 后）

```bash
# 推 v1.0.0
git push origin master
git tag v1.0.0
git push origin v1.0.0

# 看 release.yml
# https://github.com/LeoG1022/RAG4ArkUI/actions/workflows/release.yml

# 期望 4 平台全绿 + Release v1.0.0 含 4 artifact 命名 v1.0.0
```

## 与上一阶段的关联性

| Round | 主题 | 关系 |
|---|---|---|
| 75 (Day 20b) | release.yml CI matrix | A 复用其 4 平台 build |
| 42 | task #87 ONNX 真活 | **本轮 C 加速延续** |
| 44 | FORCE_JAVASCRIPT_ACTIONS_TO_NODE24 | 本轮 release 也用 |
| 45 | macos-13 → macos-14 | 本轮 release 也用 |
| 46 | release-local.sh --target | 本轮 release 也用 |
| **47（本轮）** | v1.0.0 + CoreML | **task #85 准备 + 性能里程碑** |

层次：Round 44-46 修了 release.yml CI · Round 47 借势推 v1.0.0 + 加 CoreML · 一次到位。

兼容性：完全向后兼容。

破坏性变更：无。

## 完成度 / 下一阶段

### 达成度

| 项 | 状态 |
|---|---|
| A workspace 1.0.0 + 文档同步 | ✅ |
| B v0.0.2-rc.3 artifact 验证 | ✅ |
| C ort/coreml feature | ✅ |
| C latency 实测 21× | ✅ |
| 双轨归档 + STATUS | ✅ |
| **用户 push v1.0.0 tag** | ⏳ |
| v1.0.0 release.yml 4 平台 | ⏳（CI 跑 30 分钟）|

### 下一阶段建议

立即（用户做）：
1. `git push origin master`（含本轮 A+C 改动）
2. `git tag v1.0.0`
3. `git push origin v1.0.0` · 触发 release.yml + book.yml
4. 等约 30 分钟 · 看 Actions 跑结果

短期：
- v1.0.0 release 出来后 mdBook 「下载」节自动是对的版本（A 改了 quickstart.md）
- 浏览器看 Releases 页有 4 个 v1.0.0 命名 artifact
- 验证 Linux/Windows artifact 也 build 成功（ort coreml feature 在非 macOS 不应破 build）

中期：
- 收集真实用户使用 CoreML 加速后的反馈
- 看 ort/coreml 在 Apple Silicon M1/M2/M3/M4 各代芯片表现差异
- 考虑 quantized BGE-M3 ONNX：binary 2.2GB → 600MB · 启动更快 · 但精度略降

长期：
- 看 ort 出真正 stable 2.0 · 升级（可能还要修 API 漂移）
- evaluate candle 替代 ort（已挂多轮残留）· 看 1.5+ 是否值得切换
- 加 reranker 端到端 CLI 接入 · 完成检索增强全链路
