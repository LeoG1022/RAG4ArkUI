# STATUS · Day 20b · CI Matrix 自动 Release

> 日期：2026-05-28
> 对应 commit：[本 commit · Day 20b CI Matrix Release]
> 对应 feature log：[`feedback/features/rag4arkui-core/21-2026-05-28-day20b-ci-matrix.md`](../feedback/features/rag4arkui-core/21-2026-05-28-day20b-ci-matrix.md)
> 对应 meta：[`feedback/meta/8-2026-05-28-release-ci-matrix.md`](../feedback/meta/8-2026-05-28-release-ci-matrix.md)
> 上一阶段：[`STATUS-day20a-release-local.md`](STATUS-day20a-release-local.md)
> 下一阶段：`STATUS-day21-corpus-pull.md`（推荐 · `corpus model-pull` 真活）

> 🎯 **里程碑**：**Week 6 进度 3/4 · 4 平台 CI matrix 自动 release · tag `v*` 触发 GitHub Releases ⭐**

---

## 当前状态

| 模块 | 变化 |
|---|---|
| `.github/workflows/release.yml` | **新建** ~120 行 · push tag `v*` 触发 · 4 平台 matrix · 自动上传 GitHub Releases |
| `scripts/release-local.sh` | 小调 · windows 也用 tar.gz · `BIN_SUFFIX=".exe"` 支持 |
| `docs/RELEASE.md` | 实装 Day 20b 章节 · 双 remote 配置 · 失败重试流程 |
| `README.md` | Download 章节加 GitHub Releases curl 路径 |
| `docs/ROADMAP.md` | 第 10 次实战 · Week 6 2/4 → 3/4 ⭐ |
| `feedback/meta/8-...md` | 新增（meta · CI workflow + scripts/* + README + Makefile） |
| `feedback/features/rag4arkui-core/21-...md` | 新增（business · 业务进展） |

### 4 平台 matrix

| target | runner | 输出 |
|---|---|---|
| `aarch64-apple-darwin` | macos-14 | `arkui-rag-v<VER>-aarch64-apple-darwin.tar.gz` |
| `x86_64-apple-darwin` | macos-13 | `arkui-rag-v<VER>-x86_64-apple-darwin.tar.gz` |
| `x86_64-unknown-linux-gnu` | ubuntu-latest | `arkui-rag-v<VER>-x86_64-unknown-linux-gnu.tar.gz` |
| `x86_64-pc-windows-msvc` | windows-latest（git-bash） | `arkui-rag-v<VER>-x86_64-pc-windows-msvc.tar.gz` |

---

## 输入契约

### Agent 操作（本轮）

无（仅写 workflow + 静态校验 · 实际跑要用户 push tag）

### 用户操作（推 1.0 release 时）

```bash
# 1. 推 master 到 GitHub
git push github master

# 2. 打 tag
git tag v0.0.2
git push github v0.0.2

# 3. （可选）同步 tag 到 gitcode mirror
git push gitcode master --tags

# 4. 等 5-15 分钟看 Actions Release workflow 完成
#    Releases 页面会自动出现 v0.0.2 + 4 个 tarball + SHA256SUMS
```

### 双 remote 配置（GitHub 主 · gitcode mirror）

```bash
# 方案 A：保留两个独立 remote 名
git remote add gitcode git@gitcode.com:LeoG1022/RAG4ArkUI.git
# push 时分别：
git push github master
git push gitcode master

# 方案 B：origin 单 push 到两个 URL（推荐）
git remote set-url --add --push origin git@gitcode.com:LeoG1022/RAG4ArkUI.git
# git push origin 同步推到 github + gitcode
```

---

## 输出契约

### release.yml workflow 拓扑

```text
on:
  push.tags=v*  +  workflow_dispatch
  ↓
build matrix (4 jobs · parallel · fail-fast=false)
  each:
    actions/checkout@v4
    dtolnay/rust-toolchain@stable + targets:<target>
    Swatinem/rust-cache@v2
    bash scripts/release-local.sh --features "http,mcp,lsp,tantivy"
    actions/upload-artifact@v4 (7 day retention)
  ↓
release job (ubuntu-latest)
  actions/download-artifact@v4 (merge-multiple)
  compute consolidated SHA256SUMS
  softprops/action-gh-release@v2:
    tag_name: 自动从 ref 取
    name: 同 tag
    generate_release_notes: true
    fail_on_unmatched_files: true
    files: dist-all/arkui-rag-v*.tar.gz + SHA256SUMS
```

### GitHub Release page 长这样

```
v0.0.2
Released 2026-05-30 · 4 assets

📦 Assets
- arkui-rag-v0.0.2-aarch64-apple-darwin.tar.gz       (~3 MB)
- arkui-rag-v0.0.2-x86_64-apple-darwin.tar.gz        (~3 MB)
- arkui-rag-v0.0.2-x86_64-unknown-linux-gnu.tar.gz   (~3 MB)
- arkui-rag-v0.0.2-x86_64-pc-windows-msvc.tar.gz     (~3 MB)
- SHA256SUMS                                          (~500 bytes)
- Source code (zip / tar.gz)                          (auto)

📝 What's Changed（自动 generate_release_notes）
- feat: ... (#PR1)
- fix: ... (#PR2)
...
```

### 用户下载流程

```bash
# 1. 看 https://github.com/LeoG1022/RAG4ArkUI/releases
# 2. 复制目标平台的 tarball URL
curl -LO https://github.com/LeoG1022/RAG4ArkUI/releases/download/v0.0.2/arkui-rag-v0.0.2-aarch64-apple-darwin.tar.gz

# 3. （可选）校验 sha256
curl -LO https://github.com/LeoG1022/RAG4ArkUI/releases/download/v0.0.2/SHA256SUMS
shasum -c SHA256SUMS

# 4. 解压 + 跑
tar -xzf arkui-rag-v0.0.2-aarch64-apple-darwin.tar.gz
cd arkui-rag-v0.0.2-aarch64-apple-darwin
./arkui-rag --version
```

---

## 验证手段

### 静态校验（Agent 本地能跑）

| 手段 | 范围 | 状态 |
|---|---|---|
| ruby YAML 解析 release.yml | 检查 yml 合法 + 2 jobs 都在 | ✅ |
| `make release-local` 在 script 微调后跑 | 复用 release-local.sh 不破坏本地路径 | ✅ |
| `make release-local-verify` 端到端 | 解压 + --version 全过 | ✅（沿用 Day 20a 链路） |

### 动态校验（必须 push tag 才能跑）

| 手段 | 范围 | 状态 |
|---|---|---|
| 推 `v0.0.2-rc.1` 试触发 release workflow | 4 平台 matrix build 是否全过 | ⏳ 用户操作 |
| 看 Actions 页面 4 个 build job 都 ✅ | 跨平台 cargo build 成功 | ⏳ |
| Release page 自动出现 + 4 个 tarball | softprops/action-gh-release 工作 | ⏳ |
| 下载每平台 tarball 解压 + 跑 --version | 真跨平台用 | ⏳ |

### 自动化

| 手段 | 范围 | 状态 |
|---|---|---|
| YAML 语法校验 | 提交前防低级错误 | ✅ |
| `make release-local-verify` | 本地端到端（沿用 Day 20a） | ✅ |
| **M-STATUS-PER-ROUND** | Round 21 + STATUS-day20b 配套 | ✅ |
| **ROADMAP 维护约定（第 10 次实战）** | 当前位置 + Week 6 进度 + 已完成表 | ✅ |

### 暂未自动化（明确缺口）

- ❌ release.yml 第一次实跑（必须用户 push tag）
- ❌ Windows 平台 cargo build 跨平台兼容性（tantivy 0.22+ 应 OK 但需实测）
- ❌ macOS x86_64 兼容性（macos-13 runner deprecate 风险）
- ❌ GitHub Actions release status badge（待第一次 release 成功后加）
- ❌ ONNX feature 进 release matrix（Day 20c）
- ❌ corpus 内置（Day 21）

---

## 与上一阶段（STATUS-day20a）的关联性

### 增量

| 维度 | Day 20a 完成时 | Day 20b 完成时 |
|---|---|---|
| Crate 数 | 9 | 不变 |
| 分发能力 | 仅本地 host 平台 | **4 平台自动 build** |
| 触发方式 | `make release-local` 手动 | **`git tag v* && git push` 自动** |
| Release page 自动化 | 无 | ✅ softprops/action-gh-release |
| Changelog | 无 | ✅ `generate_release_notes: true` 自动 |
| SHA256SUMS | 仅本地 | ✅ CI 合并 4 平台 |
| 跨平台覆盖 | 1 平台 | **4 平台** ⭐ |
| Week 6 进度 | 2/4 | **3/4** ⭐ |

### 与 Week 6 进度

| Week 6 切片 | 状态 |
|---|---|
| 评估报告（Day 6） | ✅ |
| 本地 release artifact（Day 20a） | ✅ |
| **CI matrix 自动 release（Day 20b 本轮）** | ✅ ⭐ |
| Corpus 分发管道（Day 21） | ⏳ |
| 文档站 + 1.0（Day 22） | ⏳ |

Week 6 进度 **3/4** ✅。

### 兼容性

- ✅ 无破坏性变更（只增不改）
- ✅ 现有 ci.yml 不动（仍是 PR 校验 · Linux only）
- ✅ scripts/release-local.sh 本地路径仍通畅（windows tar.gz + BIN_SUFFIX 不影响 macOS / linux）
- ✅ dist/ 仍 gitignored

---

## 完成度 / 下一阶段

### Day 20b 完成度

| 项 | 状态 |
|---|---|
| `.github/workflows/release.yml` 4 平台 matrix | ✅ |
| tag `v*` + workflow_dispatch 双触发 | ✅ |
| softprops/action-gh-release 自动 Release page | ✅ |
| `generate_release_notes` 自动 changelog | ✅ |
| 合并 SHA256SUMS（4 平台） | ✅ |
| `fail-fast: false` 容错 | ✅ |
| `scripts/release-local.sh` windows 兼容（tar.gz + .exe） | ✅ |
| `docs/RELEASE.md` Day 20b 章节 + 双 remote 配置 | ✅ |
| README Download 章节加 GitHub Releases 路径 | ✅ |
| 静态 YAML 校验 | ✅ |
| 第一次 release 实跑 | ⏳ 用户操作 |
| Windows tantivy build 实测 | ⏳ 第一次 release 后 |
| macos-13 切跨编 | ⏳ deprecate 后 |
| ONNX feature release 分发 | ⏳ Day 20c |

### 6 周路线图达成度更新

| 章节 | 状态 |
|---|---|
| Week 1: Rust 骨架 + tree-sitter + LanceDB + Tantivy + BGE-M3 | **7/7** ✅ |
| Week 2: 混合检索 + Reranker + HyDE + 评估集 | **4/4** ✅ |
| Week 3: HTTP + MCP + CLI | **3/3** ✅ |
| Week 4: 协议层（HTTP + MCP + LSP） | **3/3** ✅ ⭐ |
| Week 5: Claude Code 接入 | **1/1** ✅ |
| **Week 6: 发布 + 文档站 + 评估报告** | **3/4** ✅（评估 ✓ · 本地 release ✓ · **CI matrix release ✓**） |

**总完成度估算：~80%**（Day 20a 78% + Day 20b 2%）

### 下一阶段建议（按推荐优先级）

| 候选 | 价值 | 工作量 |
|---|---|---|
| 🟢 **用户跑第一次 release（试 tag）** | 验证 4 平台 CI 真能跑通 | 0.5 commit · 出问题再 fix |
| 🟢 **Day 21 corpus model-pull 真活** | 用户无脑接入 · 不用自己投放 corpus | 2 commit |
| 🟢 Day 22 mdBook 文档站 + 1.0 release | 公开发布 | 1-2 commit |
| 🟡 pre-existing 阻塞清理（lancedb + typescript） | 解锁 full feature release | 1-2 commit |
| 🟡 Day 20c onnx release 分发 | 真语义 embedding 可用 | 2 commit |
| 🟡 Day 17 DevEco Plugin MVP | 关键路径主战场 · 工作量大 | 5+ commit |

**Agent 推荐**：**先让用户试推一个 `v0.0.2-rc.1` 验证 CI matrix 真能跑通**，再进 Day 21。理由：
1. 静态 YAML 校验只能查语法 · 实际跑可能浮出 Windows tantivy build / macos-13 runner deprecate 等问题
2. CI 跑通后才能在 README 挂 status badge（公开可信度）
3. Day 21 corpus 分发依赖有可工作的 release 渠道

**备选**：**Day 21 corpus 分发**（如果用户不急试 release · 可以先丰富 corpus 能力）。

### 重要的"非完成"项

- ❌ release.yml 第一次实跑验证
- ❌ Windows 平台 cargo build 跨平台兼容性实测
- ❌ macos-13 deprecate 切跨编方案
- ❌ ONNX 真语义 embedding 包分发（Day 20c）
- ❌ pre-existing 阻塞：tree-sitter-typescript / lancedb 编译失败
- ❌ DevEco Plugin / VSCode Extension 真实接入端到端
- ❌ corpus 内置默认包（Day 21）
- ❌ 1.0 release page（Day 22）
