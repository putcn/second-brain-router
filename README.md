# 🧠 Second Brain Router

> **The missing context layer between you and your work.**
> Not a chatbot. Not a search engine. A local-first AI daemon that silently captures what you see, reads, and discuss — and surfaces the right context at the right moment.

---

## Why This Exists

The biggest productivity killer isn't lack of intelligence — it's **context reconstruction overhead**.

Every day, knowledge workers spend hours re-answering:
- *Why was this written this way?*
- *Who made that decision, and when?*
- *What did we conclude in that meeting last week?*
- *Has this problem been solved before?*

Second Brain Router eliminates that. It runs locally, captures passively, and routes proactively.

---

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Your Daily Work                      │
│    (Browser · Slack · Docs · Email · Meetings)          │
└────────────────────┬────────────────────────────────────┘
                     │ passive capture (AX API + screenshot)
                     ▼
┌─────────────────────────────────────────────────────────┐
│           sbr-daemon  (Rust, runs locally)              │
│                                                         │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  │
│  │  ax_watcher  │  │   chunker    │  │   embedder   │  │
│  │  (AX API)    │→ │  + dedup     │→ │  (local LLM) │  │
│  │  screenshot  │  │  + filter    │  │              │  │
│  └──────────────┘  └──────────────┘  └──────┬───────┘  │
│                                             │           │
│  ┌──────────────────────────────────────────▼────────┐  │
│  │              Memory Store (ChromaDB / qdrant)     │  │
│  └──────────────────────────────────────────┬────────┘  │
│                                             │           │
│  ┌──────────────────────────────────────────▼────────┐  │
│  │         Router Engine (context match + ranking)   │  │
│  └──────────────────────────────────────────┬────────┘  │
└───────────────────────────────────────────  │ ──────────┘
                                              │ non-intrusive hint
                                              ▼
                                   ┌─────────────────┐
                                   │  Overlay / CLI  │
                                   │  (Tauri or tui) │
                                   └─────────────────┘
```

### Capture Strategy (by priority)

| Layer | Method | Coverage | Privacy Risk |
|-------|--------|----------|--------------|
| Focused element text | AX `kAXSelectedTextAttribute` | Selected text | 🟢 Low |
| Full window UI tree | AX recursive traverse | All visible text in App | 🟢 Low |
| Screenshot + Vision | `mss` + local vision model | Images, Canvas, Video | 🟡 Medium |
| Microphone | local Whisper (opt-in) | Meetings, voice | 🔴 High |

AX API covers ~80% of normal user scenarios (browser, docs, Slack, email). Vision model is the fallback for Canvas-based apps (Figma, YouTube, etc).

---

## Tech Stack

| Layer | Technology | Reason |
|-------|-----------|--------|
| Core daemon | **Rust** | Native macOS AX API via FFI, zero overhead, memory safe |
| macOS AX binding | `accessibility` + `core-foundation` crates | Clean FFI to AXUIElement |
| Screenshot | `xcap` crate | Cross-platform screen capture |
| Embedding | Ollama HTTP API (local) | No cloud, pluggable model |
| Vector store | `qdrant` (local docker) | Fast ANN search, rich filtering |
| Desktop UI | Tauri + React | Rust backend, lightweight overlay |
| Config | TOML | Simple, human-readable |

---

## Project Structure

```
second-brain-router/
├── crates/
│   ├── sbr-daemon/         # main capture + routing daemon (Rust)
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   ├── capture/
│   │   │   │   ├── ax_watcher.rs   # AX API: focused element + UI tree
│   │   │   │   └── screenshot.rs   # fallback: screen capture
│   │   │   ├── memory/
│   │   │   │   ├── chunker.rs      # text chunking + dedup
│   │   │   │   ├── embedder.rs     # calls local Ollama embed API
│   │   │   │   └── store.rs        # qdrant client
│   │   │   ├── router/
│   │   │   │   ├── context.rs      # detect current task context
│   │   │   │   └── engine.rs       # retrieval + ranking + hint decision
│   │   │   └── config.rs           # TOML config loader
│   │   └── Cargo.toml
│   └── sbr-ui/             # Tauri overlay app (future)
├── config/
│   └── default.toml        # default privacy + capture settings
├── TODOS.md
└── README.md
```

---

## Roadmap

| Version | Focus |
|---------|-------|
| **v0.1** | AX watcher: capture focused + full window text, print to stdout |
| **v0.2** | Chunker + dedup + local embedding via Ollama + qdrant store |
| **v0.3** | Router engine: context detection + top-k retrieval + CLI hint |
| **v0.4** | Screenshot fallback + local vision model (qwen-vl via Ollama) |
| **v0.5** | Tauri overlay UI: non-intrusive floating hint window |
| **v0.6** | Microphone capture + local Whisper transcription (opt-in) |

---

## Privacy Model

- **Everything is local.** No data ever leaves your machine.
- **Opt-in by layer.** AX text is on by default; screenshot and microphone are off by default.
- **Per-app exclusions.** You can blacklist specific apps (e.g. banking, password managers).
- **Zero telemetry.** The daemon has no network calls except to `localhost` (Ollama + qdrant).

---

## Philosophy

> "The goal is not to remember everything. The goal is to never waste time remembering things that don't require judgment."

This is not a replacement for thinking. It reduces **cognitive load from retrieval** so mental energy goes toward judgment, creation, and execution.

---

## License

MIT
