//! Editor state management - selection, movement, and tool context.

use crate::core::commands::{Command, DrawCommand};
use crate::core::history::History;
use crate::core::selection::{Selection, SelectionClipboard};
use crate::core::tools::{DrawOp, Tool, ToolContext, ToolId};
use crate::core::EditorState;

/// Manages selection moving operations
pub struct MoveManager {
    /// Content and position when moving a selection
    pub clipboard: Option<SelectionClipboard>,
    /// Original selection before move started
    pub original_selection: Option<Selection>,
}

impl MoveManager {
    pub fn new() -> Self {
        Self {
            clipboard: None,
            original_selection: None,
        }
    }

    /// Capture the current selection content before starting a move.
    pub fn start_move(&mut self, sel: &Selection, state: &EditorState) {
        let (min_x, min_y, max_x, max_y) = sel.bounds();
        let width = max_x - min_x + 1;
        let height = max_y - min_y + 1;

        let mut cells = Vec::new();
        for y in min_y..=max_y {
            for x in min_x..=max_x {
                if let Some(cell) = state.grid.get(x, y) {
                    cells.push((x - min_x, y - min_y, *cell));
                }
            }
        }

        self.clipboard = Some(SelectionClipboard {
            cells,
            width,
            height,
        });
        self.original_selection = Some(sel.clone());
    }

    /// Generate preview ops for the current move operation.
    pub fn generate_preview_ops(
        &self,
        orig_sel: &Selection,
        curr_sel: &Selection,
        state: &EditorState,
    ) -> Vec<DrawOp> {
        let mut ops = Vec::new();

        if let Some(ref move_clip) = self.clipboard {
            let (orig_x, orig_y, orig_x2, orig_y2) = orig_sel.bounds();
            let (curr_x, curr_y, ..) = curr_sel.bounds();

            // Only generate ops if we've actually moved
            if orig_x != curr_x || orig_y != curr_y {
                // Clear original area
                for y in orig_y..=orig_y2 {
                    for x in orig_x..=orig_x2 {
                        ops.push(DrawOp::new(x, y, ' '));
                    }
                }

                // Draw at new position
                for (rel_x, rel_y, cell) in &move_clip.cells {
                    let new_x = curr_x + rel_x;
                    let new_y = curr_y + rel_y;

                    if state.grid.in_bounds(new_x, new_y) {
                        ops.push(DrawOp::new(new_x, new_y, cell.ch));
                    }
                }
            }
        }

        ops
    }

    /// Clear move state after commit.
    pub fn clear(&mut self) {
        self.clipboard = None;
        self.original_selection = None;
    }

    /// Check if currently moving a selection.
    pub fn is_moving(&self) -> bool {
        self.clipboard.is_some() && self.original_selection.is_some()
    }
}

/// Helper methods for editor state management
pub trait StateHelpers {
    /// Create tool context from current state
    fn create_tool_context(&self) -> ToolContext;

    /// Commit operations to history and mark dirty
    fn commit_ops(&mut self, ops: &[DrawOp]);

    /// Check if current tool commits incrementally during drag
    fn is_incremental_tool(&self) -> bool;

    /// Check if select tool is currently moving a selection
    fn is_select_moving(&self) -> bool;
}

impl StateHelpers for super::AsciiEditor {
    fn create_tool_context(&self) -> ToolContext {
        ToolContext {
            grid_width: self.state.grid.width(),
            grid_height: self.state.grid.height(),
            border_style: self.state.border_style,
        }
    }

    fn commit_ops(&mut self, ops: &[DrawOp]) {
        if ops.is_empty() {
            return;
        }

        let mut cmd = DrawCommand::new(ops.to_vec());
        cmd.apply(&mut self.state.grid);
        self.history.push(Box::new(cmd));

        for op in ops {
            self.dirty_tracker.mark_dirty(op.x, op.y);
        }
    }

    /// Check if current tool commits incrementally during drag
    /// (freehand draws points as you move, eraser clears as you move).
    fn is_incremental_tool(&self) -> bool {
        matches!(self.tool_id, ToolId::Freehand | ToolId::Eraser)
    }

    /// Check if select tool is currently moving a selection.
    fn is_select_moving(&self) -> bool {
        self.tool_id == ToolId::Select && self.is_moving_selection
    }
}

/// Generate preview ops for the current move operation.
pub fn generate_move_preview_ops(
    orig_sel: &Selection,
    curr_sel: &Selection,
    move_clip: &SelectionClipboard,
    state: &EditorState,
) -> Vec<DrawOp> {
    let mut ops = Vec::new();

    let (orig_x, orig_y, orig_x2, orig_y2) = orig_sel.bounds();
    let (curr_x, curr_y, ..) = curr_sel.bounds();

    // Only generate ops if we've actually moved
    if orig_x != curr_x || orig_y != curr_y {
        // Clear original area
        for y in orig_y..=orig_y2 {
            for x in orig_x..=orig_x2 {
                ops.push(DrawOp::new(x, y, ' '));
            }
        }

        // Draw at new position
        for (rel_x, rel_y, cell) in &move_clip.cells {
            let new_x = curr_x + rel_x;
            let new_y = curr_y + rel_y;

            if state.grid.in_bounds(new_x, new_y) {
                ops.push(DrawOp::new(new_x, new_y, cell.ch));
            }
        }
    }

    ops
}
