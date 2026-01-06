use crate::types::AgentRole;
use crate::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[cfg(all(feature = "semantic", feature = "ort"))]
use ort::session::Session;

#[cfg(feature = "semantic")]
use tokenizers::Tokenizer;

pub const DEFAULT_EMBEDDING_DIM: usize = 384;

#[derive(Debug, Clone)]
pub struct SemanticEngine {
    #[cfg(all(feature = "semantic", feature = "ort"))]
    session: Option<Arc<Mutex<Session>>>,
    #[cfg(feature = "semantic")]
    tokenizer: Option<Tokenizer>,
    config: ModelConfig,
    model_path: PathBuf,
    use_fallback: bool,
    fallback_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub hidden_size: usize,
    pub vocab_size: usize,
    pub num_hidden_layers: usize,
    pub num_attention_heads: usize,
    pub intermediate_size: usize,
    pub hidden_act: String,
    pub layer_norm_eps: f64,
    pub pad_token_id: u32,
    pub bos_token_id: u32,
    pub eos_token_id: u32,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            hidden_size: 384,
            vocab_size: 30522,
            num_hidden_layers: 6,
            num_attention_heads: 12,
            intermediate_size: 1536,
            hidden_act: "gelu".to_string(),
            layer_norm_eps: 1e-12,
            pad_token_id: 0,
            bos_token_id: 101,
            eos_token_id: 102,
        }
    }
}

impl SemanticEngine {
    pub fn new() -> Self {
        Self::with_path(PathBuf::from("models"))
    }

    pub fn with_path(model_path: PathBuf) -> Self {
        Self {
            #[cfg(all(feature = "semantic", feature = "ort"))]
            session: None,
            #[cfg(feature = "semantic")]
            tokenizer: None,
            config: ModelConfig::default(),
            model_path,
            use_fallback: false,
            fallback_enabled: true,
        }
    }

