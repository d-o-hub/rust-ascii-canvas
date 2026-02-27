//! WASM bindings - main entry point for JavaScript interop.

use crate::core::ascii_export::{export_grid, ExportOptions};
use crate::core::commands::{Command, DrawCommand};
use crate::core::history::History;
use crate::core::selection::{Selection, SelectionClipboard};
use crate::core::tools::{
    ArrowTool, BorderStyle, DiamondTool, DrawOp, EraserTool, FreehandTool, LineTool, RectangleTool,
    SelectTool, TextTool, Tool, ToolContext, ToolId,
};
use crate::core::EditorState;
use crate::render::{CanvasRenderer, DirtyTracker, FontMetrics};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;

/// Main ASCII editor class exposed to JavaScript.
#[wasm_bindgen]
pub struct AsciiEditor {
    /// Editor state
    state: EditorState,
    /// History for undo/redo
    history: History,
    /// Canvas renderer
    renderer: CanvasRenderer,
    /// Dirty region tracker
    dirty_tracker: DirtyTracker,
    /// Active tool
    active_tool: Box<dyn Tool>,
    /// Select tool shared reference for copy/paste operations
    select_tool: Option<Rc<RefCell<SelectTool>>>,
    /// Currently active tool ID
    tool_id: ToolId,
    /// Preview operations (shown during drag but not committed)
    preview_ops: Vec<DrawOp>,
    /// Current selection (for copy/paste)
    current_selection: Option<Selection>,
    /// Internal clipboard for copy/paste
    clipboard: SelectionClipboard,
    /// Space key held for panning
    space_held: bool,
    /// Currently panning
    is_panning: bool,
    /// Last pan position
    last_pan_pos: Option<(f64, f64)>,
}

