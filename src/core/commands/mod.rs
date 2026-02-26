//! Commands module - Command pattern for undo/redo operations.

mod draw;
mod composite;

pub use draw::DrawCommand;
pub use composite::CompositeCommand;

use crate::core::grid::Grid;
use crate::core::cell::Cell;

/// Trait for undoable commands.
pub trait Command {
    /// Apply the command to the grid.
    fn apply(&mut self, grid: &mut Grid);

    /// Undo the command on the grid.
    fn undo(&mut self, grid: &mut Grid);

    /// Get a description of the command.
    fn description(&self) -> &str;

    /// Whether this command can be merged with another.
    fn can_merge(&self, _other: &dyn Command) -> bool {
        false
    }

    /// Merge another command into this one.
    fn merge(&mut self, _other: Box<dyn Command>) {
        // Default: do nothing
    }

    /// Get a reference to Any for downcasting.
    fn as_any(&self) -> &dyn std::any::Any;
}

/// Command to set a single cell.
pub struct SetCellCommand {
    x: i32,
    y: i32,
    old_cell: Option<Cell>,
    new_cell: Cell,
    applied: bool,
}

impl SetCellCommand {
    /// Create a new command to set a cell.
    pub fn new(x: i32, y: i32, new_cell: Cell) -> Self {
        Self {
            x,
            y,
            old_cell: None,
            new_cell,
            applied: false,
        }
    }

    /// Create a command with known old cell value.
    pub fn with_old(x: i32, y: i32, old_cell: Cell, new_cell: Cell) -> Self {
        Self {
            x,
            y,
            old_cell: Some(old_cell),
            new_cell,
            applied: false,
        }
    }
}

impl Command for SetCellCommand {
    fn apply(&mut self, grid: &mut Grid) {
        if !self.applied {
            self.old_cell = grid.get(self.x, self.y).copied();
            grid.set(self.x, self.y, self.new_cell);
            self.applied = true;
        }
    }

    fn undo(&mut self, grid: &mut Grid) {
        if self.applied {
            if let Some(old) = self.old_cell {
                grid.set(self.x, self.y, old);
            } else {
                grid.clear_cell(self.x, self.y);
            }
            self.applied = false;
        }
    }

    fn description(&self) -> &str {
        "Set cell"
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Command to clear a single cell.
pub struct ClearCellCommand {
    x: i32,
    y: i32,
    old_cell: Option<Cell>,
    applied: bool,
}

impl ClearCellCommand {
    /// Create a new command to clear a cell.
    pub fn new(x: i32, y: i32) -> Self {
        Self {
            x,
            y,
            old_cell: None,
            applied: false,
        }
    }
}

impl Command for ClearCellCommand {
    fn apply(&mut self, grid: &mut Grid) {
        if !self.applied {
            self.old_cell = grid.get(self.x, self.y).copied();
            grid.clear_cell(self.x, self.y);
            self.applied = true;
        }
    }

    fn undo(&mut self, grid: &mut Grid) {
        if self.applied {
            if let Some(old) = self.old_cell {
                grid.set(self.x, self.y, old);
            }
            self.applied = false;
        }
    }

    fn description(&self) -> &str {
        "Clear cell"
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Command to clear the entire grid.
pub struct ClearGridCommand {
    old_cells: Option<Vec<Cell>>,
    width: usize,
    height: usize,
    applied: bool,
}

impl ClearGridCommand {
    /// Create a new command to clear the entire grid.
    pub fn new() -> Self {
        Self {
            old_cells: None,
            width: 0,
            height: 0,
            applied: false,
        }
    }
}

impl Default for ClearGridCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl Command for ClearGridCommand {
    fn apply(&mut self, grid: &mut Grid) {
        if !self.applied {
            self.old_cells = Some(grid.cells().to_vec());
            self.width = grid.width();
            self.height = grid.height();
            grid.clear();
            self.applied = true;
        }
    }

    fn undo(&mut self, grid: &mut Grid) {
        if self.applied {
            if let Some(ref cells) = self.old_cells {
                *grid = Grid::from_cells(cells.clone(), self.width, self.height);
            }
            self.applied = false;
        }
    }

    fn description(&self) -> &str {
        "Clear canvas"
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_cell_command() {
        let mut grid = Grid::new(10, 10);
        let mut cmd = SetCellCommand::new(5, 5, Cell::new('X'));

        cmd.apply(&mut grid);
        assert_eq!(grid.get(5, 5).unwrap().ch, 'X');

        cmd.undo(&mut grid);
        assert!(grid.get(5, 5).unwrap().is_empty());
    }

    #[test]
    fn test_clear_grid_command() {
        let mut grid = Grid::new(10, 10);
        grid.set_char(5, 5, 'X');

        let mut cmd = ClearGridCommand::new();
        cmd.apply(&mut grid);

        assert!(grid.get(5, 5).unwrap().is_empty());

        cmd.undo(&mut grid);
        assert_eq!(grid.get(5, 5).unwrap().ch, 'X');
    }
}
