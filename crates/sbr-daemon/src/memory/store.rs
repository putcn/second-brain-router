#![allow(dead_code)]

use anyhow::{Context, Result};
use qdrant_client::{
    qdrant::{
        CreateCollectionBuilder, Distance, PointStruct, UpsertPointsBuilder, VectorParamsBuilder,
    },
    Qdrant,
};
use std::collections::HashMap;
use uuid::Uuid;

/// Qdrant collection name for all captured memories.
pub const COLLECTION: &str = "sbr_memory";
/// Embedding dimension for `nomic-embed-text`.
pub const VECTOR_DIM: u64 = 768;

#[derive(Debug, Clone)]
pub struct MemoryPayload {
    pub text: String,
    pub content_hash: String,
    pub app_name: String,
    pub window_title: String,
    pub timestamp: String,
    pub source: String,
}

#[derive(Clone)]
pub struct MemoryStore {
    client: Qdrant,
}

impl MemoryStore {
    /// Connect to qdrant at `url` (e.g. `"http://localhost:6334"`).
    pub async fn connect(url: &str) -> Result<Self> {
        let client = Qdrant::from_url(url)
            .build()
            .with_context(|| format!("failed to connect to qdrant at {}", url))?;
        Ok(MemoryStore { client })
    }

    /// Ensure the collection exists. Safe to call on every startup (no-ops if already exists).
    pub async fn ensure_collection(&self) -> Result<()> {
        let exists = self
            .client
            .collection_exists(COLLECTION)
            .await
            .context("qdrant collection_exists check failed")?;

        if !exists {
            self.client
                .create_collection(
                    CreateCollectionBuilder::new(COLLECTION)
                        .vectors_config(VectorParamsBuilder::new(VECTOR_DIM, Distance::Cosine)),
                )
                .await
                .context("failed to create qdrant collection")?;

            tracing::info!("created qdrant collection '{}'", COLLECTION);
        }

        Ok(())
    }

    /// Upsert a single memory chunk with its embedding vector.
    pub async fn upsert(&self, payload: &MemoryPayload, vector: Vec<f32>) -> Result<()> {
        let id = Uuid::new_v4().to_string();

        let mut fields: HashMap<String, qdrant_client::qdrant::Value> = HashMap::new();
        fields.insert("text".into(), payload.text.clone().into());
        fields.insert("content_hash".into(), payload.content_hash.clone().into());
        fields.insert("app_name".into(), payload.app_name.clone().into());
        fields.insert("window_title".into(), payload.window_title.clone().into());
        fields.insert("timestamp".into(), payload.timestamp.clone().into());
        fields.insert("source".into(), payload.source.clone().into());

        let point = PointStruct::new(id, vector, fields);

        self.client
            .upsert_points(UpsertPointsBuilder::new(COLLECTION, vec![point]))
            .await
            .context("qdrant upsert failed")?;

        Ok(())
    }

    /// Check whether a chunk with the given content hash already exists (for dedup).
    pub async fn exists_by_hash(&self, content_hash: &str) -> Result<bool> {
        use qdrant_client::qdrant::{Condition, Filter};

        let filter = Filter::must([Condition::matches("content_hash", content_hash.to_string())]);

        let result = self
            .client
            .count(
                qdrant_client::qdrant::CountPointsBuilder::new(COLLECTION)
                    .filter(filter)
                    .exact(true),
            )
            .await
            .context("qdrant count failed")?;

        Ok(result.result.map(|r| r.count).unwrap_or(0) > 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collection_name_constant() {
        assert_eq!(COLLECTION, "sbr_memory");
    }

    #[test]
    fn test_vector_dim_constant() {
        assert_eq!(VECTOR_DIM, 768);
    }

    #[test]
    fn test_memory_payload_fields() {
        let p = MemoryPayload {
            text: "hello".into(),
            content_hash: "abc".into(),
            app_name: "Safari".into(),
            window_title: "GitHub".into(),
            timestamp: "2026-01-01T00:00:00Z".into(),
            source: "ax".into(),
        };
        assert_eq!(p.source, "ax");
        assert_eq!(p.app_name, "Safari");
    }
}
