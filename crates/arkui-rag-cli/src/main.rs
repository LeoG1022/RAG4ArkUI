//! arkui-rag 二进制入口。
//!
//! Day 2：`index` 与 `query` 端到端真实可用（用 MockEmbedder + InMemoryVectorStore）。
//! 索引持久化到 `corpus/_index/index.json`，让 index 和 query 跨进程共享。

use arkui_rag_chunker::MarkdownChunker;
use arkui_rag_core::{Citation, EnhancedQuery, Retriever};
use arkui_rag_embedding::MockEmbedder;
use arkui_rag_indexer::Indexer;
use arkui_rag_retrieval::HybridRetriever;
use arkui_rag_storage::{InMemoryBM25Index, InMemoryVectorStore};
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::Arc;

/// 默认 Mock embedding 维度（Day 2 占位；Week 2 起换 BGE-M3 = 1024）。
const DEFAULT_MOCK_DIM: usize = 384;
/// 索引产物默认路径。
const DEFAULT_INDEX_PATH: &str = "corpus/_index/index.json";

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
    /// 启动常驻服务（HTTP / MCP / LSP，至少选其一）—— Week 4 实装
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
        /// 待索引的目录（默认 corpus/）
        #[arg(long, default_value = "corpus")]
        source: PathBuf,
        /// 索引产物保存路径
        #[arg(long, default_value = DEFAULT_INDEX_PATH)]
        index_path: PathBuf,
        /// Mock embedding 维度
        #[arg(long, default_value_t = DEFAULT_MOCK_DIM)]
        dim: usize,
    },
    /// 检索一次并打印 Top-K 命中
    Query {
        #[arg(short, long)]
        text: String,
        #[arg(short, long, default_value_t = 5)]
        k: usize,
        /// 索引产物路径
        #[arg(long, default_value = DEFAULT_INDEX_PATH)]
        index_path: PathBuf,
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
    /// 拉取 / 更新本地模型（BGE-M3 等）—— Week 2 实装
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
            eprintln!("error: {:#}", e);
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
            println!(
                "arkui-rag serve (stub) — http={} mcp={} lsp={}",
                http, mcp, lsp
            );
            println!("⏳ Week 4 backlog：协议路由实际监听（见 docs/RAG4ArkUI-完整技术方案.md §4.4 / §9 图 8）");
            Ok(())
        }
        Cmd::Index {
            source,
            index_path,
            dim,
        } => cmd_index(&source, &index_path, dim).await,
        Cmd::Query {
            text,
            k,
            index_path,
        } => cmd_query(&text, k, &index_path).await,
        Cmd::Corpus { op } => corpus_op(op).await,
    }
}

async fn cmd_index(
    source: &std::path::Path,
    index_path: &std::path::Path,
    dim: usize,
) -> anyhow::Result<()> {
    if !source.exists() {
        anyhow::bail!("源目录不存在：{}", source.display());
    }
    let model_id = format!("mock-{}", dim);
    let embedder = Arc::new(MockEmbedder::new(dim));
    let vector = Arc::new(InMemoryVectorStore::new(model_id.clone(), dim));
    let bm25 = Arc::new(InMemoryBM25Index);
    let chunker = Arc::new(MarkdownChunker::new());

    let indexer = Indexer::new(chunker, embedder, vector.clone(), bm25);
    let stats = indexer.index_directory(source).await?;
    vector.save_to(index_path).await?;

    println!("✅ 索引完成");
    println!("   embedder    : {}", stats.embedder_model_id);
    println!("   files       : {}", stats.files);
    println!("   chunks      : {}", stats.chunks);
    println!("   skipped     : {}", stats.skipped);
    println!("   elapsed_ms  : {}", stats.elapsed_ms);
    println!("   saved to    : {}", index_path.display());
    Ok(())
}

async fn cmd_query(text: &str, k: usize, index_path: &std::path::Path) -> anyhow::Result<()> {
    if !index_path.exists() {
        anyhow::bail!(
            "索引文件不存在：{}（先跑 arkui-rag index 建索引）",
            index_path.display()
        );
    }
    let vector = Arc::new(InMemoryVectorStore::load_from(index_path).await?);
    let dim = vector.dim();
    let model_id = vector.embedder_model_id().to_string();
    let embedder = Arc::new(MockEmbedder::new(dim));
    if embedder.dim() != dim {
        anyhow::bail!(
            "embedder dim ({}) 与索引 dim ({}) 不匹配；索引由 {} 创建",
            embedder.dim(),
            dim,
            model_id
        );
    }
    let bm25 = Arc::new(InMemoryBM25Index);

    let retriever = HybridRetriever::new(embedder, vector, bm25);
    let q = EnhancedQuery::passthrough(text);
    let hits = retriever.retrieve(&q, k).await?;

    if hits.is_empty() {
        println!("⚠️  无命中。索引文件：{}", index_path.display());
        return Ok(());
    }

    println!("✅ Top-{} hits (using {})", hits.len(), model_id);
    println!();
    for (i, h) in hits.iter().enumerate() {
        let citation = Citation::from(h);
        let head = if citation.heading_path.is_empty() {
            "(root)".to_string()
        } else {
            citation.heading_path.join(" > ")
        };
        let lines = citation
            .line_range
            .map(|(a, b)| format!("L{}-{}", a, b))
            .unwrap_or_else(|| "L?".to_string());
        println!("─── [{}] score={:.4} ──────────────────", i + 1, h.score);
        println!("  source : {} {}", citation.source, lines);
        println!("  heading: {}", head);
        // 内容预览：前 200 字符
        let preview: String = h.chunk.content.chars().take(200).collect();
        let preview = preview.replace('\n', " ");
        println!("  preview: {}{}", preview, if h.chunk.content.len() > 200 { "…" } else { "" });
        println!();
    }
    Ok(())
}

async fn corpus_op(op: CorpusOp) -> anyhow::Result<()> {
    match op {
        CorpusOp::List => {
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
            println!("arkui-rag corpus model-pull --name {} (stub)", name);
            println!("⏳ Week 2 backlog：从 HuggingFace / ModelScope 拉模型到 ~/.arkui-rag/models/");
            Ok(())
        }
    }
}
