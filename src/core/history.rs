//! History module - Undo/Redo system using a ring buffer.

use crate::core::commands::Command;
use crate::core::grid::Grid;
use std::collections::VecDeque;

/// Maximum history depth.
const DEFAULT_MAX_DEPTH: usize = 100;

/// Undo/Redo history manager.
pub struct History {
    /// Undo stack
    undo_stack: VecDeque<Box<dyn Command>>,
    /// Redo stack
    redo_stack: VecDeque<Box<dyn Command>>,
    /// Maximum depth
    max_depth: usize,
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

    /// Push a command onto the undo stack.
    /// Clears the redo stack.
    pub fn push(&mut self, command: Box<dyn Command>) {
        // Clear redo stack when new action is performed
        self.redo_stack.clear();

        // Enforce max depth
        if self.undo_stack.len() >= self.max_depth {
            self.undo_stack.pop_front();
        }

        self.undo_stack.push_back(command);
    }

    /// Perform undo, returning the command to the redo stack.
    pub fn undo(&mut self, grid: &mut Grid) -> bool {
        if let Some(mut cmd) = self.undo_stack.pop_back() {
            cmd.undo(grid);
            self.redo_stack.push_back(cmd);
            true
        } else {
            false
        }
    }

    /// Perform redo, returning the command to the undo stack.
    pub fn redo(&mut self, grid: &mut Grid) -> bool {
        if let Some(mut cmd) = self.redo_stack.pop_back() {
            cmd.apply(grid);
            self.undo_stack.push_back(cmd);
            true
        } else {
            false
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

    /// Get description of next undo command.
    pub fn undo_description(&self) -> Option<&str> {
        self.undo_stack.back().map(|c| c.description())
    }

    /// Get description of next redo command.
    pub fn redo_description(&self) -> Option<&str> {
        self.redo_stack.back().map(|c| c.description())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::cell::Cell;
    use crate::core::commands::SetCellCommand;

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
        let mut history = History::new(10);

        // Push and apply a command
        let mut cmd = SetCellCommand::new(5, 5, Cell::new('X'));
        cmd.apply(&mut grid);
        history.push(Box::new(cmd));

        assert_eq!(grid.get(5, 5).unwrap().ch, 'X');

        // Undo
        history.undo(&mut grid);
        assert!(grid.get(5, 5).unwrap().is_empty());
        assert!(!history.can_undo());
        assert!(history.can_redo());

        // Redo
        history.redo(&mut grid);
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
        let mut history = History::new(10);

        // Push and apply first command
        let mut cmd1 = SetCellCommand::new(0, 0, Cell::new('A'));
        cmd1.apply(&mut grid);
        history.push(Box::new(cmd1));

        // Undo
        history.undo(&mut grid);
        assert_eq!(history.redo_count(), 1);

        // Push new command - should clear redo
        let mut cmd2 = SetCellCommand::new(1, 0, Cell::new('B'));
        cmd2.apply(&mut grid);
        history.push(Box::new(cmd2));

        assert_eq!(history.redo_count(), 0);
    }
}
