//! Structural layer commands - the undoable side of the layer stack (ADR-043).
//!
//! `Command` in the parent module operates on a single `Grid`; these operate on
//! the whole [`LayerStack`], so they are a separate trait rather than a
//! genericised `Command<T>` (ADR-043 decision 2).
//!
//! Every command carries bounded data (old/new values, or the layer that was
//! inserted/removed/merged) so undo is a data restore and cannot drift from
//! apply. Each command is recorded on the *active* layer's history, so undo
//! never crosses layers (ADR-043 decision 1).

use crate::core::cell::Cell;
use crate::core::layer::{Layer, LayerStack};

/// Trait for undoable structural layer operations.
pub trait LayerCommand {
    /// Apply the command to the layer stack.
    fn apply(&mut self, stack: &mut LayerStack);

    /// Undo the command on the layer stack.
    fn undo(&mut self, stack: &mut LayerStack);

    /// Get a description used for undo/redo labels.
    fn description(&self) -> &str;

    /// Get a reference to Any for downcasting.
    fn as_any(&self) -> &dyn std::any::Any;

    /// Get a mutable reference to Any for downcasting.
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

/// Command to rename a layer.
pub struct SetLayerNameCommand {
    index: usize,
    old_name: String,
    new_name: String,
}

impl SetLayerNameCommand {
    /// Create a rename command carrying both names.
    pub fn new(index: usize, old_name: String, new_name: String) -> Self {
        Self {
            index,
            old_name,
            new_name,
        }
    }
}

impl LayerCommand for SetLayerNameCommand {
    fn apply(&mut self, stack: &mut LayerStack) {
        stack.set_name(self.index, self.new_name.clone());
    }

    fn undo(&mut self, stack: &mut LayerStack) {
        stack.set_name(self.index, self.old_name.clone());
    }

