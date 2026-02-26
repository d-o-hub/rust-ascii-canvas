//! Composite command - groups multiple commands into one atomic operation.

use super::Command;
use crate::core::grid::Grid;

/// A composite command that groups multiple commands together.
pub struct CompositeCommand {
    /// Child commands
    commands: Vec<Box<dyn Command>>,
    /// Description
    description: String,
    /// Whether applied
    applied: bool,
}

impl CompositeCommand {
    /// Create a new empty composite command.
    pub fn new(description: impl Into<String>) -> Self {
        Self {
            commands: Vec::new(),
            description: description.into(),
            applied: false,
        }
    }

    /// Add a command to the composite.
    pub fn add(&mut self, command: Box<dyn Command>) {
        self.commands.push(command);
    }

    /// Add a command and return self for chaining.
    pub fn with(mut self, command: Box<dyn Command>) -> Self {
        self.commands.push(command);
        self
    }

    /// Get the number of commands.
    pub fn len(&self) -> usize {
        self.commands.len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }
}

impl Command for CompositeCommand {
    fn apply(&mut self, grid: &mut Grid) {
        if !self.applied {
            for cmd in &mut self.commands {
                cmd.apply(grid);
            }
            self.applied = true;
        }
    }

    fn undo(&mut self, grid: &mut Grid) {
        if self.applied {
            // Undo in reverse order
            for cmd in self.commands.iter_mut().rev() {
                cmd.undo(grid);
            }
            self.applied = false;
        }
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::cell::Cell;
    use crate::core::commands::SetCellCommand;

    #[test]
    fn test_composite_command() {
        let mut grid = Grid::new(10, 10);

        let mut composite = CompositeCommand::new("Multiple changes");
        composite.add(Box::new(SetCellCommand::new(0, 0, Cell::new('A'))));
        composite.add(Box::new(SetCellCommand::new(1, 0, Cell::new('B'))));
        composite.add(Box::new(SetCellCommand::new(2, 0, Cell::new('C'))));

        composite.apply(&mut grid);

        assert_eq!(grid.get(0, 0).unwrap().ch, 'A');
        assert_eq!(grid.get(1, 0).unwrap().ch, 'B');
        assert_eq!(grid.get(2, 0).unwrap().ch, 'C');

        composite.undo(&mut grid);

        assert!(grid.get(0, 0).unwrap().is_empty());
        assert!(grid.get(1, 0).unwrap().is_empty());
        assert!(grid.get(2, 0).unwrap().is_empty());
    }
}
