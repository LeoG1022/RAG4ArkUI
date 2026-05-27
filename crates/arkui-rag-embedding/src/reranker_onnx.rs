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

use anyhow::{Context, Result as AnyResult};
use ndarray::{Array2, Axis};
use ort::{
    execution_providers::{CPUExecutionProvider, CoreMLExecutionProvider, CUDAExecutionProvider},
    session::{builder::GraphOptimizationLevel, Session},
    value::Tensor,
};
use std::path::Path;
use std::sync::Arc;
use tokenizers::Tokenizer;

/// 底层 Reranker 同步 API（设计与 EmbeddingModel 平行）。
pub struct RerankerModel {
    session: Session,
    tokenizer: Tokenizer,
    max_length: usize,
}

impl RerankerModel {
    /// 加载 BGE-Reranker-v2-m3 ONNX 模型。
    ///
    /// 模型目录约定：`<dir>/model.onnx` + `<dir>/tokenizer.json`。
    pub fn load(model_dir: &Path) -> AnyResult<Self> {
        ort::init()
            .with_name("arkui-rag-rerank")
            .with_execution_providers([
                CoreMLExecutionProvider::default().build(),
                CUDAExecutionProvider::default().build(),
                CPUExecutionProvider::default().with_arena_allocator(true).build(),
            ])
            .commit()?;

        let model_path = model_dir.join("model.onnx");
        let session = Session::builder()?
            .with_optimization_level(GraphOptimizationLevel::Level3)?
            .with_intra_threads(4)?
            .commit_from_file(&model_path)
            .with_context(|| {
                format!("加载 Reranker ONNX 模型失败: {}", model_path.display())
            })?;

        let tokenizer = Tokenizer::from_file(model_dir.join("tokenizer.json"))
            .map_err(|e| anyhow::anyhow!("加载 reranker tokenizer 失败: {}", e))?;

        Ok(Self {
            session,
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

        let outputs = self.session.run(ort::inputs![
            "input_ids" => Tensor::from_array(input_ids)?,
            "attention_mask" => Tensor::from_array(attention_mask)?,
        ]?)?;

        // 输出 shape 通常是 [batch, 1] 或 [batch]，取出 logits
        let logits = outputs[0].try_extract_tensor::<f32>()?;
        let shape = logits.shape().to_vec();
        let flat: Vec<f32> = logits.iter().copied().collect();

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
            _ => {
                return Err(anyhow::anyhow!(
                    "Reranker 输出 shape 异常: {:?}",
                    shape
                ))
            }
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