    pub fn initialize(&mut self) -> Result<()> {
        #[cfg(feature = "semantic")]
        {
            // First, ensure model files exist (download if needed)
            self.ensure_models_exist()?;

            // Try to load tokenizer
            let tokenizer_path = self.model_path.join("tokenizer.json");
            if tokenizer_path.exists() {
                match Tokenizer::from_file(&tokenizer_path) {
                    Ok(tokenizer) => {
                        self.tokenizer = Some(tokenizer);
                    }
                    Err(e) => {
                        eprintln!("Warning: Could not load tokenizer: {}", e);
                    }
                }
            }

            // Try to load ONNX model (only when ort feature is enabled)
            #[cfg(feature = "ort")]
            {
                let onnx_path = self.model_path.join("model.onnx");
                if onnx_path.exists() {
                    #[cfg(windows)]
                    {
                        // On Windows, use load-dynamic to load ONNX Runtime DLL at runtime
                        let ort_dll_path = self.model_path.join("onnxruntime.dll");
                        if ort_dll_path.exists() {
                            if ort::init_from(ort_dll_path.to_string_lossy().as_ref())
                                .commit()
                                .is_ok()
                            {
                                match Session::builder() {
                                    Ok(session_builder) => {
                                        match session_builder.commit_from_file(&onnx_path) {
                                            Ok(inference_session) => {
                                                self.session =
                                                    Some(Arc::new(Mutex::new(inference_session)));
                                                self.fallback_enabled = false;
                                                self.use_fallback = false;
                                                eprintln!("[SEMANTIC] Loaded ONNX embedding model (Windows runtime DLL)");
                                            }
                                            Err(e) => {
                                                eprintln!(
                                                    "Warning: Could not load ONNX model: {}",
                                                    e
                                                );
                                                self.enable_fallback();
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        eprintln!(
                                            "Warning: Could not create session builder: {}",
                                            e
                                        );
                                        self.enable_fallback();
                                    }
                                }
                            } else {
                                eprintln!("Warning: Could not initialize ONNX Runtime");
                                self.enable_fallback();
                            }
                        } else {
                            eprintln!("Warning: onnxruntime.dll not found at {:?}", ort_dll_path);
                            self.enable_fallback();
                        }
                    }

                    #[cfg(not(windows))]
                    {
                        match Session::builder() {
                            Ok(session_builder) => match session_builder
                                .commit_from_file(&onnx_path)
                            {
                                Ok(inference_session) => {
                                    self.session = Some(Arc::new(Mutex::new(inference_session)));
                                    self.fallback_enabled = false;
                                    self.use_fallback = false;
                                    eprintln!("[SEMANTIC] Loaded ONNX embedding model");
                                }
                                Err(e) => {
                                    eprintln!("Warning: Could not load ONNX model: {}", e);
                                    self.enable_fallback();
                                }
                            },
                            Err(e) => {
                                eprintln!("Warning: Could not create session builder: {}", e);
                                self.enable_fallback();
                            }
                        }
                    }
                } else {
                    eprintln!("Warning: model.onnx not found at {:?}", onnx_path);
                    self.enable_fallback();
                }
            }

            // When ort is not available, use tokenizer fallback
            #[cfg(all(feature = "semantic", not(feature = "ort")))]
            {
                if self.tokenizer.is_some() {
                    eprintln!(
                        "[SEMANTIC] Using tokenizer-based embeddings (ONNX Runtime not available)"
                    );
                } else {
                    self.enable_fallback();
                }
            }
        }

        #[cfg(not(feature = "semantic"))]
        {
            self.use_fallback = true;
        }

        Ok(())
    }

    #[cfg(feature = "semantic")]
    fn ensure_models_exist(&self) -> Result<()> {
        use std::fs;
        use std::io::Write;

        let model_files = [
            ("tokenizer.json", "https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2/resolve/main/tokenizer.json"),
            ("config.json", "https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2/resolve/main/config.json"),
            ("model.onnx", "https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2/resolve/main/onnx/model.onnx"),
        ];

        if !self.model_path.exists() {
            fs::create_dir_all(&self.model_path)?;
        }

        for (filename, url) in model_files {
            let filepath = self.model_path.join(filename);
            if !filepath.exists() {
                eprintln!("[SEMANTIC] Downloading {}...", filename);
                match ureq::get(url).call() {
                    Ok(response) => {
                        let mut file = fs::File::create(&filepath)?;
                        let mut reader = response.into_reader();
                        let mut buffer = [0u8; 8192];
                        loop {
                            match reader.read(&mut buffer)? {
                                0 => break,
                                n => file.write_all(&buffer[..n])?,
                            }
                        }
                        eprintln!("[SEMANTIC] Downloaded {}", filename);
                    }
                    Err(e) => {
                        eprintln!("[SEMANTIC] Warning: Could not download {}: {}", filename, e);
                    }
                }
            }
        }

        Ok(())
    }

    pub fn is_loaded(&self) -> bool {
        #[cfg(all(feature = "semantic", feature = "ort"))]
        {
            self.session.is_some() || self.tokenizer.is_some()
        }
        #[cfg(all(feature = "semantic", not(feature = "ort")))]
        {
            self.tokenizer.is_some()
        }
        #[cfg(not(feature = "semantic"))]
        {
            false
        }
    }

    pub fn enable_fallback(&mut self) {
        self.fallback_enabled = true;
        self.use_fallback = true;
        eprintln!("[SEMANTIC] Enabled fallback mode (TF-IDF style embeddings)");
    }

    pub fn disable_fallback(&mut self) {
        self.fallback_enabled = false;
        if self.is_loaded() {
            self.use_fallback = false;
        }
    }

    pub fn embed(&self, text: &str) -> Result<Vec<f32>> {
        if self.use_fallback || !self.is_loaded() {
            return self.embed_fallback(text);
        }

        #[cfg(all(feature = "semantic", feature = "ort"))]
        {
            self.embed_onnx(text)
        }
        #[cfg(any(not(feature = "semantic"), not(feature = "ort")))]
        {
            self.embed_tokenized(text)
        }
    }

    #[cfg(all(feature = "semantic", feature = "ort"))]
    fn embed_onnx(&self, text: &str) -> Result<Vec<f32>> {
        use ndarray::Array;

        let input_ids = self.tokenize_to_ids(text)?;
        let attention_mask = vec![1i64; input_ids.len()];

        let mut session = match &self.session {
            Some(s) => match s.lock() {
                Ok(guard) => guard,
                Err(_) => return self.embed_fallback(text),
            },
            None => return self.embed_fallback(text),
        };

        let input_ids_array: Array<i64, _> = Array::from_shape_vec(
            (1, input_ids.len()),
            input_ids.iter().map(|&x| x as i64).collect(),
        )?;
        let attention_mask_array: Array<i64, _> =
            Array::from_shape_vec((1, attention_mask.len()), attention_mask)?;

        let input_ids_tensor = ort::value::Tensor::from_array(input_ids_array.into_dyn())?;
        let attention_mask_tensor =
            ort::value::Tensor::from_array(attention_mask_array.into_dyn())?;

        let inputs = ort::inputs![
            "input_ids" => input_ids_tensor,
            "attention_mask" => attention_mask_tensor,
        ];

        let outputs = session.run(inputs)?;

        let output_array = outputs[0].try_extract_array::<f32>()?;
        let data: Vec<f32> = output_array.iter().copied().collect();

        let seq_len = input_ids.len();
        let hidden_size = self.config.hidden_size;

        let mut sum = vec![0.0f32; hidden_size];
        for i in 0..seq_len {
            for j in 0..hidden_size {
                sum[j] += data[i * hidden_size + j];
            }
        }

        for j in 0..hidden_size {
            sum[j] /= seq_len as f32;
        }

        Ok(sum)
    }

    #[cfg(feature = "semantic")]
    fn embed_tokenized(&self, text: &str) -> Result<Vec<f32>> {
        match &self.tokenizer {
            Some(tokenizer) => {
                let encoding = tokenizer
                    .encode(text, false)
                    .map_err(|e| anyhow::anyhow!(e))?;

                let ids = encoding.get_ids();
                let attention_mask = encoding.get_attention_mask();
                let lowercase_text = text.to_lowercase();

                let hidden_size = self.config.hidden_size;
                let mut embedding = vec![0.0f32; hidden_size];

                // Token-based embedding
                for (i, &id) in ids.iter().enumerate() {
                    let mask_weight = if i < attention_mask.len() && attention_mask[i] > 0 {
                        1.0
                    } else {
                        0.0
                    };

                    let base_idx = id as usize % hidden_size;
                    embedding[base_idx] += mask_weight;

                    if base_idx + 1 < hidden_size {
                        embedding[base_idx + 1] += mask_weight * 0.5;
                    }
                }

                // Role-specific keyword boosting
                let role_keywords: Vec<(AgentRole, Vec<&str>)> = vec![
                    (
                        AgentRole::Extractor,
                        vec!["git", "diff", "extract", "file", "change", "delta"],
                    ),
                    (
                        AgentRole::Analyzer,
                        vec!["analyze", "metric", "pattern", "statistic", "find"],
                    ),
                    (
                        AgentRole::Reviewer,
                        vec!["review", "security", "bug", "error", "quality", "check"],
                    ),
                    (
                        AgentRole::Writer,
                        vec!["write", "doc", "update", "content", "text"],
                    ),
                    (
                        AgentRole::Synthesizer,
                        vec!["summar", "conclu", "recommend", "consolid"],
                    ),
                    (AgentRole::Tester, vec!["test", "verif", "unit", "check"]),
                    (
                        AgentRole::Documenter,
                        vec!["document", "comment", "api", "guide"],
                    ),
                    (
                        AgentRole::Optimizer,
                        vec!["optim", "perf", "refactor", "efficien"],
                    ),
                ];

                for (role, keywords) in &role_keywords {
                    for keyword in keywords {
                        if lowercase_text.contains(keyword) {
                            let boost_idx = (*role as usize) % hidden_size;
                            embedding[boost_idx] += 0.3;
                        }
                    }
                }

                // Normalize
                let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
                if norm > 0.0 {
                    for v in &mut embedding {
                        *v /= norm;
                    }
                }

                Ok(embedding)
            }
            None => self.embed_fallback(text),
        }
    }

    #[cfg(feature = "semantic")]
    fn tokenize_to_ids(&self, text: &str) -> Result<Vec<u32>> {
        match &self.tokenizer {
            Some(tokenizer) => {
                let encoding = tokenizer
                    .encode(text, false)
                    .map_err(|e| anyhow::anyhow!(e))?;
                Ok(encoding.get_ids().to_vec())
            }
            None => self.tokenize_fallback(text),
        }
    }

    fn tokenize_fallback(&self, text: &str) -> Result<Vec<u32>> {
        let mut tokens = Vec::new();
        tokens.push(self.config.bos_token_id);

        let clean_text: String = text
            .to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace())
            .collect();

        let vocab_size_u32 = self.config.vocab_size as u32;
        for word in clean_text.split_whitespace() {
            let hash = Self::simple_hash(word);
            tokens.push(hash % vocab_size_u32);
        }

        tokens.push(self.config.eos_token_id);
        Ok(tokens)
    }

    fn simple_hash(word: &str) -> u32 {
        let mut hash = 0u32;
        for byte in word.bytes() {
            hash = hash.wrapping_mul(31).wrapping_add(byte as u32);
        }
        hash
    }

    fn embed_fallback(&self, text: &str) -> Result<Vec<f32>> {
        let lowercase_text = text.to_lowercase();
        let words: Vec<&str> = lowercase_text.split_whitespace().collect();
        let mut embedding = vec![0.0f32; self.config.hidden_size];

        for (i, word) in words.iter().enumerate() {
            let hash = Self::simple_hash(word) as usize % self.config.hidden_size;
            let weight = (i as f32 + 1.0).recip();
            embedding[hash] += weight;
        }

        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for v in &mut embedding {
                *v /= norm;
            }
        }

        Ok(embedding)
    }

