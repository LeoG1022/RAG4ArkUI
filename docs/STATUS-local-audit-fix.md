# STATUS · 本地全链路审计 + 5 处 bug 修复

> 日期：2026-05-29
> 对应 commit：[本 commit · Phase A+B+C local audit]
> 对应 feature log：[feedback/features/rag4arkui-core/29-2026-05-29-local-audit-fix.md](../feedback/features/rag4arkui-core/29-2026-05-29-local-audit-fix.md)
> 对应 meta：[feedback/meta/14-2026-05-29-tracing-stderr-walk-fix.md](../feedback/meta/14-2026-05-29-tracing-stderr-walk-fix.md)
> 上一阶段：[STATUS-github-url-fix.md](STATUS-github-url-fix.md)

> 🎯 本地全链路审计：smoke + mcp-demo + cargo test + CLI 参数组合全绿 ⭐

## 当前状态

5 处 pre-existing bug 一次清完：
1. mcp-demo.sh 3 处 cargo run 加 manifest-path
2. main.rs tracing 强制 stderr（MCP/LSP stdio 协议要求）
3. 5 个 README tokio_test doctest 标 ignore
4. 2 个 README 裸 ``` 块加 text 语言
5. walk_corpus depth>0 判断（修 tempdir 跳过 bug）
6. eval test 行号约定 @9→@10

## 验证全过

| 命令 | 结果 |
|---|---|
| `make smoke` | ✅ PASS |
| `make mcp-demo` | ✅ PASS（4/4 step + 3 assertion） |
| `cargo test --workspace --no-fail-fast` | ✅ 56 passed / 0 failed / 11 ignored |
| `arkui-rag query --hyde mock` | ✅ HyDE 改写真启动 |
| `arkui-rag query --expand-parent` | ✅ 父级 chunk 正确显示 |
| `arkui-rag eval --report-path ...` | ✅ 8 query 报告生成 |

## 完成度

| 项 | 状态 |
|---|---|
| Phase A smoke + mcp-demo | ✅ |
| Phase B cargo test --workspace 全绿 | ✅ |
| Phase C CLI 参数组合验证 | ✅ |
| 用户端到端验证清单 | ⏳ 下一步 |
