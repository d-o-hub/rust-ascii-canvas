//! Select tool - selects and moves regions of the canvas.

use super::{clamp_to_grid, Tool, ToolContext, ToolId, ToolResult};
use crate::core::cell::Cell;
use crate::core::selection::Selection;
use smallvec::SmallVec;
use std::any::Any;

/// Select and move tool.
pub struct SelectTool {
    /// Current selection
    selection: Option<Selection>,
    /// Whether we're dragging to select
    selecting: bool,
    /// Whether we're moving a selection
    moving: bool,
    /// Start of drag operation
    drag_start: Option<(i32, i32)>,
    /// Offset from selection corner when moving
    move_offset: Option<(i32, i32)>,
    /// Original content before move (for undo)
    original_content: SmallVec<[Cell; 256]>,
    /// Original bounds (for clearing during move)
    original_bounds: Option<(i32, i32, i32, i32)>,
}

impl Default for SelectTool {
    fn default() -> Self {
        Self {
            selection: None,
            selecting: false,
            moving: false,
            drag_start: None,
            move_offset: None,
            original_content: SmallVec::new(),
            original_bounds: None,
        }
    }
}

impl SelectTool {
    /// Create a new select tool.
    pub fn new() -> Self {
        Self::default()
    }

    /// Get current selection bounds ( clones the selection).
    pub fn get_selection(&self) -> Option<Selection> {
        self.selection.clone()
    }

    /// Clear the current selection.
    pub fn clear_selection(&mut self) {
        self.selection = None;
        self.selecting = false;
        self.moving = false;
        self.drag_start = None;
        self.move_offset = None;
        self.original_content.clear();
        self.original_bounds = None;
    }

    /// Check if point is within current selection.
    fn point_in_selection(&self, x: i32, y: i32) -> bool {
        if let Some(ref sel) = self.selection {
            sel.contains(x, y)
        } else {
            false
        }
    }

    /// Check if we should start moving (clicked inside selection).
    fn should_start_move(&self, x: i32, y: i32) -> bool {
        self.selection.is_some() && self.point_in_selection(x, y)
    }
}

impl Tool for SelectTool {
    fn id(&self) -> ToolId {
        ToolId::Select
    }

    fn get_selection(&self) -> Option<Selection> {
        self.selection.clone()
    }

    fn on_pointer_down(&mut self, x: i32, y: i32, _ctx: &ToolContext) -> ToolResult {
        if self.should_start_move(x, y) {
            self.moving = true;
            self.move_offset = self.selection.as_ref().map(|sel| (x - sel.x1, y - sel.y1));
        } else {
            // Start new selection
            self.selecting = true;
            self.selection = None;
            self.drag_start = Some((x, y));
        }

        ToolResult::new()
    }

    fn on_pointer_move(&mut self, x: i32, y: i32, ctx: &ToolContext) -> ToolResult {
        let (x, y) = clamp_to_grid(x, y, ctx.grid_width, ctx.grid_height);

        if self.selecting {
            if let Some((sx, sy)) = self.drag_start {
                self.selection = Some(Selection::new(sx, sy, x, y));
            }
        }

        // Moving is handled by the editor state
        ToolResult::new()
    }

    fn on_pointer_up(&mut self, x: i32, y: i32, ctx: &ToolContext) -> ToolResult {
        let (x, y) = clamp_to_grid(x, y, ctx.grid_width, ctx.grid_height);

        if self.selecting {
            if let Some((sx, sy)) = self.drag_start {
                self.selection = Some(Selection::new(sx, sy, x, y));
            }
            self.selecting = false;
            self.drag_start = None;
        }

        if self.moving {
            self.moving = false;
            self.move_offset = None;
            return ToolResult::new().finish();
        }

        ToolResult::new()
    }

    fn reset(&mut self) {
        self.clear_selection();
    }

    fn is_active(&self) -> bool {
        self.selecting || self.moving
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select_tool() {
        let tool = SelectTool::new();
        assert_eq!(tool.id(), ToolId::Select);
        assert!(!tool.is_active());
    }

    #[test]
    fn test_selection_creation() {
        let mut tool = SelectTool::new();
        let ctx = ToolContext {
            grid_width: 80,
            grid_height: 40,
            border_style: Default::default(),
        };

        tool.on_pointer_down(5, 5, &ctx);
        tool.on_pointer_move(10, 10, &ctx);
        tool.on_pointer_up(10, 10, &ctx);

        assert!(tool.get_selection().is_some());
        let sel = tool.get_selection().unwrap();
        assert_eq!(sel.x1, 5);
        assert_eq!(sel.y1, 5);
        assert_eq!(sel.x2, 10);
        assert_eq!(sel.y2, 10);
    }
}
