use crate::capture::ax_watcher::CaptureEvent;

/// A snapshot of what the user is currently doing.
#[derive(Debug, Clone)]
pub struct Context {
    pub app_name: String,
    /// Reserved for v0.5 UI provenance display.
    #[allow(dead_code)]
    pub window_title: String,
    /// Concatenated visible text from the active window (used for embedding).
    pub text: String,
}

impl Context {
    /// Build a `Context` from the latest `CaptureEvent`.
    pub fn from_event(event: &CaptureEvent) -> Self {
        Context {
            app_name: event.app_name.clone(),
            window_title: event.window_title.clone(),
            text: event.texts.join(" "),
        }
    }

    /// Returns true if the context has enough text to be worth querying.
    pub fn is_meaningful(&self, min_len: usize) -> bool {
        self.text.trim().len() >= min_len
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_context(text: &str) -> Context {
        Context {
            app_name: "TestApp".into(),
            window_title: "Test Window".into(),
            text: text.into(),
        }
    }

    #[test]
    fn test_is_meaningful_above_threshold() {
        let ctx = make_context("hello world this is a test");
        assert!(ctx.is_meaningful(10));
    }

    #[test]
    fn test_is_meaningful_below_threshold() {
        let ctx = make_context("hi");
        assert!(!ctx.is_meaningful(10));
    }

    #[test]
    fn test_is_meaningful_whitespace_only() {
        let ctx = make_context("     ");
        assert!(!ctx.is_meaningful(1));
    }
}
