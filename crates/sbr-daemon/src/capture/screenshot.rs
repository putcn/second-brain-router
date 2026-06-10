//! Screenshot fallback for apps where AX returns too little text (e.g. Figma, YouTube).
//!
//! When enabled (`capture.screenshot_enabled = true`), this module:
//! 1. Detects that AX returned fewer chars than `min_chars_for_ax`
//! 2. Captures the primary display via `xcap`
//! 3. Sends the image to Ollama vision model and extracts plain text
//! 4. Returns the text so it can be fed into the chunker

use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

/// Minimum AX text length below which we fall back to screenshot.
pub const MIN_CHARS_FOR_AX: usize = 50;

// ---------------------------------------------------------------------------
// Ollama vision API types
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
struct VisionRequest<'a> {
    model: &'a str,
    prompt: &'a str,
    images: Vec<String>,
    stream: bool,
}

#[derive(Debug, Deserialize)]
struct VisionResponse {
    response: String,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Capture the primary display and extract text via Ollama vision model.
/// Returns extracted plain text, or an error.
pub async fn capture_and_extract(
    ollama_url: &str,
    model: &str,
) -> Result<String> {
    let png_bytes = capture_screen().context("screen capture failed")?;
    let b64 = B64.encode(&png_bytes);

    info!("screenshot captured ({} bytes), sending to vision model", png_bytes.len());

    extract_text_from_image(ollama_url, model, b64).await
}

/// Returns true if AX text is too sparse to be useful.
pub fn ax_text_too_sparse(ax_text: &str) -> bool {
    ax_text.trim().len() < MIN_CHARS_FOR_AX
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Capture the primary display, returning raw PNG bytes.
fn capture_screen() -> Result<Vec<u8>> {
    use xcap::Monitor;

    let monitors = Monitor::all().context("failed to enumerate monitors")?;
    let primary = monitors.into_iter().next().context("no monitor found")?;

    let image = primary.capture_image().context("monitor capture failed")?;

    let mut buf: Vec<u8> = Vec::new();
    image
        .write_to(
            &mut std::io::Cursor::new(&mut buf),
            image::ImageFormat::Png,
        )
        .context("PNG encode failed")?;

    debug!("PNG encoded: {} bytes", buf.len());
    Ok(buf)
}

/// Send a base64-encoded PNG to the Ollama vision endpoint and return extracted text.
async fn extract_text_from_image(
    ollama_url: &str,
    model: &str,
    b64_image: String,
) -> Result<String> {
    let url = format!("{}/api/generate", ollama_url);

    let body = VisionRequest {
        model,
        prompt: "Extract all visible text from this screenshot. \
                 Return only the text content, no commentary.",
        images: vec![b64_image],
        stream: false,
    };

    let client = reqwest::Client::new();
    let resp = client
        .post(&url)
        .json(&body)
        .send()
        .await
        .with_context(|| format!("POST {} failed", url))?;

    let status = resp.status();
    if !status.is_success() {
        let text = resp.text().await.unwrap_or_default();
        anyhow::bail!("Ollama vision returned {}: {}", status, text);
    }

    let parsed: VisionResponse = resp.json().await.context("failed to parse vision response")?;
    Ok(parsed.response.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ax_text_too_sparse_below_threshold() {
        assert!(ax_text_too_sparse("hi"));
        assert!(ax_text_too_sparse(""));
        assert!(ax_text_too_sparse("   "));
    }

    #[test]
    fn test_ax_text_not_sparse_above_threshold() {
        let text = "a".repeat(MIN_CHARS_FOR_AX);
        assert!(!ax_text_too_sparse(&text));
    }

    #[test]
    fn test_min_chars_constant() {
        assert_eq!(MIN_CHARS_FOR_AX, 50);
    }
}
