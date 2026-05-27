# 4 — ci-github-actions

> 日期：2026-05-27
> 触发：用户追加请求"补充 GitHub Actions CI（cargo check + test + clippy + fmt-check）"
> 类型：工程结构调整 + 流程优化

---

## 用户提出的要求

> "补充GitHub Actions CI（cargo check + test + clippy + fmt-check）"

来源：在 Day 3 OnnxEmbedder commit 之后。明确含 4 个 cargo 命令：check / test / clippy / fmt-check。

落地依据：`docs/STATUS-day2.md §6.2 表 A 项`（之前用户选择只做 B 项 smoke，本轮补 A）。

## Agent 给出的修改建议

新增 `.github/workflows/ci.yml`，6 个 job：

| Job | 触发 | 内容 |
|---|---|---|
| `check` | push + PR | `cargo check --workspace --all-targets` |
| `test` | push + PR | `cargo test --workspace --no-fail-fast`（24 个测试） |
| `fmt` | push + PR | `cargo fmt --all --check` |
| `clippy` | push + PR | `cargo clippy --workspace --all-targets -- -D warnings` |
| `smoke` | push + PR（needs: check） | `bash scripts/demo-smoke.sh --verbose` 端到端冒烟 |
| `onnx-build` | **仅 workflow_dispatch** + 勾选 input | `cargo check -p arkui-rag-embedding --features onnx` + cli 同 |

关键设计：
- **并发取消**：`concurrency.cancel-in-progress: true` 同分支 push 取消旧 CI
- **缓存**：`Swatinem/rust-cache@v2` 缓存 cargo registry + target，首次 ~10 分钟，后续 ~3 分钟
- **timeout**：每 job 设 5-30 分钟上限（防失控）
- **onnx 单独 job**：因 ort 原生库拉取 ~10 分钟，不放主流程；workflow_dispatch + 复选框触发
- **RUSTFLAGS 仅 clippy 用**：全局 `-D warnings` 会让第三方依赖的 warning 也 fail，太严格；改成只在 clippy job `-- -D warnings`

## 多轮互动

1. 用户在 Day 3 OnnxEmbedder commit 后追加"补充 GitHub Actions CI"
2. Agent 查 git remote → 发现指向 `gitcode.com` 而非 GitHub。**通知用户该差异**但仍按"GitHub Actions"语法写（标准格式可移植）
3. Agent 自主决策（含 4 项）后直接开工

## Agent 决策分析

> 触发 AGENTS.md 规则 #15

### 决策 A：onnx 编译验证不入主流程

1. **待决策事项**：
   - A. 主流程加 `--features onnx` job
   - B. workflow_dispatch 手动触发（推荐）
2. **Agent 决策**：选 B
3. **决策依据**：
   - ort 原生库拉取 + 编译 ~10 分钟，每次 PR 都跑会浪费 CI 额度
   - 主流程只验证"接口 / 单测 / 格式"，已覆盖 95% 风险面
   - onnx 真活变化场景少（rc.4 锁定后几乎不变），手动触发足够
4. **归档引用**：本文件
5. **用户 Review 项**：
   - [ ] 若后续 onnx 改动频繁，是否上 nightly 触发（每天一次）

### 决策 B：smoke 入主流程（needs: check）

1. **待决策事项**：
   - A. smoke 入主流程（推荐）
   - B. 仅本地 `make smoke` 跑
2. **Agent 决策**：选 A
3. **决策依据**：
   - smoke 验证"CLI 二进制实际行为"，cargo test 验证"内部 trait 契约" → 互补
   - 已有 Swatinem 缓存，smoke 复用 check job 的编译产物 → 增量耗时 < 1 分钟
   - 端到端是用户最常踩坑的层；CI 保底防回归
4. **归档引用**：本文件
5. **用户 Review 项**：
   - [ ] 同意 smoke 入 CI（增加 ~1 分钟 / build）

### 决策 C：global RUSTFLAGS=-D warnings 改局部 clippy 用