    fn description(&self) -> &str {
        "Rename layer"
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

/// Command to show or hide a layer.
pub struct SetLayerVisibleCommand {
    index: usize,
    old: bool,
    new: bool,
    label: String,
}

impl SetLayerVisibleCommand {
    /// Create a visibility command; the label reflects the direction.
    pub fn new(index: usize, old: bool, new: bool) -> Self {
        Self {
            index,
            old,
            new,
            label: if new { "Show layer" } else { "Hide layer" }.to_string(),
        }
    }
}

impl LayerCommand for SetLayerVisibleCommand {
    fn apply(&mut self, stack: &mut LayerStack) {
        stack.set_visible(self.index, self.new);
    }

    fn undo(&mut self, stack: &mut LayerStack) {
        stack.set_visible(self.index, self.old);
    }

    fn description(&self) -> &str {
        &self.label
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

/// Command to lock or unlock a layer.
pub struct SetLayerLockedCommand {
    index: usize,
    old: bool,
    new: bool,
    label: String,
}

impl SetLayerLockedCommand {
    /// Create a lock command; the label reflects the direction.
    pub fn new(index: usize, old: bool, new: bool) -> Self {
        Self {
            index,
            old,
            new,
            label: if new { "Lock layer" } else { "Unlock layer" }.to_string(),
        }
    }
}

impl LayerCommand for SetLayerLockedCommand {
    fn apply(&mut self, stack: &mut LayerStack) {
        stack.set_locked(self.index, self.new);
    }

    fn undo(&mut self, stack: &mut LayerStack) {
        stack.set_locked(self.index, self.old);
    }

    fn description(&self) -> &str {
        &self.label
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

/// Command to reorder a layer.
pub struct MoveLayerCommand {
    from: usize,
    to: usize,
    label: String,
}

impl MoveLayerCommand {
    /// Create a move command between two indices.
    pub fn new(from: usize, to: usize) -> Self {
        Self {
            from,
            to,
            label: if to > from {
                "Move layer up"
            } else {
                "Move layer down"
            }
            .to_string(),
        }
    }
}

impl LayerCommand for MoveLayerCommand {
    fn apply(&mut self, stack: &mut LayerStack) {
        // `move_layer` remaps the active index onto the same layer, in both
        // directions, so no snapshot of the pre-move index is needed.
        let _ = stack.move_layer(self.from, self.to);
    }

    fn undo(&mut self, stack: &mut LayerStack) {
        let _ = stack.move_layer(self.to, self.from);
    }

    fn description(&self) -> &str {
        &self.label
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

/// Command to add a layer and make it active.
pub struct AddLayerCommand {
    index: usize,
    layer: Layer,
    active_before: u64,
    applied: bool,
}

impl AddLayerCommand {
    /// Build a command that appends a layer sized and named like `LayerStack::add_layer`.
    pub fn adding_next(stack: &LayerStack) -> Self {
        let index = stack.len();
        let active = stack.active();
        let layer = Layer::with_grid(
            format!("Layer {}", index + 1),
            crate::core::grid::Grid::new(active.grid().width(), active.grid().height()),
        );
        Self {
            index,
            layer,
            active_before: stack.active_id(),
            applied: false,
        }
    }

    /// Index the new layer occupies.
    pub fn index(&self) -> usize {
        self.index
    }
}

impl LayerCommand for AddLayerCommand {
    fn apply(&mut self, stack: &mut LayerStack) {
        if !self.applied {
            stack.insert_layer_tracking(self.index, self.layer.clone());
            self.applied = true;
        }
        stack.set_active(self.index);
    }

    fn undo(&mut self, stack: &mut LayerStack) {
        if self.applied {
            let _ = stack.remove_layer_tracking(self.index);
            // Adding a layer does not imply the one before it was the active one.
            stack.set_active_id(self.active_before);
            self.applied = false;
        }
    }

    fn description(&self) -> &str {
        "Add layer"
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

/// Command to remove a layer.
pub struct DeleteLayerCommand {
    index: usize,
    layer: Option<Layer>,
    active_before: u64,
    applied: bool,
}

impl DeleteLayerCommand {
    /// Build a command for deleting `index`. Returns None for an invalid index
    /// or when it would remove the last remaining layer.
    pub fn deleting(stack: &LayerStack, index: usize) -> Option<Self> {
        if stack.len() <= 1 || index >= stack.len() {
            return None;
        }
        Some(Self {
            index,
            layer: None,
            active_before: stack.active_id(),
            applied: false,
        })
    }
}

impl LayerCommand for DeleteLayerCommand {
    fn apply(&mut self, stack: &mut LayerStack) {
        if !self.applied {
            if let Some(removed) = stack.remove_layer_tracking(self.index) {
                self.layer = Some(removed);
                self.applied = true;
            }
        }
    }

    fn undo(&mut self, stack: &mut LayerStack) {
        if self.applied {
            if let Some(layer) = self.layer.take() {
                stack.insert_layer_tracking(self.index, layer);
            }
            // Focus goes back to whatever was active before the delete, which is
            // not necessarily the layer that ended up at this index.
            stack.set_active_id(self.active_before);
            self.applied = false;
        }
    }

    fn description(&self) -> &str {
        "Delete layer"
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

/// Command to merge a layer down into the layer below it.
pub struct MergeLayerDownCommand {
    index: usize,
    upper: Option<Layer>,
    lower_cells: Vec<(i32, i32, Option<Cell>)>,
    active_before: u64,
    applied: bool,
}

impl MergeLayerDownCommand {
    /// Build a command merging layer `index` into the layer below it.
    ///
    /// Returns `None` for index 0 (nothing below it) or an out-of-range index.
    pub fn merging(stack: &LayerStack, index: usize) -> Option<Self> {
        if index == 0 || index >= stack.len() {
            return None;
        }
        Some(Self {
            index,
            upper: None,
            lower_cells: Vec::new(),
            active_before: stack.active_id(),
            applied: false,
        })
    }
}

impl LayerCommand for MergeLayerDownCommand {
    fn apply(&mut self, stack: &mut LayerStack) {
        let lower = self.index - 1;
        if !self.applied {
            let upper = match stack.remove_layer_tracking(self.index) {
                Some(upper) => upper,
                None => return,
            };
            // Record exactly the lower cells this merge overwrites (empty ones as
            // None) so undo restores the lower layer cell for cell.
            self.lower_cells = stack.merge_overlap(lower, upper.grid());
            self.upper = Some(upper);
            self.applied = true;
        }
        if let Some(upper) = self.upper.as_ref() {
            let grid = upper.grid().clone();
            stack.blit_visible_into(lower, &grid);
        }
    }

    fn undo(&mut self, stack: &mut LayerStack) {
        if self.applied {
            let lower = self.index - 1;
            stack.restore_optional_cells(lower, &self.lower_cells);
            if let Some(upper) = self.upper.take() {
                stack.insert_layer_tracking(self.index, upper);
            }
            // Merging typically leaves the lower layer active; undo returns focus
            // to the upper layer the user was on.
            stack.set_active_id(self.active_before);
            self.applied = false;
        }
    }

    fn description(&self) -> &str {
        "Merge layer down"
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
