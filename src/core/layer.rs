//! Layer model - pure layer and layer-stack state (ADR-043).
//!
//! Layer state used to live in `src/wasm` as a `LayerData` struct with the
//! active layer's history swapped in and out of the editor on every switch.
//! Moving it here makes the structural rules testable without a WASM build and
//! gives the layer commands (see [`crate::core::commands::LayerCommand`]) a
//! pure-core target to apply to.
//!
//! Each layer owns its own [`History`], so undo never crosses layers (ADR-043
//! open decision 1). The active layer's history is a single timeline holding
//! both grid commands and structural layer commands.

use crate::core::grid::Grid;
use crate::core::history::{History, DEFAULT_MAX_DEPTH};

// The stack itself lives in `layer_stack.rs`; re-exported here so the public
// path `core::layer::LayerStack` keeps working.
pub use super::layer_stack::LayerStack;

/// A single layer: metadata, cell content, and its own undo/redo history.
///
/// Every layer carries a stable id. Indices shift when layers are inserted or
/// removed, so anything that must remember *which* layer it belongs to across a
/// structural undo (a history, the previously active layer) refers to this id
/// rather than to an index.
#[derive(Debug)]
pub struct Layer {
    pub(crate) id: u64,
    pub(crate) name: String,
    pub(crate) visible: bool,
    pub(crate) locked: bool,
    pub(crate) grid: Grid,
    pub(crate) history: History,
}

impl Clone for Layer {
    /// History is intentionally reset: a cloned layer must not replay commands
    /// that were recorded against different content.
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            name: self.name.clone(),
            visible: self.visible,
            locked: self.locked,
            grid: self.grid.clone(),
            history: History::new(DEFAULT_MAX_DEPTH),
        }
    }
}

impl Layer {
    /// Create an empty layer with the given name and grid size.
    pub fn new(name: impl Into<String>, width: usize, height: usize) -> Self {
        Self {
            id: 0,
            name: name.into(),
            visible: true,
            locked: false,
            grid: Grid::new(width, height),
            history: History::new(DEFAULT_MAX_DEPTH),
        }
    }

    /// Create a layer around an existing grid.
    pub fn with_grid(name: impl Into<String>, grid: Grid) -> Self {
        Self {
            id: 0,
            name: name.into(),
            visible: true,
            locked: false,
            grid,
            history: History::new(DEFAULT_MAX_DEPTH),
        }
    }

    /// Stable identity of this layer.
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Layer name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Rename the layer.
    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    /// Whether the layer is rendered.
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Show or hide the layer.
    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    /// Whether the layer rejects edits.
    pub fn is_locked(&self) -> bool {
        self.locked
    }

    /// Lock or unlock the layer.
    pub fn set_locked(&mut self, locked: bool) {
        self.locked = locked;
    }

    /// Read-only access to the layer's cells.
    pub fn grid(&self) -> &Grid {
        &self.grid
    }

    /// Mutable access to the layer's cells.
    pub fn grid_mut(&mut self) -> &mut Grid {
        &mut self.grid
    }

    /// Read-only access to this layer's own history.
    pub fn history(&self) -> &History {
        &self.history
    }

    /// Mutable access to this layer's own history.
    pub fn history_mut(&mut self) -> &mut History {
        &mut self.history
    }
}

/// Active index after removing layer `removed` from a stack of `len_after` layers.
pub fn active_after_remove(active: usize, removed: usize, len_after: usize) -> usize {
    if active == removed {
        removed.min(len_after.saturating_sub(1))
    } else if active > removed {
        active - 1
    } else {
        active
    }
}

/// Active index after inserting a layer at `inserted`.
pub fn active_after_insert(active: usize, inserted: usize) -> usize {
    if active >= inserted {
        active + 1
    } else {
        active
    }
}

/// Active index after moving the layer at `from` to `to`.
pub fn active_after_move(active: usize, from: usize, to: usize) -> usize {
    if active == from {
        to
    } else if from < to && active > from && active <= to {
        active - 1
    } else if from > to && active >= to && active < from {
        active + 1
    } else {
        active
    }
}
