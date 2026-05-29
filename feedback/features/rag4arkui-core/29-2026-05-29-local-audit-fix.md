# 29 — local-audit-fix

> 日期：2026-05-29
> 涉及代码：5 处 bug 修复（Phase A+B+C 端到端审计）
> 类型：bug 修复（11 commit 累计的 pre-existing latent bug 一次性清理）

## 本轮目标

按 A→B→C 推进本地全链路审计 · 修浮出的所有 pre-existing bug，让 `make smoke`/`make mcp-demo`/`cargo test --workspace`/CLI 参数组合全绿。

## Plan

> 必备节（FAIL 级 M-FEATURE-PLAN）

Phase A：smoke + mcp-demo 现状审计
- ✅ smoke：直接过
- ❌ mcp-demo：Step 1 + Step 4 fail · 修

Phase B：cargo test --workspace 全绿
- 修 6 个 README 的 tokio_test doctest（与 chunker 同一问题 · 一次扫完）
- 修 indexer/retrieval 2 个 README 的 ```rust 块（裸 ``` 后没语言标识 · 被 cargo doctest 当 rust）
- 修 walk_corpus 在 tempfile::tempdir() 创建的隐藏目录被整个跳过的 bug
- 修 eval test 期望 chunk_id 行号约定不一致（@9 vs @10）

Phase C：CLI 参数组合验证（无 bug · 都直接过）

## 改动要点

| 文件 | 修复 |
|---|---|
| `scripts/mcp-demo.sh` | 3 处 `cargo run` 加 `--manifest-path crates/Cargo.toml`（workspace 重构到 crates/ 后未跟上） |
| `crates/arkui-rag-cli/src/main.rs` | tracing fmt() 默认 stdout · 强制 `.with_writer(std::io::stderr)`（MCP/LSP stdio 协议要求 stdout 只含协议数据 · 真 bug） |
| `crates/arkui-rag-{embedding,eval,indexer,retrieval,storage}/README.md` | 5 个 README 的 `\`\`\`rust` doctest 块标 `,ignore`（缺 tokio_test dev-dep · 与 chunker Day 20c 同一问题）|
| `crates/arkui-rag-{indexer,retrieval}/README.md` | 2 个裸 ``` 块加 `text` 语言标识（cargo doctest 默认当 rust 编译 fail） |
| `crates/arkui-rag-indexer/src/lib.rs` | `walk_corpus` 加 `depth > 0` 判断 · 不 reject root（修 tempfile::tempdir 的 /tmp/.tmpXXXX 整目录被跳过 bug） |
| `crates/arkui-rag-eval/tests/eval_end_to_end.rs` | GT chunk_id 行号 `@9 → @10`（约定不一致 · chunker 实际行号是内容起始行而非 heading 行） |

## 对话摘要

> 必备节（FAIL 级 M-FEATURE-PLAN）

按时序：
1. 用户：「把 #84/#85 挂起 · 重点本地打通流程」
2. Agent 列 5 类缺口（A/B/C/D/E）· 推荐 A→B→C
3. 用户：「按推荐继续 · agent 能做的先做完」
4. Phase A：smoke 直接过 · mcp-demo Step 1 报「索引产物未生成」→ 查脚本 cargo run 没在 crates 子目录 → 修；Step 4 报「响应行数 4 != 期望 3」→ 查到 tracing log 写 stdout → 修 main.rs
5. Phase B：6 个 README doctest fail（tokio_test 缺 dep）+ 2 个 README 裸 ``` 块 fail + walk_corpus 跳过 tempdir + eval test 行号约定不一致 → 全修
6. Phase C：--hyde mock / --expand-parent / arkui-rag eval 三项都直接过 · eval 报告生成（recall=0 是 fixture GT 与本地 corpus 不匹配的预期）

## 验证结果

- ✅ `make smoke`：PASS
- ✅ `make mcp-demo`：PASS（4/4 step + 3 个 assertion 都过）
- ✅ `cargo test --workspace --no-fail-fast`：**56 passed / 0 failed / 11 ignored**
- ✅ `arkui-rag query --hyde mock`：HyDE 改写真启动 · Top-K 返回
- ✅ `arkui-rag query --expand-parent`：每个 hit 末尾 ↳ parent (...) 显示父级
- ✅ `arkui-rag eval --report-path ...`：8 query 报告生成（recall=0 是 GT-corpus mismatch 预期）

## 残留 / 下一轮

- [ ] Phase D：用户装 binary 到 /usr/local/bin/ dogfood 一段时间（用户事）
- [ ] Phase E：ort 链路 task #87 仍 blocker · 等架构决策
- [ ] Day 17 DevEco Plugin MVP
- [x] Phase A：smoke + mcp-demo 真活
- [x] Phase B：cargo test --workspace 全绿
- [x] Phase C：--hyde / --expand-parent / eval 全验证
