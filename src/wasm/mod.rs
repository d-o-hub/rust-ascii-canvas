//! WASM module - WebAssembly bindings for browser integration.

mod bindings;
mod clipboard;
mod event_handlers;
mod helpers;
mod render_api;
mod render_bridge;
mod selection;
mod tool_manager;

pub use bindings::AsciiEditor;
pub use clipboard::copy_to_clipboard;
pub use render_bridge::EditorEventResult;
