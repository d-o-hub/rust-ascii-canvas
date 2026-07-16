//! WASM bindings - struct definition, constructor, and core methods.

use crate::core::history::History;
use crate::core::selection::{Selection, SelectionClipboard};
use crate::core::tools::{DrawOp, RectangleTool, Tool, ToolId};
use crate::core::EditorState;
use crate::render::{CanvasRenderer, DirtyTracker, FontAtlas, FontMetrics};
use crate::wasm::tool_manager::{
    parse_tool_id, set_border_style, set_line_direction, set_tool_by_id,
};
use wasm_bindgen::prelude::*;

/// WebAssembly-bindable ASCII editor instance for frontend integration.
///
/// `AsciiEditor` manages the state of the ASCII canvas, handles user interactions,
/// renders grid content to JSON / canvas drawing commands, and supports history
/// (undo/redo) and layer capabilities.
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
    /// Last grid cell under the pointer (for paste origin when no selection).
    pub(crate) last_cursor: Option<(i32, i32)>,
    pub(crate) full_render_count: u32,
    pub(crate) dirty_render_count: u32,
    pub(crate) pixel_buffer: Vec<u8>,
    pub(crate) font_atlas: FontAtlas,
    /// Named layers (background layers + active content mirrored in `state.grid`).
    pub(crate) layers: Vec<LayerData>,
    pub(crate) active_layer: usize,
}

/// Serializable layer metadata + content snapshot.
#[derive(Clone, Debug)]
pub(crate) struct LayerData {
    pub name: String,
    pub visible: bool,
    pub grid: crate::core::Grid,
}

#[wasm_bindgen]
impl AsciiEditor {
    /// Creates a new `AsciiEditor` instance with the given dimensions.
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
            last_cursor: None,
            full_render_count: 0,
            dirty_render_count: 0,
            pixel_buffer: vec![0u8; width * 8 * height * 20 * 4],
            font_atlas: FontAtlas::new(),
            layers: vec![LayerData {
                name: "Layer 1".to_string(),
                visible: true,
                grid: crate::core::Grid::new(width, height),
            }],
            active_layer: 0,
        }
    }

    /// Resizes the editor canvas and all its layers to the new dimensions.
    #[wasm_bindgen]
    pub fn resize(&mut self, new_width: usize, new_height: usize) {
        self.state.grid.resize(new_width, new_height);
        for layer in &mut self.layers {
            layer.grid.resize(new_width, new_height);
        }
        self.pixel_buffer = vec![0u8; new_width * 8 * new_height * 20 * 4];
        self.dirty_tracker.request_full_redraw();
    }

    /// Gets the current width of the canvas grid.
    #[wasm_bindgen(getter)]
    pub fn width(&self) -> usize {
        self.state.grid.width()
    }

    /// Gets the current height of the canvas grid.
    #[wasm_bindgen(getter)]
    pub fn height(&self) -> usize {
        self.state.grid.height()
    }

    /// Gets the name of the active drawing tool.
    #[wasm_bindgen(getter)]
    pub fn tool(&self) -> String {
        self.tool_id.name().to_string()
    }

    /// Sets the active tool by its string ID.
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

    /// Attempts to set the active tool using a keyboard shortcut character.
    /// Returns true if the shortcut is valid and the tool was switched.
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

    /// Sets the border style for shapes like Rectangles.
    #[wasm_bindgen(js_name = setBorderStyle)]
    pub fn set_border_style(&mut self, style: String) {
        set_border_style(&mut self.state, style, &mut self.active_tool);
    }

    /// Sets the line direction setting (e.g. Horizontal, Vertical, Auto) for the Line tool.
    #[wasm_bindgen(js_name = setLineDirection)]
    pub fn set_line_direction(&mut self, direction: String) {
        set_line_direction(self.tool_id, &mut self.active_tool, direction);
    }

    /// Sets the zoom scale factor of the editor viewport.
    #[wasm_bindgen(js_name = setZoom)]
    pub fn set_zoom(&mut self, zoom: f64) {
        self.renderer.set_zoom(zoom);
        self.dirty_tracker.request_full_redraw();
    }

    /// Gets the current zoom scale factor of the editor viewport.
    #[wasm_bindgen(getter)]
    pub fn zoom(&self) -> f64 {
        self.renderer.zoom()
    }

    /// Sets the pan offset (X and Y coordinates) of the editor viewport.
    #[wasm_bindgen(js_name = setPan)]
    pub fn set_pan(&mut self, x: f64, y: f64) {
        self.renderer.set_pan(x, y);
        self.dirty_tracker.request_full_redraw();
    }

    /// Gets the current pan offset of the editor viewport as a `[x, y]` list.
    #[wasm_bindgen(getter = pan)]
    pub fn get_pan(&self) -> Vec<f64> {
        let (x, y) = self.renderer.pan();
        vec![x, y]
    }

    /// Configures the font metrics for text size and grid layout spacing calculations.
    #[wasm_bindgen(js_name = setFontMetrics)]
    pub fn set_font_metrics(&mut self, char_width: f64, line_height: f64, size: f64) {
        let mut metrics = FontMetrics::new("JetBrains Mono, monospace", size);
        metrics.set_measured_width(char_width);
        metrics.line_height = line_height;
        self.renderer.set_metrics(metrics);
        self.dirty_tracker.request_full_redraw();
    }

    /// Reverts the last drawing operation. Returns true if successful.
    #[wasm_bindgen]
    pub fn undo(&mut self) -> bool {
        let result = self.history.undo(&mut self.state.grid);
        if result {
            self.dirty_tracker.request_full_redraw();
        }
        result
    }

    /// Re-applies a previously undone operation. Returns true if successful.
    #[wasm_bindgen]
    pub fn redo(&mut self) -> bool {
        let result = self.history.redo(&mut self.state.grid);
        if result {
            self.dirty_tracker.request_full_redraw();
        }
        result
    }

    /// Returns whether there is an action that can be undone in the history.
    #[wasm_bindgen(getter)]
    pub fn can_undo(&self) -> bool {
        self.history.can_undo()
    }

    /// Returns whether there is an action that can be redone in the history.
    #[wasm_bindgen(getter)]
    pub fn can_redo(&self) -> bool {
        self.history.can_redo()
    }

    /// Clears the canvas, history, clipboard, and reset layers.
    #[wasm_bindgen]
    pub fn clear(&mut self) {
        self.state.grid.clear();
        if let Some(layer) = self.layers.get_mut(self.active_layer) {
            layer.grid.clear();
        }
        self.history.clear();
        self.clipboard.clear();
        self.dirty_tracker.request_full_redraw();
    }
}
