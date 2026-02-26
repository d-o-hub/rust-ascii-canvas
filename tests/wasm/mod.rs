//! WASM tests entry point.

#![cfg(target_arch = "wasm32")]

use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

mod editor;

// Basic WASM initialization test
#[wasm_bindgen_test]
fn test_wasm_init() {
    // This test verifies the WASM module loads correctly
    assert!(true);
}
