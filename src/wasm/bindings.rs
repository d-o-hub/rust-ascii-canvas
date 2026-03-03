//! WASM bindings - main entry point for JavaScript interop.

use crate::core::commands::{Command, DrawCommand};
use crate::core::history::History;
use crate::core::selection::{Selection, SelectionClipboard};
use crate::core::tools::{DrawOp, RectangleTool, Tool, ToolContext, ToolId};
use crate::core::EditorState;
use crate::render::{CanvasRenderer, DirtyTracker, FontMetrics};
use crate::wasm::render_bridge::{
    create_event_result, create_event_result_with_copy, export_ascii, get_dirty_render_commands,
    get_render_commands, needs_redraw, request_full_redraw, EditorEventResult,
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
        }
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

    fn set_tool_by_id_impl(&mut self, _id: ToolId) {
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
        set_border_style(&mut self.state, style);
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

        self.preview_ops = result.ops.clone();

        if result.finished && result.modified {
            self.commit_ops(&result.ops);
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

        self.preview_ops = result.ops.clone();

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
            self.update_select_tool_selection();
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
        let key_char = key.chars().next().unwrap_or('\0');

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
                && self.active_tool.is_active()
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
    pub fn get_render_commands(&self) -> JsValue {
        get_render_commands(&self.renderer, &self.state.grid)
    }

    #[wasm_bindgen(js_name = getDirtyRenderCommands)]
    pub fn get_dirty_render_commands(&mut self) -> JsValue {
        get_dirty_render_commands(
            &mut self.renderer,
            &self.state.grid,
            &mut self.dirty_tracker,
        )
    }

    #[wasm_bindgen(getter = needsRedraw)]
    pub fn needs_redraw(&self) -> bool {
        needs_redraw(&self.dirty_tracker)
    }

    #[wasm_bindgen(js_name = requestRedraw)]
    pub fn request_redraw(&mut self) {
        request_full_redraw(&mut self.dirty_tracker);
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
