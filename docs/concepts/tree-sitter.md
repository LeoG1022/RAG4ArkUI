# tree-sitter

> 一句话：**跨语言的增量式 AST parser 框架** · 给一段代码 + 对应语言的语法规则 · 产出一棵抽象语法树（AST）· 改一个字符不需重 parse 整个文件 · 只 re-parse 受影响子树。

## 业界用它做什么

最广为人知 · 编辑器内的代码理解：

- **Neovim / Helix / Emacs** 的现代语法高亮（替代正则 highlighting）
- **GitHub** 网页上代码导航 / "Go to definition"（已用 tree-sitter 重写整个 stack）
- **Zed** 编辑器 / **Tauri** code lens
- **Hugging Face** 的 code retrieval 等 ML 项目做代码切分

特点：
- 支持 100+ 语言（每个语言一个 `tree-sitter-<lang>` crate · 维护方各自）
- 容错（碰到语法错误不整个崩 · 给个 `ERROR` 节点继续 parse）
- 增量（适合编辑器场景）
- C 核心 · Rust / JS / Python / Go 等都有 binding

## 本项目里它干什么

`crates/arkui-rag-chunker`（Day 10 引入）用它 **按 AST 切分代码文件**：

```text
.ts / .ets 源码
    ↓ tree-sitter-typescript parse
   AST tree
    ↓ TypeScriptChunker 遍历
   一组 Chunk · 每个 chunk = 一个函数 / class / method 这种「语义单位」
```

对比朴素方法：

- ❌ **按字符数切**（如「每 500 字符切一刀」）—— 会把函数从中间切断 · 检索时找回半截
- ❌ **按行数切**（如「每 50 行切一刀」）—— 同上 · 不尊重语法边界
- ❌ **正则切**（如「碰到 `^function` 就切」）—— 模板字符串里有 `function` 字面量会误伤
- ✅ **AST 切**（tree-sitter）—— 「function_declaration」「method_definition」等节点就是天然语义单位 · 切出来的 chunk 在 grammar 层面是完整的

为啥 RAG 系统这件事重要：检索时返回的 chunk 越「语义完整」· LLM 拿去做生成的上下文越有用。**Anthropic 的 Contextual Retrieval 论文 / OpenAI codex 都强调代码 corpus 必须用 AST 切分**。

## 本项目具体怎么接

| 文件 | 角色 |
|---|---|
| `crates/Cargo.toml` workspace deps | `tree-sitter = "0.22"` + `tree-sitter-typescript = "0.21"` |
| `crates/arkui-rag-chunker/src/typescript.rs` | TypeScriptChunker · 接收 .ts/.ets 源码 → AST → chunks |
| `crates/arkui-rag-chunker/src/dispatcher.rs` | 按扩展名路由到 typescript / markdown / kotlin / swift chunker |
| feature gate | `cargo build --features typescript` 才编进去（避免默认 binary 拉大） |

跑 `arkui-rag index --source corpus/` 时 · 如果碰到 .ts/.ets 文件 · 自动走 tree-sitter 路径。

## 当前 known gap

**ArkTS 专有的 `@Component struct X { ... }` 语法** · vanilla `tree-sitter-typescript` 把 `struct` 当 identifier 不当关键字 —— parse 失败回退到 ERROR 节点 · chunker fallback 成「整文件作为一个 chunk」（粗粒度）。

修法（已挂 follow-up · 大工程）：
- 用 `tree-sitter-cli` 写一个 `tree-sitter-arkts` grammar fork（要懂 PEG/LR 语法）
- 或 AST post-processing 把 `struct` 节点重写为 class-like

详见 `crates/arkui-rag-chunker/src/typescript.rs` 里 `#[ignore]` 的那个测试。

## 类比

| 已知 | tree-sitter 相当于 |
|---|---|
| Python `ast` 模块 | 类似 · 但更通用（多语言 · 增量） |
| ANTLR | 同类工具 · ANTLR 更老 · tree-sitter 更适合 IDE 场景 |
| Babel parser（JS） | 同类 · Babel 是 JS-only · tree-sitter 多语言 |
| LSP server | LSP 是协议 · tree-sitter 是 LSP server 内部常用的 parser 实现 |
