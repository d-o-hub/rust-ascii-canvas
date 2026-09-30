//! History module - Undo/Redo system using a ring buffer.
//!
//! Each layer owns a `History` (see [`crate::core::layer::Layer`]), so undo never
//! crosses layers. A single layer's timeline holds both kinds of entry, which is
//! what makes "draw, rename, draw, undo, undo" behave the way a user expects.

use crate::core::commands::{Command, LayerCommand};
use crate::core::grid::Grid;
use crate::core::layer::LayerStack;
use std::collections::VecDeque;

/// Maximum history depth.
pub const DEFAULT_MAX_DEPTH: usize = 100;

/// What an undo or redo call actually did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HistoryOutcome {
    /// Nothing to undo or redo.
    None,
    /// A grid command ran; the caller's grid holds the result.
    Grid,
    /// A layer command ran that only changed metadata (name, visibility, lock,
    /// order). The live drawing surface is untouched, so the caller must not
    /// reload it from the layer copy: that copy predates any drawing done since.
    LayerMeta,
    /// A layer command ran that can change layer content (add, delete, merge).
    /// The layer stack is authoritative and the caller should reload from it.
    LayerContent,
}

/// One entry on a per-layer timeline.
pub enum HistoryEntry {
    /// A drawing command operating on the active grid.
    Grid(Box<dyn Command>),
    /// A structural command operating on the layer stack.
    Layer(Box<dyn LayerCommand>),
}

impl HistoryEntry {
    /// Whether this entry changes the grid, only layer metadata, or layer content.
    pub fn kind(&self) -> HistoryOutcome {
        match self {
            HistoryEntry::Grid(_) => HistoryOutcome::Grid,
            HistoryEntry::Layer(cmd) => {
                if cmd.changes_content() {
                    HistoryOutcome::LayerContent
                } else {
                    HistoryOutcome::LayerMeta
                }
            }
        }
    }

    /// Human-readable description, used for undo/redo labels.
    pub fn description(&self) -> &str {
        match self {
            HistoryEntry::Grid(cmd) => cmd.description(),
            HistoryEntry::Layer(cmd) => cmd.description(),
        }
    }

    fn apply(&mut self, grid: &mut Grid, layers: &mut LayerStack) {
        match self {
            HistoryEntry::Grid(cmd) => cmd.apply(grid),
            HistoryEntry::Layer(cmd) => cmd.apply(layers),
        }
    }

    fn undo(&mut self, grid: &mut Grid, layers: &mut LayerStack) {
        match self {
            HistoryEntry::Grid(cmd) => cmd.undo(grid),
            HistoryEntry::Layer(cmd) => cmd.undo(layers),
        }
    }
}

/// Undo/Redo history manager.
pub struct History {
    /// Undo stack
    undo_stack: VecDeque<HistoryEntry>,
    /// Redo stack
    redo_stack: VecDeque<HistoryEntry>,
    /// Maximum depth
    max_depth: usize,
}

impl std::fmt::Debug for History {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("History")
            .field("max_depth", &self.max_depth)
            .field("undo_count", &self.undo_stack.len())
            .field("redo_count", &self.redo_stack.len())
            .finish()
    }
}

impl Default for History {
    fn default() -> Self {
        Self::new(DEFAULT_MAX_DEPTH)
    }
}

impl History {
    /// Create a new history with the given max depth.
    pub fn new(max_depth: usize) -> Self {
        Self {
            undo_stack: VecDeque::with_capacity(max_depth),
            redo_stack: VecDeque::with_capacity(max_depth),
            max_depth,
        }
    }

    /// Push a grid command onto the undo stack. Clears the redo stack.
    pub fn push(&mut self, command: Box<dyn Command>) {
        self.push_entry(HistoryEntry::Grid(command));
    }

    /// Push a structural layer command onto the undo stack. Clears the redo stack.
    ///
    /// Layer commands never merge, and only merge against a grid command that
    /// reports it can merge, so the timeline order stays faithful.
    pub fn push_layer(&mut self, command: Box<dyn LayerCommand>) {
        self.push_entry(HistoryEntry::Layer(command));
    }

