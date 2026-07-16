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
/// Requires a secure context (`https://` or `localhost`).
#[wasm_bindgen(js_name = isClipboardAvailable)]
pub fn is_clipboard_available() -> bool {
    let window = match web_sys::window() {
        Some(w) => w,
        None => return false,
    };
    if !window.is_secure_context() {
        return false;
    }
    // `navigator.clipboard()` is always present in modern browsers when secure.
    true
}
