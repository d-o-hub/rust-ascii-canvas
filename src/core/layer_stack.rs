//! Layer stack operations (ADR-043).
//!
//! Split from `layer.rs` to keep both files inside the 500-line budget. The
//! stack owns the rules that are easy to get subtly wrong: which index stays
//! active across an insert or remove, and which layer a history belongs to.

use crate::core::cell::Cell;
use crate::core::commands::{Command, LayerCommand};
use crate::core::grid::Grid;
use crate::core::history::{History, HistoryOutcome};
use crate::core::layer::{active_after_insert, active_after_move, active_after_remove, Layer};

/// The ordered set of layers plus which one is active.
#[derive(Debug)]
pub struct LayerStack {
    layers: Vec<Layer>,
    active: usize,
    next_id: u64,
}

impl LayerStack {
    /// Create a stack holding a single empty layer named `Layer 1`.
    pub fn new(width: usize, height: usize) -> Self {
        let mut stack = Self {
            layers: Vec::new(),
            active: 0,
            next_id: 1,
        };
        stack.push_layer(Layer::new("Layer 1", width, height));
        stack
    }

    /// Create a stack from existing layers and an active index (clamped).
    pub fn from_layers(layers: Vec<Layer>, active: usize) -> Self {
        let active = active.min(layers.len().saturating_sub(1));
        let next_id = layers.iter().map(Layer::id).max().unwrap_or(0) + 1;
        let mut stack = Self {
            layers,
            active,
            next_id,
        };
        for layer in &mut stack.layers {
            if layer.id == 0 {
                layer.id = stack.next_id;
                stack.next_id += 1;
            }
        }
        stack
    }

    /// Append a layer, assigning it a fresh id.
    fn push_layer(&mut self, layer: Layer) {
        let mut layer = layer;
        layer.id = self.next_id;
        self.next_id += 1;
        self.layers.push(layer);
    }

    /// Index of the layer with this id.
    pub fn index_of_id(&self, id: u64) -> Option<usize> {
        self.layers.iter().position(|layer| layer.id == id)
    }

    /// Id of the active layer.
    pub fn active_id(&self) -> u64 {
        self.layers[self.active].id()
    }

    /// Activate the layer with this id. Returns false when it is not present.
    ///
    /// This is how an undo restores *which* layer was active: indices move
    /// around during a structural change, ids do not.
    pub fn set_active_id(&mut self, id: u64) -> bool {
        match self.index_of_id(id) {
            Some(index) => {
                self.active = index;
                true
            }
            None => false,
        }
    }

    /// Number of layers.
    pub fn len(&self) -> usize {
        self.layers.len()
    }

    /// Whether the stack holds no layers.
    pub fn is_empty(&self) -> bool {
        self.layers.is_empty()
    }

    /// All layers, bottom to top.
    pub fn layers(&self) -> &[Layer] {
        &self.layers
    }

    /// Layer by index.
    pub fn get(&self, index: usize) -> Option<&Layer> {
        self.layers.get(index)
    }

    /// Mutable layer by index.
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Layer> {
        self.layers.get_mut(index)
    }

    /// Index of the active layer.
    pub fn active_index(&self) -> usize {
        self.active
    }

    /// The active layer.
    pub fn active(&self) -> &Layer {
        &self.layers[self.active]
    }

    /// The active layer, mutably.
    pub fn active_mut(&mut self) -> &mut Layer {
        &mut self.layers[self.active]
    }

    /// Whether the active layer rejects edits.
    pub fn is_active_locked(&self) -> bool {
        self.layers
            .get(self.active)
            .map(Layer::is_locked)
            .unwrap_or(false)
    }

    /// Switch the active layer. Returns false when the index is out of range.
    pub fn set_active(&mut self, index: usize) -> bool {
        if index < self.layers.len() {
            self.active = index;
            true
        } else {
            false
        }
    }

    /// Rename a layer. Returns false when the index is out of range.
    pub fn set_name(&mut self, index: usize, name: String) -> bool {
        match self.layers.get_mut(index) {
            Some(layer) => {
                layer.set_name(name);
                true
            }
            None => false,
        }
    }