    fn push_entry(&mut self, entry: HistoryEntry) {
        // Clear redo stack when new action is performed
        self.redo_stack.clear();

        match entry {
            HistoryEntry::Grid(new) => {
                // Try merging with the last grid command if possible.
                if let Some(HistoryEntry::Grid(last)) = self.undo_stack.back_mut() {
                    if last.can_merge(new.as_ref()) {
                        last.merge(new);
                        return;
                    }
                }
                self.push_raw(HistoryEntry::Grid(new));
            }
            other => self.push_raw(other),
        }
    }

    fn push_raw(&mut self, entry: HistoryEntry) {
        // Enforce max depth
        if self.undo_stack.len() >= self.max_depth {
            self.undo_stack.pop_front();
        }
        self.undo_stack.push_back(entry);
    }

    /// Perform undo, returning the entry to the redo stack.
    ///
    /// A locked active layer blocks grid entries, but must not trap the user
    /// behind a lock change they cannot undo, so a structural entry on top is
    /// always undoable (ADR-043 decision 6).
    pub fn undo(&mut self, grid: &mut Grid, layers: &mut LayerStack) -> HistoryOutcome {
        let kind = match self.undo_stack.back() {
            Some(entry) => entry.kind(),
            None => return HistoryOutcome::None,
        };
        if kind == HistoryOutcome::Grid && layers.is_active_locked() {
            return HistoryOutcome::None;
        }
        if matches!(self.undo_stack.back(), Some(HistoryEntry::Layer(cmd)) if !cmd.can_undo(layers))
        {
            return HistoryOutcome::None;
        }
        if let Some(mut entry) = self.undo_stack.pop_back() {
            entry.undo(grid, layers);
            self.redo_stack.push_back(entry);
            kind
        } else {
            HistoryOutcome::None
        }
    }

    /// Perform redo, returning the entry to the undo stack.
    ///
    /// Blocking rules match [`History::undo`]: grid entries are blocked on a
    /// locked active layer, structural entries are not.
    pub fn redo(&mut self, grid: &mut Grid, layers: &mut LayerStack) -> HistoryOutcome {
        let kind = match self.redo_stack.back() {
            Some(entry) => entry.kind(),
            None => return HistoryOutcome::None,
        };
        if kind == HistoryOutcome::Grid && layers.is_active_locked() {
            return HistoryOutcome::None;
        }
        if matches!(self.redo_stack.back(), Some(HistoryEntry::Layer(cmd)) if !cmd.can_apply(layers))
        {
            return HistoryOutcome::None;
        }
        if let Some(mut entry) = self.redo_stack.pop_back() {
            entry.apply(grid, layers);
            self.undo_stack.push_back(entry);
            kind
        } else {
            HistoryOutcome::None
        }
    }

    /// Check if undo is available.
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Check if redo is available.
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Get the number of undo steps available.
    pub fn undo_count(&self) -> usize {
        self.undo_stack.len()
    }

    /// Get the number of redo steps available.
    pub fn redo_count(&self) -> usize {
        self.redo_stack.len()
    }

