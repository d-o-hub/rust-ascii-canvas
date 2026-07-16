//! Rendering and pixel buffer API for WASM.

use wasm_bindgen::prelude::*;

use super::bindings::AsciiEditor;
use crate::wasm::render_bridge::{
    export_ascii, get_dirty_render_commands, get_render_commands, get_render_commands_full,
    needs_redraw, request_full_redraw,
};

#[wasm_bindgen]
impl AsciiEditor {
    /// Exports the composited visible canvas layers as a raw ASCII text string.
    #[wasm_bindgen(js_name = exportAscii)]
    pub fn export_ascii(&self) -> String {
        // Composite all visible layers so export matches what users expect from multi-layer docs.
        export_ascii(&self.composite_visible_grid())
    }

    /// Selection-aware export for the OS clipboard (selection region or trimmed full grid).
    #[wasm_bindgen(js_name = exportForCopy)]
    pub fn export_for_copy_public(&self) -> String {
        self.export_for_copy()
    }

    /// Serialize diagram to JSON (`.asc` format).
    #[wasm_bindgen(js_name = serializeDocument)]
    pub fn serialize_document(&self) -> String {
        self.serialize_document_impl()
    }

    /// Load diagram from JSON (`.asc` format). Returns false on parse/schema errors.
    #[wasm_bindgen(js_name = loadDocument)]
    pub fn load_document(&mut self, json: String) -> bool {
        self.load_document_impl(&json)
    }

    /// Number of layers.
    #[wasm_bindgen(getter = layerCount)]
    pub fn layer_count(&self) -> usize {
        self.layers.len()
    }

    /// Active layer index.
    #[wasm_bindgen(getter = activeLayer)]
    pub fn active_layer_index(&self) -> usize {
        self.active_layer
    }

    /// Layer name by index.
    #[wasm_bindgen(js_name = layerName)]
    pub fn layer_name(&self, index: usize) -> String {
        self.layers
            .get(index)
            .map(|l| l.name.clone())
            .unwrap_or_default()
    }

    /// Whether a layer is visible.
    #[wasm_bindgen(js_name = layerVisible)]
    pub fn layer_visible(&self, index: usize) -> bool {
        self.layers.get(index).map(|l| l.visible).unwrap_or(false)
    }

    /// Set layer visibility.
    #[wasm_bindgen(js_name = setLayerVisible)]
    pub fn set_layer_visible(&mut self, index: usize, visible: bool) {
        if let Some(layer) = self.layers.get_mut(index) {
            layer.visible = visible;
            self.dirty_tracker.request_full_redraw();
        }
    }

    /// Switch active layer (saves current grid into previous layer).
    #[wasm_bindgen(js_name = setActiveLayer)]
    pub fn set_active_layer(&mut self, index: usize) -> bool {
        self.set_active_layer_impl(index)
    }

    /// Add a new empty layer and switch to it.
    #[wasm_bindgen(js_name = addLayer)]
    pub fn add_layer(&mut self) -> usize {
        self.add_layer_impl()
    }

    /// Rename a layer.
    #[wasm_bindgen(js_name = renameLayer)]
    pub fn rename_layer(&mut self, index: usize, name: String) {
        if let Some(layer) = self.layers.get_mut(index) {
            layer.name = name;
        }
    }

    /// Returns the full list of drawing instructions/commands to render the entire canvas in JS.
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

    /// Returns only the drawing instructions/commands for regions of the canvas that have changed.
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

    /// Returns the number of times a full render was requested and executed.
    #[wasm_bindgen(getter = fullRenderCount)]
    pub fn full_render_count(&self) -> u32 {
        self.full_render_count
    }

    /// Returns the number of times a partial/dirty rect render was requested and executed.
    #[wasm_bindgen(getter = dirtyRenderCount)]
    pub fn dirty_render_count(&self) -> u32 {
        self.dirty_render_count
    }

    /// Resets full and dirty render performance metric counters to zero.
    #[wasm_bindgen(js_name = resetPerformanceMetrics)]
    pub fn reset_performance_metrics(&mut self) {
        self.full_render_count = 0;
        self.dirty_render_count = 0;
    }

    /// Returns whether the canvas currently needs to be redrawn.
    #[wasm_bindgen(getter = needsRedraw)]
    pub fn needs_redraw(&self) -> bool {
        needs_redraw(&self.dirty_tracker)
    }

    /// Explicitly flags that the entire canvas needs to be redrawn on the next frame.
    #[wasm_bindgen(js_name = requestRedraw)]
    pub fn request_redraw(&mut self) {
        request_full_redraw(&mut self.dirty_tracker);
    }

    /// Resets the dirty rendering rect and flag status.
    #[wasm_bindgen(js_name = clearDirtyState)]
    pub fn clear_dirty_state(&mut self) {
        self.dirty_tracker.clear();
    }

    /// Updates the font atlas glyph data cache for a specific Unicode character.
    #[wasm_bindgen(js_name = updateFontAtlasGlyph)]
    pub fn update_font_atlas_glyph(&mut self, ch_code: u32, glyph_data: Vec<u8>) {
        if let Some(ch) = char::from_u32(ch_code) {
            self.font_atlas.update_glyph(ch, &glyph_data);
            self.dirty_tracker.request_full_redraw();
        }
    }

    /// Returns the pointer to the underlying raw pixel buffer (RGBA format).
    #[wasm_bindgen(js_name = getPixelBufferPtr)]
    pub fn get_pixel_buffer_ptr(&self) -> *const u8 {
        self.pixel_buffer.as_ptr()
    }

    /// Returns the length of the raw pixel buffer in bytes.
    #[wasm_bindgen(js_name = getPixelBufferLen)]
    pub fn get_pixel_buffer_len(&self) -> usize {
        self.pixel_buffer.len()
    }

    /// Renders the entire canvas layers and current selection highlights into the pixel buffer.
    #[wasm_bindgen(js_name = renderToPixelBuffer)]
    pub fn render_to_pixel_buffer(&mut self) {
        let grid_width = self.state.grid.width();
        let grid_height = self.state.grid.height();
        let glyph_w = 8;
        let glyph_h = 20;
        let buffer_width = grid_width * glyph_w;
        let buffer_height = grid_height * glyph_h;

        let required_len = buffer_width * buffer_height * 4;
        if self.pixel_buffer.len() != required_len {
            self.pixel_buffer.resize(required_len, 0);
        }

        let bg_color = [30, 30, 30, 255];
        for i in 0..buffer_width * buffer_height {
            let idx = i * 4;
            self.pixel_buffer[idx] = bg_color[0];
            self.pixel_buffer[idx + 1] = bg_color[1];
            self.pixel_buffer[idx + 2] = bg_color[2];
            self.pixel_buffer[idx + 3] = bg_color[3];
        }

        let fg_color = [212, 212, 212, 255];

        // Composite all visible layers so on-screen view and PNG export match ASCII export.
        let composite = self.composite_visible_grid();

        if let Some(ref sel) = self.current_selection {
            let (min_x, min_y, max_x, max_y) = sel.bounds();
            let highlight_color = [38, 79, 120, 255];

            for gy in min_y..=max_y {
                for gx in min_x..=max_x {
                    if composite.in_bounds(gx, gy) {
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

        for (x, y, cell) in composite.iter_with_coords() {
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
