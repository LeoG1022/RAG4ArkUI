//! OnnxEmbedder —— BGE-M3 ONNX 推理。
//!
//! **代码来源**：直接迁移自技术方案 §7.2 `embedding.rs`。
//! Day 1 状态：feature-gated（`--features onnx` 启用），API 与方案文档完全一致；
//! 尚未实现 `arkui_rag_core::Embedder` trait（同步 ↔ 异步桥接是 Week 2 backlog）。
//!
//! 模型文件约定：`~/.arkui-rag/models/bge-m3/{model.onnx, tokenizer.json}`，
//! 首次运行由 CLI 拉取（Week 2 实现）。

#![allow(dead_code)]

use anyhow::{Context, Result};
use ndarray::{Array2, Axis};
use ort::{
    execution_providers::{CPUExecutionProvider, CoreMLExecutionProvider, CUDAExecutionProvider},
    session::{builder::GraphOptimizationLevel, Session},
    value::Tensor,
};
use std::path::Path;
use std::sync::Arc;
use tokenizers::Tokenizer;

/// 一个加载好的 Embedding 模型实例，常驻内存。
pub struct EmbeddingModel {
    session: Session,
    tokenizer: Tokenizer,
    max_length: usize,
    embed_dim: usize,
}

impl EmbeddingModel {
    /// 加载 BGE-M3 ONNX 模型与对应 tokenizer。
    pub fn load(model_dir: &Path) -> Result<Self> {
        // 1. 初始化 ONNX Runtime
        ort::init()
            .with_name("arkui-rag")
            .with_execution_providers([
                CoreMLExecutionProvider::default().build(),
                CUDAExecutionProvider::default().build(),
                CPUExecutionProvider::default().with_arena_allocator(true).build(),
            ])
            .commit()?;

        // 2. 加载 ONNX 模型
        let model_path = model_dir.join("model.onnx");
        let session = Session::builder()?
            .with_optimization_level(GraphOptimizationLevel::Level3)?
            .with_intra_threads(4)?
            .commit_from_file(&model_path)
            .with_context(|| format!("加载 ONNX 模型失败: {}", model_path.display()))?;

        // 3. 加载 tokenizer
        let tokenizer = Tokenizer::from_file(model_dir.join("tokenizer.json"))
            .map_err(|e| anyhow::anyhow!("加载 tokenizer 失败: {}", e))?;

        Ok(Self {
            session,
            tokenizer,
            max_length: 512,
            embed_dim: 1024,
        })
    }

    /// 对一批文本做 Embedding。
    pub fn encode(&self, texts: &[&str]) -> Result<Array2<f32>> {
        let batch_size = texts.len();
        let encodings = self
            .tokenizer
            .encode_batch(texts.to_vec(), true)
            .map_err(|e| anyhow::anyhow!("tokenize 失败: {}", e))?;

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
            "attention_mask" => Tensor::from_array(attention_mask.clone())?,
        ]?)?;

        let hidden_states = outputs[0].try_extract_tensor::<f32>()?;
        let pooled = self.mean_pooling(&hidden_states.view(), &attention_mask)?;
        Ok(Self::l2_normalize(pooled))
    }

    fn mean_pooling(
        &self,
        hidden: &ndarray::ArrayViewD<f32>,
        mask: &Array2<i64>,
    ) -> Result<Array2<f32>> {
        let (batch_size, seq_len, dim) =
            (hidden.shape()[0], hidden.shape()[1], hidden.shape()[2]);
        let mut result = Array2::<f32>::zeros((batch_size, dim));
        for b in 0..batch_size {
            let mut sum_mask = 0.0_f32;
            for s in 0..seq_len {
                let m = mask[[b, s]] as f32;
                if m > 0.0 {
                    sum_mask += m;
                    for d in 0..dim {
                        result[[b, d]] += hidden[[b, s, d]] * m;
                    }
                }
            }
            let denom = sum_mask.max(1e-9);
            for d in 0..dim {
                result[[b, d]] /= denom;
            }
        }
        Ok(result)
    }

    fn l2_normalize(mut arr: Array2<f32>) -> Array2<f32> {
        for mut row in arr.axis_iter_mut(Axis(0)) {
            let norm = row.iter().map(|x| x * x).sum::<f32>().sqrt().max(1e-9);
            row.mapv_inplace(|x| x / norm);
        }
        arr
    }

    pub fn dim(&self) -> usize {
        self.embed_dim
    }
}

pub type SharedEmbedding = Arc<EmbeddingModel>;