    /// Set layer visibility. Returns false when the index is out of range.
    pub fn set_visible(&mut self, index: usize, visible: bool) -> bool {
        match self.layers.get_mut(index) {
            Some(layer) => {
                layer.set_visible(visible);
                true
            }
            None => false,
        }
    }

    /// Set the lock state. Returns false when the index is out of range.
    pub fn set_locked(&mut self, index: usize, locked: bool) -> bool {
        match self.layers.get_mut(index) {
            Some(layer) => {
                layer.set_locked(locked);
                true
            }
            None => false,
        }
    }
}
impl LayerStack {
    /// Append an empty layer named after the current count and activate it.
    pub fn add_layer(&mut self) -> usize {
        let (width, height) = {
            let grid = self.active().grid();
            (grid.width(), grid.height())
        };
        let name = format!("Layer {}", self.layers.len() + 1);
        self.push_layer(Layer::new(name, width, height));
        self.active = self.layers.len() - 1;
        self.active
    }

    /// Insert a layer at `index`, clamped to the end of the stack.
    pub fn insert_layer(&mut self, index: usize, mut layer: Layer) {
        let index = index.min(self.layers.len());
        // A layer that already carries an id keeps it, so undoing a delete puts
        // the original layer (and its history) back under its own identity.
        if layer.id == 0 {
            layer.id = self.next_id;
            self.next_id += 1;
        }
        self.layers.insert(index, layer);
    }

    /// Remove a layer, keeping the active index on the same layer.
    ///
    /// The active index is remapped from the *live* value, which is what makes
    /// undo safe: a command that runs in reverse sees the post-apply stack, not
    /// the snapshot it was built from.
    pub fn remove_layer_tracking(&mut self, index: usize) -> Option<Layer> {
        let current = self.active;
        let removed = self.remove_layer(index)?;
        self.active = active_after_remove(current, index, self.layers.len());
        Some(removed)
    }

    /// Insert a layer, keeping the active index on the same layer.
    pub fn insert_layer_tracking(&mut self, index: usize, layer: Layer) {
        let current = self.active;
        self.insert_layer(index, layer);
        let inserted = index.min(self.layers.len() - 1);
        self.active = active_after_insert(current, inserted).min(self.layers.len() - 1);
    }

    /// Remove and return the layer at `index`.
    pub fn remove_layer(&mut self, index: usize) -> Option<Layer> {
        if index < self.layers.len() {
            Some(self.layers.remove(index))
        } else {
            None
        }
    }

    /// Move a layer, keeping the active index pointing at the same layer.
    pub fn move_layer(&mut self, from: usize, to: usize) -> bool {
        if from >= self.layers.len() || to >= self.layers.len() || from == to {
            return false;
        }
        let layer = self.layers.remove(from);
        self.layers.insert(to, layer);
        self.active = active_after_move(self.active, from, to);
        true
    }

    /// Resize every layer's grid.
    pub fn resize(&mut self, width: usize, height: usize) {
        for layer in &mut self.layers {
            layer.grid.resize(width, height);
        }
    }

    /// Clear every layer's history.
    ///
    /// Resize calls this: every layer's grid changed dimensions, so recorded draw
    /// commands (which carry coordinates, and for clear-canvas a width and a
    /// height) no longer describe the grids they would be replayed onto.
    pub fn clear_all_histories(&mut self) {
        for layer in &mut self.layers {
            layer.history.clear();
        }
    }

    /// Copy the visible cells of `source` onto layer `target`.
    pub fn blit_visible_into(&mut self, target: usize, source: &Grid) {
        if let Some(layer) = self.layers.get_mut(target) {
            for (x, y, cell) in source.iter_with_coords() {
                if cell.is_visible() {
                    let _ = layer.grid.set(x, y, *cell);
                }
            }
        }
    }

