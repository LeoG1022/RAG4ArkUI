# arkui-rag-cli

**定位**：`arkui-rag` 二进制入口。所有 IDE 插件 / Agent 都通过执行此二进制接入（启动子进程 + IPC，§5.3 理由 2、3）。

## Subcommand 一览（Day 1 全部接口完整、逻辑大多 stub）

```
arkui-rag --version
arkui-rag serve [--http] [--mcp] [--lsp]
arkui-rag index --source <path>
arkui-rag query --text "..." [--k 5]
arkui-rag corpus list
arkui-rag corpus model-pull --name bge-m3
```

每个 subcommand 当前打印对应 Week 的 TODO 后退出非 0。`--version` 与 `corpus list` 是 Day 1 已具备真实输出的。

## 用法

```bash
cd crates
cargo run -p arkui-rag-cli -- --help
cargo run -p arkui-rag-cli -- --version
cargo run -p arkui-rag-cli -- corpus list
```

## 日志

默认 `INFO`，通过 `RUST_LOG=debug arkui-rag ...` 切换。日志走 `tracing-subscriber`。
