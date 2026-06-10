use chrono::{DateTime, Utc};
use core_foundation::{
    base::{CFType, TCFType},
    string::{CFString, CFStringRef},
    array::{CFArray, CFArrayRef},
};
use std::collections::HashSet;
use tracing::{debug, warn};

use crate::config::Config;

// AX API constants (values from AXAttributeConstants.h)
const K_AX_FOCUSED_UI_ELEMENT_ATTRIBUTE: &str = "AXFocusedUIElement";
const K_AX_CHILDREN_ATTRIBUTE: &str = "AXChildren";
const K_AX_VALUE_ATTRIBUTE: &str = "AXValue";
const K_AX_SELECTED_TEXT_ATTRIBUTE: &str = "AXSelectedText";
const K_AX_ROLE_ATTRIBUTE: &str = "AXRole";
const K_AX_WINDOWS_ATTRIBUTE: &str = "AXWindows";
const K_AX_TITLE_ATTRIBUTE: &str = "AXTitle";
const K_AX_ROLE_SECURE_TEXT_FIELD: &str = "AXSecureTextField";

#[derive(Debug, Clone)]
pub struct CaptureEvent {
    pub app_name: String,
    pub window_title: String,
    pub texts: Vec<String>,
    pub selected_text: Option<String>,
    pub timestamp: DateTime<Utc>,
}

pub struct AXWatcher {
    config: Config,
    last_content_hash: Option<String>,
}

impl AXWatcher {
    pub fn new(config: Config) -> Self {
        AXWatcher {
            config,
            last_content_hash: None,
        }
    }

    pub async fn poll(&mut self) -> Option<CaptureEvent> {
        if !self.config.capture.ax_enabled {
            return None;
        }

        // Get frontmost app info via NSWorkspace (unsafe FFI)
        let (pid, app_name) = unsafe { get_frontmost_app()? };

        // Check excluded apps
        if self.config.capture.excluded_apps
            .iter()
            .any(|ex| app_name.contains(ex.as_str()))
        {
            debug!("Skipping excluded app: {}", app_name);
            return None;
        }

        // Capture via AX API
        let event = unsafe {
            capture_ax_content(pid, app_name, &self.config)
        };

        if let Some(ref ev) = event {
            let hash = compute_hash(&ev.texts);
            if Some(&hash) == self.last_content_hash.as_ref() {
                debug!("Content unchanged, skipping");
                return None;
            }
            self.last_content_hash = Some(hash);
        }

        event
    }
}

fn compute_hash(texts: &[String]) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    for t in texts {
        hasher.update(t.as_bytes());
    }
    hex::encode(hasher.finalize())
}

// ─── Unsafe FFI to macOS AX API ──────────────────────────────────────────────
//
// These functions call into macOS ApplicationServices framework directly.
// TODO: replace with safe wrappers from `accessibility` crate once v0.1 is validated.

#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXUIElementCreateApplication(pid: i32) -> *mut std::ffi::c_void;
    fn AXUIElementCopyAttributeValue(
        element: *mut std::ffi::c_void,
        attribute: CFStringRef,
        value: *mut *mut std::ffi::c_void,
    ) -> i32;
}

#[link(name = "AppKit", kind = "framework")]
extern "C" {
    // accessed via NSWorkspace Objective-C API, bridged through objc2 crate
}

unsafe fn get_frontmost_app() -> Option<(i32, String)> {
    use objc2_app_kit::NSWorkspace;
    use objc2::rc::Retained;

    let workspace = NSWorkspace::sharedWorkspace();
    let active_app = workspace.frontmostApplication()?;
    let pid = active_app.processIdentifier();
    let name = active_app
        .localizedName()
        .map(|n| n.to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    Some((pid, name))
}

unsafe fn capture_ax_content(
    pid: i32,
    app_name: String,
    config: &Config,
) -> Option<CaptureEvent> {
    let ax_app = AXUIElementCreateApplication(pid);
    if ax_app.is_null() {
        warn!("AXUIElementCreateApplication returned null for pid={}", pid);
        return None;
    }

    // Get window title
    let window_title = read_ax_string(ax_app, K_AX_WINDOWS_ATTRIBUTE)
        .unwrap_or_default();

    // Get selected text from focused element
    let mut focused_ptr: *mut std::ffi::c_void = std::ptr::null_mut();
    let attr = CFString::new(K_AX_FOCUSED_UI_ELEMENT_ATTRIBUTE);
    AXUIElementCopyAttributeValue(ax_app, attr.as_concrete_TypeRef(), &mut focused_ptr);

    let selected_text = if !focused_ptr.is_null() {
        read_ax_string(focused_ptr, K_AX_SELECTED_TEXT_ATTRIBUTE)
    } else {
        None
    };

    // Traverse UI tree for all visible text
    let mut texts: Vec<String> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    traverse_ax_tree(
        ax_app,
        config.capture.max_tree_depth,
        config.capture.min_text_length,
        &mut texts,
        &mut seen,
    );

    if texts.is_empty() && selected_text.is_none() {
        return None;
    }

    Some(CaptureEvent {
        app_name,
        window_title,
        texts,
        selected_text,
        timestamp: Utc::now(),
    })
}

unsafe fn read_ax_string(
    element: *mut std::ffi::c_void,
    attribute: &str,
) -> Option<String> {
    let mut value_ptr: *mut std::ffi::c_void = std::ptr::null_mut();
    let attr = CFString::new(attribute);
    let err = AXUIElementCopyAttributeValue(
        element,
        attr.as_concrete_TypeRef(),
        &mut value_ptr,
    );
    if err != 0 || value_ptr.is_null() {
        return None;
    }
    // Try to cast to CFString
    let cf_type = CFType::wrap_under_create_rule(value_ptr as _);
    cf_type.downcast::<CFString>().map(|s| s.to_string())
}

unsafe fn traverse_ax_tree(
    element: *mut std::ffi::c_void,
    depth: usize,
    min_len: usize,
    out: &mut Vec<String>,
    seen: &mut HashSet<String>,
) {
    if depth == 0 {
        return;
    }

    // Skip secure text fields (password inputs)
    if let Some(role) = read_ax_string(element, K_AX_ROLE_ATTRIBUTE) {
        if role == K_AX_ROLE_SECURE_TEXT_FIELD {
            return;
        }
    }

    // Read text value of current node
    if let Some(text) = read_ax_string(element, K_AX_VALUE_ATTRIBUTE) {
        let text = text.trim().to_string();
        if text.len() >= min_len && !seen.contains(&text) {
            seen.insert(text.clone());
            out.push(text);
        }
    }

    // Recurse into children
    let mut children_ptr: *mut std::ffi::c_void = std::ptr::null_mut();
    let attr = CFString::new(K_AX_CHILDREN_ATTRIBUTE);
    let err = AXUIElementCopyAttributeValue(element, attr.as_concrete_TypeRef(), &mut children_ptr);
    if err != 0 || children_ptr.is_null() {
        return;
    }

    // Cast to CFArray and iterate
    let array = CFArray::<CFType>::wrap_under_create_rule(children_ptr as _);
    for child in array.iter() {
        let child_ptr = child.as_CFTypeRef() as *mut std::ffi::c_void;
        traverse_ax_tree(child_ptr, depth - 1, min_len, out, seen);
    }
}
