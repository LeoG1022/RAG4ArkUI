# STATUS · Day 20a · 本地 Release Artifact

> 日期：2026-05-28
> 对应 commit：[本 commit · Day 20a Release Local]
> 对应 feature log：[`feedback/features/rag4arkui-core/20-2026-05-28-day20a-release-local.md`](../feedback/features/rag4arkui-core/20-2026-05-28-day20a-release-local.md)
> 上一阶段：[`STATUS-day16-lsp.md`](STATUS-day16-lsp.md)
> 下一阶段：`STATUS-day20b-ci-matrix.md`（推荐 · CI matrix 自动 release）或 `STATUS-day21-corpus-pull.md`

> 🎯 **里程碑**：**Week 6 启动 · 本地 host CLI 端到端可下载即用** ⭐（按用户指令裁剪：先不对接 DevEco · 先把单文件二进制跑通分发链路）

---

## 当前状态

| 模块 | 变化 |
|---|---|
| `scripts/release-local.sh` | **新建** ~190 行 · 自动探测 host triple + cargo build + tar/zip 打包 + sha256 |
| `Makefile` | + `release-local` / `release-local-verify` target |
| `docs/RELEASE.md` | **新建** · Release 与分发指南 |
| `README.md` | 顶部加 Download 章节 + 当前状态更新到 Day 20a |
| `docs/ROADMAP.md` | 第 9 次实战 · Week 6 进度 1/4 → 2/4 |

### 产物

| 项 | 值 |
|---|---|
| 平台 | `aarch64-apple-darwin`（Mach-O 64-bit · arm64） |
| Binary 大小 | **6.7 MB**（strip + thin-LTO + opt-level=3） |
| Tarball 大小 | **2.9 MB**（gzip ~57%） |
| 编译耗时 | 37s（冷缓存增量 < 5s） |
| 运行时依赖 | 仅 `libSystem.B.dylib` + `libiconv.2.dylib`（macOS 系统库） |
| 启用 features | `http,mcp,lsp,tantivy` |

---

## 输入契约

### Agent 操作（本轮）

```bash
make release-local              # 编译 + 打包
# 或:
bash scripts/release-local.sh
bash scripts/release-local.sh --features http,mcp,lsp     # 自定义
bash scripts/release-local.sh --skip-build                # 只重新打包
```

### 用户操作（下游消费者）

```bash
# 1. 下载 tarball（GitHub Releases · Day 20b 自动化前需手动从 dist/ 拷贝）
curl -LO https://github.com/LeoG1022/RAG4ArkUI/releases/download/v0.0.1/arkui-rag-v0.0.1-aarch64-apple-darwin.tar.gz

# 2. 解压
tar -xzf arkui-rag-v0.0.1-aarch64-apple-darwin.tar.gz
cd arkui-rag-v0.0.1-aarch64-apple-darwin

# 3. 验证
./arkui-rag --version              # arkui-rag 0.0.1

# 4. 上 PATH
cp ./arkui-rag /usr/local/bin/     # 或自己的 ~/.local/bin/

# 5. 索引 + 检索（参考 INSTALL.txt）
arkui-rag index --source ./my-corpus --bm25 tantivy ...
arkui-rag query --text "..." -k 5

# 6. 启 MCP 接 Claude Code（详见 docs/MCP-INTEGRATION-CLAUDE-CODE.md）
arkui-rag serve --mcp --index-path ...
```

---

## 输出契约

### tarball 结构

```
arkui-rag-v0.0.1-aarch64-apple-darwin.tar.gz
└── arkui-rag-v0.0.1-aarch64-apple-darwin/
    ├── arkui-rag          # 二进制 · 6.7 MB · executable
    ├── INSTALL.txt        # 6 步快速上手 + 「本包未包含的能力」明示
    ├── LICENSE            # MIT
    └── README.md          # 项目说明
```

### `dist/SHA256SUMS`

```
cf94f931ff6424d6772ef7f6f7db4d13a8d4f949931d6584f8e5a6e9202a7ea0  arkui-rag-v0.0.1-aarch64-apple-darwin.tar.gz
```

