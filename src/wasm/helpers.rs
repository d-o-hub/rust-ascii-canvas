//! Private helper methods for AsciiEditor.

use crate::core::ascii_export::export_region;
use crate::core::commands::{Command, DrawCommand};
use crate::core::selection::{Selection, SelectionClipboard};
use crate::core::tools::{DrawOp, SelectTool, ToolContext, ToolId};
use crate::wasm::render_bridge::{
    create_event_result, create_event_result_with_copy, export_ascii, EditorEventResult,
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

    /// Export text for OS clipboard: selection region if present, else composite of visible layers.
    pub(crate) fn export_for_copy(&self) -> String {
        if let Some(ref sel) = self.current_selection {
            let (min_x, min_y, max_x, max_y) = sel.bounds();
            export_region(&self.state.grid, min_x, min_y, max_x, max_y)
        } else {
            export_ascii(&self.composite_visible_grid())
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

    pub(crate) fn copy_selection_impl(&mut self) -> bool {
        // Prefer explicit selection; otherwise copy full content bounds into internal clipboard.
        if let Some(ref sel) = self.current_selection {
            let (min_x, min_y, max_x, max_y) = sel.bounds();
            let width = max_x - min_x + 1;
            let height = max_y - min_y + 1;

            // Only store visible cells so paste does not clobber destination with spaces.
            let mut cells = Vec::new();
            for y in min_y..=max_y {
                for x in min_x..=max_x {
                    if let Some(cell) = self.state.grid.get(x, y) {
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
            crate::core::ascii_export::find_content_bounds(&self.state.grid)
        {
            let width = max_x - min_x + 1;
            let height = max_y - min_y + 1;
            let mut cells = Vec::new();
            for y in min_y..=max_y {
                for x in min_x..=max_x {
                    if let Some(cell) = self.state.grid.get(x, y) {
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
                self.history.push(Box::new(cmd));
                self.dirty_tracker.request_full_redraw();
            }

            self.current_selection = None;
            return true;
        }
        false
    }

    pub(crate) fn paste_impl(&mut self) -> bool {
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

    /// Persist active drawing surface into the layer store.
    pub(crate) fn sync_active_layer(&mut self) {
        if let Some(layer) = self.layers.get_mut(self.active_layer) {
            layer.grid = self.state.grid.clone();
        }
    }

    pub(crate) fn set_active_layer_impl(&mut self, index: usize) -> bool {
        if index >= self.layers.len() || index == self.active_layer {
            return index < self.layers.len();
        }
        self.sync_active_layer();
        self.active_layer = index;
        if let Some(layer) = self.layers.get(index) {
            self.state.grid = layer.grid.clone();
        }
        self.current_selection = None;
        self.preview_ops.clear();
        self.history.clear();
        self.dirty_tracker.request_full_redraw();
        true
    }

    pub(crate) fn add_layer_impl(&mut self) -> usize {
        self.sync_active_layer();
        let w = self.state.grid.width();
        let h = self.state.grid.height();
        let name = format!("Layer {}", self.layers.len() + 1);
        self.layers.push(super::bindings::LayerData {
            name,
            visible: true,
            grid: crate::core::Grid::new(w, h),
        });
        let index = self.layers.len() - 1;
        self.active_layer = index;
        self.state.grid = crate::core::Grid::new(w, h);
        self.current_selection = None;
        self.preview_ops.clear();
        self.history.clear();
        self.dirty_tracker.request_full_redraw();
        index
    }

    /// Composite all visible layers (bottom → top) into a single grid.
    pub(crate) fn composite_visible_grid(&self) -> crate::core::Grid {
        let w = self.state.grid.width();
        let h = self.state.grid.height();
        let mut out = crate::core::Grid::new(w, h);

        for (i, layer) in self.layers.iter().enumerate() {
            if !layer.visible {
                continue;
            }
            let src = if i == self.active_layer {
                &self.state.grid
            } else {
                &layer.grid
            };
            for (x, y, cell) in src.iter_with_coords() {
                if cell.is_visible() {
                    let _ = out.set(x, y, *cell);
                }
            }
        }
        out
    }

    pub(crate) fn serialize_document_impl(&self) -> String {
        // Snapshot active layer content for serialization without requiring &mut.
        #[derive(serde::Serialize)]
        struct DocCell {
            x: i32,
            y: i32,
            ch: String,
        }
        #[derive(serde::Serialize)]
        struct DocLayer {
            name: String,
            visible: bool,
            cells: Vec<DocCell>,
        }
        #[derive(serde::Serialize)]
        struct Document {
            format: &'static str,
            version: u32,
            canvas: CanvasSize,
            active_layer: usize,
            layers: Vec<DocLayer>,
        }
        #[derive(serde::Serialize)]
        struct CanvasSize {
            width: usize,
            height: usize,
        }

        let mut layers = Vec::with_capacity(self.layers.len());
        for (i, layer) in self.layers.iter().enumerate() {
            let src = if i == self.active_layer {
                &self.state.grid
            } else {
                &layer.grid
            };
            let mut cells = Vec::new();
            for (x, y, cell) in src.iter_with_coords() {
                if cell.is_visible() {
                    cells.push(DocCell {
                        x,
                        y,
                        ch: cell.ch.to_string(),
                    });
                }
            }
            layers.push(DocLayer {
                name: layer.name.clone(),
                visible: layer.visible,
                cells,
            });
        }

        let doc = Document {
            format: "ascii-canvas",
            version: 1,
            canvas: CanvasSize {
                width: self.state.grid.width(),
                height: self.state.grid.height(),
            },
            active_layer: self.active_layer,
            layers,
        };

        serde_json::to_string(&doc).unwrap_or_else(|_| "{}".to_string())
    }

    #[cfg(test)]
    pub(crate) fn set_selection_for_test(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) {
        self.current_selection = Some(Selection::new(x1, y1, x2, y2));
    }

    pub(crate) fn load_document_impl(&mut self, json: &str) -> bool {
        #[derive(serde::Deserialize)]
        struct DocCell {
            x: i32,
            y: i32,
            ch: String,
        }
        #[derive(serde::Deserialize)]
        struct DocLayer {
            name: String,
            #[serde(default = "default_true")]
            visible: bool,
            cells: Vec<DocCell>,
        }
        fn default_true() -> bool {
            true
        }
        #[derive(serde::Deserialize)]
        struct CanvasSize {
            width: usize,
            height: usize,
        }
        #[derive(serde::Deserialize)]
        struct Document {
            format: String,
            version: u32,
            canvas: CanvasSize,
            #[serde(default)]
            active_layer: usize,
            layers: Vec<DocLayer>,
        }

        let doc: Document = match serde_json::from_str(json) {
            Ok(d) => d,
            Err(_) => return false,
        };
        if doc.format != "ascii-canvas" || doc.version == 0 || doc.layers.is_empty() {
            return false;
        }
        if doc.canvas.width == 0 || doc.canvas.height == 0 {
            return false;
        }

        let w = doc.canvas.width;
        let h = doc.canvas.height;
        let mut layers = Vec::new();
        for layer in doc.layers {
            let mut grid = crate::core::Grid::new(w, h);
            for cell in layer.cells {
                if let Some(ch) = cell.ch.chars().next() {
                    let _ = grid.set_char(cell.x, cell.y, ch);
                }
            }
            layers.push(super::bindings::LayerData {
                name: layer.name,
                visible: layer.visible,
                grid,
            });
        }

        let active = doc.active_layer.min(layers.len() - 1);
        self.layers = layers;
        self.active_layer = active;
        self.state.grid = self.layers[active].grid.clone();
        self.history.clear();
        self.clipboard.clear();
        self.current_selection = None;
        self.preview_ops.clear();
        self.pixel_buffer = vec![0u8; w * 8 * h * 20 * 4];
        self.dirty_tracker.request_full_redraw();
        true
    }
}

#[cfg(test)]
mod clipboard_tests {
    use crate::wasm::bindings::AsciiEditor;

    fn make_canvas_with_box() -> AsciiEditor {
        let mut canvas = AsciiEditor::new(10, 10);
        // ┌───┐
        // │   │
        // └───┘
        canvas.state.grid.set_char(0, 0, '┌');
        canvas.state.grid.set_char(1, 0, '─');
        canvas.state.grid.set_char(2, 0, '─');
        canvas.state.grid.set_char(3, 0, '─');
        canvas.state.grid.set_char(4, 0, '┐');
        canvas.state.grid.set_char(0, 1, '│');
        canvas.state.grid.set_char(4, 1, '│');
        canvas.state.grid.set_char(0, 2, '└');
        canvas.state.grid.set_char(1, 2, '─');
        canvas.state.grid.set_char(2, 2, '─');
        canvas.state.grid.set_char(3, 2, '─');
        canvas.state.grid.set_char(4, 2, '┘');
        canvas
    }

    #[test]
    fn test_export_for_copy_preserves_box() {
        let canvas = make_canvas_with_box();
        let ascii = canvas.export_for_copy();
        let lines: Vec<&str> = ascii.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[0].ends_with('┐'));
        assert!(lines[1].ends_with('│'));
        assert!(lines[2].ends_with('┘'));
    }

    #[test]
    fn test_copy_and_paste_at_selection_origin() {
        let mut canvas = make_canvas_with_box();
        canvas.set_selection_for_test(0, 0, 4, 2);
        assert!(canvas.copy_selection_impl());
        // Move selection to paste origin
        canvas.set_selection_for_test(5, 5, 5, 5);
        assert!(canvas.paste_impl());
        assert_eq!(canvas.state.grid.get(5, 5).map(|c| c.ch), Some('┌'));
        assert_eq!(canvas.state.grid.get(9, 5).map(|c| c.ch), Some('┐'));
        // Origin box still present
        assert_eq!(canvas.state.grid.get(0, 0).map(|c| c.ch), Some('┌'));
    }

    #[test]
    fn test_paste_does_not_clobber_with_spaces() {
        let mut canvas = AsciiEditor::new(10, 10);
        canvas.state.grid.set_char(5, 5, 'Z');
        canvas.state.grid.set_char(0, 0, 'A');
        canvas.set_selection_for_test(0, 0, 0, 0);
        assert!(canvas.copy_selection_impl());
        canvas.set_selection_for_test(5, 5, 5, 5);
        assert!(canvas.paste_impl());
        // Only A is in clipboard; A overwrites Z at paste target
        assert_eq!(canvas.state.grid.get(5, 5).map(|c| c.ch), Some('A'));
    }

    #[test]
    fn test_serialize_load_round_trip() {
        let canvas = make_canvas_with_box();
        let json = canvas.serialize_document_impl();
        let mut other = AsciiEditor::new(10, 10);
        assert!(other.load_document_impl(&json));
        assert_eq!(other.export_for_copy(), canvas.export_for_copy());
    }

    #[test]
    fn test_add_layer() {
        let mut canvas = AsciiEditor::new(8, 8);
        let idx = canvas.add_layer_impl();
        assert_eq!(idx, 1);
        assert_eq!(canvas.layers.len(), 2);
    }
}