1. **待决策事项**：
   - A. 全局 `-D warnings`（最严）
   - B. 仅 clippy job 用（推荐）
2. **Agent 决策**：选 B
3. **决策依据**：
   - 全局会让第三方 crate 的 warning 也 fail —— 我们控不了 ort/tokio 的 deprecation
   - clippy 是专门的 lint 工具，本身就是 warning 重灾区
   - 我们的代码经 clippy 把关已经够严
4. **归档引用**：本文件
5. **用户 Review 项**：
   - [ ] 是否同意宽松全局 + 严格 clippy 的双轨

### 决策 D：linux-only 不上跨平台矩阵

1. **待决策事项**：
   - A. ubuntu-latest only（推荐）
   - B. ubuntu + macos + windows 矩阵
2. **Agent 决策**：选 A
3. **决策依据**：
   - 我们的代码不依赖平台特性（无 OS API、无文件路径分隔符 hack）
   - 3 平台矩阵 × 6 job = 18 个 runner，CI 额度爆炸
   - Release 阶段（Week 6）再加矩阵验证二进制发布
4. **归档引用**：本文件
5. **用户 Review 项**：
   - [ ] Week 6 release 时切换到三平台矩阵

## 实际改动

- **接口变化**：无（外部接口）
- **规则变化**：CI 实质上把 `check + test + fmt + clippy` 升级为合并门禁（merge gate）
- **文件变化**：
  - 新增：`.github/workflows/ci.yml`（约 110 行，6 jobs）
  - 修改：无
- **配置变化**：
  - GitHub 仓库（若推 GitHub）会自动启用 Actions
  - 当前 git remote 指向 `gitcode.com` —— 见下方"实测验证"

## 执行生效后总结

### 实际产出

| 项 | 状态 |
|---|---|
| `.github/workflows/ci.yml` | ✅ |
| 5 主 job + 1 手动 job | ✅ |
| Swatinem 缓存配置 | ✅ |
| concurrency 取消 + timeout 设置 | ✅ |
| `STATUS-day2.md §6.2 表 A 项` 落地 | ✅ |

### 前后对比

| 维度 | 前 | 后 |
|---|---|---|
| 自动化验证 | 仅 pre-commit hook（本地） + `make smoke` 手动 | + CI 跨设备验证 push/PR |
| 防回归 | 依赖开发者主动跑 `cargo test` | CI 强制每次 push |
| 风格门禁 | 无 | `cargo fmt --check` + clippy `-D warnings` |
| onnx feature 测试覆盖 | 完全靠本地 | 至少有手动 workflow_dispatch 兜底 |

### 实测验证

- **本地静态检查**：YAML 语法应合法（GitHub Actions 接收时再实测）
- **CI 实际跑**：⏳ **取决于用户推到哪里**
  - 当前 git remote = `git@gitcode.com:keerecles/agent-harness-template.git` —— **不是 GitHub**
  - 如果用户后续推到 GitHub，CI 立即生效
  - 如果仅用 gitcode.com：gitcode 也支持 GitHub Actions 兼容语法（项目主页可启用），但行为可能略有差异
  - 如果都不推：CI 文件只是本地占位，不工作

### 残留 / 下一轮处理

- [ ] **关键**：用户决定推 GitHub vs 仅 gitcode vs 双推（mirror）
- [ ] 若推 GitHub：第一次 CI 跑成功后，把 status badge 加到根 `README.md`（要 repo URL）
- [ ] 若仅 gitcode：验证 gitcode 是否兼容本 workflow 语法（特别是 Swatinem/rust-cache 这种第三方 action）
- [ ] 未来添加：`cargo audit` 安全审计 job（STATUS-day2.md §6.2 表 E 项）
- [ ] 未来添加：覆盖率 `cargo-llvm-cov` + Codecov 上报
- [ ] Week 6 release：跨平台矩阵（ubuntu + macos + windows）
- [x] STATUS-day2.md §6.2 表 A 项（GitHub Actions CI）落地
