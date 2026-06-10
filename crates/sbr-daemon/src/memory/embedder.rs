#![allow(dead_code)]

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// Ollama `/api/embed` request body.
#[derive(Debug, Serialize)]
struct EmbedRequest<'a> {
    model: &'a str,
    input: &'a str,
}

/// Ollama `/api/embed` response body.
#[derive(Debug, Deserialize)]
struct EmbedResponse {
    embeddings: Vec<Vec<f32>>,
}

/// Thin async client wrapping the Ollama embed endpoint.
#[derive(Debug, Clone)]
pub struct Embedder {
    client: reqwest::Client,
    base_url: String,
    model: String,
}

impl Embedder {
    pub fn new(base_url: impl Into<String>, model: impl Into<String>) -> Self {
        Embedder {
            client: reqwest::Client::new(),
            base_url: base_url.into(),
            model: model.into(),
        }
    }

    /// Embed a single text string. Returns a 768-dim vector for `nomic-embed-text`.
    pub async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let url = format!("{}/api/embed", self.base_url);
        let body = EmbedRequest {
            model: &self.model,
            input: text,
        };

        let resp = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .with_context(|| format!("POST {} failed", url))?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Ollama embed returned {}: {}", status, body);
        }

        let parsed: EmbedResponse = resp
            .json()
            .await
            .context("failed to parse Ollama embed response")?;

        parsed
            .embeddings
            .into_iter()
            .next()
            .context("Ollama returned empty embeddings array")
    }

    /// Embed a batch of texts. Returns one vector per input, in order.
    pub async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let mut results = Vec::with_capacity(texts.len());
        for text in texts {
            results.push(self.embed(text).await?);
        }
        Ok(results)
    }
}

impl Default for Embedder {
    fn default() -> Self {
        Embedder::new("http://localhost:11434", "nomic-embed-text")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedder_default_url_and_model() {
        let e = Embedder::default();
        assert_eq!(e.base_url, "http://localhost:11434");
        assert_eq!(e.model, "nomic-embed-text");
    }

    #[test]
    fn test_embedder_custom_config() {
        let e = Embedder::new("http://192.168.1.10:11434", "mxbai-embed-large");
        assert_eq!(e.base_url, "http://192.168.1.10:11434");
        assert_eq!(e.model, "mxbai-embed-large");
    }
}
