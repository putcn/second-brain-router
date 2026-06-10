# TODOS — Second Brain Router

Progress tracker. Updated as work proceeds.

---

## v0.1 — AX Capture Daemon ✅ DONE

> Goal: a Rust binary that runs on macOS, captures text from the active window via AX API, and prints structured output to stdout.

- [x] Initialize Cargo workspace (`crates/sbr-daemon`)
- [x] Add dependencies: `core-foundation`, `tokio`, `serde`, `toml`, `tracing`, `objc2`
- [x] Set up `config.rs` with TOML loader and default config
- [x] Set up `tracing` based logging
- [x] Get frontmost app PID via `NSWorkspace` + `msg_send!`
- [x] Recursive AX UI tree traversal, filter password fields + empty strings
- [x] Emit `CaptureEvent`, poll loop with configurable interval
- [x] Content hash dedup
- [x] GitHub Actions CI on `macos-latest`

---

## v0.2 — Memory Pipeline ✅ DONE

> Goal: chunk captured text, embed it locally via Ollama, store vectors in qdrant.

- [x] `chunker.rs`: sliding window chunking + content hash dedup
- [x] `embedder.rs`: async Ollama `/api/embed` client
- [x] `store.rs`: qdrant client — collection + upsert + dedup check
- [x] Wire `ax_watcher` → `chunker` → `embedder` → `store` in `main.rs`
- [x] Docker Compose for local qdrant

---

## v0.3 — Router Engine + CLI Hint ✅ DONE

> Goal: given current context, retrieve relevant memories and surface a hint.

- [x] `context.rs`: current task context from `CaptureEvent`
- [x] `router.rs`: embed context, query qdrant top-5, threshold filter (>= 0.75)
- [x] Hint cooldown: 30s per app
- [x] `sbr-daemon ask "<query>"` manual query subcommand
- [x] `print_hints`: stdout output with app, timestamp, score, text snippet

---

## v0.4 — Screenshot Fallback 🔄 IN PROGRESS

> Goal: for apps where AX returns nothing (Figma, YouTube), fall back to screenshot + vision model.

- [x] `screenshot.rs`: `ax_text_too_sparse()` detection (< 50 chars)
- [x] `screenshot.rs`: capture primary display via `xcap`
- [x] `screenshot.rs`: send PNG to Ollama vision model (`qwen2.5vl`) via `/api/generate`
- [x] Wire into `main.rs`: fallback only when `screenshot_enabled = true` AND ax sparse
- [x] Payload `source` field set to `"screenshot"` vs `"ax"` accordingly
- [ ] Manual test: open Figma/YouTube, enable screenshot mode, verify text extracted
- [ ] Config docs: document `capture.screenshot_enabled = true` in README

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
