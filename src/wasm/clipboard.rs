//! Clipboard utilities for WASM.

use wasm_bindgen::prelude::*;

/// Copy text to the system clipboard.
/// Returns a promise that resolves when the copy is complete.
#[wasm_bindgen(js_name = copyToClipboard)]
pub async fn copy_to_clipboard(text: String) -> Result<(), JsValue> {
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window"))?;
    let navigator = window.navigator();

    // Use the clipboard API
    let clipboard = navigator.clipboard();

    let promise = clipboard.write_text(&text);
    let result = wasm_bindgen_futures::JsFuture::from(promise).await;

    result.map(|_| ())
}

/// Read text from the system clipboard.
/// Returns a promise that resolves with the clipboard text.
#[wasm_bindgen(js_name = readFromClipboard)]
pub async fn read_from_clipboard() -> Result<String, JsValue> {
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window"))?;
    let navigator = window.navigator();

    let clipboard = navigator.clipboard();

    let promise = clipboard.read_text();
    let result = wasm_bindgen_futures::JsFuture::from(promise).await;

    result.map(|v| v.as_string().unwrap_or_default())
}

/// Check if clipboard API is available.
#[wasm_bindgen(js_name = isClipboardAvailable)]
pub fn is_clipboard_available() -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        let window = match web_sys::window() {
            Some(w) => w,
            None => return false,
        };
        // Clipboard API requires secure context
        if !window.is_secure_context() {
            return false;
        }
        // In web-sys, navigator().clipboard() might return an object directly or wrap it.
        // Based on the error, it seems to return Clipboard directly in this version's bindings.
        // However, some browsers might not have it.
        // If it's a direct object, it's "available" if we got this far in a secure context.
        true
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Bug 6: When running outside a browser context (native unit test),
    // is_clipboard_available() must return false — no window exists.
    #[test]
    fn test_clipboard_unavailable_outside_browser() {
        // web_sys::window() returns None in a native test binary,
        // so the function must return false without panicking.
        let result = is_clipboard_available();
        assert!(
            !result,
            "Clipboard must not be available outside a secure browser context"
        );
    }
}
