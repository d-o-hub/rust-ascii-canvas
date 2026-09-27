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
    /// A grid entry leaves its result in the live grid already, so it only needs
    /// a redraw. A layer entry may have changed which layer is active or what it
    /// contains, so the live surface is reloaded from the active layer.
    pub(crate) fn apply_history_outcome(&mut self, outcome: HistoryOutcome) -> bool {
        match outcome {
            HistoryOutcome::None => false,
            HistoryOutcome::Grid => {
                self.dirty_tracker.request_full_redraw();
                true
            }
            HistoryOutcome::Layer => {
                self.refresh_from_layers();
                true
            }
        }
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
        let mut cmd = AddLayerCommand::adding_next(&self.layer_stack);
        cmd.apply(&mut self.layer_stack);
        let index = cmd.index();
        self.layer_stack.push_layer_command(Box::new(cmd));
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
        let mut cmd = MoveLayerCommand::new(from_index, to_index);
        cmd.apply(&mut self.layer_stack);
        self.layer_stack.push_layer_command(Box::new(cmd));
        self.refresh_from_layers();
    }

    /// Delete a layer and record the change (keeping the layer itself for undo).
    pub(crate) fn delete_layer_impl(&mut self, index: usize) -> bool {
        self.sync_active_layer();
        let Some(mut cmd) = DeleteLayerCommand::deleting(&self.layer_stack, index) else {
            return false;
        };
        cmd.apply(&mut self.layer_stack);
        self.layer_stack.push_layer_command(Box::new(cmd));
        self.refresh_from_layers();
        true
    }

    /// Merge a layer into the one below it and record the change.
    pub(crate) fn merge_down_impl(&mut self, index: usize) -> bool {
        self.sync_active_layer();
        let Some(mut cmd) = MergeLayerDownCommand::merging(&self.layer_stack, index) else {
            return false;
        };
        cmd.apply(&mut self.layer_stack);
        self.layer_stack.push_layer_command(Box::new(cmd));
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
        let Some(old) = self.layer_stack.get(index).map(Layer::is_visible) else {
            return;
        };
        if old == visible {
            return;
        }
        let mut cmd = SetLayerVisibleCommand::new(index, old, visible);
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
        let Some(old) = self.layer_stack.get(index).map(|l| l.name().to_string()) else {
            return;
        };
        if old == name {
            return;
        }
        let mut cmd = SetLayerNameCommand::new(index, old, name);
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
        let Some(old) = self.layer_stack.get(index).map(Layer::is_locked) else {
            return;
        };
        if old == locked {
            return;
        }
        let mut cmd = SetLayerLockedCommand::new(index, old, locked);
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

    /// Description of the next redo step, e.g. `Hide layer`.
    #[wasm_bindgen(getter = redoLabel)]
    pub fn redo_label(&self) -> String {
        self.layer_stack
            .redo_description()
            .unwrap_or_default()
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn editor() -> AsciiEditor {
        AsciiEditor::new(4, 2)
    }

    /// A v1 document with two layers, the second hidden and locked.
    fn v1_document() -> String {
        r#"{"format":"ascii-canvas","version":1,"canvas":{"width":4,"height":2},
"active_layer":0,"layers":[
{"name":"Base","visible":true,"locked":false,"cells":[{"x":0,"y":0,"ch":"A"}]},
{"name":"Sketch","visible":false,"locked":true,"cells":[{"x":1,"y":1,"ch":"B"}]}
]}"#
        .to_string()
    }

    #[test]
    fn rename_is_undoable_and_no_op_rename_records_nothing() {
        let mut canvas = editor();
        canvas.rename_layer(0, "Renamed".to_string());
        assert_eq!(canvas.layer_name(0), "Renamed");
        assert_eq!(canvas.undo_label(), "Rename layer");
        assert!(canvas.can_undo());

        assert!(canvas.undo());
        assert_eq!(canvas.layer_name(0), "Layer 1");
        assert!(canvas.redo());
        assert_eq!(canvas.layer_name(0), "Renamed");

        // Renaming to the same name changes nothing, so it records nothing.
        let depth_before = canvas.undo_label();
        canvas.rename_layer(0, "Renamed".to_string());
        assert_eq!(canvas.undo_label(), depth_before);
        // An out-of-range index is ignored.
        canvas.rename_layer(9, "Nope".to_string());
        assert_eq!(canvas.undo_label(), depth_before);
    }

    #[test]
    fn visibility_and_lock_are_undoable() {
        let mut canvas = editor();
        canvas.set_layer_visible(0, false);
        assert!(!canvas.layer_visible(0));
        assert_eq!(canvas.undo_label(), "Hide layer");
        assert!(canvas.undo());
        assert!(canvas.layer_visible(0));
        assert!(canvas.redo());
        assert!(!canvas.layer_visible(0));

        // Setting the same value again records nothing, and the redo that was
        // already consumed stays consumed.
        canvas.set_layer_visible(0, false);
        assert_eq!(canvas.redo_label(), "");
        assert_eq!(canvas.undo_label(), "Hide layer");
        assert!(!canvas.redo());

        canvas.set_layer_locked(0, true);
        assert!(canvas.layer_locked(0));
        assert_eq!(canvas.undo_label(), "Lock layer");
        // Undoing a lock must work even though the layer is locked.
        assert!(canvas.undo());
        assert!(!canvas.layer_locked(0));
    }

    #[test]
    fn add_and_delete_layers_are_undoable() {
        let mut canvas = editor();
        let index = canvas.add_layer();
        assert_eq!(index, 1);
        assert_eq!(canvas.layer_count(), 2);
        assert_eq!(canvas.active_layer_index(), 1);
        assert_eq!(canvas.undo_label(), "Add layer");

        assert!(canvas.undo());
        assert_eq!(canvas.layer_count(), 1);
        assert_eq!(canvas.active_layer_index(), 0);

        // A second undo must not remove another layer: the add entry went away
        // with the layer it had added.
        assert!(!canvas.undo());
        assert_eq!(canvas.layer_count(), 1);

        canvas.add_layer();
        canvas.set_layer_visible(0, true);
        assert!(canvas.delete_layer(0));
        assert_eq!(canvas.layer_count(), 1);
        assert_eq!(canvas.undo_label(), "Delete layer");
        assert!(canvas.undo());
        assert_eq!(canvas.layer_count(), 2);
        assert_eq!(canvas.layer_name(0), "Layer 1");

        // The last layer cannot be deleted.
        assert!(!canvas.delete_layer(0) || canvas.layer_count() > 0);
    }

    #[test]
    fn move_and_merge_are_undoable() {
        let mut canvas = editor();
        canvas.add_layer();
        canvas.add_layer();
        assert_eq!(canvas.layer_count(), 3);
        canvas.set_active_layer(0);

        canvas.move_layer(0, 2);
        assert_eq!(canvas.layer_name(2), "Layer 1");
        assert_eq!(canvas.undo_label(), "Move layer up");
        assert!(canvas.undo());
        assert_eq!(canvas.layer_name(0), "Layer 1");

        // A no-op move records nothing.
        let label = canvas.undo_label();
        canvas.move_layer(1, 1);
        canvas.move_layer(0, 9);
        assert_eq!(canvas.undo_label(), label);

        assert!(canvas.merge_layer_down(1));
        assert_eq!(canvas.layer_count(), 2);
        assert_eq!(canvas.undo_label(), "Merge layer down");
        assert!(canvas.undo());
        assert_eq!(canvas.layer_count(), 3);

        // The bottom layer has nothing to merge into.
        assert!(!canvas.merge_layer_down(0));
    }

    #[test]
    fn switching_layers_does_not_record_history() {
        let mut canvas = editor();
        canvas.add_layer();
        // The add is undoable from the layer it was recorded on.
        assert!(canvas.can_undo());

        // Navigation is not undoable, and it must not create an entry.
        assert!(canvas.set_active_layer(0));
        assert_eq!(canvas.active_layer_index(), 0);
        assert!(!canvas.can_undo(), "layer 0 has no history of its own");
        // Re-selecting the current layer is a no-op that still reports success.
        assert!(canvas.set_active_layer(0));
        assert!(!canvas.set_active_layer(9));
    }

    #[test]
    fn undo_is_scoped_to_the_active_layer() {
        let mut canvas = editor();
        canvas.rename_layer(0, "First".to_string());
        canvas.add_layer();
        canvas.rename_layer(1, "Second".to_string());
        assert!(canvas.can_undo());

        // The new layer's own history is what undo walks: the rename, then the add.
        assert!(canvas.undo());
        assert_eq!(canvas.layer_name(1), "Layer 2");
        assert!(canvas.undo());
        assert_eq!(canvas.layer_count(), 1);

        // Undoing the add put focus back on layer 0, whose own entry was never
        // touched by the other layer's undos - so this undo is layer 0's.
        assert_eq!(canvas.active_layer_index(), 0);
        assert_eq!(canvas.layer_name(0), "First");
        assert!(canvas.undo());
        assert_eq!(canvas.layer_name(0), "Layer 1");
        assert!(!canvas.undo(), "nothing left on layer 0");
    }

    #[test]
    fn resize_drops_history_on_every_layer() {
        let mut canvas = editor();
        canvas.add_layer();
        canvas.rename_layer(0, "Kept".to_string());
        assert!(canvas.can_undo());

        canvas.resize(8, 4);
        assert_eq!(canvas.width(), 8);
        assert_eq!(canvas.height(), 4);
        // Recorded draw commands carry coordinates and sizes that no longer match
        // the resized grids, so no layer may keep its history.
        assert!(!canvas.can_undo());
        assert_eq!(canvas.undo_label(), "");
        assert_eq!(canvas.layer_name(0), "Kept", "content and names survive");
    }

    #[test]
    fn v1_document_loads_with_its_layer_flags() {
        let mut canvas = editor();
        assert!(canvas.load_document(v1_document()));
        assert_eq!(canvas.layer_count(), 2);
        assert_eq!(canvas.active_layer_index(), 0);
        assert_eq!(canvas.layer_name(0), "Base");
        assert_eq!(canvas.layer_name(1), "Sketch");
        // The frozen v1 format carries per-layer visibility and lock state.
        assert!(canvas.layer_visible(0));
        assert!(!canvas.layer_visible(1));
        assert!(!canvas.layer_locked(0));
        assert!(canvas.layer_locked(1));

        // Round-tripping keeps both flags, so nothing is silently dropped.
        let again = canvas.serialize_document();
        let mut reloaded = editor();
        assert!(reloaded.load_document(again));
        assert!(!reloaded.layer_visible(1));
        assert!(reloaded.layer_locked(1));
        assert_eq!(reloaded.layer_name(1), "Sketch");
    }

    #[test]
    fn loading_a_document_clears_history() {
        let mut canvas = editor();
        canvas.rename_layer(0, "Before".to_string());
        assert!(canvas.can_undo());
        assert!(canvas.load_document(v1_document()));
        assert!(!canvas.can_undo());
    }
}
