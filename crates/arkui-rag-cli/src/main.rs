//! arkui-rag 二进制入口。
//!
//! Day 3：`--embedder onnx --model-path <dir>` 启用真实 BGE-M3 推理（需 `--features onnx`）。
//! Day 4：`--bm25 tantivy` 启用真实 BM25 倒排检索（需 `--features tantivy`），让 HybridRetriever
//!         RRF 真正双路融合（向量 + BM25）。默认 `--bm25 memory` 走 Day 2 空 stub 行为。
//! 索引产物校验 `embedder_model_id` 防止"用不同 embedder 查老索引"；BM25 索引目录与
//! vector 索引 (`<index-path-dir>/bm25/`) 并列存放。

use arkui_rag_chunker::MarkdownChunker;
use arkui_rag_core::{Citation, Embedder, EnhancedQuery, Reranker, Retriever};
use arkui_rag_embedding::MockEmbedder;
use arkui_rag_indexer::Indexer;
use arkui_rag_retrieval::{CrossEncoderReranker, HybridRetriever};
use arkui_rag_storage::{BM25Index, InMemoryBM25Index, InMemoryVectorStore};
use clap::{Parser, Subcommand, ValueEnum};
use std::path::{Path, PathBuf};
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum Bm25Kind {
    /// 空 stub，HybridRetriever 退化为纯向量（Day 2 默认）
    Memory,
    /// Tantivy 真实 BM25 倒排检索（Day 4，需 --features tantivy）
    Tantivy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum RerankerKind {
    /// 不重排，直接用 HybridRetriever 输出（Day 2/3/4 默认）
    None,
    /// MockReranker：identity + truncate（占位，无实际重排）
    Mock,
    /// OnnxReranker：BGE-Reranker-v2 真实推理（Day 5，需 --features onnx + 本地模型）
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
        /// BM25 后端：memory（空 stub）/ tantivy（真实倒排检索，需 --features tantivy）
        #[arg(long, value_enum, default_value_t = Bm25Kind::Memory)]
        bm25: Bm25Kind,
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
        /// BM25 后端：必须与建索引时一致
        #[arg(long, value_enum, default_value_t = Bm25Kind::Memory)]
        bm25: Bm25Kind,
        /// Reranker 后端（Day 5）：none/mock/onnx
        #[arg(long, value_enum, default_value_t = RerankerKind::None)]
        rerank: RerankerKind,
        /// Reranker 启用时检索器先取多大 Top-K 再精排（默认 50）
        #[arg(long, default_value_t = 50)]
        pre_rerank_k: usize,
        /// Reranker 模型目录（onnx 必填）
        #[arg(long)]
        reranker_model_path: Option<PathBuf>,
        /// Reranker 模型标识（onnx 默认 bge-reranker-v2-m3）
        #[arg(long, default_value = "bge-reranker-v2-m3")]
        reranker_model_id: String,
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
            bm25,
        } => {
            cmd_index(
                &source,
                &index_path,
                embedder,
                model_path.as_deref(),
                &model_id,
                dim,
                bm25,
            )
            .await
        }
        Cmd::Query {
            text,
            k,
            index_path,
            embedder,
            model_path,
            bm25,
            rerank,
            pre_rerank_k,
            reranker_model_path,
            reranker_model_id,
        } => {
            cmd_query(
                &text,
                k,
                &index_path,
                embedder,
                model_path.as_deref(),
                bm25,
                rerank,
                pre_rerank_k,
                reranker_model_path.as_deref(),
                &reranker_model_id,
            )
            .await
        }
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

/// 推导 BM25 索引目录：与 index.json 同级的 bm25/ 子目录。
fn bm25_dir_from(index_path: &Path) -> PathBuf {
    index_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("bm25")
}

/// 构造 BM25 实例 + 报告 kind 名（写入索引元数据 / 防错配）。
fn build_bm25(kind: Bm25Kind, index_path: &Path) -> anyhow::Result<(Arc<dyn BM25Index>, &'static str)> {
    match kind {
        Bm25Kind::Memory => Ok((Arc::new(InMemoryBM25Index), "memory")),
        Bm25Kind::Tantivy => build_tantivy(index_path),
    }
}

#[cfg(feature = "tantivy")]
fn build_tantivy(index_path: &Path) -> anyhow::Result<(Arc<dyn BM25Index>, &'static str)> {
    use arkui_rag_storage::TantivyBM25Index;
    let dir = bm25_dir_from(index_path);
    let bm = TantivyBM25Index::open(&dir)
        .map_err(|e| anyhow::anyhow!("打开 Tantivy 索引目录 {} 失败: {}", dir.display(), e))?;
    Ok((Arc::new(bm), "tantivy"))
}

#[cfg(not(feature = "tantivy"))]
fn build_tantivy(_index_path: &Path) -> anyhow::Result<(Arc<dyn BM25Index>, &'static str)> {
    anyhow::bail!(
        "本二进制未启用 tantivy feature。重新构建：\n\
         \tcargo build -p arkui-rag-cli --features tantivy --release\n\
         （或 --features full 启用 onnx + tantivy）"
    )
}

/// 构造 reranker 实例 + 模型 ID 名。None 时返回 None。
fn build_reranker(
    kind: RerankerKind,
    model_path: Option<&Path>,
    model_id: &str,
) -> anyhow::Result<Option<(Arc<dyn Reranker>, String)>> {
    match kind {
        RerankerKind::None => Ok(None),
        RerankerKind::Mock => {
            let m = CrossEncoderReranker::default();
            let id = m.model_id().to_string();
            Ok(Some((Arc::new(m), id)))
        }
        RerankerKind::Onnx => build_onnx_reranker(model_path, model_id).map(Some),
    }
}

#[cfg(feature = "onnx")]
fn build_onnx_reranker(
    model_path: Option<&Path>,
    model_id: &str,
) -> anyhow::Result<(Arc<dyn Reranker>, String)> {
    use arkui_rag_embedding::OnnxReranker;
    let path = model_path.ok_or_else(|| {
        anyhow::anyhow!("--rerank onnx 必须配 --reranker-model-path <模型目录>")
    })?;
    let m = OnnxReranker::load(path, model_id)
        .map_err(|e| anyhow::anyhow!("加载 Reranker ONNX 失败: {}", e))?;
    let id = m.model_id().to_string();
    Ok((Arc::new(m), id))
}

#[cfg(not(feature = "onnx"))]
fn build_onnx_reranker(
    _model_path: Option<&Path>,
    _model_id: &str,
) -> anyhow::Result<(Arc<dyn Reranker>, String)> {
    anyhow::bail!(
        "本二进制未启用 onnx feature。重新构建：\n\
         \tcargo build -p arkui-rag-cli --features onnx --release\n\
         （或 --features full 启用 onnx + tantivy）"
    )
}

async fn cmd_index(
    source: &Path,
    index_path: &Path,
    kind: EmbedderKind,
    model_path: Option<&Path>,
    model_id: &str,
    mock_dim: usize,
    bm25_kind: Bm25Kind,
) -> anyhow::Result<()> {
    if !source.exists() {
        anyhow::bail!("源目录不存在：{}", source.display());
    }
    let (embedder, model_id_used, dim) =
        build_embedder(kind, model_path, model_id, mock_dim).await?;
    let vector = Arc::new(InMemoryVectorStore::new(model_id_used.clone(), dim));
    let (bm25, bm25_name) = build_bm25(bm25_kind, index_path)?;
    let chunker = Arc::new(MarkdownChunker::new());

    let indexer = Indexer::new(chunker, embedder, vector.clone(), bm25);
    let stats = indexer.index_directory(source).await?;
    vector.save_to(index_path).await?;

    println!("✅ 索引完成");
    println!("   embedder    : {}", stats.embedder_model_id);
    println!("   dim         : {}", dim);
    println!("   bm25        : {}", bm25_name);
    println!("   files       : {}", stats.files);
    println!("   chunks      : {}", stats.chunks);
    println!("   skipped     : {}", stats.skipped);
    println!("   elapsed_ms  : {}", stats.elapsed_ms);
    println!("   saved to    : {}", index_path.display());
    if matches!(bm25_kind, Bm25Kind::Tantivy) {
        println!("   bm25 index  : {}", bm25_dir_from(index_path).display());
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn cmd_query(
    text: &str,
    k: usize,
    index_path: &Path,
    kind: EmbedderKind,
    model_path: Option<&Path>,
    bm25_kind: Bm25Kind,
    rerank_kind: RerankerKind,
    pre_rerank_k: usize,
    reranker_model_path: Option<&Path>,
    reranker_model_id: &str,
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
    let (bm25, bm25_name) = build_bm25(bm25_kind, index_path)?;
    let reranker_opt = build_reranker(rerank_kind, reranker_model_path, reranker_model_id)?;
    let rerank_name: String = reranker_opt
        .as_ref()
        .map(|(_, id)| id.clone())
        .unwrap_or_else(|| "none".to_string());

    let retriever = HybridRetriever::new(embedder, vector, bm25);
    let q = EnhancedQuery::passthrough(text);

    // 启用 reranker 时先取 pre_rerank_k 个候选送精排
    let retrieve_k = if reranker_opt.is_some() {
        pre_rerank_k.max(k)
    } else {
        k
    };
    let hits = retriever.retrieve(&q, retrieve_k).await?;
    let hits = if let Some((rr, _id)) = reranker_opt {
        rr.rerank(text, hits, k).await?
    } else {
        hits.into_iter().take(k).collect()
    };

    if hits.is_empty() {
        println!(
            "⚠️  无命中。索引文件：{}（bm25={} · rerank={}）",
            index_path.display(),
            bm25_name,
            rerank_name
        );
        return Ok(());
    }

    println!(
        "✅ Top-{} hits (embedder={} · bm25={} · rerank={})",
        hits.len(),
        query_model_id,
        bm25_name,
        rerank_name
    );
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
