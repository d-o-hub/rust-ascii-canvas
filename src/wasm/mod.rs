//! WASM module - WebAssembly bindings for browser integration.

mod bindings;
mod events;
mod clipboard;

pub use bindings::AsciiEditor;
pub use events::EventResult;
pub use clipboard::copy_to_clipboard;
