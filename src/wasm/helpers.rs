//! Private helper methods for AsciiEditor.

use crate::core::ascii_export::{export_grid, export_region_with_options, ExportOptions};
use crate::core::commands::{Command, DrawCommand};
use crate::core::selection::{Selection, SelectionClipboard};
use crate::core::tools::{DrawOp, SelectTool, ToolContext, ToolId};
use crate::wasm::render_bridge::{
    create_event_result, create_event_result_with_copy, EditorEventResult,
};

use super::bindings::AsciiEditor;

impl AsciiEditor {
    pub(crate) fn create_tool_context(&self) -> ToolContext {
        ToolContext {
            grid_width: self.state.grid.width(),
            grid_height: self.state.grid.height(),
            border_style: self.state.border_style,
        }
    }

    /// Export text for OS clipboard: selection region if present, else full composite.
    /// Always reads from the composite of visible layers so multi-layer docs match `exportAscii`.
    pub(crate) fn export_for_copy(&self) -> String {
        self.export_for_copy_with_options(&ExportOptions::default())
    }

    /// Export the current copy scope with explicit clipboard fidelity options.
    pub(crate) fn export_for_copy_with_options(&self, options: &ExportOptions) -> String {
        let composite = self.composite_visible_grid();
        if let Some(ref sel) = self.current_selection {
            let (min_x, min_y, max_x, max_y) = sel.bounds();
            export_region_with_options(&composite, min_x, min_y, max_x, max_y, options)
        } else {
            export_grid(&composite, options)
        }
    }

    /// Paste origin: selection top-left, else last cursor, else (0,0).
    pub(crate) fn paste_origin(&self) -> (i32, i32) {
        if let Some(ref sel) = self.current_selection {
            let (min_x, min_y, _, _) = sel.bounds();
            (min_x, min_y)
        } else if let Some((x, y)) = self.last_cursor {
            (x, y)
        } else {
            (0, 0)
        }
    }

    pub(crate) fn commit_ops(&mut self, ops: &[DrawOp]) {
        if self.is_active_layer_locked() {
            return;
        }
        if ops.is_empty() {
            return;
        }

        let mut cmd = DrawCommand::new(ops.to_vec());
        cmd.apply(&mut self.state.grid);
        self.layer_stack.push_grid(Box::new(cmd));

        for op in ops {
            self.dirty_tracker.mark_dirty(op.x, op.y);
        }
    }

    pub(crate) fn is_incremental_tool(&self) -> bool {
        matches!(self.tool_id, ToolId::Freehand | ToolId::Eraser)
    }

    pub(crate) fn set_tool_by_id_impl(&mut self, id: ToolId) {
        use crate::wasm::tool_manager::set_tool_by_id;
        self.reset_interaction();
        self.tool_id = id;
        set_tool_by_id(
            &mut self.active_tool,
            &mut self.tool_id,
            &mut self.preview_ops,
            &mut self.state,
            &mut self.current_selection,
            self.eraser_size,
        );
    }

    pub(crate) fn is_select_moving(&self) -> bool {
        self.tool_id == ToolId::Select && self.is_moving_selection
    }

    pub(crate) fn update_select_tool_selection(&mut self) {
        self.current_selection = self.active_tool.get_selection();
    }

    pub(crate) fn start_selection_move(&mut self) {
        if let Some(ref sel) = self.current_selection {
            let (min_x, min_y, max_x, max_y) = sel.bounds();
            let width = max_x - min_x + 1;
            let height = max_y - min_y + 1;

            // Include all cells (including spaces) so move preserves empty interior
            // and correctly clears/restores the region.
            let mut cells = Vec::new();
            for y in min_y..=max_y {
                for x in min_x..=max_x {
                    if let Some(cell) = self.state.grid.get(x, y) {
                        cells.push((x - min_x, y - min_y, *cell));
                    }
                }
            }

            self.move_clipboard = Some(SelectionClipboard {
                cells,
                width,
                height,
            });
            self.move_original_selection = Some(sel.clone());
        }
    }

    pub(crate) fn generate_move_preview_ops(&self) -> Vec<DrawOp> {
        let mut ops = Vec::new();

        if let (Some(ref orig_sel), Some(ref curr_sel), Some(ref move_clip)) = (
            &self.move_original_selection,
            &self.current_selection,
            &self.move_clipboard,
        ) {
            let (orig_x, orig_y, orig_x2, orig_y2) = orig_sel.bounds();
            let (curr_x, curr_y, _, _) = curr_sel.bounds();

            if orig_x != curr_x || orig_y != curr_y {
                for y in orig_y..=orig_y2 {
                    for x in orig_x..=orig_x2 {
                        ops.push(DrawOp::new(x, y, ' '));
                    }
                }

                for (rel_x, rel_y, cell) in &move_clip.cells {
                    let new_x = curr_x + rel_x;
                    let new_y = curr_y + rel_y;

                    if self.state.grid.in_bounds(new_x, new_y) {
                        ops.push(DrawOp::new(new_x, new_y, cell.ch));
                    }
                }
            }
        }

        ops
    }

    pub(crate) fn commit_selection_move(&mut self) {
        let ops = self.generate_move_preview_ops();

        if !ops.is_empty() {
            self.commit_ops(&ops);
        }

        self.move_clipboard = None;
        self.move_original_selection = None;
    }

    pub(crate) fn create_event_result(&self) -> EditorEventResult {
        create_event_result(
            self.needs_redraw(),
            self.tool_id.name(),
            self.can_undo(),
            self.can_redo(),
        )
    }

    pub(crate) fn create_event_result_with_copy(&self, should_copy: bool) -> EditorEventResult {
        let ascii = if should_copy {
            Some(self.export_for_copy())
        } else {
            None
        };

        create_event_result_with_copy(
            self.tool_id.name(),
            self.can_undo(),
            self.can_redo(),
            ascii.unwrap_or_default(),
        )
    }

    pub(crate) fn select_all_impl(&mut self) {
        let width = self.state.grid.width() as i32;
        let height = self.state.grid.height() as i32;

        self.set_tool_by_id_impl(ToolId::Select);

        if let Some(select_tool) = self.active_tool.as_any_mut().downcast_mut::<SelectTool>() {
            let selection = Selection::new(0, 0, width - 1, height - 1);
            select_tool.set_selection(selection.clone());
            self.current_selection = Some(selection);
            self.dirty_tracker.request_full_redraw();
        }
    }

    /// Composite all visible layers (bottom → top) into a single grid.
    pub(crate) fn composite_visible_grid(&self) -> crate::core::Grid {
        let w = self.state.grid.width();
        let h = self.state.grid.height();
        let mut out = crate::core::Grid::new(w, h);

        let active = self.layer_stack.active_index();
        let active_grid = self.committed_active_grid();
        for (i, layer) in self.layer_stack.layers().iter().enumerate() {
            if !layer.is_visible() {
                continue;
            }
            let src = if i == active {
                &active_grid
            } else {
                layer.grid()
            };
            for (x, y, cell) in src.iter_with_coords() {
                if cell.is_visible() {
                    let _ = out.set(x, y, *cell);
                }
            }
        }
        out
    }

    #[cfg(test)]
    pub(crate) fn set_selection_for_test(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) {
        self.current_selection = Some(Selection::new(x1, y1, x2, y2));
    }
}
