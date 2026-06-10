mod capture;
mod config;
mod engine;
mod memory;

use capture::screenshot;
use engine::{context::Context, router};
use memory::{
    chunker::{chunk_text, content_hash},
    embedder::Embedder,
    store::{MemoryPayload, MemoryStore},
};

use std::{
    env,
    time::{Duration, Instant},
};
use tokio::time::sleep;
use tracing::{error, info, warn};

/// Minimum ms between hints for the same app (cooldown).
const HINT_COOLDOWN_MS: u128 = 30_000;
/// Ollama base URL.
const OLLAMA_URL: &str = "http://localhost:11434";
/// Vision model for screenshot fallback.
const VISION_MODEL: &str = "qwen2.5vl";

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("sbr_daemon=debug")
        .init();

    let args: Vec<String> = env::args().collect();

    if args.get(1).map(|s| s.as_str()) == Some("ask") {
        let query_text = args[2..].join(" ");
        if query_text.is_empty() {
            eprintln!("Usage: sbr-daemon ask \"<query>\"");
            std::process::exit(1);
        }
        run_ask(&query_text).await;
        return;
    }

    run_daemon().await;
}

async fn run_ask(query_text: &str) {
    let embedder = Embedder::default();
    let store = match connect_qdrant().await {
        Some(s) => s,
        None => {
            eprintln!("qdrant unavailable");
            std::process::exit(1);
        }
    };

    match embedder.embed(query_text).await {
        Ok(vector) => match router::query(store.client_ref(), vector, None).await {
            Ok(hits) => router::print_hints(&hits),
            Err(e) => eprintln!("query failed: {}", e),
        },
        Err(e) => eprintln!("embed failed: {}", e),
    }
}

async fn run_daemon() {
    let cfg = config::Config::load_or_default();
    info!(
        "sbr-daemon starting. poll_interval={}ms screenshot={}",
        cfg.capture.poll_interval_ms, cfg.capture.screenshot_enabled
    );
    info!("excluded apps: {:?}", cfg.capture.excluded_apps);

    let embedder = Embedder::default();
    let store = connect_qdrant().await;

    if let Some(ref s) = store {
        if let Err(e) = s.ensure_collection().await {
            error!("failed to ensure qdrant collection: {}", e);
        }
    }

    let mut watcher = capture::ax_watcher::AXWatcher::new(cfg.clone());
    let mut last_hint_at: Option<Instant> = None;
    let mut last_hint_app: Option<String> = None;

    loop {
        if let Some(event) = watcher.poll().await {
            info!(
                "[{}] {} — {} text nodes",
                event.timestamp.format("%H:%M:%S"),
                event.app_name,
                event.texts.len()
            );

            // --- v0.4: screenshot fallback ---
            let text = if cfg.capture.screenshot_enabled
                && screenshot::ax_text_too_sparse(&event.texts.join(" "))
            {
                info!("AX sparse for {}, trying screenshot fallback", event.app_name);
                match screenshot::capture_and_extract(OLLAMA_URL, VISION_MODEL).await {
                    Ok(t) => {
                        info!("vision extracted {} chars", t.len());
                        t
                    }
                    Err(e) => {
                        warn!("screenshot fallback failed: {}", e);
                        event.texts.join(" ")
                    }
                }
            } else {
                event.texts.join(" ")
            };

            let ctx = Context {
                app_name: event.app_name.clone(),
                window_title: event.window_title.clone(),
                text: text.clone(),
            };

            if let Some(ref s) = store {
                // --- store chunks into memory ---
                let chunks = chunk_text(&ctx.text, 512, 64, cfg.capture.min_text_length);
                for chunk in &chunks {
                    let hash = content_hash(chunk);
                    match s.exists_by_hash(&hash).await {
                        Ok(true) => continue,
                        Ok(false) => {}
                        Err(e) => {
                            warn!("qdrant dedup check failed: {}", e);
                            continue;
                        }
                    }
                    match embedder.embed(chunk).await {
                        Ok(vector) => {
                            let payload = MemoryPayload {
                                text: chunk.clone(),
                                content_hash: hash,
                                app_name: event.app_name.clone(),
                                window_title: event.window_title.clone(),
                                timestamp: event.timestamp.to_rfc3339(),
                                source: if cfg.capture.screenshot_enabled
                                    && screenshot::ax_text_too_sparse(&event.texts.join(" "))
                                {
                                    "screenshot".into()
                                } else {
                                    "ax".into()
                                },
                            };
                            if let Err(e) = s.upsert(&payload, vector).await {
                                error!("qdrant upsert failed: {}", e);
                            }
                        }
                        Err(e) => warn!("embed failed: {}", e),
                    }
                }

                // --- router: surface hint if context is meaningful ---
                if ctx.is_meaningful(cfg.capture.min_text_length) {
                    let should_hint = !matches!(
                        (&last_hint_app, &last_hint_at),
                        (Some(app), Some(t))
                            if app == &ctx.app_name
                                && t.elapsed().as_millis() < HINT_COOLDOWN_MS
                    );

                    if should_hint {
                        match embedder.embed(&ctx.text).await {
                            Ok(vector) => {
                                match router::query(s.client_ref(), vector, Some(&ctx.app_name))
                                    .await
                                {
                                    Ok(hits) if !hits.is_empty() => {
                                        router::print_hints(&hits);
                                        last_hint_at = Some(Instant::now());
                                        last_hint_app = Some(ctx.app_name.clone());
                                    }
                                    Ok(_) => {}
                                    Err(e) => warn!("router query failed: {}", e),
                                }
                            }
                            Err(e) => warn!("context embed failed: {}", e),
                        }
                    }
                }
            }
        }

        sleep(Duration::from_millis(cfg.capture.poll_interval_ms)).await;
    }
}

async fn connect_qdrant() -> Option<MemoryStore> {
    match MemoryStore::connect("http://localhost:6334").await {
        Ok(s) => {
            info!("connected to qdrant");
            Some(s)
        }
        Err(e) => {
            warn!("qdrant unavailable, memory pipeline disabled: {}", e);
            None
        }
    }
}
