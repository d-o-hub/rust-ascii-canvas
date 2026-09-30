//! The single cancellation seam for transient tool/pointer state.

use super::AsciiEditor;
use crate::core::commands::StrokeCommand;
use crate::core::{DrawOp, Grid};

impl AsciiEditor {
    /// Cancel unfinished work before changing its document, layer, or coordinate space.
    /// Completed edits and the internal copy clipboard are intentionally retained.
    pub(crate) fn reset_interaction(&mut self) {
        if let Some(stroke) = self.stroke.take() {
            stroke.restore_original(&mut self.state.grid);
        }
        self.active_tool.reset();
        self.preview_ops.clear();
        self.current_selection = None;
        self.move_clipboard = None;
        self.move_original_selection = None;
        self.is_moving_selection = false;
        self.last_cursor = None;
        self.space_held = false;
        self.is_panning = false;
        self.last_pan_pos = None;
        self.dirty_tracker.request_full_redraw();
    }

    pub(crate) fn begin_stroke(&mut self) {
        self.stroke = Some(StrokeCommand::new(self.tool_id.name()));
    }

    pub(crate) fn extend_stroke(&mut self, ops: &[DrawOp]) {
        if let Some(stroke) = &mut self.stroke {
            stroke.extend(&mut self.state.grid, ops);
            for op in ops {
                self.dirty_tracker.mark_dirty(op.x, op.y);
            }
        }
    }

    pub(crate) fn finish_stroke(&mut self) {
        if let Some(stroke) = self.stroke.take() {
            if !stroke.is_empty() {
                self.layer_stack.push_grid(Box::new(stroke));
            }
        }
    }

    /// Snapshot committed work without the in-progress live stroke.
    pub(crate) fn committed_active_grid(&self) -> Grid {
        let mut grid = self.state.grid.clone();
        if let Some(stroke) = &self.stroke {
            stroke.restore_original(&mut grid);
        }
        grid
    }
}
