# 20 — day20a-release-local

> 日期：2026-05-28
> 涉及代码：
> - `scripts/release-local.sh`（新建 · 本地 release artifact 打包脚本 ~190 行）
> - `Makefile`（+ `release-local` / `release-local-verify` target）
> - `docs/RELEASE.md`（新建 · Release 与分发指南）
> - `README.md`（顶部加 Download / 当前状态章节）
> - `docs/ROADMAP.md`（第 9 次实战：Week 6 进度 1/4 → 2/4）
> 类型：新建（Day 20 主线 · Week 6 启动 · 本地 host CLI 端到端分发）

## 本轮目标

把 RAG4ArkUI 从「开发者自己 cargo build」推进到「用户下载 tarball 解压即用」：
1. 本地 host 平台（aarch64-apple-darwin）的 release 二进制 + tarball 打包脚本
2. Makefile 一键 target + 自验证
3. 用户向 README + 完整 RELEASE.md 文档
4. CI matrix（多平台 release.yml + GitHub Releases 上传）留 Day 20b

## Plan

> 必备节（FAIL 级 M-FEATURE-PLAN）

### 范围裁剪（按用户指令）

用户原话：「先不对接 DevEco 等公局，优先端到端本地跑 CLI」。

- ✅ 做：本地 cargo build --release · 打 tarball · 解压 + 跑 --version + query 验证
- ❌ 不做：DevEco / VSCode / IntelliJ 插件接入
- ⏳ 推迟：跨平台 CI matrix（Day 20b） · GitHub Releases 自动上传 · curl 安装脚本

### Features 组合决策

默认 `http,mcp,lsp,tantivy`（约 6.7 MB · 37s 编译）

| feature | 选 | 理由 |
|---|---|---|
| http | ✅ | Day 14 REST 协议 |
| mcp | ✅ | Day 15 Claude Code/Cursor stdio |
| lsp | ✅ | Day 16 IDE 编辑器 |
| tantivy | ✅ | 真 BM25 倒排（远胜 in-memory） |
| typescript | ❌ | pre-existing：`tree-sitter-typescript` 0.21 把 `LANGUAGE_TYPESCRIPT` 改成 `language_typescript`，chunker/src/typescript.rs 引用未跟进 |
| lancedb | ❌ | pre-existing：`arrow-arith` 0.x 在新 chrono 上的 `quarter` 方法歧义编译失败 |
| onnx | ❌ | 体积膨胀 + 用户需另装 ONNX Runtime 原生库 · 单独 release 渠道分发 |

两处 pre-existing 阻塞已挂 follow-up（与 Day 16 spawn 的「eval/indexer 回归」并列）。

### 打包契约（scripts/release-local.sh）

```
dist/
├── arkui-rag-v<VER>-<TARGET_TRIPLE>.tar.gz    # 主产物
│   └── arkui-rag-v<VER>-<TARGET_TRIPLE>/
│       ├── arkui-rag         # 二进制（strip + thin-LTO + opt-level=3）
│       ├── INSTALL.txt       # 6 步快速上手
│       ├── LICENSE
│       └── README.md
└── SHA256SUMS                # 累积校验和
```

INSTALL.txt 显式列出「本包未包含的能力」（onnx / lancedb / typescript），不让用户误以为完整 full feature。

### 与既有 CI workflow（.github/workflows/ci.yml）的关系

`ci.yml` 是 PR 校验（check/test/fmt/clippy），跑 Linux only。
本轮 `release-local.sh` 是**本地** host 打包，**不动 CI**。Day 20b 加 `release.yml` 才进 CI matrix。

### 替代方案权衡（被否）

- 备选 1：直接写 `release.yml` + matrix 一步到位
  - 否决：用户明说「优先端到端本地跑 CLI」 + CI matrix 涉及 4 平台 yml 调试，工作量爆涨到 3-4 commit
- 备选 2：用 `cargo-dist` 自动化（业界标准）
  - 否决：要给 workspace 加 metadata + 远程拉 cargo-dist · 引入新工具 · Day 20a 范围内不必
- 备选 3：把 onnx 也打进默认 release
  - 否决：~300MB ONNX Runtime 不能进二进制 · 需用户另外装 + 配 LD_LIBRARY_PATH · 体验不一致

### 测试策略

1. cargo build --release --features http,mcp,lsp,tantivy → 通过（已实测 37s）
2. `make release-local` → 产物 sha256 唯一 + 暂存目录干净
3. 解压到 /tmp + 跑 --version → 0.0.1 ✓
4. 解压版 binary 跑 query 对预建 index → Top-K 命中 ✓
5. （已在 Day 16 验证）三协议 serve 起停 OK

## 改动要点

> API 选型 / 算法 / 关键决策

