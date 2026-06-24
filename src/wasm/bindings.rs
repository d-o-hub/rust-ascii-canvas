//! WASM bindings - struct definition, constructor, and core methods.

#![allow(missing_docs)]

use crate::core::history::History;
use crate::core::selection::{Selection, SelectionClipboard};
use crate::core::tools::{DrawOp, RectangleTool, Tool, ToolId};
use crate::core::EditorState;
use crate::render::{CanvasRenderer, DirtyTracker, FontAtlas, FontMetrics};
use crate::wasm::tool_manager::{
    parse_tool_id, set_border_style, set_line_direction, set_tool_by_id,
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct AsciiEditor {
    pub(crate) state: EditorState,
    pub(crate) history: History,
    pub(crate) renderer: CanvasRenderer,
    pub(crate) dirty_tracker: DirtyTracker,
    pub(crate) active_tool: Box<dyn Tool>,
    pub(crate) tool_id: ToolId,
    pub(crate) preview_ops: Vec<DrawOp>,
    pub(crate) current_selection: Option<Selection>,
    pub(crate) clipboard: SelectionClipboard,
    pub(crate) space_held: bool,
    pub(crate) is_panning: bool,
    pub(crate) last_pan_pos: Option<(f64, f64)>,
    pub(crate) move_clipboard: Option<SelectionClipboard>,
    pub(crate) move_original_selection: Option<Selection>,
    pub(crate) is_moving_selection: bool,
    pub(crate) full_render_count: u32,
    pub(crate) dirty_render_count: u32,
    pub(crate) pixel_buffer: Vec<u8>,
    pub(crate) font_atlas: FontAtlas,
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
            self.tool_id = id;
            set_tool_by_id(
                &mut self.active_tool,
                &mut self.tool_id,
                &mut self.preview_ops,
                &mut self.state,
                &mut self.current_selection,
            );
        }
    }

    #[wasm_bindgen(js_name = setToolByShortcut)]
    pub fn set_tool_by_shortcut(&mut self, shortcut: char) -> bool {
        if let Some(id) = ToolId::from_shortcut(shortcut) {
            self.tool_id = id;
            set_tool_by_id(
                &mut self.active_tool,
                &mut self.tool_id,
                &mut self.preview_ops,
                &mut self.state,
                &mut self.current_selection,
            );
            true
        } else {
            false
        }
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
}
