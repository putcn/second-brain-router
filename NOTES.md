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

### `rustfmt` formatting rules — summary

These rules have been verified against actual CI failures.

**Function signatures**: if the entire signature fits in `max_width` (100), keep it single-line.
If not, each parameter goes on its own line.

```rust
// single-line when it fits:
pub async fn capture_and_extract(ollama_url: &str, model: &str) -> Result<String>

// multi-line when it doesn’t:
async fn extract_text_from_image(
    ollama_url: &str,
    model: &str,
    b64_image: String,
) -> Result<String>
```

**Macro calls (`info!`, `warn!`, etc.)**: if the format string + args exceed `max_width`,
`rustfmt` always expands to multi-line, even if you wrote it single-line.

```rust
// rustfmt will expand this if > 100 chars:
info!("screenshot captured ({} bytes), sending to vision model", png_bytes.len());

// → becomes:
info!(
    "screenshot captured ({} bytes), sending to vision model",
    png_bytes.len()
);
```

**Method chains with `.await`**: `rustfmt` splits each step to its own line.

```rust
// rustfmt produces:
let parsed: VisionResponse = resp
    .json()
    .await
    .context("failed to parse vision response")?;
```

**Two-arg method calls**: if both args fit on one line, `rustfmt` keeps them there,
even if you wrote them multi-line.

```rust
// rustfmt collapses to single line if it fits:
image.write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)
```

**Assignment line-break**: when the RHS doesn’t fit on one line with the `let`, `rustfmt`
breaks after `=` and keeps the RHS together:

```rust
let chunks =
    chunk_text(&ctx.text, 512, 64, cfg.capture.min_text_length);
```

**Chained `.await` into `match`**: `.await` goes on the line *after* the call, indented:

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
// clippy rejects this — use matches! instead:
let should_hint = !matches!(
    (&last_hint_app, &last_hint_at),
    (Some(app), Some(t))
        if app == &ctx.app_name && t.elapsed().as_millis() < COOLDOWN
);
```

### `#[allow(dead_code)]` for planned-but-unused fields

With `-D warnings` in clippy, unused fields/functions are compile errors.
Annotate individually with a version comment:

```rust
/// Reserved for v0.5 UI provenance display
#[allow(dead_code)]
pub window_title: String,
```

For an entire module not yet wired up: `#![allow(dead_code)]` at top of file.

### `qdrant_client::Qdrant` does not implement `Debug`

Any struct containing `Qdrant` cannot `#[derive(Debug)]`. Use `#[derive(Clone)]` only.

### `qdrant_client` payload `Value::as_str()` returns `Option<&String>`, not `Option<&str>`

```rust
// Type error — &String != &str:
.unwrap_or("unknown")

// Correct:
.map_or("unknown", |v| v)
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
cd docker && docker compose up -d
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
ollama pull nomic-embed-text   # for embedder.rs
ollama pull qwen2.5vl          # for screenshot fallback (v0.4)
```

Embed call:
```bash
curl http://localhost:11434/api/embed \
  -d '{"model": "nomic-embed-text", "input": "your text here"}'
# returns: { "embeddings": [[...768 floats...]] }
```

Vision call (screenshot fallback):
```bash
curl http://localhost:11434/api/generate \
  -d '{"model": "qwen2.5vl", "prompt": "Extract text", "images": ["<base64>"], "stream": false}'
# returns: { "response": "extracted text..." }
```

---

## Python POC (pre-Rust validation)

Before writing Rust, we validated the AX API approach in Python using
`pyobjc-framework-ApplicationServices`. Key findings:

- Python 3.9 + `pyobjc` 12.0 fails to compile due to Clang strict mode on newer Xcode
- Fix: upgrade to Python 3.12 via `pyenv install 3.12`
- `NSWorkspace.sharedWorkspace().activeApplication()` returns the frontmost app dict
- `kAXSelectedTextAttribute` returns `None` in Terminal (expected — Terminal is read-only)
- `AXIsProcessTrusted()` must return `True` before any AX call works
