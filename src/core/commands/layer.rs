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

    /// Whether this command can change layer *content* (grids, or which layer is
    /// active) rather than just metadata.
    ///
    /// The editor keeps the active layer's live grid in `state.grid` and only
    /// writes it back to the layer on the next sync, so a metadata undo must not
    /// reload the grid from the layer copy: that copy predates any drawing done
    /// since. Reloading is reserved for commands that can have changed content.
    fn changes_content(&self) -> bool {
        false
    }

    /// Get a reference to Any for downcasting.
    fn as_any(&self) -> &dyn std::any::Any;

    /// Get a mutable reference to Any for downcasting.
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

/// Command to rename a layer.
pub struct SetLayerNameCommand {
    id: u64,
    old_name: String,
    new_name: String,
}

impl SetLayerNameCommand {
    /// Create a rename command for the layer with this id, carrying both names.
    pub fn new(id: u64, old_name: String, new_name: String) -> Self {
        Self {
            id,
            old_name,
            new_name,
        }
    }
}

impl LayerCommand for SetLayerNameCommand {
    fn apply(&mut self, stack: &mut LayerStack) {
        let _ = stack.set_name_by_id(self.id, self.new_name.clone());
    }

    fn undo(&mut self, stack: &mut LayerStack) {
        let _ = stack.set_name_by_id(self.id, self.old_name.clone());
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
    id: u64,
    old: bool,
    new: bool,
    label: String,
}

impl SetLayerVisibleCommand {
    /// Create a visibility command for the layer with this id.
    pub fn new(id: u64, old: bool, new: bool) -> Self {
        Self {
            id,
            old,
            new,
            label: if new { "Show layer" } else { "Hide layer" }.to_string(),
        }
    }
}

impl LayerCommand for SetLayerVisibleCommand {
    fn apply(&mut self, stack: &mut LayerStack) {
        let _ = stack.set_visible_by_id(self.id, self.new);
    }

    fn undo(&mut self, stack: &mut LayerStack) {
        let _ = stack.set_visible_by_id(self.id, self.old);
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
    id: u64,
    old: bool,
    new: bool,
    label: String,
}

impl SetLayerLockedCommand {
    /// Create a lock command for the layer with this id.
    pub fn new(id: u64, old: bool, new: bool) -> Self {
        Self {
            id,
            old,
            new,
            label: if new { "Lock layer" } else { "Unlock layer" }.to_string(),
        }
    }
}

impl LayerCommand for SetLayerLockedCommand {
    fn apply(&mut self, stack: &mut LayerStack) {
        let _ = stack.set_locked_by_id(self.id, self.new);
    }