    pub fn cosine_similarity(&self, a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return 0.0;
        }

        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }

        dot_product / (norm_a * norm_b)
    }

    pub fn embedding_dimension(&self) -> usize {
        self.config.hidden_size
    }
}

impl Default for SemanticEngine {
    fn default() -> Self {
        let mut engine = Self::new();
        let _ = engine.initialize();
        engine
    }
}

#[derive(Debug, Clone)]
pub struct RoleEmbeddingStore {
    engine: Arc<SemanticEngine>,
    role_embeddings: HashMap<AgentRole, Vec<f32>>,
}

impl RoleEmbeddingStore {
    pub fn new(engine: Arc<SemanticEngine>) -> Self {
        let mut store = Self {
            engine,
            role_embeddings: HashMap::new(),
        };

        let role_descriptions = [
            (
                AgentRole::Extractor,
                "Extract code changes, file deltas, git diff, modifications, additions".to_string(),
            ),
            (
                AgentRole::Analyzer,
                "Analyze code patterns, metrics, statistics, findings, trends".to_string(),
            ),
            (
                AgentRole::Writer,
                "Write documentation, content, updates, revisions, text".to_string(),
            ),
            (
                AgentRole::Reviewer,
                "Review code, security issues, bugs, errors, violations, quality gate".to_string(),
            ),
            (
                AgentRole::Synthesizer,
                "Synthesize summaries, findings, consolidations, conclusions, recommendations"
                    .to_string(),
            ),
            (
                AgentRole::Tester,
                "Test code, run tests, execute verification, unit tests".to_string(),
            ),
            (
                AgentRole::Documenter,
                "Write documentation, comments, API docs, guides".to_string(),
            ),
            (
                AgentRole::Optimizer,
                "Optimize performance, refactor code, improve efficiency".to_string(),
            ),
            (
                AgentRole::Specialist,
                "Handle specialized tasks, domain-specific requirements".to_string(),
            ),
            (
                AgentRole::General,
                "General purpose tasks, communication, messaging".to_string(),
            ),
        ];

        for (role, description) in role_descriptions {
            if let Ok(embedding) = store.engine.embed(&description) {
                store.role_embeddings.insert(role, embedding);
            }
        }

        store
    }