每次跑 `release-local` 会累加新一行。

### `make release-local-verify` 输出

```
[1/4] cargo build --release --features http,mcp,lsp,tantivy
[2/4] 烟雾测试 --version → arkui-rag 0.0.1
[3/4] 暂存产物到 dist/.staging/...
[4/4] 打包为 .tar.gz
✅ Release artifact 完成
━━━ 解压验证 ━━━
arkui-rag 0.0.1
✅ release tarball 端到端可用
```

---

## 验证手段

### 用户手动

```bash
# 全链路自验证（推荐）
make release-local-verify

# 或：拆解执行
make release-local
ls -lh dist/
tar -tzf dist/arkui-rag-v0.0.1-aarch64-apple-darwin.tar.gz | head

# 验证 binary 是否真能独立跑（不依赖 /Users/leo/...）
mkdir -p /tmp/test && tar -xzf dist/*.tar.gz -C /tmp/test
/tmp/test/arkui-rag-v0.0.1-*/arkui-rag --version
```

### 端到端 5 路径（本轮全过）

| 路径 | 验证命令 | 结果 |
|---|---|---|
| 编译 | `cargo build --release --features http,mcp,lsp,tantivy` | ✅ 37s |
| 打包 | `make release-local` | ✅ tarball 2.9 MB + SHA256 |
| 解压 + 跑 | `tar -xzf dist/*.tar.gz -C /tmp && /tmp/.../arkui-rag --version` | ✅ `arkui-rag 0.0.1` |
| Index | `arkui-rag index --source corpus --bm25 tantivy` | ✅ 3 files → 22 chunks · 132ms |
| Query | `arkui-rag query --text "下拉刷新" --bm25 tantivy -k 3` | ✅ Top-3 命中 + 引用溯源 |
| Serve HTTP | `arkui-rag serve --http` + `curl /health /search` | ✅ JSON 响应正确 |
| Serve MCP | `arkui-rag serve --mcp` + `initialize` + `tools/list` | ✅ 4 tools 暴露 |
| Serve LSP | `arkui-rag serve --lsp` + Content-Length `initialize` | ✅ capabilities 返回 |

### 自动化

| 手段 | 范围 | 状态 |
|---|---|---|
| `make release-local-verify` | 打包 + 解压 + --version | ✅ |
| **M-STATUS-PER-ROUND** | Round 20 + STATUS-day20a 配套 | ✅ |
| **ROADMAP 维护约定（第 9 次实战）** | 当前位置 + Week 6 进度 + 已完成表 | ✅ |

### 暂未自动化（明确缺口）

- ❌ `release.yml` GitHub Actions matrix 自动 release（Day 20b）
- ❌ tarball 内置 `arkui-rag --version` 在 CI 中验证（Day 20b）
- ❌ 跨平台真编 + 跨平台跑（需 GitHub Actions runner）
- ❌ ONNX feature 包 release（需用户额外装 ONNX Runtime · Day 20c）
- ❌ corpus 默认包内置（Day 21）

---

## 与上一阶段（STATUS-day16）的关联性

### 增量

| 维度 | Day 16 完成时 | 本轮（Day 20a）后 |
|---|---|---|
| Crate 数 | 9 | 不变 |
| 协议层数 | 3（HTTP + MCP + LSP） | 不变 |
| **可分发产物** | ❌ 仅源码 + `cargo build` | ✅ **tarball + sha256 + INSTALL.txt** |
| CLI 编译耗时 | 未实测 | **37s（cold + LTO）** |
| Binary 大小 | 未实测 | **6.7 MB** |
| Tarball 大小 | — | **2.9 MB** |
| 端到端验证脚本 | smoke + mcp-demo | + **release-local-verify** |

### 与 Week 6 进度

| Week 6 切片 | 状态 |
|---|---|
| 评估报告（Day 6） | ✅ |
| **本地 release artifact**（Day 20a 本轮） | ✅ ⭐ |
| CI matrix + GitHub Releases（Day 20b） | ⏳ |
| Corpus 分发管道（Day 21） | ⏳ |
| 文档站 + 1.0（Day 22） | ⏳ |

