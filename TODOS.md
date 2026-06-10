# TODOS — Second Brain Router

Progress tracker. Updated as work proceeds.

---

## v0.1 — AX Capture Daemon (current)

> Goal: a Rust binary that runs on macOS, captures text from the active window via AX API, and prints structured output to stdout.

### Setup
- [ ] Initialize Cargo workspace (`crates/sbr-daemon`)
- [ ] Add dependencies: `accessibility`, `core-foundation`, `tokio`, `serde`, `toml`, `tracing`
- [ ] Set up `config.rs` with TOML loader and default config
- [ ] Set up `tracing` based logging

### Capture: AX Watcher (`capture/ax_watcher.rs`)
- [ ] Get frontmost app PID via `NSWorkspace` (via objc crate)
- [ ] Create `AXUIElementCreateApplication(pid)` handle
- [ ] Read `kAXFocusedUIElementAttribute` → focused element
- [ ] Read `kAXSelectedTextAttribute` from focused element
- [ ] Recursive UI tree traversal: read `kAXChildrenAttribute` + `kAXValueAttribute`
- [ ] Filter out password fields (`kAXSecureTextField` role)
- [ ] Filter out empty / whitespace-only strings
- [ ] Emit structured `CaptureEvent { app, window_title, texts, timestamp }` on change
- [ ] Poll loop with configurable interval (default 1s)

### Config (`config/default.toml`)
- [ ] `capture.ax_enabled = true`
- [ ] `capture.screenshot_enabled = false`
- [ ] `capture.poll_interval_ms = 1000`
- [ ] `capture.excluded_apps = ["1Password", "Keychain"]`

### Testing v0.1
- [ ] Run daemon, switch between Chrome / Notes / Slack, verify text is captured
- [ ] Verify password fields are not captured
- [ ] Verify excluded apps produce no output

---

## v0.2 — Memory Pipeline

> Goal: chunk captured text, embed it locally, store in qdrant.

- [ ] `chunker.rs`: sliding window chunking (512 tokens, 64 overlap)
- [ ] `chunker.rs`: content hash dedup (skip already-seen chunks)
- [ ] `embedder.rs`: call Ollama `/api/embed` endpoint (model: `nomic-embed-text`)
- [ ] `store.rs`: qdrant client, create collection, upsert vectors with payload
- [ ] Payload schema: `{ text, app, window_title, timestamp, source: "ax" }`
- [ ] Docker compose for local qdrant

---

## v0.3 — Router Engine + CLI Hint

> Goal: given current context, retrieve relevant memories and decide whether to surface a hint.

- [ ] `context.rs`: detect current task context (active app + window title + focused text)
- [ ] `engine.rs`: embed current context, query qdrant top-k
- [ ] `engine.rs`: relevance threshold filter (skip if score < 0.75)
- [ ] `engine.rs`: hint decision logic (don't spam, cooldown per app)
- [ ] CLI output: print hint to stdout with source provenance
- [ ] `sbr ask "<query>"` manual query mode

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
- [ ] IPC between daemon and UI via local socket or Tauri commands
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
