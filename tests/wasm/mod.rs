//! JS/WASM boundary tests. No DOM APIs: wasm-bindgen's default Node runner.
//! Browser behavior is covered separately by Playwright.

#![cfg(target_arch = "wasm32")]

mod editor;
