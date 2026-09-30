//! A gesture-sized, non-coalescing draw transaction, bounded by affected cells.

use std::collections::BTreeMap;

use super::Command;
use crate::core::{Cell, DrawOp, Grid};

/// Accumulates live stroke samples while retaining each cell's original value.
/// It enters history only when the gesture completes; undo also cancels it.
pub struct StrokeCommand {
    cells: BTreeMap<(i32, i32), (Cell, Cell)>,
    description: &'static str,
}

impl StrokeCommand {
    /// Begin an empty gesture.
    pub fn new(description: &'static str) -> Self {
        Self {
            cells: BTreeMap::new(),
            description,
        }
    }

    /// Apply a sample without recording another history entry.
    pub fn extend(&mut self, grid: &mut Grid, ops: &[DrawOp]) {
        for op in ops {
            let Some(original) = grid.get(op.x, op.y).copied() else {
                continue;
            };
            if original == op.cell {
                continue;
            }
            self.cells
                .entry((op.x, op.y))
                .and_modify(|(_, current)| *current = op.cell)
                .or_insert((original, op.cell));
            grid.set(op.x, op.y, op.cell);
        }
    }

    /// Whether the gesture has no net change to committed cell values.
    pub fn is_empty(&self) -> bool {
        self.cells
            .values()
            .all(|(original, current)| original == current)
    }

    /// Restore the committed surface for cancellation or a clean snapshot.
    pub fn restore_original(&self, grid: &mut Grid) {
        for (&(x, y), &(original, _)) in &self.cells {
            grid.set(x, y, original);
        }
    }
}

impl Command for StrokeCommand {
    fn apply(&mut self, grid: &mut Grid) {
        for (&(x, y), &(_, current)) in &self.cells {
            grid.set(x, y, current);
        }
    }

    fn undo(&mut self, grid: &mut Grid) {
        self.restore_original(grid);
    }

    fn description(&self) -> &str {
        self.description
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{CellStyle, History, LayerStack};

    #[test]
    fn unchanged_and_reverted_cells_have_no_net_effect() {
        let mut grid = Grid::new(2, 1);
        grid.set_char(0, 0, 'X');
        let mut stroke = StrokeCommand::new("Freehand");
        stroke.extend(&mut grid, &[DrawOp::new(0, 0, 'X')]);
        assert!(stroke.is_empty());
        stroke.extend(&mut grid, &[DrawOp::new(0, 0, 'Y')]);
        assert!(!stroke.is_empty());
        stroke.extend(&mut grid, &[DrawOp::new(0, 0, 'X')]);
        assert!(stroke.is_empty());
    }

    #[test]
    fn revisited_cells_keep_the_original_style_and_final_value() {
        let mut grid = Grid::new(4, 2);
        let original = Cell {
            ch: 'X',
            style: CellStyle::BOLD,
        };
        grid.set(1, 0, original);
        let mut stroke = StrokeCommand::new("Freehand");
        for ch in ['A', 'B', 'C'] {
            stroke.extend(&mut grid, &[DrawOp::new(1, 0, ch)]);
        }
        assert_eq!(stroke.cells.len(), 1);
        stroke.undo(&mut grid);
        assert_eq!(grid.get(1, 0), Some(&original));
        stroke.apply(&mut grid);
        assert_eq!(grid.get(1, 0).unwrap().ch, 'C');
    }

    #[test]
    fn separate_gestures_never_coalesce_in_history() {
        let mut grid = Grid::new(320, 2);
        let mut history = History::default();
        let mut layers = LayerStack::new(320, 2);
        for y in 0..2 {
            let mut stroke = StrokeCommand::new("Freehand");
            for x in 0..300 {
                stroke.extend(&mut grid, &[DrawOp::new(x, y, 'A')]);
            }
            history.push(Box::new(stroke));
        }
        assert_eq!(history.undo_count(), 2);
        history.undo(&mut grid, &mut layers);
        assert_eq!(grid.get(0, 0).unwrap().ch, 'A');
        assert_eq!(grid.get(0, 1).unwrap().ch, ' ');
        history.undo(&mut grid, &mut layers);
        assert_eq!(grid.get(0, 0).unwrap().ch, ' ');
    }
}
