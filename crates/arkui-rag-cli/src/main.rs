//! arkui-rag 二进制入口。
//!
//! Day 3：`--embedder onnx --model-path <dir>` 启用真实 BGE-M3 推理（需 `--features onnx`）。
//! 默认 `--embedder mock` 保持 Day 2 行为（无依赖 demo）。
//! 索引产物校验 `embedder_model_id` 防止"用不同 embedder 查老索引"。

use arkui_rag_chunker::MarkdownChunker;
use arkui_rag_core::{Citation, Embedder, EnhancedQuery, Retriever};
use arkui_rag_embedding::MockEmbedder;
use arkui_rag_indexer::Indexer;
use arkui_rag_retrieval::HybridRetriever;
use arkui_rag_storage::{InMemoryBM25Index, InMemoryVectorStore};
use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::Arc;

const DEFAULT_MOCK_DIM: usize = 384;
const DEFAULT_INDEX_PATH: &str = "corpus/_index/index.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum EmbedderKind {
    /// MockEmbedder：哈希派生确定性向量，零依赖，Day 2 默认
    Mock,
    /// OnnxEmbedder：真实 BGE-M3 / Qwen3 推理，需 --features onnx + 本地模型
    Onnx,
}

#[derive(Parser, Debug)]
#[command(
    name = "arkui-rag",
    version,
    about = "本地 RAG 引擎 for ArkUI-X / OpenHarmony",
    long_about = "完整方案见 docs/RAG4ArkUI-完整技术方案.md；当前阶段见 docs/STATUS-day*.md"
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
        #[arg(long, default_value = "corpus")]
        source: PathBuf,
        #[arg(long, default_value = DEFAULT_INDEX_PATH)]
        index_path: PathBuf,
        /// embedder 类型
        #[arg(long, value_enum, default_value_t = EmbedderKind::Mock)]
        embedder: EmbedderKind,
        /// onnx embedder 需要：模型目录（含 model.onnx 和 tokenizer.json）
        #[arg(long)]
        model_path: Option<PathBuf>,
        /// onnx embedder 需要：模型标识（如 bge-m3 / qwen3-embedding-0.6b）
        #[arg(long, default_value = "bge-m3")]
        model_id: String,
        /// mock 模式下的向量维度（onnx 模式忽略，从模型读）
        #[arg(long, default_value_t = DEFAULT_MOCK_DIM)]
        dim: usize,
    },
    /// 检索一次并打印 Top-K 命中
    Query {
        #[arg(short, long)]
        text: String,
        #[arg(short, long, default_value_t = 5)]
        k: usize,
        #[arg(long, default_value = DEFAULT_INDEX_PATH)]
        index_path: PathBuf,
        /// embedder 类型；必须与建索引时一致（否则 model_id 校验报错）
        #[arg(long, value_enum, default_value_t = EmbedderKind::Mock)]
        embedder: EmbedderKind,
        /// onnx embedder 需要的模型目录
        #[arg(long)]
        model_path: Option<PathBuf>,
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
    /// 拉取 / 更新本地模型 —— Week 2-3 backlog（真实下载）
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
            println!("arkui-rag serve (stub) — http={} mcp={} lsp={}", http, mcp, lsp);
            println!("⏳ Week 4 backlog：协议路由实际监听（见 docs/RAG4ArkUI-完整技术方案.md §4.4）");
            Ok(())
        }
        Cmd::Index {
            source,
            index_path,
            embedder,
            model_path,
            model_id,
            dim,
        } => cmd_index(&source, &index_path, embedder, model_path.as_deref(), &model_id, dim).await,
        Cmd::Query {
            text,
            k,
            index_path,
            embedder,
            model_path,
        } => cmd_query(&text, k, &index_path, embedder, model_path.as_deref()).await,
        Cmd::Corpus { op } => corpus_op(op).await,
    }
}

/// 构造 embedder 实例 + 报告 (model_id, dim)。
async fn build_embedder(
    kind: EmbedderKind,
    model_path: Option<&std::path::Path>,
    model_id: &str,
    mock_dim: usize,
) -> anyhow::Result<(Arc<dyn Embedder>, String, usize)> {
    match kind {
        EmbedderKind::Mock => {
            let m = MockEmbedder::new(mock_dim);
            let id = m.model_id().to_string();
            let dim = m.dim();
            Ok((Arc::new(m), id, dim))
        }
        EmbedderKind::Onnx => build_onnx(model_path, model_id),
    }
}

#[cfg(feature = "onnx")]
fn build_onnx(
    model_path: Option<&std::path::Path>,
    model_id: &str,
) -> anyhow::Result<(Arc<dyn Embedder>, String, usize)> {
    use arkui_rag_embedding::OnnxEmbedder;
    let path = model_path.ok_or_else(|| {
        anyhow::anyhow!("--embedder onnx 必须配 --model-path <模型目录>")
    })?;
    let m = OnnxEmbedder::load(path, model_id)
        .map_err(|e| anyhow::anyhow!("加载 ONNX 模型失败: {}", e))?;
    let id = m.model_id().to_string();
    let dim = m.dim();
    Ok((Arc::new(m), id, dim))
}

