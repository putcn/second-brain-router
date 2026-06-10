# NOTES — Second Brain Router

Implementation learnings, gotchas, and decisions recorded as we go.

---

## macOS AX API in Rust

### Use `msg_send!` for `processIdentifier`, not method call

`objc2-app-kit` gates many `NSRunningApplication` methods behind individual feature flags.
`processIdentifier` is one of them. Enabling it causes feature conflicts with `NSWorkspace`.

**Solution**: use `objc2`'s `msg_send!` macro to call it directly via ObjC runtime:

```rust
use objc2::msg_send;
let pid: i32 = msg_send![&*active_app, processIdentifier];
```

This bypasses the feature gate entirely. `processIdentifier` is a stable ObjC method
on `NSRunningApplication` since macOS 10.6, so this is safe.

---

### `AXUIElementCreateSystemWide()` returns `-25204` for focused element

The system-wide AX element does **not** support `kAXFocusedUIElementAttribute` directly.
You must first get the frontmost app's PID, then create a per-app AX handle:

```rust
// Wrong: AXUIElementCreateSystemWide() → err -25204
// Correct:
let ax_app = AXUIElementCreateApplication(pid);
```

Error code reference:
| Code | Meaning |
|------|---------|
| `0` | Success |
| `-25201` | No value (kAXErrorNoValue) |
| `-25204` | Attribute unsupported (kAXErrorAttributeUnsupported) |
| `-25211` | API disabled — no Accessibility permission |
| `-25212` | Not implemented by this app |

---

### `AXIsProcessTrusted()` for permission check

Before running, verify Accessibility permission is granted:

```python
# Quick Python check during dev
from ApplicationServices import AXIsProcessTrusted
print(AXIsProcessTrusted())  # must be True
```

On macOS: **System Settings → Privacy & Security → Accessibility** → enable Terminal / your app.

Without this, all AX calls silently return `None` / error with no exception.

---

## Rust / Clippy / rustfmt Rules Learned

### `rustfmt` struct initializer formatting

`rustfmt` always expands struct literals with more than one field to multi-line,
regardless of `max_width`. Single-line struct init only works for unit structs or
structs with one field.

```rust
// Will be reformatted by rustfmt — don't write this:
AXWatcher { config, last_content_hash: None }

// Write this instead:
AXWatcher {
    config,
    last_content_hash: None,
}
```

### `rustfmt` function call line-breaking rules

- If the entire call fits within `max_width` (100), it stays on one line.
- If it doesn't fit, `rustfmt` breaks **at the assignment**, not by expanding args:

```rust
// rustfmt prefers this (args fit on one line after the =):
let chunks = chunk_text(&ctx.text, 512, 64, cfg.capture.min_text_length);

// If args don't fit, it breaks at the = and keeps args together:
let chunks =
    chunk_text(&ctx.text, 512, 64, cfg.capture.min_text_length);
```

- For chained `.await` into a `match`, `rustfmt` puts `.await` on the next line:

```rust
match router::query(s.client_ref(), vector, Some(&ctx.app_name))
    .await
{
    ...
}
```

### `clippy::match_like_matches_macro`

When a `match` expression only returns `true` or `false`, clippy requires `matches!`:

```rust
// clippy rejects this:
let should_hint = match (&last_hint_app, &last_hint_at) {
    (Some(app), Some(t)) if app == &ctx.app_name && t.elapsed().as_millis() < COOLDOWN => false,
    _ => true,
};

// Use this instead:
let should_hint = !matches!(
    (&last_hint_app, &last_hint_at),
    (Some(app), Some(t))
        if app == &ctx.app_name && t.elapsed().as_millis() < COOLDOWN
);
```

### `#[allow(dead_code)]` for planned-but-unused fields

With `-D warnings` in clippy, unused fields/functions are compile errors.
For fields pre-written for future versions, annotate individually:

```rust
/// Reserved for v0.5 UI provenance display
#[allow(dead_code)]
pub window_title: String,
```

For an entire module not yet wired up:

```rust
#![allow(dead_code)]  // at top of file, removed once module is used
```

### `qdrant_client::Qdrant` does not implement `Debug`

Any struct containing `Qdrant` cannot `#[derive(Debug)]`. Use `#[derive(Clone)]` only.

```rust
// Compile error:
#[derive(Debug, Clone)]
pub struct MemoryStore { client: Qdrant }

// Correct:
#[derive(Clone)]
pub struct MemoryStore { client: Qdrant }
```

### `qdrant_client` payload `Value::as_str()` returns `Option<&String>`, not `Option<&str>`

Using `.unwrap_or("literal")` after `.and_then(|v| v.as_str())` causes a type mismatch
because `&String != &str`. Use `.map_or()` instead:

```rust
// Type error:
payload.get("app_name").and_then(|v| v.as_str()).unwrap_or("unknown").to_string()

// Correct:
payload.get("app_name").and_then(|v| v.as_str()).map_or("unknown", |v| v).to_string()
```

---

## CI

### CI runs on `macos-latest`

AX API and `AppKit` are macOS-only frameworks. The CI workflow is pinned to
`macos-latest` runner. Linux/Windows builds are not supported until we abstract
the capture layer behind a trait.

---

## Qdrant

### Run locally via Docker

```bash
# Use docker-compose (preferred):
cd docker && docker compose up -d

# Or run directly:
docker run -p 6333:6333 -p 6334:6334 \
  -v $(pwd)/qdrant_storage:/qdrant/storage \
  qdrant/qdrant
```

REST API + dashboard at `http://localhost:6333/dashboard`. gRPC at `6334` (used by sbr-daemon).

### Collection setup

- Collection name: `sbr_memory`
- Vector dimension: `768` (nomic-embed-text output)
- Distance metric: `Cosine`
- `ensure_collection()` is idempotent — safe to call on every startup.

---

## Ollama

### Embedding model

```bash
brew install ollama
ollama serve &
ollama pull nomic-embed-text
```

Call via REST:
```bash
curl http://localhost:11434/api/embed \
  -d '{"model": "nomic-embed-text", "input": "your text here"}'
```

Returns `{ "embeddings": [[...768 floats...]] }`. The outer array supports batch input;
we currently call one text at a time.

---

## Python POC (pre-Rust validation)

Before writing Rust, we validated the AX API approach in Python using
`pyobjc-framework-ApplicationServices`. Key findings:

- Python 3.9 + `pyobjc` 12.0 fails to compile due to Clang strict mode on newer Xcode
- Fix: upgrade to Python 3.12 via `pyenv install 3.12`
- `NSWorkspace.sharedWorkspace().activeApplication()` returns the frontmost app dict
- `kAXSelectedTextAttribute` returns `None` in Terminal (expected — Terminal is read-only)
- `AXIsProcessTrusted()` must return `True` before any AX call works
