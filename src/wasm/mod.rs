//! WASM module - WebAssembly bindings for browser integration.

mod bindings;
mod clipboard;
mod clipboard_helpers;
mod document_api;
mod event_handlers;
mod helpers;
mod interaction;
mod layer_api;
mod pixel_export;
mod render_api;
mod render_bridge;
mod selection;
mod tool_manager;

#[cfg(test)]
mod helpers_tests;
#[cfg(all(test, target_arch = "wasm32"))]
mod input_regression_tests;
#[cfg(test)]
mod pixel_export_tests;
#[cfg(test)]
mod safety_regression_tests;

pub use bindings::AsciiEditor;
pub use clipboard::copy_to_clipboard;
pub use render_bridge::EditorEventResult;
