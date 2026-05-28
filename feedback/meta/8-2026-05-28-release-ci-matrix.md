# 8 — release-ci-matrix

> 日期：2026-05-28
> 触发：用户「继续」指令 + AskUserQuestion 决策（双推 GitHub + gitcode · tag 触发）
> 类型：工程结构调整（新增 `.github/workflows/release.yml` · CI 自动 release · 双 remote 工作流）

---

## 用户提出的要求

承接 Day 20a 完成后的 agent 推荐「Day 20b CI matrix」。用户「继续」前，agent 主动用 AskUserQuestion 问了两个决策点：

| 决策 | 用户选 |
|---|---|
| release.yml 写给哪个 CI / 推哪个远端? | **双推 GitHub + gitcode（mirror）** |
| release 触发方式? | **git tag v* 触发（业界标准）** |

含义：
- workflow 只在 GitHub Actions 跑（gitcode 仅 mirror 代码 · 不跑 CI）
- 用户本地配双 remote：`origin` push to both / 或独立 `gitcode` remote
- 1.0 release 流程：`git tag v1.0.0 && git push --tags`

## Agent 给出的修改建议

新增 `.github/workflows/release.yml`（独立于现有 ci.yml 的 PR 校验 workflow）：

| 设计点 | 选择 | 理由 |
|---|---|---|
| 触发 | `push.tags=v*` + `workflow_dispatch` | 业界标准 + 失败重试方便 |
| Build matrix | 4 平台原生 build（不跨编） | 每 runner `rustc -vV host:` 就是 target · 大幅简化 yml |
| 打包脚本 | 复用 `scripts/release-local.sh` | 与本地 dev 完全一致 · 单一信任源 |
| 打包格式 | 全平台 tar.gz（含 windows） | Win10+ 内置 tar.exe · 避免 zip 依赖 · 跨平台一致 |
| Artifact 上传 | `actions/upload-artifact@v4` 保留 7 天 | debug 用 · 不长占空间 |
| Release 上传 | `softprops/action-gh-release@v2` | Rust 社区事实标准 · 自动建 Release page |
| Release notes | `generate_release_notes: true` | GitHub 自动从 commit 生成 changelog |
| Fail behavior | `fail-fast: false` | 第一次 release 容错 · 一平台失败不阻塞其他 |

### 替代方案权衡（被否）

- 用 `cargo-dist`（业界自动化标准）：workspace 要加 metadata · 远程拉工具 · 手撸 yml 更显式
- 全 matrix 改 macos-14 跨编 x86_64：跨编 macOS 需 SDK 配置 · macos-13 当前还能用 · 留 follow-up
- 把 onnx 进默认 matrix：ONNX Runtime 跨平台分发复杂 · Day 20c 独立处理

## 多轮互动

按时序：
1. Day 20a commit 后 agent 推荐 Day 20b · 用户回「Day 20 跨平台二进制构建（轻量解锁分发）是什么」（指向 Day 20a 解释）
2. Agent 解释 Day 20 全局 · 用户选「先不对接 DevEco · 优先端到端本地跑 CLI」→ 执行 Day 20a
3. Day 20a 完成后 agent 推荐 Day 20b · 用户回「继续」
4. Agent 用 AskUserQuestion 问决策 · 用户选「双推 + tag 触发」
5. Agent 直接执行（无返工）

## 实际改动

- **接口变化**：无（不动 Rust 代码 / CLI 参数）
- **规则变化**：无
- **文件变化**：
  - 新增 `.github/workflows/release.yml`（~120 行 · meta）
  - 改 `scripts/release-local.sh`（windows 用 tar.gz + BIN_SUFFIX 支持 .exe · meta）
  - 改 `docs/RELEASE.md`（实装 Day 20b 章节 + 双 remote 配置说明 · business）
  - 改 `README.md`（顶部 Download 章节加 GitHub Releases 路径 · meta）
  - 改 `docs/ROADMAP.md`（第 10 次实战 · Week 6 进度 2/4 → 3/4 · business）
- **配置变化**：
  - GitHub Actions 新增 release workflow（push tag 触发）
  - `permissions: contents: write` 用于 Release page 创建 + artifact 上传

## 执行生效后总结

### 实际产出

| 产物 | 状态 |
|---|---|
| `.github/workflows/release.yml` | 新建 · 静态 YAML 校验通过（ruby YAML.load_file 解析 2 jobs：build + release） |
| `scripts/release-local.sh` | windows tar.gz + BIN_SUFFIX 兼容（本地 macOS 实测仍通过） |
| 4 平台 matrix runner 配置 | macos-14 / macos-13 / ubuntu-latest / windows-latest |
| GitHub Release page 自动化 | softprops/action-gh-release@v2 + generate_release_notes |

### 前后对比

| 维度 | Day 20a 完成时 | Day 20b 完成时 |
|---|---|---|
| 分发能力 | 仅本地 host 平台 | **4 平台自动 build + 上传** |
| 用户操作 | 本地 `make release-local` | 仅 `git tag v* && git push --tags` |
| Release page | 无 | 自动建 + changelog 自动生成 |
| 跨平台覆盖 | 1 平台（开发机） | **4 平台**（aarch64/x86_64 darwin · x86_64 linux/windows） |
| Week 6 进度 | 2/4 | **3/4** ⭐ |

### 实测验证

```
$ ruby -ryaml -e "puts YAML.load_file('/Users/leo/work/RAG4ArkUI/.github/workflows/release.yml')['jobs'].keys.inspect"
["build", "release"]
✅ YAML 合法 · 2 jobs 解析正确

$ make release-local 2>&1 | tail
✅ Release artifact 完成
  产物 : dist/arkui-rag-v0.0.1-aarch64-apple-darwin.tar.gz (2.9M)
  sha  : 09d1f0906a5dfc41fda704b2a69dfbb6dae97d8aa3e9b7dcfdc16970ff4254e0
```

工作流首次跑必须 push tag · 静态校验已尽 agent 范围内的最大努力 · 余下风险由用户首次 tag 触发暴露。

### 残留 / 下一轮处理

- [ ] **关键**：用户跑第一次 release（建议先 `v0.0.2-rc.1` 试跑 · 失败可删 tag · 详见 docs/RELEASE.md「失败重试」）
- [ ] **第一次 release 后**：观察 Windows tantivy build 是否成功（tantivy 0.22+ 应跨平台 · 不保证）
- [ ] **第一次 release 后**：把 GitHub Actions release status badge 加到 README.md
- [ ] **第一次 release 后**：把 status badge 加到 docs/RELEASE.md
- [ ] **macos-13 deprecate 跟进**：未来切到 `macos-14` + 跨编 `--target=x86_64-apple-darwin`
- [ ] **Day 20c**：单独发 onnx 版 release（需 ONNX Runtime 跨平台分发指南 · 复杂）
- [ ] gitcode 双推流程的用户文档（docs/RELEASE.md 已写 · 等用户验证）
- [x] release.yml workflow 写完 + 静态校验
- [x] release-local.sh windows 兼容（tar.gz + BIN_SUFFIX）
- [x] docs/RELEASE.md 实装 Day 20b 章节
- [x] README Download 章节加 GitHub Releases 路径
- [x] 双轨归档（本 meta 8 + feature/rag4arkui-core/21）
