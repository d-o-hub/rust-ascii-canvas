//! Render and export bridge - rendering and ASCII export functionality.

use crate::core::ascii_export::{export_grid, ExportOptions};
use crate::render::{CanvasRenderer, DirtyTracker};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

/// Result of an editor event, returned to JavaScript.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EditorEventResult {
    /// Whether a redraw is needed.
    pub needs_redraw: bool,
    /// The current tool name.
    pub tool: String,
    /// Whether undo is available.
    pub can_undo: bool,
    /// Whether redo is available.
    pub can_redo: bool,
    /// Whether to copy to clipboard.
    pub should_copy: bool,
    /// ASCII representation if copying.
    pub ascii: Option<String>,
}

pub(crate) fn create_event_result(
    needs_redraw: bool,
    tool: &str,
    can_undo: bool,
    can_redo: bool,
) -> EditorEventResult {
    EditorEventResult {
        needs_redraw,
        tool: tool.to_string(),
        can_undo,
        can_redo,
        should_copy: false,
        ascii: None,
    }
}

pub(crate) fn create_event_result_with_copy(
    tool: &str,
    can_undo: bool,
    can_redo: bool,
    ascii: String,
) -> EditorEventResult {
    EditorEventResult {
        needs_redraw: false,
        tool: tool.to_string(),
        can_undo,
        can_redo,
        should_copy: true,
        ascii: Some(ascii),
    }
}

pub(crate) fn export_ascii(grid: &crate::core::Grid) -> String {
    let options = ExportOptions::default();
    export_grid(grid, &options)
}

pub(crate) fn get_render_commands(renderer: &CanvasRenderer, grid: &crate::core::Grid) -> JsValue {
    let commands = renderer.build_full_render(grid);
    serde_wasm_bindgen::to_value(&commands).unwrap_or(JsValue::NULL)
}

pub(crate) fn get_render_commands_full(
    renderer: &CanvasRenderer,
    grid: &crate::core::Grid,
    preview_ops: &[crate::core::tools::DrawOp],
    selection: Option<&crate::core::selection::Selection>,
) -> JsValue {
    let commands =
        renderer.build_full_render_with_preview_and_selection(grid, preview_ops, selection);
    serde_wasm_bindgen::to_value(&commands).unwrap_or(JsValue::NULL)
}

pub(crate) fn get_dirty_render_commands(
    renderer: &mut CanvasRenderer,
    grid: &crate::core::Grid,
    dirty_tracker: &mut DirtyTracker,
) -> JsValue {
    if dirty_tracker.needs_full_redraw() {
        dirty_tracker.clear();
        return get_render_commands(renderer, grid);
    }

    let dirty = *dirty_tracker.dirty_rect();
    dirty_tracker.clear();

    let commands = renderer.build_render_commands(grid, &dirty);
    serde_wasm_bindgen::to_value(&commands).unwrap_or(JsValue::NULL)
}

pub(crate) fn needs_redraw(dirty_tracker: &DirtyTracker) -> bool {
    dirty_tracker.needs_full_redraw() || !dirty_tracker.dirty_rect().is_empty()
}

pub(crate) fn request_full_redraw(dirty_tracker: &mut DirtyTracker) {
    dirty_tracker.request_full_redraw();
}