    pub fn route_task(&self, user_prompt: &str) -> AgentRole {
        let prompt_embedding = match self.engine.embed(user_prompt) {
            Ok(e) => e,
            Err(_) => {
                return AgentRole::General;
            }
        };

        let mut best_role = AgentRole::General;
        let mut best_score = 0.0f32;

        for (role, role_embedding) in &self.role_embeddings {
            let score = self
                .engine
                .cosine_similarity(&prompt_embedding, role_embedding);
            if score > best_score {
                best_score = score;
                best_role = *role;
            }
        }

        best_role
    }

    pub fn get_all_scores(&self, user_prompt: &str) -> Vec<(AgentRole, f32)> {
        let prompt_embedding = match self.engine.embed(user_prompt) {
            Ok(e) => e,
            Err(_) => {
                return Vec::new();
            }
        };

        let mut scores: Vec<(AgentRole, f32)> = self
            .role_embeddings
            .iter()
            .map(|(role, role_embedding)| {
                let score = self
                    .engine
                    .cosine_similarity(&prompt_embedding, role_embedding);
                (*role, score)
            })
            .collect();

        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        scores
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity() {
        let mut engine = SemanticEngine::new();
        engine.initialize().ok();

        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        let c = vec![0.0, 1.0, 0.0];

        assert!((engine.cosine_similarity(&a, &b) - 1.0).abs() < 0.001);
        assert!((engine.cosine_similarity(&a, &c) - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_embedding_dimension() {
        let engine = SemanticEngine::new();
        assert_eq!(engine.embedding_dimension(), 384);
    }

    #[test]
    fn test_role_embedding_store() {
        let mut engine = SemanticEngine::new();
        engine.initialize().ok();
        let engine = Arc::new(engine);

        let store = RoleEmbeddingStore::new(engine);

        let code_review_task = "Review this pull request for security issues";
        let role = store.route_task(code_review_task);
        assert_eq!(role, AgentRole::Reviewer);

        let extraction_task = "Show me the git diff for the recent changes";
        let role = store.route_task(extraction_task);
        assert_eq!(role, AgentRole::Extractor);
    }

    #[test]
    fn test_all_scores() {
        let mut engine = SemanticEngine::new();
        engine.initialize().ok();
        let engine = Arc::new(engine);

        let store = RoleEmbeddingStore::new(engine);
        let scores = store.get_all_scores("Analyze the codebase metrics");

        assert!(!scores.is_empty());
        assert!(scores[0].1 >= scores[1].1);
    }
}
