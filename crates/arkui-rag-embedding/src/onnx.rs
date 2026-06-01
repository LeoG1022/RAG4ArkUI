//! OnnxEmbedder —— BGE-M3 ONNX 推理。
//!
//! **代码来源**：直接迁移自技术方案 §7.2 `embedding.rs`。
//! Day 1 状态：feature-gated（`--features onnx` 启用），API 与方案文档完全一致；
//! 尚未实现 `arkui_rag_core::Embedder` trait（同步 ↔ 异步桥接是 Week 2 backlog）。
//!
//! 模型文件约定：`~/.arkui-rag/models/bge-m3/{model.onnx, tokenizer.json}`，
//! 首次运行由 CLI 拉取（Week 2 实现）。

#![allow(dead_code)]

use anyhow::Result;
use ndarray::{Array2, Axis};
use ort::{
    execution_providers::{CPUExecutionProvider, CoreMLExecutionProvider, CUDAExecutionProvider},
    session::{builder::GraphOptimizationLevel, Session},
    value::Tensor,
};
use std::path::Path;
use std::sync::Arc;
use tokenizers::Tokenizer;

/// Round 40 Phase 2 helper：把 ort 任何 Error map 到 anyhow::Error。
///
/// rc.12 起 ort 用 typed `Error<T>`（如 `Error<SessionBuilder>`）· 含
/// `NonNull<*>` / `dyn Any` 等非 Send/Sync 字段 · `?` 自动 From 转 anyhow
/// （要 Send + Sync）失败 · 必须显式 map_err。
/// Generic E + Display bound 兼容所有 ort Error 类型。
fn ort_err<E: std::fmt::Display>(prefix: &'static str) -> impl FnOnce(E) -> anyhow::Error {
    move |e| anyhow::anyhow!("{}: {}", prefix, e)
}

/// 一个加载好的 Embedding 模型实例，常驻内存。
///
/// rc.12: `Session::run` 签名变为 `&mut self` · encode(&self) 不能直接调 ·
/// 用 `Mutex<Session>` 内部可变（Send + Sync · 适合 spawn_blocking）。
pub struct EmbeddingModel {
    session: std::sync::Mutex<Session>,
    tokenizer: Tokenizer,
    max_length: usize,
    embed_dim: usize,
}

impl EmbeddingModel {
    /// 加载 BGE-M3 ONNX 模型与对应 tokenizer。
    pub fn load(model_dir: &Path) -> Result<Self> {
        // 1. 初始化 ONNX Runtime
        // Round 49.5 Phase 2: 自动检测 external data 文件 · 跳 CoreML EP
        // 触发条件：模型目录含 *.onnx_data 文件（BGE-M3 等大模型用 external data 存权重）
        // 原因：ort rc.12 + CoreML + external data 互不兼容 · CoreML EP 把
        //       model.onnx 当目录处理 · 找 model.onnx/model.onnx_data 报 "Not a directory"
        // env 兜底：ARKUI_RAG_DISABLE_COREML=1 也能强制禁用（debug / 用户自定义）
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
            .with_name("arkui-rag")
            .with_execution_providers(providers)
            .commit();  // rc.12: 返回 bool（true=首次初始化生效 · false=已初始化）· 不是 Result

        // 2. 加载 ONNX 模型
        let model_path = model_dir.join("model.onnx");
        let session = Session::builder()
            .map_err(ort_err("Session::builder"))?
            .with_optimization_level(GraphOptimizationLevel::Level3)
            .map_err(ort_err("with_optimization_level"))?
            .with_intra_threads(4)
            .map_err(ort_err("with_intra_threads"))?
            .commit_from_file(&model_path)
            .map_err(|e| {
                anyhow::anyhow!("加载 ONNX 模型失败 {}: {}", model_path.display(), e)
            })?;

        // 3. 加载 tokenizer
        let tokenizer = Tokenizer::from_file(model_dir.join("tokenizer.json"))
            .map_err(|e| anyhow::anyhow!("加载 tokenizer 失败: {}", e))?;

        Ok(Self {
            session: std::sync::Mutex::new(session),
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

        // rc.12: Tensor::from_array 需 (shape, data) tuple · 不再直接吃 ndarray
        let input_ids_shape = [input_ids.nrows() as i64, input_ids.ncols() as i64];
        let input_ids_data: Vec<i64> = input_ids.iter().copied().collect();
        let attention_mask_shape = [attention_mask.nrows() as i64, attention_mask.ncols() as i64];
        let attention_mask_data: Vec<i64> = attention_mask.iter().copied().collect();

        let input_ids_tensor = Tensor::from_array((input_ids_shape, input_ids_data))
            .map_err(ort_err("Tensor::from_array(input_ids)"))?;
        let attention_mask_tensor =
            Tensor::from_array((attention_mask_shape, attention_mask_data))
                .map_err(ort_err("Tensor::from_array(attention_mask)"))?;

        // rc.12: ort::inputs! 宏返回 Vec 不是 Result · 去掉宏后 ?
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
            .map_err(ort_err("session.run(embed)"))?;

        // rc.12: try_extract_tensor 返回 (Shape, &[f32]) 元组
        let (shape, flat) = outputs[0]
            .try_extract_tensor::<f32>()
            .map_err(ort_err("try_extract_tensor(hidden)"))?;
        // rc.12: Shape: Deref<Target = [i64]> · 直接当 slice 用
        let dims: &[i64] = &shape;
        if dims.len() != 3 {
            return Err(anyhow::anyhow!(
                "embed 输出 shape 异常 · 期望 [batch, seq_len, dim] · 实际 {:?}",
                dims
            ));
        }
        let pooled = self.mean_pooling_from_flat(
            flat,
            dims[0] as usize,
            dims[1] as usize,
            dims[2] as usize,
            &attention_mask,
        );
        Ok(Self::l2_normalize(pooled))
    }

    /// rc.12 后的新 mean pooling · 直接吃 flat &[f32] + shape · 不需 ArrayViewD
    fn mean_pooling_from_flat(
        &self,
        flat: &[f32],
        batch_size: usize,
        seq_len: usize,
        dim: usize,
        mask: &Array2<i64>,
    ) -> Array2<f32> {
        let mut result = Array2::<f32>::zeros((batch_size, dim));
        for b in 0..batch_size {
            let mut sum_mask = 0.0_f32;
            for s in 0..seq_len {
                let m = mask[[b, s]] as f32;
                if m > 0.0 {
                    sum_mask += m;
                    let off = (b * seq_len + s) * dim;
                    for d in 0..dim {
                        result[[b, d]] += flat[off + d] * m;
                    }
                }
            }
            let denom = sum_mask.max(1e-9);
            for d in 0..dim {
                result[[b, d]] /= denom;
            }
        }
        result
    }

    /// 保留旧 ArrayViewD 版本（兼容历史调用 · 当前未使用）
    #[allow(dead_code)]
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