    /// Clear all history.
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }

    /// Get description of next undo entry.
    pub fn undo_description(&self) -> Option<&str> {
        self.undo_stack.back().map(HistoryEntry::description)
    }

    /// Get description of next redo entry.
    pub fn redo_description(&self) -> Option<&str> {
        self.redo_stack.back().map(HistoryEntry::description)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::cell::Cell;
    use crate::core::commands::SetCellCommand;
    use crate::core::layer::LayerStack;

    /// Minimal structural command for timeline tests: renames a layer.
    struct RenameForTest {
        index: usize,
        old_name: String,
        new_name: String,
    }

    impl LayerCommand for RenameForTest {
        fn apply(&mut self, layers: &mut LayerStack) {
            layers.set_name(self.index, self.new_name.clone());
        }

        fn undo(&mut self, layers: &mut LayerStack) {
            layers.set_name(self.index, self.old_name.clone());
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

    fn rename(index: usize, old_name: &str, new_name: &str) -> Box<dyn LayerCommand> {
        Box::new(RenameForTest {
            index,
            old_name: old_name.to_string(),
            new_name: new_name.to_string(),
        })
    }

    fn stack() -> LayerStack {
        LayerStack::new(10, 10)
    }

    #[test]
    fn test_history_push() {
        let mut history = History::new(10);
        history.push(Box::new(SetCellCommand::new(0, 0, Cell::new('A'))));

        assert!(history.can_undo());
        assert!(!history.can_redo());
        assert_eq!(history.undo_count(), 1);
    }

    #[test]
    fn test_history_undo_redo() {
        let mut grid = Grid::new(10, 10);
        let mut layers = stack();
        let mut history = History::new(10);

        let mut cmd = SetCellCommand::new(5, 5, Cell::new('X'));
        cmd.apply(&mut grid);
        history.push(Box::new(cmd));

        assert_eq!(grid.get(5, 5).unwrap().ch, 'X');

        assert_eq!(history.undo(&mut grid, &mut layers), HistoryOutcome::Grid);
        assert!(grid.get(5, 5).unwrap().is_empty());
        assert!(!history.can_undo());
        assert!(history.can_redo());

        assert_eq!(history.redo(&mut grid, &mut layers), HistoryOutcome::Grid);
        assert_eq!(grid.get(5, 5).unwrap().ch, 'X');
        assert!(history.can_undo());
        assert!(!history.can_redo());
    }

    #[test]
    fn test_history_max_depth() {
        let mut history = History::new(5);

        for i in 0..10 {
            history.push(Box::new(SetCellCommand::new(i, 0, Cell::new('X'))));
        }

        assert_eq!(history.undo_count(), 5);
    }

    #[test]
    fn test_history_clear_on_push() {
        let mut grid = Grid::new(10, 10);
        let mut layers = stack();
        let mut history = History::new(10);

        let mut cmd1 = SetCellCommand::new(0, 0, Cell::new('A'));
        cmd1.apply(&mut grid);
        history.push(Box::new(cmd1));

        history.undo(&mut grid, &mut layers);
        assert_eq!(history.redo_count(), 1);

        let mut cmd2 = SetCellCommand::new(1, 0, Cell::new('B'));
        cmd2.apply(&mut grid);
        history.push(Box::new(cmd2));

        assert_eq!(history.redo_count(), 0);
    }

    #[test]
    fn test_layer_entry_undo_and_redo() {
        let mut grid = Grid::new(10, 10);
        let mut layers = stack();
        let mut history = History::new(10);

        let mut cmd = rename(0, "Layer 1", "Renamed");
        cmd.apply(&mut layers);
        history.push_layer(cmd);
        assert_eq!(layers.active().name(), "Renamed");
        assert_eq!(history.undo_description(), Some("Rename layer"));

        assert_eq!(
            history.undo(&mut grid, &mut layers),
            HistoryOutcome::LayerMeta
        );
        assert_eq!(layers.active().name(), "Layer 1");
        assert_eq!(history.redo_description(), Some("Rename layer"));

        assert_eq!(
            history.redo(&mut grid, &mut layers),
            HistoryOutcome::LayerMeta
        );
        assert_eq!(layers.active().name(), "Renamed");
    }

    #[test]
    fn test_grid_and_layer_entries_share_one_timeline() {
        let mut grid = Grid::new(10, 10);
        let mut layers = stack();
        let mut history = History::new(10);

        let mut draw = SetCellCommand::new(0, 0, Cell::new('A'));
        draw.apply(&mut grid);
        history.push(Box::new(draw));

        let mut rename_cmd = rename(0, "Layer 1", "Second");
        rename_cmd.apply(&mut layers);
        history.push_layer(rename_cmd);

        assert_eq!(history.undo_count(), 2);
        // The structural entry is on top, so it undoes first even though the
        // active layer is locked: a lock must not trap the user.
        layers.set_locked(0, true);
        assert_eq!(
            history.undo(&mut grid, &mut layers),
            HistoryOutcome::LayerMeta
        );
        assert_eq!(layers.active().name(), "Layer 1");

        // Now a grid entry is on top and the lock blocks it. The entry stays on
        // the stack: a blocked undo skips the command, it does not discard it.
        assert_eq!(history.undo(&mut grid, &mut layers), HistoryOutcome::None);
        assert!(history.can_undo());
        assert_eq!(history.undo_count(), 1);
        assert_eq!(history.redo_count(), 1);
        assert_eq!(grid.get(0, 0).unwrap().ch, 'A');

        // Unlocking makes the same entry undoable again.
        layers.set_locked(0, false);
        assert_eq!(history.undo(&mut grid, &mut layers), HistoryOutcome::Grid);
        assert!(grid.get(0, 0).unwrap().is_empty());
    }

    #[test]
    fn test_grid_undo_blocked_while_active_layer_locked() {
        let mut grid = Grid::new(10, 10);
        let mut layers = stack();
        let mut history = History::new(10);

        let mut draw = SetCellCommand::new(1, 1, Cell::new('B'));
        draw.apply(&mut grid);
        history.push(Box::new(draw));
        layers.set_locked(0, true);

        assert_eq!(history.undo(&mut grid, &mut layers), HistoryOutcome::None);
        assert_eq!(history.undo_count(), 1, "the entry must not be consumed");
        assert_eq!(grid.get(1, 1).unwrap().ch, 'B');

        layers.set_locked(0, false);
        assert_eq!(history.undo(&mut grid, &mut layers), HistoryOutcome::Grid);
        assert!(grid.get(1, 1).unwrap().is_empty());
    }

    #[test]
    fn test_grid_redo_blocked_while_active_layer_locked() {
        let mut grid = Grid::new(10, 10);
        let mut layers = stack();
        let mut history = History::new(10);

        let mut draw = SetCellCommand::new(1, 1, Cell::new('B'));
        draw.apply(&mut grid);
        history.push(Box::new(draw));
        history.undo(&mut grid, &mut layers);

        layers.set_locked(0, true);
        assert_eq!(history.redo(&mut grid, &mut layers), HistoryOutcome::None);
        assert_eq!(history.redo_count(), 1, "the entry must not be consumed");

        layers.set_locked(0, false);
        assert_eq!(history.redo(&mut grid, &mut layers), HistoryOutcome::Grid);
        assert_eq!(grid.get(1, 1).unwrap().ch, 'B');
    }

    #[test]
    fn test_push_layer_clears_redo_stack() {
        let mut grid = Grid::new(10, 10);
        let mut layers = stack();
        let mut history = History::new(10);

        let mut cmd = rename(0, "Layer 1", "Renamed");
        cmd.apply(&mut layers);
        history.push_layer(cmd);
        history.undo(&mut grid, &mut layers);
        assert_eq!(history.redo_count(), 1);

        let mut next = rename(0, "Layer 1", "Other");
        next.apply(&mut layers);
        history.push_layer(next);
        assert_eq!(history.redo_count(), 0);
    }

    #[test]
    fn test_layer_entries_never_merge() {
        let mut history = History::new(10);
        history.push_layer(rename(0, "a", "b"));
        history.push_layer(rename(0, "b", "c"));
        assert_eq!(history.undo_count(), 2);
    }

    #[test]
    fn test_empty_history_reports_none() {
        let mut grid = Grid::new(4, 4);
        let mut layers = stack();
        let mut history = History::new(10);
        assert_eq!(history.undo(&mut grid, &mut layers), HistoryOutcome::None);
        assert_eq!(history.redo(&mut grid, &mut layers), HistoryOutcome::None);
        assert_eq!(history.undo_description(), None);
        assert_eq!(history.redo_description(), None);
    }
}
