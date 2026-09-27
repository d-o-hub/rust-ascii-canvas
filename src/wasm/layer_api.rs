//! Layer operations for WASM (ADR-043).
//!
//! This lives in its own module because `render_api.rs` is at its 500-line
//! budget. Every structural operation is recorded on the active layer's history
//! after it is applied, so undo/redo covers layer changes as well as drawing
//! (per-layer history: undo never crosses layers, ADR-043 decision 1).
//!
//! Navigation (`setActiveLayer`) is not undoable, and a mutation that changes
//! nothing records nothing (ADR-043 decision 5).

use super::bindings::AsciiEditor;
use crate::core::commands::{
    AddLayerCommand, DeleteLayerCommand, LayerCommand, MergeLayerDownCommand, MoveLayerCommand,
    SetLayerLockedCommand, SetLayerNameCommand, SetLayerVisibleCommand,
};
use crate::core::history::HistoryOutcome;
use crate::core::layer::Layer;
use wasm_bindgen::prelude::*;

impl AsciiEditor {
    /// Copy the live drawing surface into the active layer.
    pub(crate) fn sync_active_layer(&mut self) {
        let grid = self.state.grid.clone();
        *self.layer_stack.active_mut().grid_mut() = grid;
    }

    /// Pull the active layer back into the live surface after a structural change.
    pub(crate) fn refresh_from_layers(&mut self) {
        self.state.grid = self.layer_stack.active().grid().clone();
        self.current_selection = None;
        self.preview_ops.clear();
        self.dirty_tracker.request_full_redraw();
    }

    /// Whether the active layer rejects edits.
    pub(crate) fn is_active_layer_locked(&self) -> bool {
        self.layer_stack.is_active_locked()
    }

    /// Fold an undo/redo result back into the editor.
    ///
    /// Grid and metadata entries leave their result in the live surface already,
    /// so they only need a redraw. Reloading the surface from the layer copy is
    /// reserved for entries that can change layer content, because that copy is
    /// only written back on the next sync and would otherwise drop drawing done
    /// since the last one.
    pub(crate) fn apply_history_outcome(&mut self, outcome: HistoryOutcome) -> bool {
        match outcome {
            HistoryOutcome::None => false,
            HistoryOutcome::Grid | HistoryOutcome::LayerMeta => {
                self.dirty_tracker.request_full_redraw();
                true
            }
            HistoryOutcome::LayerContent => {
                self.refresh_from_layers();
                true
            }
        }
    }

    /// Record a structural command on the layer that was active before the
    /// change, falling back to the layer active after it when that layer is gone
    /// (deleting the active layer). Recording on the user's own layer means both
    /// undo and redo are reachable there, and undo returns focus to it.
    pub(crate) fn record_layer_command(
        &mut self,
        active_before: u64,
        command: Box<dyn LayerCommand>,
    ) {
        self.layer_stack
            .push_layer_command_on_id(active_before, command);
    }

    /// Switch the active layer, keeping each layer's own history.
    pub(crate) fn set_active_layer_impl(&mut self, index: usize) -> bool {
        if index >= self.layer_stack.len() || index == self.layer_stack.active_index() {
            return index < self.layer_stack.len();
        }
        self.sync_active_layer();
        self.layer_stack.set_active(index);
        self.refresh_from_layers();
        true
    }

    /// Add an empty layer, activate it, and record the change.
    pub(crate) fn add_layer_impl(&mut self) -> usize {
        self.sync_active_layer();
        let active_before = self.layer_stack.active_id();
        let mut cmd = AddLayerCommand::adding_next(&mut self.layer_stack);
        cmd.apply(&mut self.layer_stack);
        let index = cmd.index();
        self.record_layer_command(active_before, Box::new(cmd));
        self.refresh_from_layers();
        index
    }

    /// Reorder a layer and record the change.
    pub(crate) fn move_layer_impl(&mut self, from_index: usize, to_index: usize) {
        let len = self.layer_stack.len();
        if from_index >= len || to_index >= len || from_index == to_index {
            return;
        }
        self.sync_active_layer();
        let active_before = self.layer_stack.active_id();
        let Some(id) = self.layer_stack.layer_id_at(from_index) else {
            return;
        };
        let mut cmd = MoveLayerCommand::new(from_index, to_index, id);
        cmd.apply(&mut self.layer_stack);
        self.record_layer_command(active_before, Box::new(cmd));
        self.refresh_from_layers();
    }

    /// Delete a layer and record the change (keeping the layer itself for undo).
    pub(crate) fn delete_layer_impl(&mut self, index: usize) -> bool {
        self.sync_active_layer();
        let active_before = self.layer_stack.active_id();
        let Some(mut cmd) = DeleteLayerCommand::deleting(&self.layer_stack, index) else {
            return false;
        };
        cmd.apply(&mut self.layer_stack);
        self.record_layer_command(active_before, Box::new(cmd));
        self.refresh_from_layers();
        true
    }