Week 6 进度 **2/4** ✅。

### 兼容性

- ✅ 无破坏性变更（只增不改）
- ✅ 不动现有 CI workflow（ci.yml 保持 Linux only check/test/fmt/clippy）
- ✅ dist/ 已在 .gitignore（产物不入库）
- ✅ Cargo.toml release profile 复用（strip / thin-LTO / opt-level=3 已存在）

---

## 完成度 / 下一阶段

### Day 20a 完成度

| 项 | 状态 |
|---|---|
| 本地 host release artifact（aarch64-apple-darwin） | ✅ |
| `scripts/release-local.sh` 自动探测 + tar/zip + sha256 | ✅ |
| Makefile `release-local` + `release-local-verify` | ✅ |
| `docs/RELEASE.md` 完整发布指南 | ✅ |
| README 顶部 Download 章节 | ✅ |
| 端到端 7 路径全验证（编译 → 打包 → 解压 → index → query → 三协议 serve） | ✅ |
| CI matrix（4 平台 release.yml） | ⏳ Day 20b |
| GitHub Releases 自动上传 | ⏳ Day 20b |
| ONNX feature 分发渠道 | ⏳ Day 20c |
| pre-existing 阻塞（lancedb / typescript） | ⏳ 独立 follow-up |

### 6 周路线图达成度更新

| 章节 | 状态 |
|---|---|
| Week 1: Rust 骨架 + tree-sitter + LanceDB + Tantivy + BGE-M3 | **7/7** ✅ |
| Week 2: 混合检索 + Reranker + HyDE + 评估集 | **4/4** ✅ |
| Week 3: HTTP + MCP + CLI | **3/3** ✅ |
| Week 4: 协议层（HTTP + MCP + LSP） | **3/3** ✅ ⭐ |
| Week 5: Claude Code 接入 | **1/1** ✅ |
| **Week 6: 发布 + 文档站 + 评估报告** | **2/4** ✅（评估 ✓ · 本地 release ✓） |

**总完成度估算：~78%**（Day 16 75% + Day 20a 3%）

### 下一阶段建议（按推荐优先级）

| 候选 | 价值 | 工作量 |
|---|---|---|
| 🟢 **Day 20b CI matrix + GitHub Releases** | 多平台自动分发 · 1.0 必备 | 1 commit · 主要是 release.yml 调试 |
| 🟢 Day 21 corpus model-pull 真活 | 用户无脑接入 · 不用自己投放 corpus | 2 commit |
| 🟢 Day 22 mdBook 文档站 + 1.0 release | 公开发布 | 1-2 commit |
| 🟡 Day 17 DevEco Plugin MVP | 关键路径主战场 · 但工作量大 | 5+ commit |
| 🟡 pre-existing 阻塞清理（lancedb + typescript） | 解锁 full feature release | 1-2 commit · 看 chrono/arrow-arith 版本博弈 |
| 🟡 Day 20c onnx release 分发 | 让用户可以用真语义 embedding | 2 commit |

**Agent 推荐**：**Day 20b CI matrix**（顺势推进 · 4 平台 release.yml）。理由：
1. Day 20a 已经验证本地分发链路通畅 · 把同一脚本推进 CI 就能解锁 4 平台
2. 工作量 1 commit（主要是 GitHub Actions yml 调试 + secret 配置）
3. 完成后整个 RAG4ArkUI 可被 Mac / Linux / Windows 用户从 GitHub Releases 一键下载

**备选**：**Day 21 corpus 分发**（用户不用自己投放 corpus · 一键 model-pull）。

### 重要的"非完成"项

- ❌ CI matrix release.yml + softprops/action-gh-release 自动上传
- ❌ ONNX 真语义 embedding 包分发
- ❌ 跨平台真编（仅 host arm64 验证）
- ❌ pre-existing 阻塞：tree-sitter-typescript / lancedb 编译失败
- ❌ DevEco Plugin / VSCode Extension 真实接入端到端
