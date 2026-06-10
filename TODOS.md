# TODOS — Second Brain Router

Progress tracker. Updated as work proceeds.

---

## v0.1 — AX Capture Daemon ✅ DONE

> Goal: a Rust binary that runs on macOS, captures text from the active window via AX API, and prints structured output to stdout.

### Setup
- [x] Initialize Cargo workspace (`crates/sbr-daemon`)
- [x] Add dependencies: `core-foundation`, `tokio`, `serde`, `toml`, `tracing`, `objc2`
- [x] Set up `config.rs` with TOML loader and default config
- [x] Set up `tracing` based logging

### Capture: AX Watcher (`capture/ax_watcher.rs`)
- [x] Get frontmost app PID via `NSWorkspace` + `msg_send!`
- [x] Create `AXUIElementCreateApplication(pid)` handle
- [x] Read `kAXFocusedUIElementAttribute` → focused element
- [x] Read `kAXSelectedTextAttribute` from focused element
- [x] Recursive UI tree traversal: read `kAXChildrenAttribute` + `kAXValueAttribute`
- [x] Filter out password fields (`kAXSecureTextField` role)
- [x] Filter out empty / whitespace-only strings
- [x] Emit structured `CaptureEvent { app, window_title, texts, timestamp }` on change
- [x] Poll loop with configurable interval (default 1s)
- [x] Content hash dedup (skip unchanged windows)

### Config
- [x] `capture.ax_enabled = true`
- [x] `capture.screenshot_enabled = false`
- [x] `capture.poll_interval_ms = 1000`
- [x] `capture.excluded_apps = ["1Password", "Keychain", ...]`

### CI
- [x] GitHub Actions CI on `macos-latest`
- [x] `cargo fmt --check`, `cargo clippy -D warnings`, `cargo build`, `cargo test`
- [x] `rustfmt.toml` pinned to `max_width = 100`

---

## v0.2 — Memory Pipeline ✅ DONE

> Goal: chunk captured text, embed it locally via Ollama, store vectors in qdrant.

- [x] `chunker.rs`: sliding window chunking (configurable size + overlap)
- [x] `chunker.rs`: content hash dedup
- [x] `embedder.rs`: async HTTP client calling Ollama `/api/embed`
- [x] `store.rs`: qdrant client — create collection + upsert vectors with payload
- [x] `store.rs`: payload schema `{ text, app_name, window_title, timestamp, source }`
- [x] Wire `ax_watcher` → `chunker` → `embedder` → `store` in `main.rs`
- [x] Docker Compose for local qdrant (`docker/docker-compose.yml`)

---

## v0.3 — Router Engine + CLI Hint 🔄 IN PROGRESS

> Goal: given current context, retrieve relevant memories and decide whether to surface a hint.

- [x] `context.rs`: detect current task context (active app + window title + focused text)
- [x] `engine.rs` / `router.rs`: embed current context, query qdrant top-k
- [x] `router.rs`: relevance threshold filter (score >= 0.75)
- [x] `router.rs`: hint decision logic (30s cooldown per app)
- [x] CLI output: print hint to stdout with source provenance
- [x] `sbr-daemon ask "<query>"` manual query subcommand
- [ ] Integration smoke test: run daemon for 60s, verify hints appear in stdout

---

## v0.4 — Screenshot Fallback

> Goal: for apps where AX returns nothing (Figma, YouTube), fall back to screenshot + vision model.

- [ ] `screenshot.rs`: detect when AX tree returns < N chars
- [ ] `screenshot.rs`: capture screen with `xcap` crate
- [ ] `screenshot.rs`: send image to Ollama vision model (`qwen2.5vl`)
- [ ] Parse vision model response → plain text → feed into chunker
- [ ] Config: `capture.screenshot_enabled = false` (opt-in)

---

## v0.5 — Tauri Overlay UI

> Goal: non-intrusive floating hint window that appears at the right moment.

- [ ] Init Tauri app (`crates/sbr-ui`)
- [ ] IPC between daemon and UI via local Unix socket
- [ ] Floating hint window (always on top, click-through when idle)
- [ ] Dismiss on click or timeout (5s)
- [ ] Show source provenance (app name + timestamp)

---

## v0.6 — Microphone + Whisper (opt-in)

> Goal: transcribe meetings and conversations locally.

- [ ] Microphone capture via `cpal` crate
- [ ] VAD (voice activity detection) to skip silence
- [ ] Send audio chunks to local `faster-whisper` HTTP server
- [ ] Speaker diarization (basic, by silence gap)
- [ ] Feed transcript into chunker → memory pipeline
- [ ] Config: `capture.microphone_enabled = false`

---

## Backlog / Ideas

- [ ] Browser extension (Chrome): send current page URL + visible text via native messaging
- [ ] Context graph UI: visualize memory as a timeline + knowledge graph
- [ ] Per-project memory isolation
- [ ] Export / backup memory store
- [ ] Windows support (`UI Automation` API instead of AX)
