use sha2::{Digest, Sha256};

/// A single text chunk with provenance metadata.
#[derive(Debug, Clone, PartialEq)]
pub struct Chunk {
    pub text: String,
    pub content_hash: String,
    pub app_name: String,
    pub window_title: String,
    pub source: ChunkSource,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ChunkSource {
    AxTree,
    Screenshot,
    Microphone,
}

/// Split `text` into overlapping chunks of `chunk_size` chars with `overlap` chars overlap.
/// Filters out chunks shorter than `min_length`.
pub fn chunk_text(
    text: &str,
    chunk_size: usize,
    overlap: usize,
    min_length: usize,
) -> Vec<String> {
    if text.trim().len() < min_length {
        return vec![];
    }
    let chars: Vec<char> = text.chars().collect();
    let total = chars.len();
    if total <= chunk_size {
        return vec![text.trim().to_string()];
    }

    let step = chunk_size.saturating_sub(overlap).max(1);
    let mut chunks = Vec::new();
    let mut start = 0;

    while start < total {
        let end = (start + chunk_size).min(total);
        let chunk: String = chars[start..end].iter().collect();
        let trimmed = chunk.trim().to_string();
        if trimmed.len() >= min_length {
            chunks.push(trimmed);
        }
        if end == total {
            break;
        }
        start += step;
    }

    chunks
}

/// Compute a stable SHA-256 content hash for dedup.
pub fn content_hash(text: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text.trim().as_bytes());
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── chunk_text ─────────────────────────────────────────────────────────────────

    #[test]
    fn test_short_text_returned_as_single_chunk() {
        let text = "Hello world";
        let chunks = chunk_text(text, 100, 10, 5);
        assert_eq!(chunks, vec!["Hello world"]);
    }

    #[test]
    fn test_text_below_min_length_returns_empty() {
        let chunks = chunk_text("Hi", 100, 10, 10);
        assert!(chunks.is_empty());
    }

    #[test]
    fn test_long_text_splits_into_multiple_chunks() {
        // 50 chars, chunk_size=20, overlap=5 => step=15
        let text = "a".repeat(50);
        let chunks = chunk_text(&text, 20, 5, 1);
        assert!(chunks.len() > 1);
        // every chunk should be <= 20 chars
        for c in &chunks {
            assert!(c.len() <= 20);
        }
    }

    #[test]
    fn test_overlap_means_chunks_share_content() {
        let text: String = (0..40).map(|i| char::from_digit(i % 10, 10).unwrap()).collect();
        let chunks = chunk_text(&text, 10, 5, 1);
        // With overlap=5, chunk[1] should start 5 chars into chunk[0]
        assert!(chunks.len() >= 2);
        let overlap_region = &chunks[0][5..10];
        assert!(chunks[1].starts_with(overlap_region));
    }

    #[test]
    fn test_exact_chunk_size_text_is_single_chunk() {
        let text = "x".repeat(20);
        let chunks = chunk_text(&text, 20, 5, 1);
        assert_eq!(chunks.len(), 1);
    }

    #[test]
    fn test_whitespace_only_text_returns_empty() {
        let chunks = chunk_text("     \n\t  ", 100, 10, 5);
        assert!(chunks.is_empty());
    }

    #[test]
    fn test_last_chunk_covers_end_of_text() {
        let text = "a".repeat(35);
        let chunks = chunk_text(&text, 20, 5, 1);
        let last = chunks.last().unwrap();
        // last chunk must end at the last char of text
        assert!(text.ends_with(last.as_str()));
    }

    // ── content_hash ───────────────────────────────────────────────────────────

    #[test]
    fn test_same_text_produces_same_hash() {
        let h1 = content_hash("hello world");
        let h2 = content_hash("hello world");
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_different_text_produces_different_hash() {
        let h1 = content_hash("hello world");
        let h2 = content_hash("hello world!");
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_hash_trims_whitespace() {
        let h1 = content_hash("hello");
        let h2 = content_hash("  hello  ");
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_hash_is_hex_string() {
        let h = content_hash("test");
        assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
        assert_eq!(h.len(), 64); // SHA-256 = 32 bytes = 64 hex chars
    }
}
