use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Serialize)]
struct EmbedRequest {
    model: String,
    prompt: String,
}

#[derive(Debug, Deserialize)]
struct EmbedResponse {
    embedding: Vec<f32>,
}

#[derive(Debug, Serialize)]
struct GenerateRequest {
    model: String,
    prompt: String,
    stream: bool,
}

#[derive(Debug, Deserialize)]
struct GenerateResponse {
    response: String,
    done: bool,
}

#[derive(Debug, Deserialize)]
struct ModelsResponse {
    models: Vec<ModelInfo>,
}

#[derive(Debug, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub model: String,
}

pub struct OllamaClient {
    client: Client,
    base_url: String,
}

impl OllamaClient {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(300))
                .build()
                .expect("Failed to create HTTP client"),
            base_url: "http://localhost:11434".to_string(),
        }
    }

    pub fn with_url(base_url: &str) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(300))
                .build()
                .expect("Failed to create HTTP client"),
            base_url: base_url.to_string(),
        }
    }

    pub async fn is_available(&self) -> bool {
        match self.client
            .get(format!("{}/api/tags", self.base_url))
            .send()
            .await
        {
            Ok(resp) => resp.status().is_success(),
            Err(_) => false,
        }
    }

    pub async fn list_models(&self) -> Result<Vec<ModelInfo>> {
        let resp = self.client
            .get(format!("{}/api/tags", self.base_url))
            .send()
            .await?
            .json::<ModelsResponse>()
            .await?;
        Ok(resp.models)
    }

    pub async fn generate_embedding(&self, model: &str, text: &str) -> Result<Vec<f32>> {
        let request = EmbedRequest {
            model: model.to_string(),
            prompt: text.to_string(),
        };

        let resp = self.client
            .post(format!("{}/api/embeddings", self.base_url))
            .json(&request)
            .send()
            .await?;

        let embed_response: EmbedResponse = resp.json().await?;
        Ok(embed_response.embedding)
    }

    pub async fn generate(&self, model: &str, prompt: &str) -> Result<String> {
        let request = GenerateRequest {
            model: model.to_string(),
            prompt: prompt.to_string(),
            stream: false,
        };

        let resp = self.client
            .post(format!("{}/api/generate", self.base_url))
            .json(&request)
            .send()
            .await?;

        let generate_response: GenerateResponse = resp.json().await?;
        Ok(generate_response.response)
    }
}

impl Default for OllamaClient {
    fn default() -> Self {
        Self::new()
    }
}
