# 7 — release-local-script

> 日期：2026-05-28
> 触发：用户指令 「继续 Day 20 先不对接 DevEco 等公局，优先端到端本地跑 CLI」
> 类型：工具脚本（meta · scripts/* 新增）

---

## 用户提出的要求

> 「继续Day20 先不对接DevEco等公局，优先端到端本地跑CLI」

裁剪范围：
- 优先做：本地 host 平台 release binary + 端到端 CLI 验证
- 不做：DevEco / VSCode / IntelliJ 插件接入
- 推迟：跨平台 CI matrix（Day 20b）+ GitHub Releases 自动上传

## Agent 给出的修改建议

新增 `scripts/release-local.sh` 一个独立工具脚本（不动 cargo / cargo.toml / 业务代码）：

| 设计点 | 选择 | 理由 |
|---|---|---|
| 语言 | bash shell | 与 demo-smoke.sh / mcp-demo.sh 风格一致；零额外依赖 |
| Target triple 探测 | `rustc -vV | awk '/^host:/ ...'` | 内置工具 · 跨平台兼容 |
| 打包格式 | macOS/linux 用 `.tar.gz` · Windows 用 `.zip` | 业界标准 |
| SHA256 | 累积写入 `dist/SHA256SUMS` | GitHub Releases 校验和的标准位置 |
| 默认 features | `http,mcp,lsp,tantivy` | 避开 pre-existing 阻塞（lancedb / typescript）· 主协议层完整 |
| INSTALL.txt | 每个 tarball 内置 6 步快速上手 | 用户拿到包不用回 README |
| Makefile 入口 | `release-local` + `release-local-verify` | 后者一条龙做 打包 + 解压 + --version 验证 |

替代方案权衡（被否）：
- 用 `cargo-dist`（业界标准 · Astral uv 同款）：要给 workspace 加 metadata + 远程拉工具 · 本轮范围内不必
- 直接写 `.github/workflows/release.yml`：用户明确说「先不对接」 · 4 平台 CI yml 调试涉及 3-4 commit · 太大

## 多轮互动

无 —— 用户给出明确范围裁剪后 agent 直接执行。

## 实际改动

- **接口变化**：无（不动 Rust 代码 / Cargo.toml / cargo features）
- **规则变化**：无（不动 AGENTS.md / CLAUDE.md / hooks）
- **文件变化**：
  - 新增 `scripts/release-local.sh`（~190 行 · meta）
  - 改 `Makefile`（+ `release-local` / `release-local-verify` target + help 行）
  - 改 `README.md`（顶部加 Download + 当前状态更新到 Day 20a）
  - 新增 `docs/RELEASE.md`（用户向发布指南 · business）
- **配置变化**：无（`dist/` 已在 .gitignore line 10）

## 执行生效后总结

### 实际产出

| 产物 | 大小 | 说明 |
|---|---|---|
| `dist/arkui-rag-v0.0.1-aarch64-apple-darwin.tar.gz` | 2.9 MB | 主分发产物 |
| `dist/SHA256SUMS` | — | 累积 sha256 |
| INSTALL.txt（包内） | 1.7 KB | 6 步快速上手 |
| arkui-rag binary（包内） | 6.7 MB | strip + thin-LTO + opt-level=3 |

### 前后对比

| 维度 | Day 16 完成时 | Day 20a 完成时 |
|---|---|---|
| 分发能力 | 仅源码 + 用户 `cargo build` | tarball + sha256 + INSTALL.txt |
| 一键打包入口 | 无 | `make release-local` |
| 一键自验证 | 无 | `make release-local-verify` |
| 用户向 README | 无 Download 章节 | 顶部含完整使用流程 |
| `docs/RELEASE.md` | 无 | 1 份完整发布指南 |

### 实测验证

```
$ make release-local-verify
[1/4] cargo build --release --features http,mcp,lsp,tantivy
   Finished `release` profile [optimized] target(s) in 37.67s
[2/4] 烟雾测试 --version → arkui-rag 0.0.1
[3/4] 暂存产物到 dist/.staging/arkui-rag-v0.0.1-aarch64-apple-darwin/
[4/4] 打包为 .tar.gz
✅ Release artifact 完成
  产物 : dist/arkui-rag-v0.0.1-aarch64-apple-darwin.tar.gz (2.9M)
  sha  : cf94f931ff6424d6772ef7f6f7db4d13a8d4f949931d6584f8e5a6e9202a7ea0
━━━ 解压验证 ━━━
arkui-rag 0.0.1
✅ release tarball 端到端可用
```

端到端 5 路径全过：
- ✅ 编译：`cargo build --release --features http,mcp,lsp,tantivy` · 37s · 6.7 MB
- ✅ 打包：tarball 2.9 MB + sha256
- ✅ 解压 + 跑：`tar -xzf ... && ./arkui-rag --version` · `arkui-rag 0.0.1`
- ✅ 业务流：解压版 binary 跑 query 真返回 Top-K 命中
- ✅ 三协议：HTTP /health + /search · MCP initialize + tools/list · LSP Content-Length initialize

### 残留 / 下一轮处理

- [ ] Day 20b：写 `.github/workflows/release.yml` · matrix 跑 4 平台 · softprops/action-gh-release 自动上传
- [ ] Day 20b 前置：用户决策推 GitHub 还是 gitcode（meta-4 残留项 · 仍未决）
- [ ] pre-existing 阻塞：tree-sitter-typescript 0.21 API 漂移 / arrow-arith trait method 歧义（已挂 follow-up）
- [ ] 添加 `make release-local-clean` target 清 `dist/` 与 `SHA256SUMS`（可选）
- [x] scripts/release-local.sh 一键打包脚本
- [x] Makefile `release-local` + `release-local-verify` target
- [x] README 顶部 Download 章节
- [x] docs/RELEASE.md 完整发布指南
- [x] 双轨归档（本 meta 7 + 配套 feature/rag4arkui-core/20）
