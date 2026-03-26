use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingConfig {
    pub model: String,
    pub dimension: usize,
    pub batch_size: usize,
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            model: "nomic-embed-text".to_string(),
            dimension: 768,
            batch_size: 32,
        }
    }
}

pub struct EmbeddingStore {
    config: EmbeddingConfig,
    chroma_dir: PathBuf,
}

impl EmbeddingStore {
    pub fn new(app_data_dir: &PathBuf) -> Result<Self, String> {
        let chroma_dir = app_data_dir.join("chroma");
        fs::create_dir_all(&chroma_dir)
            .map_err(|e| format!("Failed to create chroma directory: {}", e))?;

        Ok(Self {
            config: EmbeddingConfig::default(),
            chroma_dir,
        })
    }

    pub fn get_chroma_dir(&self) -> &PathBuf {
        &self.chroma_dir
    }

    pub fn get_config(&self) -> &EmbeddingConfig {
        &self.config
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChromaCollection {
    pub name: String,
    pub id: String,
    pub embedding_dimension: usize,
    pub document_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChromaEntry {
    pub id: String,
    pub embedding: Vec<f32>,
    pub document: String,
    pub metadata: serde_json::Value,
}

pub fn save_chroma_index(chroma_dir: &PathBuf, entries: &[ChromaEntry], collection_name: &str) -> Result<(), String> {
    let index_path = chroma_dir.join(format!("{}.json", collection_name));
    let json = serde_json::to_string_pretty(entries)
        .map_err(|e| format!("Failed to serialize entries: {}", e))?;
    fs::write(&index_path, json)
        .map_err(|e| format!("Failed to write index: {}", e))?;
    Ok(())
}

pub fn load_chroma_index(chroma_dir: &PathBuf, collection_name: &str) -> Result<Vec<ChromaEntry>, String> {
    let index_path = chroma_dir.join(format!("{}.json", collection_name));
    if !index_path.exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(&index_path)
        .map_err(|e| format!("Failed to read index: {}", e))?;
    serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse index: {}", e))
}

pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }
    
    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let magnitude_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let magnitude_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    
    if magnitude_a == 0.0 || magnitude_b == 0.0 {
        return 0.0;
    }
    
    dot_product / (magnitude_a * magnitude_b)
}

pub fn find_similar(entries: &[ChromaEntry], query_embedding: &[f32], top_k: usize) -> Vec<(String, f32)> {
    let mut similarities: Vec<(String, f32)> = entries
        .iter()
        .map(|e| (e.id.clone(), cosine_similarity(&e.embedding, query_embedding)))
        .collect();
    
    similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    similarities.truncate(top_k);
    similarities
}