    /// Merge a layer into the one below it and record the change.
    pub(crate) fn merge_down_impl(&mut self, index: usize) -> bool {
        self.sync_active_layer();
        let active_before = self.layer_stack.active_id();
        let Some(mut cmd) = MergeLayerDownCommand::merging(&self.layer_stack, index) else {
            return false;
        };
        cmd.apply(&mut self.layer_stack);
        self.record_layer_command(active_before, Box::new(cmd));
        self.refresh_from_layers();
        true
    }
}

#[wasm_bindgen]
impl AsciiEditor {
    /// Number of layers.
    #[wasm_bindgen(getter = layerCount)]
    pub fn layer_count(&self) -> usize {
        self.layer_stack.len()
    }

    /// Active layer index.
    #[wasm_bindgen(getter = activeLayer)]
    pub fn active_layer_index(&self) -> usize {
        self.layer_stack.active_index()
    }

    /// Layer name by index.
    #[wasm_bindgen(js_name = layerName)]
    pub fn layer_name(&self, index: usize) -> String {
        self.layer_stack
            .get(index)
            .map(|l| l.name().to_string())
            .unwrap_or_default()
    }

    /// Whether a layer is visible.
    #[wasm_bindgen(js_name = layerVisible)]
    pub fn layer_visible(&self, index: usize) -> bool {
        self.layer_stack
            .get(index)
            .map(Layer::is_visible)
            .unwrap_or(false)
    }

    /// Set layer visibility.
    #[wasm_bindgen(js_name = setLayerVisible)]
    pub fn set_layer_visible(&mut self, index: usize, visible: bool) {
        let Some((id, old)) = self
            .layer_stack
            .get(index)
            .map(|layer| (layer.id(), layer.is_visible()))
        else {
            return;
        };
        if old == visible {
            return;
        }
        let mut cmd = SetLayerVisibleCommand::new(id, old, visible);
        cmd.apply(&mut self.layer_stack);
        self.layer_stack.push_layer_command(Box::new(cmd));
        self.dirty_tracker.request_full_redraw();
    }

    /// Switch active layer (saves current grid into the previous layer).
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
        let Some((id, old)) = self
            .layer_stack
            .get(index)
            .map(|layer| (layer.id(), layer.name().to_string()))
        else {
            return;
        };
        if old == name {
            return;
        }
        let mut cmd = SetLayerNameCommand::new(id, old, name);
        cmd.apply(&mut self.layer_stack);
        self.layer_stack.push_layer_command(Box::new(cmd));
    }

    /// Whether a layer is locked.
    #[wasm_bindgen(js_name = layerLocked)]
    pub fn layer_locked(&self, index: usize) -> bool {
        self.layer_stack
            .get(index)
            .map(Layer::is_locked)
            .unwrap_or(false)
    }

    /// Set layer lock state.
    #[wasm_bindgen(js_name = setLayerLocked)]
    pub fn set_layer_locked(&mut self, index: usize, locked: bool) {
        let Some((id, old)) = self
            .layer_stack
            .get(index)
            .map(|layer| (layer.id(), layer.is_locked()))
        else {
            return;
        };
        if old == locked {
            return;
        }
        let mut cmd = SetLayerLockedCommand::new(id, old, locked);
        cmd.apply(&mut self.layer_stack);
        self.layer_stack.push_layer_command(Box::new(cmd));
        self.dirty_tracker.request_full_redraw();
    }

    /// Move a layer to a new index in the stack.
    #[wasm_bindgen(js_name = moveLayer)]
    pub fn move_layer(&mut self, from_index: usize, to_index: usize) {
        self.move_layer_impl(from_index, to_index);
    }

    /// Delete a layer.
    #[wasm_bindgen(js_name = deleteLayer)]
    pub fn delete_layer(&mut self, index: usize) -> bool {
        self.delete_layer_impl(index)
    }

    /// Merge the specified layer down into the one below it.
    #[wasm_bindgen(js_name = mergeLayerDown)]
    pub fn merge_layer_down(&mut self, index: usize) -> bool {
        self.merge_down_impl(index)
    }

    /// Description of the next undo step, e.g. `Rename layer`.
    #[wasm_bindgen(getter = undoLabel)]
    pub fn undo_label(&self) -> String {
        self.layer_stack
            .undo_description()
            .unwrap_or_default()
            .to_string()
    }

    /// Number of undo steps on the active layer.
    #[wasm_bindgen(getter = undoCount)]
    pub fn undo_count(&self) -> usize {
        self.layer_stack.undo_count()
    }

    /// Number of redo steps on the active layer.
    #[wasm_bindgen(getter = redoCount)]
    pub fn redo_count(&self) -> usize {
        self.layer_stack.redo_count()
    }

    /// Description of the next redo step, e.g. `Hide layer`.
    #[wasm_bindgen(getter = redoLabel)]
    pub fn redo_label(&self) -> String {
        self.layer_stack
            .redo_description()
            .unwrap_or_default()
            .to_string()
    }
}

// The test bodies live beside this file to respect the 500-line budget.
#[cfg(test)]
#[path = "layer_api_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "layer_api_regression_tests.rs"]
mod regression_tests;
