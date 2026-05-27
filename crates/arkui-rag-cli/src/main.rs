//! arkui-rag 二进制入口。
//!
//! Day 1 主要目标：把 CLI 接口形状（subcommand / 参数 / 退出码约定）锁死。
//! 实际功能多数走 stub 路径，打印 TODO 后退出。

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser, Debug)]
#[command(
    name = "arkui-rag",
    version,
    about = "本地 RAG 引擎 for ArkUI-X / OpenHarmony",
    long_about = "完整方案见 docs/RAG4ArkUI-完整技术方案.md"
)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    /// 启动常驻服务（HTTP / MCP / LSP，至少选其一）
    Serve {
        #[arg(long)]
        http: bool,
        #[arg(long)]
        mcp: bool,
        #[arg(long)]
        lsp: bool,
    },
    /// 对指定目录建索引
    Index {
        #[arg(long)]
        source: PathBuf,
    },
    /// 检索一次并打印 Top-K 命中
    Query {
        #[arg(short, long)]
        text: String,
        #[arg(short, long, default_value_t = 5)]
        k: usize,
    },
    /// Corpus 管理
    Corpus {
        #[command(subcommand)]
        op: CorpusOp,
    },
}

#[derive(Subcommand, Debug)]
enum CorpusOp {
    /// 列出 corpus/ 下的子目录与文档数
    List,
    /// 拉取 / 更新本地模型（BGE-M3 等）
    ModelPull {
        #[arg(long)]
        name: String,
    },
}

#[tokio::main]
async fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();
    match run(cli).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {}", e);
            ExitCode::from(2)
        }
    }
}

async fn run(cli: Cli) -> anyhow::Result<()> {
    match cli.cmd {
        Cmd::Serve { http, mcp, lsp } => {
            if !http && !mcp && !lsp {
                anyhow::bail!("必须指定至少一个协议：--http / --mcp / --lsp");
            }
            println!("arkui-rag serve (Day 1 stub) — http={} mcp={} lsp={}", http, mcp, lsp);
            println!("⏳ Week 4 backlog：协议路由实际监听");
            Ok(())
        }
        Cmd::Index { source } => {
            println!("arkui-rag index --source {} (Day 1 stub)", source.display());
            println!("⏳ Week 2 backlog：跑 Chunker → Embedder → Storage 的索引流水线");
            Ok(())
        }
        Cmd::Query { text, k } => {
            println!("arkui-rag query --text \"{}\" --k {} (Day 1 stub)", text, k);
            println!("⏳ Week 3 backlog：HybridRetriever + Reranker + Context Assembler");
            Ok(())
        }
        Cmd::Corpus { op } => corpus_op(op).await,
    }
}

async fn corpus_op(op: CorpusOp) -> anyhow::Result<()> {
    match op {
        CorpusOp::List => {
            // 这是 Day 1 唯一真实实现的非 trivial subcommand
            let candidates = ["official", "samples", "migration", "errors", "custom"];
            println!("corpus/ 子目录：");
            for d in candidates {
                let path = std::path::Path::new("corpus").join(d);
                let count = if path.exists() {
                    std::fs::read_dir(&path)
                        .map(|rd| {
                            rd.filter_map(|e| e.ok())
                                .filter(|e| !e.file_name().to_string_lossy().starts_with('.'))
                                .count()
                        })
                        .unwrap_or(0)
                } else {
                    0
                };
                let mark = if path.exists() { "✅" } else { "❌" };
                println!("  {} {:<10} ({} 个文档)", mark, d, count);
            }
            Ok(())
        }
        CorpusOp::ModelPull { name } => {
            println!("arkui-rag corpus model-pull --name {} (Day 1 stub)", name);
            println!("⏳ Week 2 backlog：从 HuggingFace / ModelScope 拉模型到 ~/.arkui-rag/models/");
            Ok(())
        }
    }
}
