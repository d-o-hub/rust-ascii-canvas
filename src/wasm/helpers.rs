//! Private helper methods for AsciiEditor.

use crate::core::ascii_export::export_region;
use crate::core::commands::{Command, DrawCommand};
use crate::core::history::{History, DEFAULT_MAX_DEPTH};
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

    /// Export text for OS clipboard: selection region if present, else full composite.
    /// Always reads from the composite of visible layers so multi-layer docs match `exportAscii`.
    pub(crate) fn export_for_copy(&self) -> String {
        let composite = self.composite_visible_grid();
        if let Some(ref sel) = self.current_selection {
            let (min_x, min_y, max_x, max_y) = sel.bounds();
            export_region(&composite, min_x, min_y, max_x, max_y)
        } else {
            export_ascii(&composite)
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
                self.history.push(Box::new(cmd));
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

        if !ops.is_empty() {
            self.commit_ops(&ops);
            self.dirty_tracker.request_full_redraw();
            return true;
        }
        false
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

        let old_active = self.active_layer;
        self.active_layer = index;

        // Swap history!
        let mut temp_history = std::mem::take(&mut self.history);
        std::mem::swap(&mut temp_history, &mut self.layers[old_active].history);
        self.history = std::mem::take(&mut self.layers[index].history);

        if let Some(layer) = self.layers.get(index) {
            self.state.grid = layer.grid.clone();
        }
        self.current_selection = None;
        self.preview_ops.clear();
        self.dirty_tracker.request_full_redraw();
        true
    }

    pub(crate) fn add_layer_impl(&mut self) -> usize {
        self.sync_active_layer();
        let w = self.state.grid.width();
        let h = self.state.grid.height();
        let name = format!("Layer {}", self.layers.len() + 1);

        // Swap active history into old active layer's history
        let mut temp_history = std::mem::take(&mut self.history);
        std::mem::swap(
            &mut temp_history,
            &mut self.layers[self.active_layer].history,
        );

        self.layers.push(super::bindings::LayerData {
            name,
            visible: true,
            locked: false,
            grid: crate::core::Grid::new(w, h),
            history: History::new(DEFAULT_MAX_DEPTH),
        });
        let index = self.layers.len() - 1;
        self.active_layer = index;
        self.state.grid = crate::core::Grid::new(w, h);
        self.history = History::new(DEFAULT_MAX_DEPTH); // fresh history for the new layer
        self.current_selection = None;
        self.preview_ops.clear();
        self.dirty_tracker.request_full_redraw();
        index
    }

    pub(crate) fn move_layer_impl(&mut self, from_index: usize, to_index: usize) {
        if from_index >= self.layers.len()
            || to_index >= self.layers.len()
            || from_index == to_index
        {
            return;
        }
        self.sync_active_layer();

        // Temporarily swap active history back to active layer for moving
        let mut temp_history = std::mem::take(&mut self.history);
        std::mem::swap(
            &mut temp_history,
            &mut self.layers[self.active_layer].history,
        );

        let layer = self.layers.remove(from_index);
        self.layers.insert(to_index, layer);

        if self.active_layer == from_index {
            self.active_layer = to_index;
        } else if from_index < to_index
            && self.active_layer > from_index
            && self.active_layer <= to_index
        {
            self.active_layer -= 1;
        } else if from_index > to_index
            && self.active_layer >= to_index
            && self.active_layer < from_index
        {
            self.active_layer += 1;
        }

        // Swap history back from the new active layer
        let mut temp_history = std::mem::take(&mut self.layers[self.active_layer].history);
        std::mem::swap(&mut temp_history, &mut self.history);

        self.state.grid = self.layers[self.active_layer].grid.clone();
        self.dirty_tracker.request_full_redraw();
    }

    pub(crate) fn delete_layer_impl(&mut self, index: usize) -> bool {
        if self.layers.len() <= 1 || index >= self.layers.len() {
            return false;
        }
        self.sync_active_layer();

        // Temporarily swap active history back to active layer before structural changes
        let mut temp_history = std::mem::take(&mut self.history);
        std::mem::swap(
            &mut temp_history,
            &mut self.layers[self.active_layer].history,
        );

        self.layers.remove(index);

        if self.active_layer == index {
            if self.active_layer >= self.layers.len() {
                self.active_layer = self.layers.len() - 1;
            }
        } else if self.active_layer > index {
            self.active_layer -= 1;
        }

        // Restore active grid and history
        self.state.grid = self.layers[self.active_layer].grid.clone();
        self.history = std::mem::take(&mut self.layers[self.active_layer].history);

        self.current_selection = None;
        self.preview_ops.clear();
        self.dirty_tracker.request_full_redraw();
        true
    }

    pub(crate) fn merge_down_impl(&mut self, index: usize) -> bool {
        if index == 0 || index >= self.layers.len() {
            return false;
        }
        self.sync_active_layer();

        // Temporarily swap active history back to active layer before structural changes
        let mut temp_history = std::mem::take(&mut self.history);
        std::mem::swap(
            &mut temp_history,
            &mut self.layers[self.active_layer].history,
        );

        // Clone upper layer's grid
        let upper_grid = self.layers[index].grid.clone();

        // Merge into lower layer
        {
            let lower_layer = &mut self.layers[index - 1];
            for (x, y, cell) in upper_grid.iter_with_coords() {
                if cell.is_visible() {
                    lower_layer.grid.set(x, y, *cell);
                }
            }
        }

        self.layers.remove(index);

        if self.active_layer == index {
            self.active_layer = index - 1;
        } else if self.active_layer > index {
            self.active_layer -= 1;
        }

        self.state.grid = self.layers[self.active_layer].grid.clone();
        self.history = std::mem::take(&mut self.layers[self.active_layer].history);

        self.current_selection = None;
        self.preview_ops.clear();
        self.dirty_tracker.request_full_redraw();
        true
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
            locked: bool,
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
                locked: layer.locked,
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
            #[serde(default = "default_false")]
            locked: bool,
            cells: Vec<DocCell>,
        }
        fn default_true() -> bool {
            true
        }
        fn default_false() -> bool {
            false
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
        // Match UI grid Apply caps (400×200) and keep layer count bounded to avoid OOM.
        const MAX_CANVAS_WIDTH: usize = 400;
        const MAX_CANVAS_HEIGHT: usize = 200;
        const MAX_LAYERS: usize = 32;
        if doc.canvas.width == 0
            || doc.canvas.height == 0
            || doc.canvas.width > MAX_CANVAS_WIDTH
            || doc.canvas.height > MAX_CANVAS_HEIGHT
            || doc.layers.len() > MAX_LAYERS
        {
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
                locked: layer.locked,
                grid,
                history: History::new(DEFAULT_MAX_DEPTH),
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
    fn test_export_svg() {
        let canvas = make_canvas_with_box();
        let svg = canvas.export_svg();
        assert!(svg.starts_with("<svg"));
        assert!(svg.contains("<rect"));
        assert!(svg.contains("<g fill="));
        assert!(svg.contains("dominant-baseline=\"hanging\""));
        assert!(svg.contains("┌"));
        assert!(svg.contains("┐"));
        assert!(svg.contains("┘"));
        assert!(svg.contains("└"));
        assert!(svg.ends_with("</g></svg>"));
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

    #[test]
    fn test_load_document_rejects_oversized_canvas() {
        let mut canvas = AsciiEditor::new(10, 10);
        let json = r#"{
            "format":"ascii-canvas",
            "version":1,
            "canvas":{"width":50000,"height":50000},
            "active_layer":0,
            "layers":[{"name":"Layer 1","visible":true,"cells":[]}]
        }"#;
        assert!(!canvas.load_document_impl(json));
        // Original canvas unchanged
        assert_eq!(canvas.state.grid.width(), 10);
        assert_eq!(canvas.state.grid.height(), 10);
    }

    #[test]
    fn test_load_document_rejects_too_many_layers() {
        let mut canvas = AsciiEditor::new(10, 10);
        let layers: String = (0..40)
            .map(|i| format!(r#"{{"name":"L{i}","visible":true,"cells":[]}}"#))
            .collect::<Vec<_>>()
            .join(",");
        let json = format!(
            r#"{{"format":"ascii-canvas","version":1,"canvas":{{"width":10,"height":10}},"active_layer":0,"layers":[{layers}]}}"#
        );
        assert!(!canvas.load_document_impl(&json));
    }

    #[test]
    fn test_export_and_copy_use_composite_layers() {
        let mut canvas = AsciiEditor::new(10, 10);
        // Layer 0: 'A' at (0,0)
        canvas.state.grid.set_char(0, 0, 'A');
        // Sync layer 0 snapshot then add layer 1 with 'B' at (1,0)
        let idx = canvas.add_layer_impl();
        assert_eq!(idx, 1);
        canvas.state.grid.set_char(1, 0, 'B');
        // Both layers visible — export/copy must include A and B
        let ascii = canvas.export_for_copy();
        assert!(
            ascii.contains('A'),
            "composite export missing layer 0: {ascii:?}"
        );
        assert!(
            ascii.contains('B'),
            "composite export missing layer 1: {ascii:?}"
        );

        assert!(canvas.copy_selection_impl());
        // Paste onto a clean area and verify composite was captured
        canvas.set_selection_for_test(5, 5, 5, 5);
        assert!(canvas.paste_impl());
        assert_eq!(canvas.state.grid.get(5, 5).map(|c| c.ch), Some('A'));
        assert_eq!(canvas.state.grid.get(6, 5).map(|c| c.ch), Some('B'));
    }

    #[test]
    fn test_selection_export_uses_composite() {
        let mut canvas = AsciiEditor::new(10, 10);
        canvas.state.grid.set_char(0, 0, 'A');
        let _ = canvas.add_layer_impl();
        canvas.state.grid.set_char(1, 0, 'B');
        // Active layer is 1; selection covers both cells
        canvas.set_selection_for_test(0, 0, 1, 0);
        let ascii = canvas.export_for_copy();
        assert!(
            ascii.contains('A') && ascii.contains('B'),
            "selection export must composite layers: {ascii:?}"
        );
    }

    #[test]
    fn test_layer_lock_prevents_draw_ops() {
        let mut canvas = AsciiEditor::new(10, 10);
        // Pre-populate a cell
        use crate::core::tools::DrawOp;
        canvas.commit_ops(&[DrawOp::new(1, 1, 'A')]);
        assert_eq!(canvas.state.grid.get(1, 1).unwrap().ch, 'A');

        // Lock the layer
        canvas.set_layer_locked(0, true);
        assert!(canvas.layer_locked(0));

        // Try to write a character (e.g. via commit_ops) while locked
        canvas.commit_ops(&[DrawOp::new(2, 2, 'X')]);
        // Cell (2, 2) should remain empty
        assert_eq!(canvas.state.grid.get(2, 2).unwrap().ch, ' ');

        // Try to clear while locked
        canvas.clear();
        // Cell (1, 1) should STILL be 'A' because clear was blocked by lock
        assert_eq!(canvas.state.grid.get(1, 1).unwrap().ch, 'A');

        // Unlock the layer
        canvas.set_layer_locked(0, false);
        assert!(!canvas.layer_locked(0));

        // Try clearing now that it is unlocked
        canvas.clear();
        // Cell should now be cleared
        assert!(!canvas.state.grid.get(1, 1).unwrap().is_visible());
    }

    #[test]
    fn test_reorder_layers_preserves_indices() {
        let mut canvas = AsciiEditor::new(10, 10);
        canvas.state.grid.set_char(0, 0, 'A');

        let _idx1 = canvas.add_layer_impl();
        canvas.state.grid.set_char(1, 1, 'B');

        let _idx2 = canvas.add_layer_impl();
        canvas.state.grid.set_char(2, 2, 'C');

        assert_eq!(canvas.layers.len(), 3);
        assert_eq!(canvas.active_layer, 2);

        // Move active layer (2) down to index 1
        canvas.move_layer(2, 1);
        assert_eq!(canvas.active_layer, 1);
        assert_eq!(canvas.layers[1].name, "Layer 3"); // C should now be at index 1
        assert_eq!(canvas.layers[2].name, "Layer 2"); // B should now be at index 2
    }

    #[test]
    fn test_delete_layer_prevents_deleting_last_layer() {
        let mut canvas = AsciiEditor::new(10, 10);
        assert_eq!(canvas.layers.len(), 1);

        // Try deleting layer 0 (the only layer)
        assert!(!canvas.delete_layer(0));
        assert_eq!(canvas.layers.len(), 1);

        // Add a layer and delete it
        canvas.add_layer_impl();
        assert_eq!(canvas.layers.len(), 2);
        assert!(canvas.delete_layer(1));
        assert_eq!(canvas.layers.len(), 1);
        assert_eq!(canvas.active_layer, 0);
    }

    #[test]
    fn test_merge_layer_down_composites_cells() {
        let mut canvas = AsciiEditor::new(10, 10);
        canvas.state.grid.set_char(0, 0, 'A');

        canvas.add_layer_impl();
        canvas.state.grid.set_char(1, 1, 'B');

        // Merge layer 1 down to layer 0
        assert!(canvas.merge_layer_down(1));
        assert_eq!(canvas.layers.len(), 1);
        assert_eq!(canvas.active_layer, 0);

        // Cell 'A' from bottom and 'B' from top should now both be in the bottom grid
        assert_eq!(canvas.state.grid.get(0, 0).unwrap().ch, 'A');
        assert_eq!(canvas.state.grid.get(1, 1).unwrap().ch, 'B');
    }

    #[test]
    fn test_layer_history_preservation_across_switches() {
        let mut canvas = AsciiEditor::new(10, 10);

        // Draw 'A' on Layer 0 (creates an undo step on Layer 0)
        use crate::core::tools::DrawOp;
        canvas.commit_ops(&[DrawOp::new(0, 0, 'A')]);
        assert_eq!(canvas.state.grid.get(0, 0).unwrap().ch, 'A');
        assert!(canvas.can_undo());

        // Add Layer 1 and draw 'B'
        canvas.add_layer_impl();
        canvas.commit_ops(&[DrawOp::new(1, 1, 'B')]);
        assert_eq!(canvas.state.grid.get(1, 1).unwrap().ch, 'B');
        assert!(canvas.can_undo());

        // Switch to Layer 0
        canvas.set_active_layer(0);
        assert_eq!(canvas.state.grid.get(0, 0).unwrap().ch, 'A');
        // Undo on Layer 0 should undo 'A' but keep 'B' on Layer 1 intact
        assert!(canvas.undo());
        assert_eq!(canvas.state.grid.get(0, 0).unwrap().ch, ' ');

        // Switch back to Layer 1 and verify 'B' is still there
        canvas.set_active_layer(1);
        assert_eq!(canvas.state.grid.get(1, 1).unwrap().ch, 'B');
    }

    #[test]
    fn test_paste_text_basic() {
        let mut canvas = AsciiEditor::new(10, 10);
        // Paste plain text at default origin (0, 0)
        assert!(canvas.paste_text_impl("HELLO"));
        assert_eq!(canvas.state.grid.get(0, 0).unwrap().ch, 'H');
        assert_eq!(canvas.state.grid.get(1, 0).unwrap().ch, 'E');
        assert_eq!(canvas.state.grid.get(2, 0).unwrap().ch, 'L');
        assert_eq!(canvas.state.grid.get(3, 0).unwrap().ch, 'L');
        assert_eq!(canvas.state.grid.get(4, 0).unwrap().ch, 'O');
    }

    #[test]
    fn test_paste_text_multiline_normalization() {
        let mut canvas = AsciiEditor::new(10, 10);
        // Paste multiline text with mixed LF/CRLF
        let content = "AB\nCD\r\nEF";
        assert!(canvas.paste_text_impl(content));

        // Row 0
        assert_eq!(canvas.state.grid.get(0, 0).unwrap().ch, 'A');
        assert_eq!(canvas.state.grid.get(1, 0).unwrap().ch, 'B');

        // Row 1
        assert_eq!(canvas.state.grid.get(0, 1).unwrap().ch, 'C');
        assert_eq!(canvas.state.grid.get(1, 1).unwrap().ch, 'D');

        // Row 2
        assert_eq!(canvas.state.grid.get(0, 2).unwrap().ch, 'E');
        assert_eq!(canvas.state.grid.get(1, 2).unwrap().ch, 'F');
    }

    #[test]
    fn test_paste_text_respect_bounds() {
        let mut canvas = AsciiEditor::new(10, 10);
        // Set selection at (8, 8) to set paste origin
        canvas.set_selection_for_test(8, 8, 8, 8);

        // Paste text that exceeds right and bottom boundaries
        assert!(canvas.paste_text_impl("XYZ\n123"));

        // At (8,8) and (9,8)
        assert_eq!(canvas.state.grid.get(8, 8).unwrap().ch, 'X');
        assert_eq!(canvas.state.grid.get(9, 8).unwrap().ch, 'Y');
        // 'Z' should be out of bounds at (10, 8), grid width is 10
        assert!(canvas.state.grid.get(10, 8).is_none());

        // At (8,9) and (9,9)
        assert_eq!(canvas.state.grid.get(8, 9).unwrap().ch, '1');
        assert_eq!(canvas.state.grid.get(9, 9).unwrap().ch, '2');
        // '3' should be out of bounds at (10, 9)
        assert!(canvas.state.grid.get(10, 9).is_none());
    }

    #[test]
    fn test_paste_text_do_not_clobber_with_spaces() {
        let mut canvas = AsciiEditor::new(10, 10);
        // Pre-fill canvas
        canvas.state.grid.set_char(0, 0, '1');
        canvas.state.grid.set_char(1, 0, '2');
        canvas.state.grid.set_char(2, 0, '3');
        canvas.state.grid.set_char(0, 1, '4');
        canvas.state.grid.set_char(1, 1, '5');
        canvas.state.grid.set_char(2, 1, '6');

        // Paste text with spaces: "A C" and a line with leading/trailing spaces
        let content = "A C\n B ";
        assert!(canvas.paste_text_impl(content));

        // Row 0: '1' is overwritten by 'A', '2' is NOT overwritten by space, '3' is overwritten by 'C'
        assert_eq!(canvas.state.grid.get(0, 0).unwrap().ch, 'A');
        assert_eq!(canvas.state.grid.get(1, 0).unwrap().ch, '2');
        assert_eq!(canvas.state.grid.get(2, 0).unwrap().ch, 'C');

        // Row 1: '4' is NOT overwritten by space, '5' is overwritten by 'B', '6' is NOT overwritten by space
        assert_eq!(canvas.state.grid.get(0, 1).unwrap().ch, '4');
        assert_eq!(canvas.state.grid.get(1, 1).unwrap().ch, 'B');
        assert_eq!(canvas.state.grid.get(2, 1).unwrap().ch, '6');
    }
}
