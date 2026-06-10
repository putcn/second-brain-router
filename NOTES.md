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

---

### CI runs on `macos-latest`

AX API and `AppKit` are macOS-only frameworks. The CI workflow is pinned to
`macos-latest` runner. Linux/Windows builds are not supported until we abstract
the capture layer behind a trait.

---

### `#[allow(dead_code)]` for planned-but-unused code

With `-D warnings` in clippy, unused fields/functions are compile errors.
For code that is intentionally pre-written for future versions, annotate with:

```rust
/// Reserved for v0.2 memory pipeline
#[allow(dead_code)]
pub window_title: String,
```

For an entire module of future code (e.g. `chunker.rs` before v0.2 wires it up):

```rust
#![allow(dead_code)]  // at top of file
```

Remove these annotations once the code is actively used.

---

## Qdrant (coming in v0.2)

### Run locally via Docker

```bash
docker run -p 6333:6333 -p 6334:6334 \
  -v $(pwd)/qdrant_storage:/qdrant/storage \
  qdrant/qdrant
```

REST API at `http://localhost:6333`, gRPC at `6334`.
Dashboard at `http://localhost:6333/dashboard`.

---

## Ollama (coming in v0.2)

### Embedding model

```bash
ollama pull nomic-embed-text
```

Call via REST:
```bash
curl http://localhost:11434/api/embed \
  -d '{"model": "nomic-embed-text", "input": "your text here"}'
```

Returns a 768-dim float vector. Use this as the qdrant vector dimension.

---

## Python POC (pre-Rust validation)

Before writing Rust, we validated the AX API approach in Python using
`pyobjc-framework-ApplicationServices`. Key findings:

- Python 3.9 + `pyobjc` 12.0 fails to compile due to Clang strict mode on newer Xcode
- Fix: upgrade to Python 3.12 via `pyenv install 3.12`
- `NSWorkspace.sharedWorkspace().activeApplication()` returns the frontmost app dict
- `kAXSelectedTextAttribute` returns `None` in Terminal (expected — Terminal is read-only)
- `AXIsProcessTrusted()` must return `True` before any AX call works
