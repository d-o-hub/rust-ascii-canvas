//! WASM bindings - main entry point for JavaScript interop.

#![allow(missing_docs)]

use crate::core::commands::{Command, DrawCommand};
use crate::core::history::History;
use crate::core::selection::{Selection, SelectionClipboard};
use crate::core::tools::{DrawOp, RectangleTool, Tool, ToolContext, ToolId};
use crate::core::EditorState;
use crate::render::{CanvasRenderer, DirtyTracker, FontAtlas, FontMetrics};
use crate::wasm::render_bridge::{
    create_event_result, create_event_result_with_copy, export_ascii, get_dirty_render_commands,
    get_render_commands, get_render_commands_full, needs_redraw, request_full_redraw,
    EditorEventResult,
};
use crate::wasm::tool_manager::{
    parse_tool_id, set_border_style, set_line_direction, set_tool_by_id,
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct AsciiEditor {
    state: EditorState,
    history: History,
    renderer: CanvasRenderer,
    dirty_tracker: DirtyTracker,
    active_tool: Box<dyn Tool>,
    tool_id: ToolId,
    preview_ops: Vec<DrawOp>,
    current_selection: Option<Selection>,
    clipboard: SelectionClipboard,
    space_held: bool,
    is_panning: bool,
    last_pan_pos: Option<(f64, f64)>,
    /// Content and position when moving a selection
    move_clipboard: Option<SelectionClipboard>,
    move_original_selection: Option<Selection>,
    /// Track if select tool is currently moving
    is_moving_selection: bool,
    /// Performance: number of full renders
    full_render_count: u32,
    /// Performance: number of dirty renders
    dirty_render_count: u32,
    /// Pixel buffer for direct WASM-to-Canvas rendering
    pixel_buffer: Vec<u8>,
    /// Font atlas for pixel buffer rendering
    font_atlas: FontAtlas,
}

#[wasm_bindgen]
impl AsciiEditor {
    #[wasm_bindgen(constructor)]
    pub fn new(width: usize, height: usize) -> Self {
        let state = EditorState::new(width, height);
        let renderer = CanvasRenderer::new();

        Self {
            state,
            history: History::new(100),
            renderer,
            dirty_tracker: DirtyTracker::new(),
            active_tool: Box::new(RectangleTool::new()),
            tool_id: ToolId::Rectangle,
            preview_ops: Vec::new(),
            current_selection: None,
            clipboard: SelectionClipboard::new(),
            space_held: false,
            is_panning: false,
            last_pan_pos: None,
            move_clipboard: None,
            move_original_selection: None,
            is_moving_selection: false,
            full_render_count: 0,
            dirty_render_count: 0,
            pixel_buffer: vec![0u8; width * 8 * height * 20 * 4],
            font_atlas: FontAtlas::new(),
        }
    }

    /// Resize the grid, preserving existing content within the new bounds.
    #[wasm_bindgen]
    pub fn resize(&mut self, new_width: usize, new_height: usize) {
        self.state.grid.resize(new_width, new_height);
        self.pixel_buffer = vec![0u8; new_width * 8 * new_height * 20 * 4];
        self.dirty_tracker.request_full_redraw();
    }

    #[wasm_bindgen(getter)]
    pub fn width(&self) -> usize {
        self.state.grid.width()
    }

    #[wasm_bindgen(getter)]
    pub fn height(&self) -> usize {
        self.state.grid.height()
    }

    #[wasm_bindgen(getter)]
    pub fn tool(&self) -> String {
        self.tool_id.name().to_string()
    }

    #[wasm_bindgen(js_name = setTool)]
    pub fn set_tool(&mut self, tool_id: String) {
        if let Some(id) = parse_tool_id(&tool_id) {
            self.set_tool_by_id_impl(id);
        }
    }

    #[wasm_bindgen(js_name = setToolByShortcut)]
    pub fn set_tool_by_shortcut(&mut self, shortcut: char) -> bool {
        if let Some(id) = ToolId::from_shortcut(shortcut) {
            self.set_tool_by_id_impl(id);
            true
        } else {
            false
        }
    }

    fn set_tool_by_id_impl(&mut self, id: ToolId) {
        self.tool_id = id;
        set_tool_by_id(
            &mut self.active_tool,
            &mut self.tool_id,
            &mut self.preview_ops,
            &mut self.state,
            &mut self.current_selection,
        );
    }

    #[wasm_bindgen(js_name = setBorderStyle)]
    pub fn set_border_style(&mut self, style: String) {
        set_border_style(&mut self.state, style, &mut self.active_tool);
    }

    #[wasm_bindgen(js_name = setLineDirection)]
    pub fn set_line_direction(&mut self, direction: String) {
        set_line_direction(self.tool_id, &mut self.active_tool, direction);
    }

    #[wasm_bindgen(js_name = setZoom)]
    pub fn set_zoom(&mut self, zoom: f64) {
        self.renderer.set_zoom(zoom);
        self.dirty_tracker.request_full_redraw();
    }

    #[wasm_bindgen(getter)]
    pub fn zoom(&self) -> f64 {
        self.renderer.zoom()
    }

    #[wasm_bindgen(js_name = setPan)]
    pub fn set_pan(&mut self, x: f64, y: f64) {
        self.renderer.set_pan(x, y);
        self.dirty_tracker.request_full_redraw();
    }

    #[wasm_bindgen(getter = pan)]
    pub fn get_pan(&self) -> Vec<f64> {
        let (x, y) = self.renderer.pan();
        vec![x, y]
    }

    #[wasm_bindgen(js_name = setFontMetrics)]
    pub fn set_font_metrics(&mut self, char_width: f64, line_height: f64, size: f64) {
        let mut metrics = FontMetrics::new("JetBrains Mono, monospace", size);
        metrics.set_measured_width(char_width);
        metrics.line_height = line_height;
        self.renderer.set_metrics(metrics);
        self.dirty_tracker.request_full_redraw();
    }

    #[wasm_bindgen(js_name = onPointerDown)]
    pub fn on_pointer_down(&mut self, screen_x: f64, screen_y: f64) -> JsValue {
        if self.space_held {
            self.is_panning = true;
            self.last_pan_pos = Some((screen_x, screen_y));
            return JsValue::NULL;
        }

        let (x, y) = self.renderer.screen_to_grid(screen_x, screen_y);

        let ctx = self.create_tool_context();
        let result = self.active_tool.on_pointer_down(x, y, &ctx);

        // If select tool and clicking inside selection, start moving
        if self.tool_id == ToolId::Select {
            if let Some(ref sel) = self.current_selection {
                if sel.contains(x, y) {
                    self.is_moving_selection = true;
                    self.start_selection_move();
                } else {
                    self.is_moving_selection = false;
                }
            }
        }

        if self.is_incremental_tool() && result.modified {
            // For freehand/eraser, commit each stroke point immediately
            self.commit_ops(&result.ops);
            self.preview_ops.clear();
        } else {
            self.preview_ops = result.ops.clone();
            if !self.preview_ops.is_empty() {
                self.dirty_tracker.request_full_redraw();
            }
        }

        let event_result = self.create_event_result();
        serde_wasm_bindgen::to_value(&event_result).unwrap_or(JsValue::NULL)
    }

    #[wasm_bindgen(js_name = onPointerMove)]
    pub fn on_pointer_move(&mut self, screen_x: f64, screen_y: f64) -> JsValue {
        if self.is_panning {
            if let Some((lx, ly)) = self.last_pan_pos {
                let dx = screen_x - lx;
                let dy = screen_y - ly;
                let (px, py) = self.renderer.pan();
                self.renderer.set_pan(px + dx, py + dy);
                self.dirty_tracker.request_full_redraw();
            }
            self.last_pan_pos = Some((screen_x, screen_y));
            let event_result = self.create_event_result();
            return serde_wasm_bindgen::to_value(&event_result).unwrap_or(JsValue::NULL);
        }

        let (x, y) = self.renderer.screen_to_grid(screen_x, screen_y);

        let ctx = self.create_tool_context();
        let result = self.active_tool.on_pointer_move(x, y, &ctx);

        if self.is_incremental_tool() && result.modified {
            // For freehand/eraser, commit each stroke segment immediately
            self.commit_ops(&result.ops);
            self.preview_ops.clear();
        } else if !result.ops.is_empty() {
            // For shape tools (rect, line, etc.), store as preview overlay
            self.preview_ops = result.ops.clone();
            self.dirty_tracker.request_full_redraw();
        }

        // Update selection during select tool drag (triggers redraw for highlight)
        if self.tool_id == ToolId::Select {
            self.update_select_tool_selection();

            // If moving, generate preview ops
            if self.is_select_moving() {
                self.preview_ops = self.generate_move_preview_ops();
            }

            if self.current_selection.is_some() || !self.preview_ops.is_empty() {
                self.dirty_tracker.request_full_redraw();
            }
        }

        let event_result = self.create_event_result();
        serde_wasm_bindgen::to_value(&event_result).unwrap_or(JsValue::NULL)
    }

    #[wasm_bindgen(js_name = onPointerUp)]
    pub fn on_pointer_up(&mut self, screen_x: f64, screen_y: f64) -> JsValue {
        if self.is_panning {
            self.is_panning = false;
            self.last_pan_pos = None;
            let event_result = self.create_event_result();
            return serde_wasm_bindgen::to_value(&event_result).unwrap_or(JsValue::NULL);
        }

        let (x, y) = self.renderer.screen_to_grid(screen_x, screen_y);

        let ctx = self.create_tool_context();
        let result = self.active_tool.on_pointer_up(x, y, &ctx);

        self.preview_ops.clear();

        if self.tool_id == ToolId::Select {
            // If we were moving, commit the move
            if self.is_moving_selection {
                self.commit_selection_move();
                self.is_moving_selection = false;
            }

            self.update_select_tool_selection();
            // Always redraw to show/update selection highlight
            self.dirty_tracker.request_full_redraw();
        }

        if result.modified {
            self.commit_ops(&result.ops);
        }

        let event_result = self.create_event_result();
        serde_wasm_bindgen::to_value(&event_result).unwrap_or(JsValue::NULL)
    }

    fn update_select_tool_selection(&mut self) {
        self.current_selection = self.active_tool.get_selection();
    }

    #[wasm_bindgen(js_name = onKeyDown)]
    pub fn on_key_down(&mut self, key: String, ctrl: bool, shift: bool) -> JsValue {
        let key_char = if key.len() == 1 {
            key.chars().next().unwrap_or('\0')
        } else {
            match key.as_str() {
                "Enter" => '\n',
                "Backspace" => '\x08',
                "Delete" => '\0',
                "Tab" => '\t',
                _ => '\0',
            }
        };

        if key == "Escape" {
            self.active_tool.reset();
            self.preview_ops.clear();
            let event_result = self.create_event_result();
            return serde_wasm_bindgen::to_value(&event_result).unwrap_or(JsValue::NULL);
        }

        if key_char == ' ' && !ctrl && !shift {
            self.space_held = true;
            let event_result = self.create_event_result();
            return serde_wasm_bindgen::to_value(&event_result).unwrap_or(JsValue::NULL);
        }

        if ctrl && !shift && key.to_lowercase() == "z" {
            self.undo();
            let event_result = self.create_event_result();
            return serde_wasm_bindgen::to_value(&event_result).unwrap_or(JsValue::NULL);
        }

        if ctrl && shift && key.to_lowercase() == "z" {
            self.redo();
            let event_result = self.create_event_result();
            return serde_wasm_bindgen::to_value(&event_result).unwrap_or(JsValue::NULL);
        }

        if ctrl && key.to_lowercase() == "c" {
            let event_result = self.create_event_result_with_copy(true);
            return serde_wasm_bindgen::to_value(&event_result).unwrap_or(JsValue::NULL);
        }

        if ctrl && key.to_lowercase() == "x" && self.cut_selection() {
            let event_result = self.create_event_result();
            return serde_wasm_bindgen::to_value(&event_result).unwrap_or(JsValue::NULL);
        }

        if ctrl && key.to_lowercase() == "v" && self.paste() {
            let event_result = self.create_event_result();
            return serde_wasm_bindgen::to_value(&event_result).unwrap_or(JsValue::NULL);
        }

        if !ctrl && (key == "Delete" || key == "Backspace") {
            if self.tool_id == ToolId::Select
                && self.current_selection.is_some()
                && self.delete_selection()
            {
                let event_result = self.create_event_result();
                return serde_wasm_bindgen::to_value(&event_result).unwrap_or(JsValue::NULL);
            }
            if self.tool_id == ToolId::Text && self.active_tool.is_active() {
                let ctx = self.create_tool_context();
                let delete_char = if key == "Delete" { '\0' } else { '\x08' };
                let result = self.active_tool.on_key(delete_char, &ctx);
                if result.modified {
                    self.commit_ops(&result.ops);
                }
                let event_result = self.create_event_result();
                return serde_wasm_bindgen::to_value(&event_result).unwrap_or(JsValue::NULL);
            }
        }

        if !ctrl && !shift && !self.active_tool.is_active() {
            if let Some(tool_id) = ToolId::from_shortcut(key_char) {
                self.set_tool_by_id_impl(tool_id);
                let event_result = self.create_event_result();
                return serde_wasm_bindgen::to_value(&event_result).unwrap_or(JsValue::NULL);
            }
        }

        if self.tool_id == ToolId::Text && self.active_tool.is_active() {
            let ctx = self.create_tool_context();
            let result = self.active_tool.on_key(key_char, &ctx);
            if result.modified {
                self.commit_ops(&result.ops);
            }
        }

        let event_result = self.create_event_result();
        serde_wasm_bindgen::to_value(&event_result).unwrap_or(JsValue::NULL)
    }

    #[wasm_bindgen(js_name = onKeyUp)]
    pub fn on_key_up(&mut self, key: String) {
        if key == " " {
            self.space_held = false;
            self.is_panning = false;
        }
    }

    #[wasm_bindgen(js_name = onWheel)]
    pub fn on_wheel(&mut self, delta: f64, screen_x: f64, screen_y: f64) -> JsValue {
        let zoom_factor = if delta > 0.0 { 0.9 } else { 1.1 };
        let new_zoom = self.renderer.zoom() * zoom_factor;

        let (px, py) = self.renderer.pan();
        let zoom_ratio = new_zoom / self.renderer.zoom();

        let new_pan_x = screen_x - (screen_x - px) * zoom_ratio;
        let new_pan_y = screen_y - (screen_y - py) * zoom_ratio;

        self.renderer.set_zoom(new_zoom);
        self.renderer.set_pan(new_pan_x, new_pan_y);
        self.dirty_tracker.request_full_redraw();

        let event_result = self.create_event_result();
        serde_wasm_bindgen::to_value(&event_result).unwrap_or(JsValue::NULL)
    }

    #[wasm_bindgen]
    pub fn undo(&mut self) -> bool {
        let result = self.history.undo(&mut self.state.grid);
        if result {
            self.dirty_tracker.request_full_redraw();
        }
        result
    }

    #[wasm_bindgen]
    pub fn redo(&mut self) -> bool {
        let result = self.history.redo(&mut self.state.grid);
        if result {
            self.dirty_tracker.request_full_redraw();
        }
        result
    }

    #[wasm_bindgen(getter)]
    pub fn can_undo(&self) -> bool {
        self.history.can_undo()
    }

    #[wasm_bindgen(getter)]
    pub fn can_redo(&self) -> bool {
        self.history.can_redo()
    }

    #[wasm_bindgen]
    pub fn clear(&mut self) {
        self.state.grid.clear();
        self.history.clear();
        self.clipboard.clear();
        self.dirty_tracker.request_full_redraw();
    }

    #[wasm_bindgen(js_name = copySelection)]
    pub fn copy_selection(&mut self) -> bool {
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

    #[wasm_bindgen(js_name = cutSelection)]
    pub fn cut_selection(&mut self) -> bool {
        if !self.copy_selection() {
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

    #[wasm_bindgen]
    pub fn paste(&mut self) -> bool {
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

    #[wasm_bindgen(js_name = deleteSelection)]
    pub fn delete_selection(&mut self) -> bool {
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

    #[wasm_bindgen(getter)]
    pub fn has_clipboard(&self) -> bool {
        !self.clipboard.is_empty()
    }

    #[wasm_bindgen(getter)]
    pub fn has_selection(&self) -> bool {
        self.current_selection.is_some()
    }

    #[wasm_bindgen(js_name = exportAscii)]
    pub fn export_ascii(&self) -> String {
        export_ascii(&self.state.grid)
    }

    #[wasm_bindgen(js_name = getRenderCommands)]
    pub fn get_render_commands(&mut self) -> JsValue {
        self.full_render_count += 1;
        let has_preview = !self.preview_ops.is_empty();
        let has_selection = self.current_selection.is_some();

        if !has_preview && !has_selection {
            get_render_commands(&self.renderer, &self.state.grid)
        } else {
            get_render_commands_full(
                &self.renderer,
                &self.state.grid,
                &self.preview_ops,
                self.current_selection.as_ref(),
            )
        }
    }

    #[wasm_bindgen(js_name = getDirtyRenderCommands)]
    pub fn get_dirty_render_commands(&mut self) -> JsValue {
        if self.dirty_tracker.needs_full_redraw() {
            self.full_render_count += 1;
        } else if !self.dirty_tracker.dirty_rect().is_empty() {
            self.dirty_render_count += 1;
        }

        get_dirty_render_commands(
            &mut self.renderer,
            &self.state.grid,
            &mut self.dirty_tracker,
        )
    }

    #[wasm_bindgen(getter = fullRenderCount)]
    pub fn full_render_count(&self) -> u32 {
        self.full_render_count
    }

    #[wasm_bindgen(getter = dirtyRenderCount)]
    pub fn dirty_render_count(&self) -> u32 {
        self.dirty_render_count
    }

    #[wasm_bindgen(js_name = resetPerformanceMetrics)]
    pub fn reset_performance_metrics(&mut self) {
        self.full_render_count = 0;
        self.dirty_render_count = 0;
    }

    /// Check if a redraw is needed.
    #[wasm_bindgen(getter = needsRedraw)]
    pub fn needs_redraw(&self) -> bool {
        needs_redraw(&self.dirty_tracker)
    }

    /// Request a full redraw on the next frame.
    #[wasm_bindgen(js_name = requestRedraw)]
    pub fn request_redraw(&mut self) {
        request_full_redraw(&mut self.dirty_tracker);
    }

    /// Clear dirty state after rendering.
    #[wasm_bindgen(js_name = clearDirtyState)]
    pub fn clear_dirty_state(&mut self) {
        self.dirty_tracker.clear();
    }

    #[wasm_bindgen(js_name = getPixelBufferPtr)]
    pub fn get_pixel_buffer_ptr(&self) -> *const u8 {
        self.pixel_buffer.as_ptr()
    }

    #[wasm_bindgen(js_name = getPixelBufferLen)]
    pub fn get_pixel_buffer_len(&self) -> usize {
        self.pixel_buffer.len()
    }

    #[wasm_bindgen(js_name = renderToPixelBuffer)]
    pub fn render_to_pixel_buffer(&mut self) {
        let grid_width = self.state.grid.width();
        let grid_height = self.state.grid.height();
        let glyph_w = 8;
        let glyph_h = 20;
        let buffer_width = grid_width * glyph_w;
        let buffer_height = grid_height * glyph_h;

        // Resize pixel buffer if needed
        let required_len = buffer_width * buffer_height * 4;
        if self.pixel_buffer.len() != required_len {
            self.pixel_buffer.resize(required_len, 0);
        }

        // Fill with background color
        let bg_color = [30, 30, 30, 255]; // Matching --bg: #1e1e1e
        for i in 0..buffer_width * buffer_height {
            let idx = i * 4;
            self.pixel_buffer[idx] = bg_color[0];
            self.pixel_buffer[idx + 1] = bg_color[1];
            self.pixel_buffer[idx + 2] = bg_color[2];
            self.pixel_buffer[idx + 3] = bg_color[3];
        }

        let fg_color = [212, 212, 212, 255]; // Matching --fg: #d4d4d4

        // Draw selection highlight
        if let Some(ref sel) = self.current_selection {
            let (min_x, min_y, max_x, max_y) = sel.bounds();
            let highlight_color = [38, 79, 120, 255]; // #264f78

            for gy in min_y..=max_y {
                for gx in min_x..=max_x {
                    if self.state.grid.in_bounds(gx, gy) {
                        let sx = gx as usize * glyph_w;
                        let sy = gy as usize * glyph_h;

                        for y in 0..glyph_h {
                            let buffer_y = sy + y;
                            let buffer_row_start = (buffer_y * buffer_width + sx) * 4;
                            for x in 0..glyph_w {
                                let pixel_idx = buffer_row_start + x * 4;
                                self.pixel_buffer[pixel_idx] = highlight_color[0];
                                self.pixel_buffer[pixel_idx + 1] = highlight_color[1];
                                self.pixel_buffer[pixel_idx + 2] = highlight_color[2];
                                self.pixel_buffer[pixel_idx + 3] = highlight_color[3];
                            }
                        }
                    }
                }
            }
        }

        // Draw cells from grid
        for (x, y, cell) in self.state.grid.iter_with_coords() {
            if cell.is_visible() {
                self.font_atlas.render_glyph(
                    &mut self.pixel_buffer,
                    buffer_width,
                    x as usize * glyph_w,
                    y as usize * glyph_h,
                    cell.ch,
                    fg_color,
                );
            }
        }

        // Draw preview overlay
        for op in &self.preview_ops {
            if op.cell.is_visible() {
                self.font_atlas.render_glyph(
                    &mut self.pixel_buffer,
                    buffer_width,
                    op.x as usize * glyph_w,
                    op.y as usize * glyph_h,
                    op.cell.ch,
                    fg_color,
                );
            }
        }
    }
}

impl AsciiEditor {
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

    /// Capture the current selection content before starting a move.
    fn start_selection_move(&mut self) {
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

    /// Generate preview ops for the current move operation.
    fn generate_move_preview_ops(&self) -> Vec<DrawOp> {
        let mut ops = Vec::new();

        if let (Some(ref orig_sel), Some(ref curr_sel), Some(ref move_clip)) = (
            &self.move_original_selection,
            &self.current_selection,
            &self.move_clipboard,
        ) {
            let (orig_x, orig_y, orig_x2, orig_y2) = orig_sel.bounds();
            let (curr_x, curr_y, _, _) = curr_sel.bounds();

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

                    if self.state.grid.in_bounds(new_x, new_y) {
                        ops.push(DrawOp::new(new_x, new_y, cell.ch));
                    }
                }
            }
        }

        ops
    }

    /// Commit the move operation.
    fn commit_selection_move(&mut self) {
        let ops = self.generate_move_preview_ops();

        if !ops.is_empty() {
            self.commit_ops(&ops);
        }

        // Clear move state
        self.move_clipboard = None;
        self.move_original_selection = None;
    }

    fn create_event_result(&self) -> EditorEventResult {
        create_event_result(
            self.needs_redraw(),
            self.tool_id.name(),
            self.can_undo(),
            self.can_redo(),
        )
    }

    fn create_event_result_with_copy(&self, should_copy: bool) -> EditorEventResult {
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
}