    fn undo(&mut self, stack: &mut LayerStack) {
        let _ = stack.set_locked_by_id(self.id, self.old);
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
    id: u64,
    label: String,
}

impl MoveLayerCommand {
    /// Create a move command for the layer with this id, between two indices.
    pub fn new(from: usize, to: usize, id: u64) -> Self {
        Self {
            from,
            to,
            id,
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
        self.run(stack, self.from, self.to);
    }

    fn undo(&mut self, stack: &mut LayerStack) {
        self.run(stack, self.to, self.from);
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

impl MoveLayerCommand {
    /// Move only while the indices still describe this layer: if the stack was
    /// rearranged by something else, a positional move would move the wrong one.
    fn run(&self, stack: &mut LayerStack, from: usize, to: usize) {
        if stack.get(from).map(Layer::id) == Some(self.id) {
            let _ = stack.move_layer(from, to);
        }
    }
}

/// Command to add a layer and make it active.
pub struct AddLayerCommand {
    index: usize,
    /// Id of the layer that sat directly below the new one when it was first
    /// applied. Indices shift when layers are reordered or removed, so a redo
    /// re-derives the insert position from this anchor instead of trusting the
    /// recorded index. `None` means the new layer went to the bottom.
    below_id: Option<u64>,
    layer: Layer,
    active_before: u64,
    applied: bool,
}

impl AddLayerCommand {
    /// Build a command that appends a layer sized and named like `LayerStack::add_layer`.
    pub fn adding_next(stack: &mut LayerStack) -> Self {
        let index = stack.len();
        let active = stack.active();
        let mut layer = Layer::with_grid(
            format!("Layer {}", index + 1),
            crate::core::grid::Grid::new(active.grid().width(), active.grid().height()),
        );
        // Take the id now so apply and undo can find this exact layer later.
        layer.set_id(stack.reserve_layer_id());
        let below_id = below_id_at(stack, index);
        Self {
            index,
            below_id,
            layer,
            active_before: stack.active_id(),
            applied: false,
        }
    }

    /// Id the added layer will carry.
    pub fn layer_id(&self) -> u64 {
        self.layer.id()
    }

    /// Index the new layer occupies.
    pub fn index(&self) -> usize {
        self.index
    }
}

/// Id of the layer directly below `index`, or `None` at the bottom of the stack.
/// This is the anchor a command records so a replayed layer lands in the right
/// *place* rather than at an index that has since shifted.
fn below_id_at(stack: &LayerStack, index: usize) -> Option<u64> {
    if index == 0 {
        None
    } else {
        stack.layer_id_at(index - 1)
    }
}

/// Where a layer should be (re-)inserted, re-derived from the `below_id` anchor.
/// The recorded index is only a fallback for when that neighbour is gone.
fn insert_position(stack: &LayerStack, below_id: Option<u64>, recorded: usize) -> usize {
    match below_id.and_then(|id| stack.index_of_id(id)) {
        Some(below) => below + 1,
        None => recorded.min(stack.len()),
    }
}

impl LayerCommand for AddLayerCommand {
    fn apply(&mut self, stack: &mut LayerStack) {
        if !self.applied {
            let target = insert_position(stack, self.below_id, self.index);
            stack.insert_layer_tracking(target, self.layer.clone());
            self.applied = true;
        }
        // Focus the layer by id, not by the index it happened to land on.
        stack.set_active_id(self.layer.id());
    }

    fn undo(&mut self, stack: &mut LayerStack) {
        if self.applied {
            let _ = stack.remove_layer_by_id(self.layer.id());
            // Adding a layer does not imply the one before it was the active one.
            stack.set_active_id(self.active_before);
            self.applied = false;
        }
    }

    fn changes_content(&self) -> bool {
        true
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
    id: u64,
    /// Id of the layer that sat directly below the deleted one. The undo re-inserts
    /// after this anchor so the layer returns to the right *place*, not merely to
    /// the index it was removed from. `None` means it was the bottom layer.
    below_id: Option<u64>,
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
        let id = stack.layer_id_at(index)?;
        let below_id = below_id_at(stack, index);
        Some(Self {
            index,
            // The id is captured up front so apply finds that layer even if the
            // stack was renumbered between recording and running.
            id,
            below_id,
            layer: None,
            active_before: stack.active_id(),
            applied: false,
        })
    }
}

impl LayerCommand for DeleteLayerCommand {
    fn apply(&mut self, stack: &mut LayerStack) {
        if !self.applied {
            if let Some(removed) = stack.remove_layer_by_id(self.id) {
                self.layer = Some(removed);
                self.applied = true;
            }
        }
    }

    fn undo(&mut self, stack: &mut LayerStack) {
        if self.applied {
            if let Some(layer) = self.layer.take() {
                // Back where it belonged, not merely at the index it came from.
                let target = insert_position(stack, self.below_id, self.index);
                stack.insert_layer_tracking(target, layer);
            }
            // Focus goes back to whatever was active before the delete, which is
            // not necessarily the layer that ended up at this index.
            stack.set_active_id(self.active_before);
            self.applied = false;
        }
    }

    fn changes_content(&self) -> bool {
        true
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
    upper_id: u64,
    lower_id: u64,
    upper: Option<Layer>,
    lower_cells: Vec<(i32, i32, Option<Cell>)>,
    active_before: u64,
    applied: bool,
}

impl MergeLayerDownCommand {
    /// Build a command merging layer `index` into the layer below it.
    ///
    /// Returns `None` for index 0 (nothing below it) or an out-of-range index.
    /// Both layers are captured by id, so a later redo cannot merge or restore
    /// whichever layers happen to sit at those indices by then.
    pub fn merging(stack: &LayerStack, index: usize) -> Option<Self> {
        if index == 0 || index >= stack.len() {
            return None;
        }
        Some(Self {
            upper_id: stack.layer_id_at(index)?,
            lower_id: stack.layer_id_at(index - 1)?,
            upper: None,
            lower_cells: Vec::new(),
            active_before: stack.active_id(),
            applied: false,
        })
    }
}

impl LayerCommand for MergeLayerDownCommand {
    fn apply(&mut self, stack: &mut LayerStack) {
        if !self.applied {
            // Locate both layers by id: refuse to merge if the stack no longer has
            // the pair this command was built for.
            let upper_index = match stack.index_of_id(self.upper_id) {
                Some(index) if index > 0 => index,
                _ => return,
            };
            if stack.layer_id_at(upper_index - 1) != Some(self.lower_id) {
                return;
            }
            let upper = match stack.remove_layer_by_id(self.upper_id) {
                Some(upper) => upper,
                None => return,
            };
            // Record exactly the lower cells this merge overwrites (empty ones as
            // None) so undo restores the lower layer cell for cell.
            self.lower_cells = stack.merge_overlap(upper_index - 1, upper.grid());
            self.upper = Some(upper);
            self.applied = true;
        }
        let (Some(upper), Some(lower)) = (self.upper.as_ref(), stack.index_of_id(self.lower_id))
        else {
            return;
        };
        let grid = upper.grid().clone();
        stack.blit_visible_into(lower, &grid);
    }

    fn undo(&mut self, stack: &mut LayerStack) {
        if self.applied {
            if let Some(lower) = stack.index_of_id(self.lower_id) {
                stack.restore_optional_cells(lower, &self.lower_cells);
            }
            if let Some(upper) = self.upper.take() {
                // Put the upper layer back directly above the layer it was merged
                // into, wherever that layer now sits.
                let index = stack
                    .index_of_id(self.lower_id)
                    .map(|index| index + 1)
                    .unwrap_or(stack.len());
                stack.insert_layer_tracking(index, upper);
            }
            // Focus returns to the layer that was active before the merge.
            stack.set_active_id(self.active_before);
            self.applied = false;
        }
    }

    fn changes_content(&self) -> bool {
        true
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
