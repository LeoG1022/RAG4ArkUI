# 30 — user-verification

> 日期：2026-05-29
> 涉及代码：`docs/USER-VERIFICATION.md`（新建 · 9 步可执行验证清单）+ mdbook 引用
> 类型：新建（用户向操作文档）

## 本轮目标

Phase A/B/C 都跑通后 · 写一份 step-by-step 让用户自己 dogfood 跑一遍确认。

## Plan

> 必备节（FAIL 级 M-FEATURE-PLAN）

9 节内容：
- 0. 环境准备（rust / protoc / mdbook）
- 1. make check（默认编译）
- 2. cargo test --workspace
- 3. make smoke
- 4. make mcp-demo
- 5. make release-local-verify
- 6. CLI 7 子步骤（corpus pull / model-pull / index / query / hyde / expand-parent / eval）
- 7. 三协议 server（HTTP / MCP / LSP）
- 8. lancedb feature（可选）
- 9. mdBook 文档站（可选）

每步给：**命令 + 期望输出 + 失败时怎么办**。

## 改动要点

- 单一文件 `docs/USER-VERIFICATION.md` · 不拆碎
- mdbook 引用 + SUMMARY 「上手」节加链接
- 末尾「故障常见原因」表 + 「跑完之后」收尾

## 对话摘要

> 必备节（FAIL 级 M-FEATURE-PLAN）

按时序：
1. Phase A+B+C 跑完 + commit af03030
2. 用户要求「最后梳理端到端本地验证清单」
3. Agent 写 docs/USER-VERIFICATION.md + 链入 mdbook

## 验证结果

- ✅ mdbook build clean · verify page 在「上手」节
- ⏳ 用户实际跑一遍（dogfood）

## 残留 / 下一轮

- [ ] 用户实际跑一遍 9 节验证
- [ ] 失败的步骤反馈 agent 修
- [x] 写完整 9 节清单
- [x] mdbook 链入
