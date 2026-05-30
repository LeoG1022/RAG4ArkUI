# 34 — cli-default-features

> 日期：2026-05-30
> 涉及代码：`crates/arkui-rag-cli/Cargo.toml`（default features）· `docs/USER-VERIFICATION.md`
> 类型：默认值调整（DX 优化 · 修文档实操坑）

## 本轮目标

修两个坑：
1. **裸 `cargo build --release -p arkui-rag-cli` 编出来的 binary 默认 `features = []` · 三协议 / Tantivy 全没有 · 用户装到 PATH 后跑 `arkui-rag serve --mcp` 或 `--bm25 tantivy` 直接报「未启用 feature」**
2. USER-VERIFICATION.md 没告诉用户「不走 `make release-local` 就必须显式 `--features ...`」

后果：用户按上一轮接 Claude Code 流程操作时被绊倒 · 报错信息看似明确但用户不知道为什么 release-local-verify 跑过了的 binary 仍出错（因为他们其实是用别的命令重 build 过 · 装了个 default=[] 的 binary 到 PATH）。

## Plan

修正方案两步：

### 步骤 1 · Cargo.toml `default` 与 release-local 对齐

`crates/arkui-rag-cli/Cargo.toml`：

```toml
# before
default = []

# after
default = ["http", "mcp", "lsp", "tantivy", "typescript", "corpus-pull"]
```

选这六个的依据 = 跟 `scripts/release-local.sh:32 DEFAULT_FEATURES` 完全一致 · 保证：

- 裸 `cargo build --release -p arkui-rag-cli` 产物 ≡ `make release-local` 产物
- 用户怎么编都一样的可用 binary

排除项（仍 opt-in）：
- `onnx` —— ort 2.0 RC 整链路 broken（task #87）· 强制 opt-in 防止默认编译失败
- `lancedb` —— arrow/lance 编译 10+ 分钟 + 需要 protoc · 用户决定何时承担
- `kotlin` / `swift` —— 小众 chunker · 暂无 corpus 用例

### 步骤 2 · USER-VERIFICATION.md 第 1 节加警告框

在「1. 默认 features 编译」之后插入 `> 💡` 警告框：
- 列三种 build 方式（A 推荐 release-local / B 裸 cargo build / C 显式覆盖）
- 列默认不含的 feature + 原因（onnx broken · lancedb 大 · kotlin/swift 小众）
- 给「症状识别 → 修法」：报「未启用 tantivy feature」= 装的是老/裸 build binary

### 不动的

- `scripts/release-local.sh DEFAULT_FEATURES` 不动——已经对了，本轮是让 Cargo.toml 跟它对齐而不是反过来
- `crates/Cargo.toml` workspace 配置不动——这是 cli crate 自己的事
- `scripts/classify-change.sh` 不动——它把 `crates/**/Cargo.toml` 判 business 实际应该 meta · 但这是另一个坑 · 留残留项 · 本轮聚焦默认 features

### 替代选项权衡

| 选项 | 优点 | 缺点 | 选了吗 |
|---|---|---|---|
| A · `default = []` + 文档警告 | 控制粒度最细 · 用户必须显式声明 | 用户必踩坑（如本轮所见） | ❌ 已被本轮否决 |
| B · `default = [六个全]` + release-local 同步 | 跟 release-local 一致 · 默认即可用 | 默认 build 多拉若干 deps（~30 秒）| ✅ **采纳** |
| C · `default = [全部 + onnx + lancedb]` | 完全无脑 build | onnx broken 时默认编译失败 · lancedb 拉 10 分钟 | ❌ 风险过大 |
| D · `default = [仅 mcp]`（最小可对接 Claude）| 编译最快 | HTTP/LSP/Tantivy 仍坑 | ❌ 不够 |

选 B · 理由：用户**期待**「下载 / `cargo build` 就能用」 · default 应反映「能用」最低门槛 · 真重的 feature（onnx/lancedb）仍 opt-in 防意外。

## 对话摘要

| 轮次 | 用户 | Agent |
|---|---|---|
| 1 | 跑 `arkui-rag index --bm25 tantivy` 报「本二进制未启用 tantivy feature」 | 诊断：`/usr/local/bin/arkui-rag` 是 2.2MB（default=[]）· 不是 10.7MB（带 tantivy）· 帮重 build · 验证 MCP 真活 |
| 2 | 同意 agent 提出的两条改进（cli default features + USER-VERIFICATION 警告框） | 本轮实施 |

无方向调整 · 用户直接 "要做"。

## 改动要点

- `crates/arkui-rag-cli/Cargo.toml:34-45`：default 从 `[]` 改成 `["http","mcp","lsp","tantivy","typescript","corpus-pull"]` · 加 7 行注释解释为什么选这六、为什么排除 onnx/lancedb/kotlin/swift
- `docs/USER-VERIFICATION.md:36-67`：「失败时」之后插入 32 行 `> 💡` 警告块 · 含三种 build 方式 + 默认排除清单 + 症状识别
- 验证：`cd crates && cargo build --release -p arkui-rag-cli`（无 `--features`）· 产物 10.7MB · `arkui-rag index --help` 含 tantivy enum value
- 与 Round 33 关系：Round 33 = AGENTS.md #18 行为规则；Round 34 = 实战中发现的文档/默认值坑 · 独立 slice

## 验证结果

- 编译：`cd crates && cargo build --release -p arkui-rag-cli` ✓ `Finished in 0.44s`（缓存命中 · 实际无新 deps · 因为之前手动加 --features 时已编过同样组合）
- check-api-parity：N/A（不动业务代码）
- MCP smoke：`printf <jsonrpc> | arkui-rag serve --mcp ...` ✓ `tools/list` 返回 4 个工具（arkui_search_docs / arkui_search_code / arkui_migrate_snippet / arkui_validate_api）
- Tantivy 索引：`arkui-rag index --source .../references --bm25 tantivy` ✓ 11 文件 / 107 chunks
- 用户操作验证：用户需 `sudo cp crates/target/release/arkui-rag /usr/local/bin/` 重装 · `arkui-rag index --help | grep tantivy` 看到 enum value

## 残留 / 下一轮

- [x] cli/Cargo.toml default 调整
- [x] USER-VERIFICATION.md Step 1 加警告
- [x] feature log Round 34
- [x] STATUS-cli-default-features.md
- [ ] **scripts/classify-change.sh 分类边界** · `crates/**/Cargo.toml` 应判 meta 而非 business（manifest 改动影响构建语义 · 是基础设施） · 本轮没改 · 留作独立 round（避免本轮散注意力）
- [ ] **用户重装 binary 验证步骤生效** · `sudo cp` + `arkui-rag index --bm25 tantivy` 在 Claude Code 配 mcp.json 之前完成
- [ ] CI 跑 `cargo check --workspace` 时间是否显著增加（理论 +30 秒）· 跑过一次后看