#[cfg(not(feature = "onnx"))]
fn build_onnx(
    _model_path: Option<&std::path::Path>,
    _model_id: &str,
) -> anyhow::Result<(Arc<dyn Embedder>, String, usize)> {
    anyhow::bail!(
        "本二进制未启用 onnx feature。重新构建：\n\
         \tcargo build -p arkui-rag-cli --features onnx --release"
    )
}

async fn cmd_index(
    source: &std::path::Path,
    index_path: &std::path::Path,
    kind: EmbedderKind,
    model_path: Option<&std::path::Path>,
    model_id: &str,
    mock_dim: usize,
) -> anyhow::Result<()> {
    if !source.exists() {
        anyhow::bail!("源目录不存在：{}", source.display());
    }
    let (embedder, model_id_used, dim) =
        build_embedder(kind, model_path, model_id, mock_dim).await?;
    let vector = Arc::new(InMemoryVectorStore::new(model_id_used.clone(), dim));
    let bm25 = Arc::new(InMemoryBM25Index);
    let chunker = Arc::new(MarkdownChunker::new());

    let indexer = Indexer::new(chunker, embedder, vector.clone(), bm25);
    let stats = indexer.index_directory(source).await?;
    vector.save_to(index_path).await?;

    println!("✅ 索引完成");
    println!("   embedder    : {}", stats.embedder_model_id);
    println!("   dim         : {}", dim);
    println!("   files       : {}", stats.files);
    println!("   chunks      : {}", stats.chunks);
    println!("   skipped     : {}", stats.skipped);
    println!("   elapsed_ms  : {}", stats.elapsed_ms);
    println!("   saved to    : {}", index_path.display());
    Ok(())
}

async fn cmd_query(
    text: &str,
    k: usize,
    index_path: &std::path::Path,
    kind: EmbedderKind,
    model_path: Option<&std::path::Path>,
) -> anyhow::Result<()> {
    if !index_path.exists() {
        anyhow::bail!(
            "索引文件不存在：{}（先跑 arkui-rag index 建索引）",
            index_path.display()
        );
    }
    let vector = Arc::new(InMemoryVectorStore::load_from(index_path).await?);
    let dim = vector.dim();
    let index_model_id = vector.embedder_model_id().to_string();

    let (embedder, query_model_id, query_dim) =
        build_embedder(kind, model_path, &index_model_id, dim).await?;

    // 防错配：索引时的 model_id 必须与查询时一致
    if query_model_id != index_model_id {
        anyhow::bail!(
            "embedder model_id 不匹配：索引由 '{}' 建，查询用 '{}'。\n\
             重建索引：arkui-rag index --embedder {} {} ...",
            index_model_id,
            query_model_id,
            match kind {
                EmbedderKind::Mock => "mock",
                EmbedderKind::Onnx => "onnx",
            },
            if matches!(kind, EmbedderKind::Onnx) {
                "--model-path <PATH>"
            } else {
                ""
            }
        );
    }
    if query_dim != dim {
        anyhow::bail!("embedder dim ({}) 与索引 dim ({}) 不匹配", query_dim, dim);
    }
    let bm25 = Arc::new(InMemoryBM25Index);

    let retriever = HybridRetriever::new(embedder, vector, bm25);
    let q = EnhancedQuery::passthrough(text);
    let hits = retriever.retrieve(&q, k).await?;

    if hits.is_empty() {
        println!("⚠️  无命中。索引文件：{}", index_path.display());
        return Ok(());
    }

    println!("✅ Top-{} hits (using {})", hits.len(), query_model_id);
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
        let preview: String = h.chunk.content.chars().take(200).collect();
        let preview = preview.replace('\n', " ");
        println!(
            "  preview: {}{}",
            preview,
            if h.chunk.content.len() > 200 { "…" } else { "" }
        );
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
            println!("⏳ Week 2-3 backlog：从 HuggingFace / ModelScope 拉模型到 ~/.arkui-rag/models/");
            println!();
            println!("当前手动获取方式：");
            println!("  # BGE-M3");
            println!("  git lfs install");
            println!("  git clone https://huggingface.co/BAAI/bge-m3 ~/.arkui-rag/models/bge-m3");
            println!("  # 或国内镜像：");
            println!("  git clone https://www.modelscope.cn/Xorbits/bge-m3.git ~/.arkui-rag/models/bge-m3");
            println!();
            println!("  之后导出 ONNX（一次性）：");
            println!("  pip install optimum[onnxruntime]");
            println!("  optimum-cli export onnx --model ~/.arkui-rag/models/bge-m3 \\");
            println!("      --task feature-extraction --opset 17 ~/.arkui-rag/models/bge-m3-onnx");
            Ok(())
        }
    }
}
