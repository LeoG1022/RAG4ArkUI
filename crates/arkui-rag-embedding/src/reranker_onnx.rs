//! OnnxReranker —— BGE-Reranker-v2-m3 cross-encoder 真实推理。
//!
//! 与 OnnxEmbedder 对比：
//! - Embedder：单输入文本 → 单向量（双塔模型）
//! - Reranker：(query, doc) 对 → 单分数（cross-encoder，精度高但慢）
//!
//! 输入拼接：`[CLS] query [SEP] doc [SEP]`，由 tokenizer 自动处理 special tokens。
//! 输出：单 logit（num_labels=1），越高表示越相关。
//!
//! 桥接策略：与 OnnxEmbedder 同样用 `tokio::task::spawn_blocking`。

#![allow(dead_code)]

use anyhow::Result as AnyResult;
use ndarray::Array2;
use ort::{
    execution_providers::{CPUExecutionProvider, CUDAExecutionProvider, CoreMLExecutionProvider},
    session::{builder::GraphOptimizationLevel, Session},
    value::Tensor,
};
use std::path::Path;
use std::sync::Arc;
use tokenizers::Tokenizer;

/// Round 40 Phase 2 helper · 同 onnx.rs 同款 · ort Error → anyhow Error
/// Generic E + Display bound 兼容 ort 所有 typed `Error<T>` 变体
fn ort_err<E: std::fmt::Display>(prefix: &'static str) -> impl FnOnce(E) -> anyhow::Error {
    move |e| anyhow::anyhow!("{}: {}", prefix, e)
}

/// 底层 Reranker 同步 API（设计与 EmbeddingModel 平行）。
///
/// rc.12: Mutex 包 Session · 因为 Session::run 改为 &mut self · score(&self) 不能直接调
pub struct RerankerModel {
    session: std::sync::Mutex<Session>,
    tokenizer: Tokenizer,
    max_length: usize,
}

impl RerankerModel {
    /// 加载 BGE-Reranker-v2-m3 ONNX 模型。
    ///
    /// 模型目录约定：`<dir>/model.onnx` + `<dir>/tokenizer.json`。
    pub fn load(model_dir: &Path) -> AnyResult<Self> {
        // Round 49.5 Phase 2: 同 onnx.rs · 自动检测 external data · 跳 CoreML EP
        let env_disable = std::env::var("ARKUI_RAG_DISABLE_COREML").is_ok();
        let has_external_data = std::fs::read_dir(model_dir)
            .map(|rd| {
                rd.filter_map(|e| e.ok()).any(|e| {
                    e.file_name()
                        .to_str()
                        .map(|n| n.ends_with(".onnx_data") || n.ends_with("_data"))
                        .unwrap_or(false)
                })
            })
            .unwrap_or(false);
        let disable_coreml = env_disable || has_external_data;
        let mut providers = Vec::new();
        if !disable_coreml {
            providers.push(CoreMLExecutionProvider::default().build());
        }
        providers.push(CUDAExecutionProvider::default().build());
        providers.push(
            CPUExecutionProvider::default()
                .with_arena_allocator(true)
                .build(),
        );

        ort::init()
            .with_name("arkui-rag-rerank")
            .with_execution_providers(providers)
            .commit(); // rc.12: bool 返回 · 不是 Result

        let model_path = model_dir.join("model.onnx");
        let session = Session::builder()
            .map_err(ort_err("Session::builder"))?
            .with_optimization_level(GraphOptimizationLevel::Level3)
            .map_err(ort_err("with_optimization_level"))?
            .with_intra_threads(4)
            .map_err(ort_err("with_intra_threads"))?
            .commit_from_file(&model_path)
            .map_err(|e| {
                anyhow::anyhow!(
                    "加载 Reranker ONNX 模型失败 {}: {}",
                    model_path.display(),
                    e
                )
            })?;

        let tokenizer = Tokenizer::from_file(model_dir.join("tokenizer.json"))
            .map_err(|e| anyhow::anyhow!("加载 reranker tokenizer 失败: {}", e))?;

        Ok(Self {
            session: std::sync::Mutex::new(session),
            tokenizer,
            max_length: 512,
        })
    }

