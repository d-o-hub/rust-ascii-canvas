//! Internal clipboard operations.

use super::AsciiEditor;
use crate::core::commands::{Command, DrawCommand};
use crate::core::selection::SelectionClipboard;
use crate::core::tools::{DrawOp, ToolId};

impl AsciiEditor {
    pub(crate) fn copy_selection_impl(&mut self) -> bool {
        // Always source from the composite of visible layers so internal paste matches
        // OS clipboard / exportAscii for multi-layer documents.
        let composite = self.composite_visible_grid();

        // Prefer explicit selection; otherwise copy full content bounds into internal clipboard.
        if let Some(ref sel) = self.current_selection {
            let (min_x, min_y, max_x, max_y) = sel.bounds();
            let width = max_x - min_x + 1;
            let height = max_y - min_y + 1;

            // Only store visible cells so paste does not clobber destination with spaces.
            let mut cells = Vec::new();
            for y in min_y..=max_y {
                for x in min_x..=max_x {
                    if let Some(cell) = composite.get(x, y) {
                        if cell.is_visible() {
                            cells.push((x - min_x, y - min_y, *cell));
                        }
                    }
                }
            }

            self.clipboard = SelectionClipboard {
                cells,
                width,
                height,
            };
            return !self.clipboard.is_empty() || width > 0;
        }

        // No selection: copy entire content bounding box (visible cells only).
        if let Some((min_x, min_y, max_x, max_y)) =
            crate::core::ascii_export::find_content_bounds(&composite)
        {
            let width = max_x - min_x + 1;
            let height = max_y - min_y + 1;
            let mut cells = Vec::new();
            for y in min_y..=max_y {
                for x in min_x..=max_x {
                    if let Some(cell) = composite.get(x, y) {
                        if cell.is_visible() {
                            cells.push((x - min_x, y - min_y, *cell));
                        }
                    }
                }
            }
            self.clipboard = SelectionClipboard {
                cells,
                width,
                height,
            };
            return !self.clipboard.is_empty();
        }

        false
    }

    pub(crate) fn cut_selection_impl(&mut self) -> bool {
        if self.is_active_layer_locked() {
            return false;
        }
        if self.current_selection.is_none() {
            return false;
        }
        if !self.copy_selection_impl() {
            return false;
        }

        if let Some(ref sel) = self.current_selection {
            let (min_x, min_y, max_x, max_y) = sel.bounds();
            let mut ops = Vec::new();

            for y in min_y..=max_y {
                for x in min_x..=max_x {
                    ops.push(DrawOp::new(x, y, ' '));
                }
            }

            if !ops.is_empty() {
                let mut cmd = DrawCommand::new(ops);
                cmd.apply(&mut self.state.grid);
                self.layer_stack.push_grid(Box::new(cmd));
                self.dirty_tracker.request_full_redraw();
            }

            self.current_selection = None;
            return true;
        }
        false
    }

    pub(crate) fn paste_text_impl(&mut self, text: &str) -> bool {
        if self.is_active_layer_locked() {
            return false;
        }
        if text.is_empty() {
            return false;
        }

        let (offset_x, offset_y) = self.paste_origin();
        let grid_width = self.state.grid.width() as i32;
        let grid_height = self.state.grid.height() as i32;
        let mut ops = Vec::new();

        for (row_idx, line) in text.lines().enumerate() {
            let y = offset_y + row_idx as i32;
            if y < 0 || y >= grid_height {
                continue;
            }

            for (col_idx, ch) in line.chars().enumerate() {
                let x = offset_x + col_idx as i32;
                if x < 0 || x >= grid_width {
                    continue;
                }

                if ch != ' ' && !ch.is_whitespace() && !ch.is_control() {
                    ops.push(DrawOp::new(x, y, ch));
                }
            }
        }

        self.apply_paste_ops(ops)
    }

    pub(crate) fn paste_impl(&mut self) -> bool {
        if self.is_active_layer_locked() {
            return false;
        }
        if self.clipboard.is_empty() {
            return false;
        }

        let (offset_x, offset_y) = self.paste_origin();
        let grid_width = self.state.grid.width() as i32;
        let grid_height = self.state.grid.height() as i32;
        let mut ops = Vec::new();

        for (rel_x, rel_y, cell) in &self.clipboard.cells {
            let x = offset_x + *rel_x;
            let y = offset_y + *rel_y;

            if x >= 0 && x < grid_width && y >= 0 && y < grid_height {
                ops.push(DrawOp::new(x, y, cell.ch));
            }
        }

        self.apply_paste_ops(ops)
    }

    fn apply_paste_ops(&mut self, ops: Vec<DrawOp>) -> bool {
        if ops.is_empty() {
            return false;
        }
        // Capture the paste origin first, then roll back provisional cells before
        // the paste records its undo values. Rejected paste leaves the stroke alone.
        if self.stroke.is_some() {
            self.reset_interaction();
        }
        self.commit_ops(&ops);
        self.dirty_tracker.request_full_redraw();
        true
    }

    pub(crate) fn delete_selection_impl(&mut self) -> bool {
        if self.is_active_layer_locked() {
            return false;
        }
        if self.tool_id != ToolId::Select {
            return false;
        }

        if let Some(ref sel) = self.current_selection {
            let (min_x, min_y, max_x, max_y) = sel.bounds();
            let mut ops = Vec::new();

            for y in min_y..=max_y {
                for x in min_x..=max_x {
                    ops.push(DrawOp::new(x, y, ' '));
                }
            }

            if !ops.is_empty() {
                let mut cmd = DrawCommand::new(ops);
                cmd.apply(&mut self.state.grid);
                self.layer_stack.push_grid(Box::new(cmd));
                self.dirty_tracker.request_full_redraw();
            }

            self.current_selection = None;
            return true;
        }
        false
    }
}
