//! WASM module - WebAssembly bindings for browser integration.

mod bindings;
mod clipboard;
mod editor_state;
mod events;
mod render_bridge;
mod tool_manager;

pub use bindings::AsciiEditor;
pub use clipboard::copy_to_clipboard;
pub use editor_state::{generate_move_preview_ops, MoveManager, StateHelpers};
pub use render_bridge::EditorEventResult;
