use anyhow::Result;
use qdrant_client::qdrant::{Condition, Filter, SearchPointsBuilder};
use qdrant_client::Qdrant;
use tracing::debug;

use crate::memory::store::COLLECTION;

/// Minimum cosine similarity score to surface a hint (0.0 – 1.0).
pub const RELEVANCE_THRESHOLD: f32 = 0.75;
/// Number of nearest neighbours to retrieve from qdrant.
const TOP_K: u64 = 5;

/// A single retrieved memory with its relevance score.
#[derive(Debug, Clone)]
pub struct Hit {
    pub text: String,
    pub app_name: String,
    pub window_title: String,
    pub timestamp: String,
    pub score: f32,
}

/// Query qdrant with a context embedding and return relevant hits above threshold.
pub async fn query(
    client: &Qdrant,
    vector: Vec<f32>,
    app_filter: Option<&str>,
) -> Result<Vec<Hit>> {
    let mut builder = SearchPointsBuilder::new(COLLECTION, vector, TOP_K).with_payload(true);

    // Optionally restrict search to memories from the same app
    if let Some(app) = app_filter {
        let filter = Filter::must([Condition::matches("app_name", app.to_string())]);
        builder = builder.filter(filter);
    }

    let response = client
        .search_points(builder)
        .await
        .map_err(|e| anyhow::anyhow!("qdrant search failed: {}", e))?;

    let hits: Vec<Hit> = response
        .result
        .into_iter()
        .filter(|r| r.score >= RELEVANCE_THRESHOLD)
        .filter_map(|r| {
            let payload = &r.payload;
            let text = payload.get("text")?.as_str()?.to_string();
            let app_name = payload
                .get("app_name")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();
            let window_title = payload
                .get("window_title")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let timestamp = payload
                .get("timestamp")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            Some(Hit {
                text,
                app_name,
                window_title,
                timestamp,
                score: r.score,
            })
        })
        .collect();

    debug!("query returned {} hits above threshold", hits.len());
    Ok(hits)
}

/// Print hits to stdout in a human-readable format.
pub fn print_hints(hits: &[Hit]) {
    if hits.is_empty() {
        println!("No relevant memories found.");
        return;
    }
    println!("\n── Relevant memories ──");
    for (i, hit) in hits.iter().enumerate() {
        println!(
            "[{}] ({:.2}) {} | {}\n    {}",
            i + 1,
            hit.score,
            hit.app_name,
            hit.timestamp,
            &hit.text[..hit.text.len().min(200)]
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relevance_threshold_value() {
        assert_eq!(RELEVANCE_THRESHOLD, 0.75);
    }

    #[test]
    fn test_print_hints_empty() {
        // smoke test — just verify it doesn't panic
        print_hints(&[]);
    }

    #[test]
    fn test_print_hints_with_hit() {
        let hit = Hit {
            text: "hello world".into(),
            app_name: "Safari".into(),
            window_title: "GitHub".into(),
            timestamp: "2026-01-01T00:00:00Z".into(),
            score: 0.92,
        };
        print_hints(&[hit]);
    }
}
