//! ASCII Canvas Editor
//!
//! A fast, browser-based ASCII diagram editor built with Rust and WebAssembly.
//!
//! # Features
//!
//! - Multiple drawing tools (Rectangle, Line, Arrow, Diamond, Text, Freehand, Select, Eraser)
//! - Multiple border styles (Single, Double, Heavy, Rounded, ASCII, Dotted)
//! - Full undo/redo support
//! - Zoom and pan navigation
//! - Copy ASCII output to clipboard
//! - Dark theme inspired by Figma
//! - 60 FPS rendering with dirty-rect optimization
//!
//! # Architecture
//!
//! The editor is organized into several layers:
//!
//! - **Core**: Pure Rust logic for grid, tools, commands, and history
//! - **Render**: Canvas rendering with dirty-rect optimization
//! - **WASM**: WebAssembly bindings for JavaScript interop
//! - **UI**: Toolbar, shortcuts, and theming
//!
//! # Example
//!
//! ```rust
//! use ascii_canvas::{EditorState, Grid, ToolId};
//!
//! // Create a new grid
//! let mut grid = Grid::new(80, 40);
//!
//! // Create editor state
//! let mut state = EditorState::new(80, 40);
//! state.set_tool(ToolId::Rectangle);
//! ```

#![allow(dead_code)]
#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod core;
pub mod render;
pub mod ui;
pub mod utils;
pub mod wasm;

// Re-exports for convenience
pub use core::{
    export_grid, BorderStyle, Cell, CellStyle, Command, DrawOp, EditorState, ExportOptions, Grid,
    History, Selection, Tool, ToolId, ToolResult,
};
pub use render::{CanvasRenderer, DirtyRect, FontMetrics};
pub use ui::{ShortcutManager, Theme, ToolbarConfig};
pub use wasm::AsciiEditor;

/// Current version of the library.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Initialize the WASM module.
/// Call this once when the application starts.
pub fn init() {
    // Set up panic hook for better error messages in browser
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();
}

// Internal helper for logging
#[cfg(target_arch = "wasm32")]
mod console {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = console)]
        pub fn log(s: &str);
    }

    /// Log a debug message to the browser console.
    pub fn debug(s: &str) {
        log(&format!("[DEBUG] {}", s));
    }

    /// Log an info message to the browser console.
    pub fn info(s: &str) {
        log(&format!("[INFO] {}", s));
    }

    /// Log a warning message to the browser console.
    pub fn warn(s: &str) {
        log(&format!("[WARN] {}", s));
    }

    /// Log an error message to the browser console.
    pub fn error(s: &str) {
        log(&format!("[ERROR] {}", s));
    }
}

#[cfg(target_arch = "wasm32")]
pub use console::{debug, error, info, warn};

#[cfg(not(target_arch = "wasm32"))]
mod console {
    /// Log a debug message.
    pub fn debug(s: &str) {
        println!("[DEBUG] {}", s);
    }
    /// Log an info message.
    pub fn info(s: &str) {
        println!("[INFO] {}", s);
    }
    /// Log a warning message.
    pub fn warn(s: &str) {
        println!("[WARN] {}", s);
    }
    /// Log an error message.
    pub fn error(s: &str) {
        eprintln!("[ERROR] {}", s);
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub use console::{debug, error, info, warn};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn test_re_exports() {
        let grid = Grid::new(10, 10);
        assert_eq!(grid.width(), 10);

        let state = EditorState::new(10, 10);
        assert_eq!(state.tool, ToolId::Rectangle);
    }
}