    /// The cells of layer `lower` that `upper` would overwrite, with their current
    /// values. `None` marks a cell that is currently empty, so undo can clear it.
    pub fn merge_overlap(&self, lower: usize, upper: &Grid) -> Vec<(i32, i32, Option<Cell>)> {
        let Some(layer) = self.layers.get(lower) else {
            return Vec::new();
        };
        let mut out = Vec::new();
        for (x, y, cell) in upper.iter_with_coords() {
            if cell.is_visible() {
                let existing = layer
                    .grid
                    .get(x, y)
                    .filter(|existing| existing.is_visible())
                    .copied();
                out.push((x, y, existing));
            }
        }
        out
    }

    /// Write a recorded set of cells back into a layer (used to undo a merge).
    pub fn restore_optional_cells(&mut self, index: usize, cells: &[(i32, i32, Option<Cell>)]) {
        if let Some(layer) = self.layers.get_mut(index) {
            for (x, y, cell) in cells {
                match cell {
                    Some(cell) => {
                        let _ = layer.grid.set(*x, *y, *cell);
                    }
                    None => {
                        let _ = layer.grid.clear_cell(*x, *y);
                    }
                }
            }
        }
    }
}

impl LayerStack {
    /// Record a grid command on the active layer's history.
    pub fn push_grid(&mut self, command: Box<dyn Command>) {
        self.active_mut().history.push(command);
    }

    /// Record a structural layer command on the active layer's history.
    pub fn push_layer_command(&mut self, command: Box<dyn LayerCommand>) {
        self.active_mut().history.push_layer(command);
    }

    /// Whether the active layer can undo.
    pub fn can_undo(&self) -> bool {
        self.active().history.can_undo()
    }

    /// Whether the active layer can redo.
    pub fn can_redo(&self) -> bool {
        self.active().history.can_redo()
    }

    /// Description of the next undo entry on the active layer, if any.
    pub fn undo_description(&self) -> Option<&str> {
        self.active().history.undo_description()
    }

    /// Description of the next redo entry on the active layer, if any.
    pub fn redo_description(&self) -> Option<&str> {
        self.active().history.redo_description()
    }

    /// Undo the newest entry on the active layer's history.
    ///
    /// The history is moved out for the call so a command can mutate this stack
    /// without aliasing it, and handed back to the layer it came from. When that
    /// layer is the one the command removed (undoing "add layer"), the history
    /// dies with the layer rather than leaking onto another one.
    pub fn undo_active(&mut self, grid: &mut Grid) -> HistoryOutcome {
        self.run_on_active_history(|stack, history| history.undo(grid, stack))
    }

    /// Redo the newest undone entry on the active layer's history.
    pub fn redo_active(&mut self, grid: &mut Grid) -> HistoryOutcome {
        self.run_on_active_history(|stack, history| history.redo(grid, stack))
    }

    fn run_on_active_history(
        &mut self,
        run: impl FnOnce(&mut LayerStack, &mut History) -> HistoryOutcome,
    ) -> HistoryOutcome {
        // Hand the history to the command, then give it back to the layer it came
        // from. Looking that layer up by id matters: a structural undo can put a
        // different layer at the same index, and writing to the index would
        // overwrite the layer that undo just restored. If the owning layer is
        // gone (undoing "add layer"), its history goes with it.
        let id = self.active_id();
        let index = self.active;
        let mut history = std::mem::replace(&mut self.layers[index].history, History::new(0));
        let outcome = run(self, &mut history);
        if let Some(index) = self.index_of_id(id) {
            self.layers[index].history = history;
        }
        outcome
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::commands::SetCellCommand;

    #[test]
    fn layer_clone_resets_history() {
        let mut layer = Layer::new("A", 3, 3);
        layer
            .history_mut()
            .push(Box::new(SetCellCommand::new(0, 0, Cell::new('Z'))));
        let copy = layer.clone();
        assert!(!copy.history().can_undo());
        assert_eq!(copy.name(), "A");
    }

    #[test]
    fn with_grid_keeps_the_given_cells() {
        let mut grid = Grid::new(2, 2);
        grid.set_char(1, 1, 'Q');
        let layer = Layer::with_grid("Kept", grid);
        assert_eq!(layer.name(), "Kept");
        assert_eq!(layer.grid().get(1, 1).map(|c| c.ch), Some('Q'));
    }
}
