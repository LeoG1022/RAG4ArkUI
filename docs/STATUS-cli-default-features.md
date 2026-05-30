# STATUS — cli-default-features

> 配套 feature log：`feedback/features/rag4arkui-core/34-2026-05-30-cli-default-features.md`
> 日期：2026-05-30

---

## 当前状态

`arkui-rag-cli` 的 `default` features 从空 `[]` 调整为「能用」组合 `["http","mcp","lsp","tantivy","typescript","corpus-pull"]` · 与 `scripts/release-local.sh` DEFAULT_FEATURES 完全对齐。

本阶段交付：

- 裸 `cargo build --release -p arkui-rag-cli` 产物（10.7MB）≡ `make release-local` 产物
- `docs/USER-VERIFICATION.md` Step 1 加 32 行警告块 · 说明三种 build 方式 + 默认排除项 + 报错诊断
- 用户对接 Claude Code 的实操路径不再被「未启用 feature」报错绊倒

实战触发：用户跑「接 Claude Code」流程时报错 → agent 诊断发现 PATH binary 是 default=[] · 修根本原因。

## 输入契约

无 CLI 参数 / API 变化。

**Cargo features 契约变化**（影响 build 命令语义）：

| 命令 | Before（本轮前） | After（本轮后） |
|---|---|---|
| `cargo build --release -p arkui-rag-cli` | 空 binary（无三协议无 tantivy）· 2.2MB | 与 release-local 同款 binary · 10.7MB |
| `cargo build --release -p arkui-rag-cli --features http,mcp,lsp,tantivy,typescript,corpus-pull` | 同上 · 显式声明 | 同 default · 显式声明仍可用（幂等）|
| `cargo build --release -p arkui-rag-cli --no-default-features` | 不适用（default 本来就空）| 回到旧的空 binary 行为（如需）|
| `make release-local` / `release-local-verify` | 不变 · 显式指定 features | 不变 · 与 default 对齐而已 |

opt-in 不变的 feature：
- `--features onnx` —— ort 2.0 RC broken（task #87）
- `--features lancedb` —— arrow/lance 编译慢
- `--features kotlin` / `--features swift` —— 按需开

## 输出契约

无运行时输出格式变化。

**新增「能用」语义保证**：
- 裸 build 的 binary 必有 `arkui-rag serve --mcp` / `--http` / `--lsp` 真活
- 裸 build 的 binary 必有 `--bm25 tantivy` 真活
- 裸 build 的 binary 必有 `corpus pull --from-file` 真活
- 裸 build 的 binary 必有 `arkui-rag index --source <ts/tsx/ets>` 切分能力

新失败模式：
- 默认 build 会拉 axum / tantivy / ureq / flate2 / tar / tree-sitter-typescript 等 deps · 首次 build 时间 +30 秒 ~ +1 分钟
- 如有用户**不**想要这些 deps · 必须 `--no-default-features` 显式声明

## 验证手段

### 用户手动

```bash
# 1. clean build 验证（确认默认产物可用）
cd crates && cargo clean -p arkui-rag-cli
cargo build --release -p arkui-rag-cli 2>&1 | tail -3
ls -la target/release/arkui-rag
# 期望：10MB+ · 不是 2.2MB

# 2. 验证 tantivy 选项暴露
./target/release/arkui-rag index --help | grep tantivy
# 期望：看到 enum value `tantivy: Tantivy 真实 BM25 倒排检索`

# 3. 验证 MCP 真活
printf '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"m","version":"1"}}}\n{"jsonrpc":"2.0","id":2,"method":"tools/list"}\n' \
  | ./target/release/arkui-rag serve --mcp \
      --index-path ~/.arkui-rag/index.json --bm25 tantivy \
  2>/dev/null | head -c 500
# 期望：第 1 行 protocolVersion 2024-11-05 · 第 2 行 tools/list 含 arkui_search_docs
```

### 自动化

- `cargo build --release -p arkui-rag-cli`（CI 已含 · 现在默认 features 多）
- `cargo test --workspace`（无破坏 · 测试基线已含三协议 + tantivy）
- 推荐补充（未来）：CI 加 `cargo build --release -p arkui-rag-cli --no-default-features` 矩阵 step · 防 default 漂移破坏 no-features path

## 与上一阶段的关联性

| 阶段 | Round | 产出 | 角色 |
|---|---|---|---|
| Round 32 | concepts-kb | docs/concepts/ 知识库目录 + 5 步流程 | 静态资产基础设施 |
| Round 33 | concepts-archive-rule | AGENTS.md #18 触发机制 | 行为契约 |
| **Round 34 (本轮)** | cli-default-features | `default` 含「能用」六件套 + 文档警告 | DX 修复（实操坑）|

增量：Round 33 解决「概念问答如何沉淀」的元问题 · Round 34 解决「裸 build 出来不能用」的具体 DX 坑 · 两者独立。

兼容性：
- **向后兼容**：所有显式声明 `--features X` 的命令行为不变（幂等）
- **行为变化**：裸 `cargo build` 产物语义变了（从「啥都没有的 stub binary」变成「能接 Claude Code 的 release-local 同款」）
- **CI 影响**：`make check` / `cargo check --workspace` 现在会额外拉 axum/tantivy 等 deps · 首次 +30 秒（缓存后无差异）

破坏性变更：仅一处 · 想要旧 default=[] 行为的用户必须 `--no-default-features` · 在 USER-VERIFICATION.md 警告块已说明。

## 完成度 / 下一阶段

### 达成度

| 项 | 状态 |
|---|---|
| Cargo.toml default 调整 | ✅ |
| USER-VERIFICATION.md 警告块 | ✅ |
| 裸 build 验证（10.7MB · MCP 真活 · tantivy 真活）| ✅ |
| feature log Round 34 | ✅ |
| STATUS-cli-default-features（本文件）| ✅ |
| 用户重装 binary | ⏳（需用户 `sudo cp` 覆盖 /usr/local/bin/）|

### 下一阶段建议

立即（用户做）：
1. `sudo cp /Users/leo/work/RAG4ArkUI/crates/target/release/arkui-rag /usr/local/bin/`
2. `ls -la $(which arkui-rag)` 验证大小 10MB+
3. 继续接 Claude Code 流程（mcp.json + 重启）

短期（可选 · 1-2 round）：
- `scripts/classify-change.sh` 把 `crates/**/Cargo.toml` 归 meta（manifest 改默认 features 影响构建语义 · 是基础设施变更）· 本轮特意没动 · 留作独立 round 避免散注意力
- CI release.yml 在 matrix 加一个 `--no-default-features` step · 防漂移

长期：
- 第一次 release v0.0.2-rc.1 推上去后 · 观察用户反馈是否还有 "用法不知道 build 哪个 features" 类困惑
- 如果还有 · 考虑 `arkui-rag --help` 加段「我用哪些 features 编的」 introspection 输出（通过 build script + `env!()`）
