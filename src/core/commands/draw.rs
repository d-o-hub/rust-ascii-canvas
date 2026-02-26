//! Draw command - applies multiple draw operations atomically.

use super::Command;
use crate::core::cell::Cell;
use crate::core::grid::Grid;
use crate::core::tools::DrawOp;

/// Command that applies multiple draw operations.
pub struct DrawCommand {
    /// Operations to apply
    ops: Vec<DrawOp>,
    /// Previous cell states for undo
    previous: Vec<(i32, i32, Option<Cell>)>,
    /// Whether the command has been applied
    applied: bool,
    /// Description of the operation
    description: String,
}

impl DrawCommand {
    /// Create a new draw command from operations.
    pub fn new(ops: Vec<DrawOp>) -> Self {
        let count = ops.len();
        Self {
            ops,
            previous: Vec::new(),
            applied: false,
            description: if count == 1 {
                "Draw".to_string()
            } else {
                format!("Draw {} cells", count)
            },
        }
    }

    /// Create with a description.
    pub fn with_description(ops: Vec<DrawOp>, description: impl Into<String>) -> Self {
        Self {
            ops,
            previous: Vec::new(),
            applied: false,
            description: description.into(),
        }
    }

    /// Create from a single operation.
    pub fn single(x: i32, y: i32, ch: char) -> Self {
        Self::new(vec![DrawOp::new(x, y, ch)])
    }

    /// Get the number of operations.
    pub fn len(&self) -> usize {
        self.ops.len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.ops.is_empty()
    }
}

impl Command for DrawCommand {
    fn apply(&mut self, grid: &mut Grid) {
        if !self.applied && !self.ops.is_empty() {
            // Save previous states
            self.previous.clear();
            self.previous.reserve(self.ops.len());

            for op in &self.ops {
                let prev = grid.get(op.x, op.y).copied();
                self.previous.push((op.x, op.y, prev));
            }

            // Apply all operations
            for op in &self.ops {
                grid.set(op.x, op.y, op.cell);
            }

            self.applied = true;
        }
    }

    fn undo(&mut self, grid: &mut Grid) {
        if self.applied {
            // Restore previous states in reverse order
            for (x, y, prev) in self.previous.iter().rev() {
                if let Some(cell) = prev {
                    grid.set(*x, *y, *cell);
                } else {
                    grid.clear_cell(*x, *y);
                }
            }
            self.applied = false;
        }
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn can_merge(&self, other: &dyn Command) -> bool {
        // Can merge if the other is also a DrawCommand and we're still small
        if let Some(other_draw) = other.as_any().downcast_ref::<DrawCommand>() {
            self.ops.len() + other_draw.ops.len() < 1000
        } else {
            false
        }
    }

    fn merge(&mut self, other: Box<dyn Command>) {
        if let Some(other_draw) = other.as_any().downcast_ref::<DrawCommand>() {
            // Only merge if not yet applied
            if !self.applied {
                self.ops.extend(other_draw.ops.iter().cloned());
                self.description = format!("Draw {} cells", self.ops.len());
            }
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_draw_command() {
        let mut grid = Grid::new(10, 10);
        let ops = vec![
            DrawOp::new(0, 0, 'A'),
            DrawOp::new(1, 0, 'B'),
            DrawOp::new(2, 0, 'C'),
        ];

        let mut cmd = DrawCommand::new(ops);
        cmd.apply(&mut grid);

        assert_eq!(grid.get(0, 0).unwrap().ch, 'A');
        assert_eq!(grid.get(1, 0).unwrap().ch, 'B');
        assert_eq!(grid.get(2, 0).unwrap().ch, 'C');

        cmd.undo(&mut grid);

        assert!(grid.get(0, 0).unwrap().is_empty());
        assert!(grid.get(1, 0).unwrap().is_empty());
        assert!(grid.get(2, 0).unwrap().is_empty());
    }

    #[test]
    fn test_empty_draw_command() {
        let mut grid = Grid::new(10, 10);
        let mut cmd = DrawCommand::new(vec![]);

        cmd.apply(&mut grid);
        assert!(cmd.is_empty());
    }
}