#[wasm_bindgen]
impl AsciiEditor {
    /// Create a new ASCII editor.
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
            select_tool: None,
            tool_id: ToolId::Rectangle,
            preview_ops: Vec::new(),
            current_selection: None,
            clipboard: SelectionClipboard::new(),
            space_held: false,
            is_panning: false,
            last_pan_pos: None,
        }
    }

    /// Get grid width.
    #[wasm_bindgen(getter)]
    pub fn width(&self) -> usize {
        self.state.grid.width()
    }

    /// Get grid height.
    #[wasm_bindgen(getter)]
    pub fn height(&self) -> usize {
        self.state.grid.height()
    }

    /// Get current tool ID.
    #[wasm_bindgen(getter)]
    pub fn tool(&self) -> String {
        self.tool_id.name().to_string()
    }

    /// Set the current tool by ID.
    #[wasm_bindgen(js_name = setTool)]
    pub fn set_tool(&mut self, tool_id: String) {
        if let Some(id) = self.parse_tool_id(&tool_id) {
            self.set_tool_by_id(id);
        }
    }

    /// Set tool by keyboard shortcut.
    #[wasm_bindgen(js_name = setToolByShortcut)]
    pub fn set_tool_by_shortcut(&mut self, shortcut: char) -> bool {
        if let Some(id) = ToolId::from_shortcut(shortcut) {
            self.set_tool_by_id(id);
            true
        } else {
            false
        }
    }

    fn parse_tool_id(&self, s: &str) -> Option<ToolId> {
        match s.to_lowercase().as_str() {
            "rectangle" | "rect" | "r" => Some(ToolId::Rectangle),
            "line" | "l" => Some(ToolId::Line),
            "arrow" | "a" => Some(ToolId::Arrow),
            "diamond" | "d" => Some(ToolId::Diamond),
            "text" | "t" => Some(ToolId::Text),
            "freehand" | "f" => Some(ToolId::Freehand),
            "select" | "v" => Some(ToolId::Select),
            "eraser" | "e" => Some(ToolId::Eraser),
            _ => None,
        }
    }

    fn set_tool_by_id(&mut self, id: ToolId) {
        self.active_tool.reset();
        self.preview_ops.clear();
        self.tool_id = id;
        self.state.tool = id;

        if id != ToolId::Select {
            self.current_selection = None;
            self.select_tool = None;
        }

        match id {
            ToolId::Rectangle => {
                self.active_tool = Box::new(RectangleTool::new());
            }
            ToolId::Line => {
                self.active_tool = Box::new(LineTool::new());
            }
            ToolId::Arrow => {
                self.active_tool = Box::new(ArrowTool::new());
            }
            ToolId::Diamond => {
                self.active_tool = Box::new(DiamondTool::new());
            }
            ToolId::Text => {
                self.active_tool = Box::new(TextTool::new());
            }
            ToolId::Freehand => {
                self.active_tool = Box::new(FreehandTool::new());
            }
            ToolId::Select => {
                let tool = Rc::new(RefCell::new(SelectTool::new()));
                self.select_tool = Some(Rc::clone(&tool));
                self.active_tool = Box::new(SelectTool::new());
            }
            ToolId::Eraser => {
                self.active_tool = Box::new(EraserTool::new());
            }
        }
    }

    /// Set border style for shapes.
    #[wasm_bindgen(js_name = setBorderStyle)]
    pub fn set_border_style(&mut self, style: String) {
        let style = match style.to_lowercase().as_str() {
            "single" => BorderStyle::Single,
            "double" => BorderStyle::Double,
            "heavy" => BorderStyle::Heavy,
            "rounded" => BorderStyle::Rounded,
            "ascii" => BorderStyle::Ascii,
            "dotted" => BorderStyle::Dotted,
            _ => BorderStyle::Single,
        };
        self.state.border_style = style;
    }

    /// Set zoom level.
    #[wasm_bindgen(js_name = setZoom)]
    pub fn set_zoom(&mut self, zoom: f64) {
        self.renderer.set_zoom(zoom);
        self.dirty_tracker.request_full_redraw();
    }

    /// Get zoom level.
    #[wasm_bindgen(getter)]
    pub fn zoom(&self) -> f64 {
        self.renderer.zoom()
    }

    /// Set pan offset.
    #[wasm_bindgen(js_name = setPan)]
    pub fn set_pan(&mut self, x: f64, y: f64) {
        self.renderer.set_pan(x, y);
        self.dirty_tracker.request_full_redraw();
    }

    /// Get pan offset.
    #[wasm_bindgen(getter = pan)]
    pub fn get_pan(&self) -> Vec<f64> {
        let (x, y) = self.renderer.pan();
        vec![x, y]
    }

    /// Set font metrics from measured values.
    #[wasm_bindgen(js_name = setFontMetrics)]
    pub fn set_font_metrics(&mut self, char_width: f64, line_height: f64, size: f64) {
        let mut metrics = FontMetrics::new("JetBrains Mono, monospace", size);
        metrics.set_measured_width(char_width);
        metrics.line_height = line_height;
        self.renderer.set_metrics(metrics);
        self.dirty_tracker.request_full_redraw();
    }

    /// Handle pointer down event.
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

        serde_wasm_bindgen::to_value(&self.create_event_result()).unwrap_or(JsValue::NULL)
    }

    /// Handle pointer move event.
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
            return serde_wasm_bindgen::to_value(&self.create_event_result())
                .unwrap_or(JsValue::NULL);
        }

        let (x, y) = self.renderer.screen_to_grid(screen_x, screen_y);

        let ctx = self.create_tool_context();
        let result = self.active_tool.on_pointer_move(x, y, &ctx);

        self.preview_ops = result.ops.clone();

        serde_wasm_bindgen::to_value(&self.create_event_result()).unwrap_or(JsValue::NULL)
    }

    /// Handle pointer up event.
    #[wasm_bindgen(js_name = onPointerUp)]
    pub fn on_pointer_up(&mut self, screen_x: f64, screen_y: f64) -> JsValue {
        if self.is_panning {
            self.is_panning = false;
            self.last_pan_pos = None;
            return serde_wasm_bindgen::to_value(&self.create_event_result())
                .unwrap_or(JsValue::NULL);
        }

        let (x, y) = self.renderer.screen_to_grid(screen_x, screen_y);

        let ctx = self.create_tool_context();
        let result = self.active_tool.on_pointer_up(x, y, &ctx);

        self.preview_ops.clear();

        // Update current selection for Select tool after operation
        if self.tool_id == ToolId::Select {
            self.update_select_tool_selection();
        }

        if result.modified {
            self.commit_ops(&result.ops);
        }

        serde_wasm_bindgen::to_value(&self.create_event_result()).unwrap_or(JsValue::NULL)
    }

    fn update_select_tool_selection(&mut self) {
        self.current_selection = self.active_tool.get_selection();
    }

    /// Handle keyboard input.
    #[wasm_bindgen(js_name = onKeyDown)]
    pub fn on_key_down(&mut self, key: String, ctrl: bool, shift: bool) -> JsValue {
        let key_char = key.chars().next().unwrap_or('\0');

        // Handle escape to cancel active operations
        if key == "Escape" {
            self.active_tool.reset();
            self.preview_ops.clear();
            return serde_wasm_bindgen::to_value(&self.create_event_result())
                .unwrap_or(JsValue::NULL);
        }

        // Handle space for panning
        if key_char == ' ' && !ctrl && !shift {
            self.space_held = true;
            return serde_wasm_bindgen::to_value(&self.create_event_result())
                .unwrap_or(JsValue::NULL);
        }

        // Handle undo/redo
        if ctrl && !shift && key.to_lowercase() == "z" {
            self.undo();
            return serde_wasm_bindgen::to_value(&self.create_event_result())
                .unwrap_or(JsValue::NULL);
        }

        if ctrl && shift && key.to_lowercase() == "z" {
            self.redo();
            return serde_wasm_bindgen::to_value(&self.create_event_result())
                .unwrap_or(JsValue::NULL);
        }

        // Handle copy (Ctrl+C) - copy to system clipboard
        if ctrl && key.to_lowercase() == "c" {
            return serde_wasm_bindgen::to_value(&self.create_event_result_with_copy(true))
                .unwrap_or(JsValue::NULL);
        }

        // Handle cut (Ctrl+X) - copy selection to clipboard and clear
        if ctrl && key.to_lowercase() == "x" {
            if self.cut_selection() {
                return serde_wasm_bindgen::to_value(&self.create_event_result())
                    .unwrap_or(JsValue::NULL);
            }
        }

        // Handle paste (Ctrl+V) - paste from internal clipboard
        if ctrl && key.to_lowercase() == "v" {
            if self.paste() {
                return serde_wasm_bindgen::to_value(&self.create_event_result())
                    .unwrap_or(JsValue::NULL);
            }
        }

        // Handle delete (Delete/Backspace) - delete selection or character
        if !ctrl && (key == "Delete" || key == "Backspace") {
            // For Select tool with active selection, delete the selection
            if self.tool_id == ToolId::Select && self.active_tool.is_active() {
                if self.delete_selection() {
                    return serde_wasm_bindgen::to_value(&self.create_event_result())
                        .unwrap_or(JsValue::NULL);
                }
            }
            // For Text tool, handle backspace/delete in the tool
            if self.tool_id == ToolId::Text && self.active_tool.is_active() {
                let ctx = self.create_tool_context();
                let delete_char = if key == "Delete" { '\0' } else { '\x08' };
                let result = self.active_tool.on_key(delete_char, &ctx);
                if result.modified {
                    self.commit_ops(&result.ops);
                }
                return serde_wasm_bindgen::to_value(&self.create_event_result())
                    .unwrap_or(JsValue::NULL);
            }
        }

        // Handle tool shortcuts only when tool is not active
        // This prevents shortcuts from interrupting text input, selection, etc.
        if !ctrl && !shift && !self.active_tool.is_active() {
            if let Some(tool_id) = ToolId::from_shortcut(key_char) {
                self.set_tool_by_id(tool_id);
                return serde_wasm_bindgen::to_value(&self.create_event_result())
                    .unwrap_or(JsValue::NULL);
            }
        }

        // Handle text input
        if self.tool_id == ToolId::Text && self.active_tool.is_active() {
            let ctx = self.create_tool_context();
            let result = self.active_tool.on_key(key_char, &ctx);
            if result.modified {
                self.commit_ops(&result.ops);
            }
        }

        serde_wasm_bindgen::to_value(&self.create_event_result()).unwrap_or(JsValue::NULL)
    }

    /// Handle key up event.
    #[wasm_bindgen(js_name = onKeyUp)]
    pub fn on_key_up(&mut self, key: String) {
        if key == " " {
            self.space_held = false;
            self.is_panning = false;
        }
    }

    /// Handle mouse wheel for zoom.
    #[wasm_bindgen(js_name = onWheel)]
    pub fn on_wheel(&mut self, delta: f64, screen_x: f64, screen_y: f64) -> JsValue {
        // Calculate new zoom
        let zoom_factor = if delta > 0.0 { 0.9 } else { 1.1 };
        let new_zoom = self.renderer.zoom() * zoom_factor;

        // Adjust pan to zoom toward cursor position
        let (px, py) = self.renderer.pan();
        let zoom_ratio = new_zoom / self.renderer.zoom();

        let new_pan_x = screen_x - (screen_x - px) * zoom_ratio;
        let new_pan_y = screen_y - (screen_y - py) * zoom_ratio;

        self.renderer.set_zoom(new_zoom);
        self.renderer.set_pan(new_pan_x, new_pan_y);
        self.dirty_tracker.request_full_redraw();

        serde_wasm_bindgen::to_value(&self.create_event_result()).unwrap_or(JsValue::NULL)
    }

    /// Perform undo.
    #[wasm_bindgen]
    pub fn undo(&mut self) -> bool {
        let result = self.history.undo(&mut self.state.grid);
        if result {
            self.dirty_tracker.request_full_redraw();
        }
        result
    }

    /// Perform redo.
    #[wasm_bindgen]
    pub fn redo(&mut self) -> bool {
        let result = self.history.redo(&mut self.state.grid);
        if result {
            self.dirty_tracker.request_full_redraw();
        }
        result
    }

    /// Check if undo is available.
    #[wasm_bindgen(getter)]
    pub fn can_undo(&self) -> bool {
        self.history.can_undo()
    }

    /// Check if redo is available.
    #[wasm_bindgen(getter)]
    pub fn can_redo(&self) -> bool {
        self.history.can_redo()
    }

    /// Clear the canvas.
    #[wasm_bindgen]
    pub fn clear(&mut self) {
        self.state.grid.clear();
        self.history.clear();
        self.clipboard.clear();
        self.dirty_tracker.request_full_redraw();
    }

    /// Copy selection to internal clipboard.
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

    /// Cut selection to internal clipboard.
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

    /// Paste from internal clipboard.
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

    /// Delete current selection.
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

    /// Check if clipboard has content.
    #[wasm_bindgen(getter)]
    pub fn has_clipboard(&self) -> bool {
        !self.clipboard.is_empty()
    }

    /// Check if there is an active selection.
    #[wasm_bindgen(getter)]
    pub fn has_selection(&self) -> bool {
        self.current_selection.is_some()
    }

    /// Export ASCII to string.
    #[wasm_bindgen(js_name = exportAscii)]
    pub fn export_ascii(&self) -> String {
        let options = ExportOptions::default();
        export_grid(&self.state.grid, &options)
    }

    /// Get render commands for JavaScript.
    #[wasm_bindgen(js_name = getRenderCommands)]
    pub fn get_render_commands(&self) -> JsValue {
        let commands = self.renderer.build_full_render(&self.state.grid);
        serde_wasm_bindgen::to_value(&commands).unwrap_or(JsValue::NULL)
    }

    /// Get dirty render commands.
    #[wasm_bindgen(js_name = getDirtyRenderCommands)]
    pub fn get_dirty_render_commands(&mut self) -> JsValue {
        if self.dirty_tracker.needs_full_redraw() {
            self.dirty_tracker.clear();
            return self.get_render_commands();
        }

        let dirty = *self.dirty_tracker.dirty_rect();
        self.dirty_tracker.clear();

        let commands = self
            .renderer
            .build_render_commands(&self.state.grid, &dirty);
        serde_wasm_bindgen::to_value(&commands).unwrap_or(JsValue::NULL)
    }

    /// Check if full redraw is needed.
    #[wasm_bindgen(getter = needsRedraw)]
    pub fn needs_redraw(&self) -> bool {
        self.dirty_tracker.needs_full_redraw() || !self.dirty_tracker.dirty_rect().is_empty()
    }

    /// Request a full redraw.
    #[wasm_bindgen(js_name = requestRedraw)]
    pub fn request_redraw(&mut self) {
        self.dirty_tracker.request_full_redraw();
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

        // Mark dirty regions
        for op in ops {
            self.dirty_tracker.mark_dirty(op.x, op.y);
        }
    }

    fn create_event_result(&self) -> EditorEventResult {
        EditorEventResult {
            needs_redraw: self.needs_redraw(),
            tool: self.tool_id.name().to_string(),
            can_undo: self.can_undo(),
            can_redo: self.can_redo(),
            should_copy: false,
            ascii: None,
        }
    }

    fn create_event_result_with_copy(&self, should_copy: bool) -> EditorEventResult {
        let ascii = if should_copy {
            Some(self.export_ascii())
        } else {
            None
        };

        EditorEventResult {
            needs_redraw: false,
            tool: self.tool_id.name().to_string(),
            can_undo: self.can_undo(),
            can_redo: self.can_redo(),
            should_copy,
            ascii,
        }
    }
}

/// Result of an editor event.
#[derive(Clone, Debug, Serialize, Deserialize)]
struct EditorEventResult {
    needs_redraw: bool,
    tool: String,
    can_undo: bool,
    can_redo: bool,
    should_copy: bool,
    ascii: Option<String>,
}
