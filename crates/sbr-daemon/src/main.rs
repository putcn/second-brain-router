mod capture;
mod config;

use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, warn};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("sbr_daemon=debug")
        .init();

    let cfg = config::Config::load_or_default();
    info!("sbr-daemon starting. poll_interval={}ms", cfg.capture.poll_interval_ms);
    info!("excluded apps: {:?}", cfg.capture.excluded_apps);

    let mut watcher = capture::ax_watcher::AXWatcher::new(cfg.clone());

    loop {
        match watcher.poll().await {
            Some(event) => {
                // v0.1: just print to stdout
                // v0.2: this will feed into chunker -> embedder -> store
                println!(
                    "[{}] {} | {} chars captured",
                    event.timestamp.format("%H:%M:%S"),
                    event.app_name,
                    event.texts.iter().map(|t| t.len()).sum::<usize>()
                );
                for text in &event.texts {
                    println!("  > {}", &text[..text.len().min(120)]);
                }
            }
            None => {
                warn!("no content captured this cycle");
            }
        }
        sleep(Duration::from_millis(cfg.capture.poll_interval_ms)).await;
    }
}