    /// 对 `(query, doc)` 对批量打分。返回每对的 logit 分数（越高越相关）。
    pub fn score(&self, pairs: &[(String, String)]) -> AnyResult<Vec<f32>> {
        if pairs.is_empty() {
            return Ok(Vec::new());
        }
        // tokenizer.encode_batch 支持 pair 输入：传 (query, doc) tuple
        let inputs: Vec<(String, String)> = pairs.to_vec();
        let encodings = self
            .tokenizer
            .encode_batch(inputs, true)
            .map_err(|e| anyhow::anyhow!("reranker tokenize 失败: {}", e))?;

        let batch_size = encodings.len();
        let mut input_ids = Array2::<i64>::zeros((batch_size, self.max_length));
        let mut attention_mask = Array2::<i64>::zeros((batch_size, self.max_length));

        for (i, enc) in encodings.iter().enumerate() {
            let ids = enc.get_ids();
            let mask = enc.get_attention_mask();
            let len = ids.len().min(self.max_length);
            for j in 0..len {
                input_ids[[i, j]] = ids[j] as i64;
                attention_mask[[i, j]] = mask[j] as i64;
            }
        }

        // rc.12: Tensor::from_array 需 (shape, data) tuple
        let input_ids_shape = [input_ids.nrows() as i64, input_ids.ncols() as i64];
        let input_ids_data: Vec<i64> = input_ids.iter().copied().collect();
        let attention_mask_shape = [attention_mask.nrows() as i64, attention_mask.ncols() as i64];
        let attention_mask_data: Vec<i64> = attention_mask.iter().copied().collect();

        let input_ids_tensor = Tensor::from_array((input_ids_shape, input_ids_data))
            .map_err(ort_err("Tensor::from_array(input_ids)"))?;
        let attention_mask_tensor = Tensor::from_array((attention_mask_shape, attention_mask_data))
            .map_err(ort_err("Tensor::from_array(attention_mask)"))?;

        // rc.12: Session::run 签名 &mut self · 通过 Mutex 拿可变借用
        let mut session = self
            .session
            .lock()
            .map_err(|e| anyhow::anyhow!("session mutex poisoned: {}", e))?;
        let outputs = session
            .run(ort::inputs![
                "input_ids" => input_ids_tensor,
                "attention_mask" => attention_mask_tensor,
            ])
            .map_err(ort_err("session.run(rerank)"))?;

        // rc.12: try_extract_tensor 返回 (Shape, &[f32]) 元组
        let (shape_obj, flat_slice) = outputs[0]
            .try_extract_tensor::<f32>()
            .map_err(ort_err("try_extract_tensor(rerank logits)"))?;
        // rc.12: Shape: Deref<Target = [i64]> · 直接当 slice 用
        let shape: Vec<usize> = (&*shape_obj).iter().map(|&d| d as usize).collect();
        let flat: Vec<f32> = flat_slice.to_vec();

        // 兼容 [batch, 1] 和 [batch] 两种 shape
        let scores: Vec<f32> = match shape.len() {
            1 => flat,
            2 if shape[1] == 1 => flat,
            2 => {
                // [batch, num_labels>1]：取索引 1（label "relevant"），常见 binary classification
                let stride = shape[1];
                (0..batch_size)
                    .map(|i| flat[i * stride + (stride.min(2) - 1)])
                    .collect()
            }
            _ => return Err(anyhow::anyhow!("Reranker 输出 shape 异常: {:?}", shape)),
        };

        if scores.len() != batch_size {
            return Err(anyhow::anyhow!(
                "Reranker scores 数 {} 与 batch_size {} 不匹配",
                scores.len(),
                batch_size
            ));
        }
        Ok(scores)
    }
}

pub type SharedReranker = Arc<RerankerModel>;
