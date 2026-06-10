mod capture;
mod config;
mod memory;

use memory::{
    chunker::{chunk_text, content_hash},
    embedder::Embedder,
    store::{MemoryPayload, MemoryStore},
};

use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info, warn};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("sbr_daemon=debug")
        .init();

    let cfg = config::Config::load_or_default();
    info!(
        "sbr-daemon starting. poll_interval={}ms",
        cfg.capture.poll_interval_ms
    );
    info!("excluded apps: {:?}", cfg.capture.excluded_apps);

    // --- v0.2 memory pipeline setup ---
    let embedder = Embedder::default();

    let store = match MemoryStore::connect("http://localhost:6334").await {
        Ok(s) => {
            info!("connected to qdrant");
            Some(s)
        }
        Err(e) => {
            warn!("qdrant unavailable, memory pipeline disabled: {}", e);
            None
        }
    };

    if let Some(ref s) = store {
        if let Err(e) = s.ensure_collection().await {
            error!("failed to ensure qdrant collection: {}", e);
        }
    }

    // --- capture loop ---
    let mut watcher = capture::ax_watcher::AXWatcher::new(cfg.clone());

    loop {
        if let Some(event) = watcher.poll().await {
            info!(
                "[{}] {} — {} text nodes",
                event.timestamp.format("%H:%M:%S"),
                event.app_name,
                event.texts.len()
            );

            // Feed into memory pipeline if qdrant is available
            if let Some(ref s) = store {
                let chunks =
                    chunk_text(&event.texts.join(" "), 512, 64, cfg.capture.min_text_length);

                for chunk in chunks {
                    let hash = content_hash(&chunk);

                    // Skip already-stored chunks
                    match s.exists_by_hash(&hash).await {
                        Ok(true) => continue,
                        Ok(false) => {}
                        Err(e) => {
                            warn!("qdrant dedup check failed: {}", e);
                            continue;
                        }
                    }

                    // Embed and store
                    match embedder.embed(&chunk).await {
                        Ok(vector) => {
                            let payload = MemoryPayload {
                                text: chunk.clone(),
                                content_hash: hash,
                                app_name: event.app_name.clone(),
                                window_title: event.window_title.clone(),
                                timestamp: event.timestamp.to_rfc3339(),
                                source: "ax".into(),
                            };
                            if let Err(e) = s.upsert(&payload, vector).await {
                                error!("qdrant upsert failed: {}", e);
                            }
                        }
                        Err(e) => {
                            warn!("embed failed for chunk: {}", e);
                        }
                    }
                }
            }
        }

        sleep(Duration::from_millis(cfg.capture.poll_interval_ms)).await;
    }
}
