//! Core module - pure Rust logic with no WASM/web dependencies.
//!
//! This module contains:
//! - Grid model for ASCII canvas
//! - Cell representation
//! - Drawing tools
//! - Command pattern for undo/redo
//! - History management
//! - ASCII export

pub mod cell;
pub mod grid;
pub mod tools;
pub mod commands;
pub mod history;
pub mod selection;
pub mod ascii_export;

// Re-exports
pub use cell::{Cell, CellStyle};
pub use grid::Grid;
pub use tools::{Tool, ToolId, ToolResult, BorderStyle, DrawOp};
pub use commands::Command;
pub use history::History;
pub use selection::Selection;
pub use ascii_export::{export_grid, ExportOptions};

use serde::{Deserialize, Serialize};

/// Editor state container.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EditorState {
    /// The ASCII grid
    pub grid: Grid,
    /// Currently selected tool
    pub tool: ToolId,
    /// Current border style for shapes
    pub border_style: BorderStyle,
}

impl EditorState {
    /// Create a new editor state with the given grid dimensions.
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            grid: Grid::new(width, height),
            tool: ToolId::default(),
            border_style: BorderStyle::default(),
        }
    }

    /// Create with custom grid.
    pub fn with_grid(grid: Grid) -> Self {
        Self {
            grid,
            tool: ToolId::default(),
            border_style: BorderStyle::default(),
        }
    }

    /// Get grid width.
    pub fn width(&self) -> usize {
        self.grid.width()
    }

    /// Get grid height.
    pub fn height(&self) -> usize {
        self.grid.height()
    }

    /// Set the current tool.
    pub fn set_tool(&mut self, tool: ToolId) {
        self.tool = tool;
    }

    /// Set the border style.
    pub fn set_border_style(&mut self, style: BorderStyle) {
        self.border_style = style;
    }

    /// Clear the grid.
    pub fn clear(&mut self) {
        self.grid.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_editor_state() {
        let state = EditorState::new(80, 40);
        assert_eq!(state.width(), 80);
        assert_eq!(state.height(), 40);
        assert_eq!(state.tool, ToolId::Rectangle);
    }

    #[test]
    fn test_editor_set_tool() {
        let mut state = EditorState::new(80, 40);
        state.set_tool(ToolId::Line);
        assert_eq!(state.tool, ToolId::Line);
    }
}
