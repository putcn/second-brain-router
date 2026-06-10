# 🧠 Second Brain Router

> **The missing context layer between you and your work.**  
> Not a chatbot. Not a search engine. A personal AI system that knows *when* to surface the right memory, decision, or knowledge — and when to stay silent.

---

## Why This Exists

The biggest productivity killer isn't lack of intelligence — it's **context reconstruction overhead**.

Every day, knowledge workers spend hours answering questions like:
- *Why was this code written this way?*
- *Who made that decision, and when?*
- *What did we conclude in that meeting last week?*
- *Has this bug been seen before?*

Second Brain Router eliminates that. It acts as a **persistent, queryable, privacy-first context layer** that sits between your brain and your tools.

---

## Core Concept

```
┌─────────────────────────────────────────────────────────┐
│                    Your Daily Work                      │
│   (Code Editor · Meetings · Slack · Browser · Docs)    │
└────────────────────┬────────────────────────────────────┘
                     │ continuous passive capture
                     ▼
┌─────────────────────────────────────────────────────────┐
│              Second Brain Router (local)                │
│                                                         │
│  ┌─────────────┐  ┌──────────────┐  ┌───────────────┐  │
│  │  Ingestion  │  │   Memory     │  │    Router     │  │
│  │  Pipeline   │→ │   Store      │→ │   Engine      │  │
│  │             │  │  (semantic)  │  │               │  │
│  └─────────────┘  └──────────────┘  └───────┬───────┘  │
└──────────────────────────────────────────────┼──────────┘
                                               │ push at right moment
                                               ▼
                                    ┌──────────────────┐
                                    │  Contextual Hint │
                                    │  (non-intrusive) │
                                    └──────────────────┘
```

The Router's prime directive: **know when to speak, and when to shut up.**

---

## What It Does

### 🔍 Passive Context Capture
- Screen content (OCR + vision model, opt-in)
- Voice / meeting transcription (local Whisper)
- Files, code, documents via file system watcher
- Browser activity via lightweight extension
- Git history, PR descriptions, commit messages

### 🗂️ Semantic Memory Store
- All captured content compressed into **traceable semantic chunks**
- Organized by: person, project, decision, timeline
- Local vector store (no cloud, no leaks)
- Full provenance: every memory links back to its source

### 🧭 Routing Engine
- Detects your current task context (what file, what window, what conversation)
- Retrieves top-k relevant memory chunks silently in background
- Pushes a **non-intrusive hint** only when confidence is high enough
- Three output modes:
  - 💡 **Evidence only** — "This was changed in PR #142 two weeks ago"
  - 🔔 **Soft alert** — "This decision has a known tradeoff, want context?"
  - 🤐 **Silent** — stores and learns, says nothing

### 🔒 Privacy First
- Everything runs **100% locally** by default (Ollama + local Whisper)
- Zero telemetry
- Per-project data isolation
- Explicit opt-in for each capture source

---

## Tech Stack (Planned)

| Layer | Technology |
|-------|-----------|
| Local LLM inference | Ollama (Llama 3 / Qwen2.5) |
| Speech-to-text | faster-whisper (local) |
| Embedding & retrieval | sentence-transformers + ChromaDB |
| File watching | watchdog (Python) |
| Screen capture | mss + vision model (opt-in) |
| Desktop overlay UI | Tauri (Rust + React) |
| Orchestration | Python + asyncio |
| Config & privacy rules | YAML / TOML |

---

## MVP Scope (v0.1)

The first version focuses on **code + Git context only** — the highest-signal, lowest-privacy-risk source.

- [ ] Git repo watcher: index commits, PRs, and inline code comments
- [ ] Local embedding pipeline with ChromaDB
- [ ] CLI query interface: `sbr ask "why was this function refactored?"`
- [ ] VS Code extension: show relevant context in sidebar when a file opens
- [ ] Basic provenance: every result shows source file, commit, and date

**Out of scope for v0.1:** screen capture, voice, browser extension, real-time push

---

## Project Structure (Planned)

```
second-brain-router/
├── ingestion/          # capture pipelines (git, file, voice, screen)
├── memory/             # embedding, chunking, vector store
├── router/             # context detection + retrieval logic
├── ui/                 # Tauri desktop overlay
├── extensions/         # VS Code, browser
├── config/             # privacy rules, source toggles
└── cli/                # sbr CLI tool
```

---

## Roadmap

| Version | Focus |
|---------|-------|
| v0.1 | Git + code context, CLI + VS Code sidebar |
| v0.2 | File & document ingestion, project timeline view |
| v0.3 | Local meeting transcription + speaker-tagged memory |
| v0.4 | Real-time routing engine with non-intrusive overlay |
| v0.5 | Screen capture (opt-in), full context graph UI |

---

## Philosophy

This is not a replacement for thinking. It's a reduction of **cognitive load from retrieval** so that more mental energy goes toward **judgment, creation, and execution**.

> "The goal is not to remember everything. The goal is to never waste time remembering things that don't require judgment."

---

## Contributing

This project is in early design phase. Issues and RFC-style discussions are very welcome.

If you're interested in contributing to the ingestion pipeline, routing logic, or UI layer — open an issue to discuss.

---

## License

MIT
