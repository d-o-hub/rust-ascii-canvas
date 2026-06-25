//! Pointer, keyboard, and wheel event handlers for WASM.

#![allow(missing_docs)]

use wasm_bindgen::prelude::*;

use super::bindings::AsciiEditor;
use crate::core::tools::ToolId;

#[wasm_bindgen]
impl AsciiEditor {
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

        if !result.ops.is_empty() && (self.is_incremental_tool() || self.tool_id == ToolId::Text) {
            self.commit_ops(&result.ops);
        }

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
            self.commit_ops(&result.ops);
            self.preview_ops.clear();
        } else {
            self.preview_ops = result.ops.clone();
            if !self.preview_ops.is_empty() {
                self.dirty_tracker.request_full_redraw();
            }
        }

        self.js_event_result()
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
            return self.js_event_result();
        }

        let (x, y) = self.renderer.screen_to_grid(screen_x, screen_y);
        let ctx = self.create_tool_context();
        let result = self.active_tool.on_pointer_move(x, y, &ctx);

        if self.is_incremental_tool() && result.modified {
            self.commit_ops(&result.ops);
            self.preview_ops.clear();
        } else if !result.ops.is_empty() {
            self.preview_ops = result.ops.clone();
            self.dirty_tracker.request_full_redraw();
        }

        if self.tool_id == ToolId::Select {
            self.update_select_tool_selection();
            if self.is_select_moving() {
                self.preview_ops = self.generate_move_preview_ops();
            }
            if self.current_selection.is_some() || !self.preview_ops.is_empty() {
                self.dirty_tracker.request_full_redraw();
            }
        }

        self.js_event_result()
    }

    #[wasm_bindgen(js_name = onPointerUp)]
    pub fn on_pointer_up(&mut self, screen_x: f64, screen_y: f64) -> JsValue {
        if self.is_panning {
            self.is_panning = false;
            self.last_pan_pos = None;
            return self.js_event_result();
        }

        let (x, y) = self.renderer.screen_to_grid(screen_x, screen_y);
        let ctx = self.create_tool_context();
        let result = self.active_tool.on_pointer_up(x, y, &ctx);

        self.preview_ops.clear();

        if self.tool_id == ToolId::Select {
            if self.is_moving_selection {
                self.commit_selection_move();
                self.is_moving_selection = false;
            }
            self.update_select_tool_selection();
            self.dirty_tracker.request_full_redraw();
        }

        if result.modified {
            self.commit_ops(&result.ops);
        }

        self.js_event_result()
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
            self.current_selection = None;
            self.preview_ops.clear();
            self.dirty_tracker.request_full_redraw();
            return self.js_event_result();
        }

        if key_char == ' ' && !ctrl && !shift {
            self.space_held = true;
            return self.js_event_result();
        }

        if ctrl && !shift && key.to_lowercase() == "z" {
            self.undo();
            return self.js_event_result();
        }

        if ctrl && shift && key.to_lowercase() == "z" {
            self.redo();
            return self.js_event_result();
        }

        if ctrl && !shift && key.to_lowercase() == "y" {
            self.redo();
            return self.js_event_result();
        }

        if ctrl && key.to_lowercase() == "c" {
            return self.js_event_result_with_copy(true);
        }

        if ctrl && key.to_lowercase() == "a" {
            self.select_all_impl();
            return self.js_event_result();
        }

        if ctrl && key.to_lowercase() == "x" && self.cut_selection_impl() {
            return self.js_event_result();
        }

        if ctrl && key.to_lowercase() == "v" && self.paste_impl() {
            return self.js_event_result();
        }

        if !ctrl && (key == "Delete" || key == "Backspace") {
            if self.tool_id == ToolId::Select
                && self.current_selection.is_some()
                && self.delete_selection_impl()
            {
                return self.js_event_result();
            }
            if self.tool_id == ToolId::Text && self.active_tool.is_active() {
                let ctx = self.create_tool_context();
                let delete_char = if key == "Delete" { '\0' } else { '\x08' };
                let result = self.active_tool.on_key(delete_char, &ctx);
                if result.modified {
                    self.commit_ops(&result.ops);
                }
                return self.js_event_result();
            }
        }

        if !ctrl && !shift && !self.active_tool.is_active() {
            if let Some(tool_id) = ToolId::from_shortcut(key_char) {
                self.set_tool_by_id_impl(tool_id);
                return self.js_event_result();
            }
        }

        if self.tool_id == ToolId::Text && self.active_tool.is_active() {
            let ctx = self.create_tool_context();
            let result = self.active_tool.on_key(key_char, &ctx);
            if result.modified {
                self.commit_ops(&result.ops);
            }
            return self.js_event_result();
        }

        self.js_event_result()
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

        self.js_event_result()
    }
}

impl AsciiEditor {
    fn js_event_result(&self) -> JsValue {
        let er = self.create_event_result();
        serde_wasm_bindgen::to_value(&er).unwrap_or(JsValue::NULL)
    }

    fn js_event_result_with_copy(&self, should_copy: bool) -> JsValue {
        let er = self.create_event_result_with_copy(should_copy);
        serde_wasm_bindgen::to_value(&er).unwrap_or(JsValue::NULL)
    }
}
