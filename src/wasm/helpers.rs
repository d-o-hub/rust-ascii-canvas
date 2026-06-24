//! Private helper methods for AsciiEditor.

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

    pub(crate) fn commit_ops(&mut self, ops: &[DrawOp]) {
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

    pub(crate) fn is_incremental_tool(&self) -> bool {
        matches!(self.tool_id, ToolId::Freehand | ToolId::Eraser)
    }

    pub(crate) fn set_tool_by_id_impl(&mut self, id: ToolId) {
        use crate::wasm::tool_manager::set_tool_by_id;
        self.tool_id = id;
        set_tool_by_id(
            &mut self.active_tool,
            &mut self.tool_id,
            &mut self.preview_ops,
            &mut self.state,
            &mut self.current_selection,
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
            Some(self.export_ascii())
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

    pub(crate) fn copy_selection_impl(&mut self) -> bool {
        if self.tool_id != ToolId::Select {
            return false;
        }

        if let Some(ref sel) = self.current_selection {
            let (min_x, min_y, max_x, max_y) = sel.bounds();
            let width = max_x - min_x + 1;
            let height = max_y - min_y + 1;

            let mut cells = Vec::new();
            for y in min_y..=max_y {
                for x in min_x..=max_x {
                    if let Some(cell) = self.state.grid.get(x, y) {
                        cells.push((x - min_x, y - min_y, *cell));
                    }
                }
            }

            self.clipboard = SelectionClipboard {
                cells,
                width,
                height,
            };
            return true;
        }
        false
    }

    pub(crate) fn cut_selection_impl(&mut self) -> bool {
        if !self.copy_selection_impl() {
            return false;
        }

        if self.tool_id == ToolId::Select {
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
                    self.history.push(Box::new(cmd));
                    self.dirty_tracker.request_full_redraw();
                }

                self.current_selection = None;
                return true;
            }
        }
        false
    }

    pub(crate) fn paste_impl(&mut self) -> bool {
        if self.clipboard.is_empty() {
            return false;
        }

        let grid_width = self.state.grid.width() as i32;
        let grid_height = self.state.grid.height() as i32;
        let mut ops = Vec::new();

        for (rel_x, rel_y, cell) in &self.clipboard.cells {
            let x = *rel_x;
            let y = *rel_y;

            if x >= 0 && x < grid_width && y >= 0 && y < grid_height {
                ops.push(DrawOp::new(x, y, cell.ch));
            }
        }

        if !ops.is_empty() {
            let mut cmd = DrawCommand::new(ops);
            cmd.apply(&mut self.state.grid);
            self.history.push(Box::new(cmd));
            self.dirty_tracker.request_full_redraw();
            return true;
        }
        false
    }

    pub(crate) fn delete_selection_impl(&mut self) -> bool {
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
                self.history.push(Box::new(cmd));
                self.dirty_tracker.request_full_redraw();
            }

            self.current_selection = None;
            return true;
        }
        false
    }
}