### 关键决策

1. **不引入 cargo-dist**：脚本手撸 ~190 行 shell · 与 mcp-demo.sh / demo-smoke.sh 风格一致
2. **不动 release profile**：workspace Cargo.toml 已有 thin-LTO + strip + opt-level=3 · 复用
3. **target triple 自动探测**：`rustc -vV | awk '/^host:/ {print $2}'` · 跨平台兼容
4. **dist/ 已在 .gitignore**：产物天然不入库 · 避免污染
5. **avoid pre-existing 阻塞**：默认 features 主动排除 lancedb/typescript · 不在本轮修这些
6. **`make release-local-verify` 一键真验证**：包含「打包 + 解压到 /tmp + 跑 --version」全链路

### 与 Day 16 的差异

- crate 数 9（不变）
- 测试数（不变 · Day 16 验证逻辑无变化）
- 新增 scripts/release-local.sh + Makefile target + docs/RELEASE.md
- README 顶部加用户向 Download 章节
- ROADMAP Week 6 从 1/4 → 2/4

## 对话摘要

> 必备节（FAIL 级 M-FEATURE-PLAN）

按时序：
1. **Day 16 commit `611cdcb` 后**，agent 建议下一步 Day 20 跨平台二进制
2. **用户问**：「Day 20 跨平台二进制构建（轻量解锁分发）是什么」
3. **Agent 答**：解释 4 个平台 / cargo-dist / 解锁分发关系 / 跟 Day 17 DevEco 工作量对比
4. **用户指令**：「继续 Day 20 先不对接 DevEco 等公局，优先端到端本地跑 CLI」
5. **Agent 自主决策**：
   - features 用 `http,mcp,lsp,tantivy`（避开 pre-existing 阻塞 lancedb/typescript）
   - 手撸 shell 脚本不引入 cargo-dist
   - 不写 release.yml（推迟到 20b）
6. **执行验证**：
   - 第一次跑 `cargo build --release --features http,mcp,lsp,tantivy,lancedb,typescript` 浮出 lancedb arrow-arith 编译错误
   - 退一步用 `http,mcp,lsp,tantivy,typescript` 又浮出 tree-sitter-typescript API 漂移
   - 最终用 `http,mcp,lsp,tantivy` 37s 编译通过
7. **端到端**：5 路径全过（index / query / HTTP / MCP / LSP）+ 解压后真跑 query 返回正确结果

## 验证结果

- ✅ `make check`（默认 features）：通过（611cdcb 后无回归）
- ✅ `make release-local`：tarball 产出 + SHA256 计算
  - 编译耗时：~37s（增量后 < 5s）
  - binary 大小：6.7 MB（arm64 mach-o · 仅依赖 libSystem/libiconv）
  - tarball 大小：2.9 MB（gzip 压缩 ~57%）
- ✅ `make release-local-verify`：解压到 /tmp + 跑 --version + 跑 query 全通过
- ✅ 端到端 5 路径（index / query / serve --http / --mcp / --lsp）：全通过（详见 STATUS-day20a §验证手段）

## 残留 / 下一轮

- [ ] **Day 20b**：写 `.github/workflows/release.yml` · matrix 跑 4 平台（aarch64/x86_64 darwin · x86_64 linux gnu · x86_64 windows msvc）· softprops/action-gh-release 自动上传
- [ ] **Day 20b 前置**：用户决策推 GitHub 还是 gitcode（meta-4 残留项）
- [ ] **pre-existing 阻塞 1**：修 `tree-sitter-typescript` 0.21 API 漂移（`language_typescript` 写法）使 typescript feature 复活
- [ ] **pre-existing 阻塞 2**：修 `lancedb`/`arrow-arith` 0.x 在新 chrono 上的 `quarter` 方法歧义（升级 arrow-arith 或 pin chrono 老版本）
- [ ] **Day 20c**（可选）：发布 `arkui-rag-full-v<VER>-<TARGET>.tar.gz` 含 onnx + 写明 ONNX Runtime 装库步骤
- [ ] **Day 21**：corpus model-pull 真实下载 + corpus 分发
- [ ] **Day 22**：mdBook 文档站 + 1.0 release
- [x] Day 20a：`scripts/release-local.sh` 一键打包 · 自动探测 host triple · sha256 计算
- [x] Day 20a：Makefile `release-local` + `release-local-verify` target
- [x] Day 20a：`docs/RELEASE.md` 完整发布指南
- [x] Day 20a：README 顶部加用户向 Download 章节 + 当前状态更新到 Day 20a
- [x] Day 20a：本地 aarch64-apple-darwin 端到端验证（编译 → 打包 → 解压 → query 真跑）
- [x] ROADMAP 维护约定第 9 次实战
