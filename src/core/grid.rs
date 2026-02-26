//! Grid module - manages the 2D ASCII canvas as a flat vector of cells.
//!
//! Uses row-major ordering for O(1) index calculations.
//! Supports efficient iteration, modification, and boundary checking.

use super::cell::Cell;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use std::ops::{Index, IndexMut};

/// A 2D grid of ASCII cells using flat storage.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Grid {
    /// Flat storage of cells in row-major order
    cells: Vec<Cell>,
    /// Grid width in cells
    width: usize,
    /// Grid height in cells
    height: usize,
}

impl Grid {
    /// Create a new grid with the given dimensions.
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            cells: vec![Cell::default(); width * height],
            width,
            height,
        }
    }

    /// Create a grid from existing cells.
    pub fn from_cells(cells: Vec<Cell>, width: usize, height: usize) -> Self {
        debug_assert_eq!(cells.len(), width * height);
        Self {
            cells,
            width,
            height,
        }
    }

    /// Get grid width.
    #[inline]
    pub fn width(&self) -> usize {
        self.width
    }

    /// Get grid height.
    #[inline]
    pub fn height(&self) -> usize {
        self.height
    }

    /// Get total cell count.
    #[inline]
    pub fn len(&self) -> usize {
        self.cells.len()
    }

    /// Check if grid is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }

    /// Convert (x, y) coordinates to flat index.
    #[inline]
    pub fn index_of(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    /// Convert flat index to (x, y) coordinates.
    #[inline]
    pub fn coords_of(&self, index: usize) -> (usize, usize) {
        (index % self.width, index / self.width)
    }

    /// Check if coordinates are within bounds.
    #[inline]
    pub fn in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height
    }

    /// Get cell at (x, y), returns None if out of bounds.
    #[inline]
    pub fn get(&self, x: i32, y: i32) -> Option<&Cell> {
        if self.in_bounds(x, y) {
            Some(&self.cells[self.index_of(x as usize, y as usize)])
        } else {
            None
        }
    }

    /// Get mutable cell at (x, y), returns None if out of bounds.
    #[inline]
    pub fn get_mut(&mut self, x: i32, y: i32) -> Option<&mut Cell> {
        if self.in_bounds(x, y) {
            let idx = self.index_of(x as usize, y as usize);
            Some(&mut self.cells[idx])
        } else {
            None
        }
    }

    /// Set cell at (x, y), returns false if out of bounds.
    #[inline]
    pub fn set(&mut self, x: i32, y: i32, cell: Cell) -> bool {
        if let Some(c) = self.get_mut(x, y) {
            *c = cell;
            true
        } else {
            false
        }
    }

    /// Set character at (x, y), returns false if out of bounds.
    #[inline]
    pub fn set_char(&mut self, x: i32, y: i32, ch: char) -> bool {
        if let Some(cell) = self.get_mut(x, y) {
            cell.ch = ch;
            true
        } else {
            false
        }
    }

    /// Clear cell at (x, y).
    #[inline]
    pub fn clear_cell(&mut self, x: i32, y: i32) -> bool {
        if let Some(cell) = self.get_mut(x, y) {
            cell.clear();
            true
        } else {
            false
        }
    }

    /// Clear all cells in the grid.
    pub fn clear(&mut self) {
        for cell in &mut self.cells {
            cell.clear();
        }
    }

    /// Get iterator over all cells with coordinates.
    pub fn iter_with_coords(&self) -> impl Iterator<Item = (i32, i32, &Cell)> {
        self.cells.iter().enumerate().map(move |(i, cell)| {
            let (x, y) = self.coords_of(i);
            (x as i32, y as i32, cell)
        })
    }

    /// Get mutable iterator over all cells with coordinates.
    pub fn iter_mut_with_coords(&mut self) -> impl Iterator<Item = (i32, i32, &mut Cell)> {
        let width = self.width;
        self.cells.iter_mut().enumerate().map(move |(i, cell)| {
            let x = i % width;
            let y = i / width;
            (x as i32, y as i32, cell)
        })
    }

    /// Get slice of cells.
    #[inline]
    pub fn cells(&self) -> &[Cell] {
        &self.cells
    }

    /// Resize the grid, keeping existing content where possible.
    pub fn resize(&mut self, new_width: usize, new_height: usize) {
        if new_width == self.width && new_height == self.height {
            return;
        }

        let mut new_cells = vec![Cell::default(); new_width * new_height];

        let copy_width = new_width.min(self.width);
        let copy_height = new_height.min(self.height);

        for y in 0..copy_height {
            let src_start = y * self.width;
            let dst_start = y * new_width;
            new_cells[dst_start..dst_start + copy_width]
                .copy_from_slice(&self.cells[src_start..src_start + copy_width]);
        }

        self.cells = new_cells;
        self.width = new_width;
        self.height = new_height;
    }

    /// Fill a rectangular region with a character.
    pub fn fill_rect(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, ch: char) {
        let (min_x, max_x) = (x1.min(x2), x1.max(x2));
        let (min_y, max_y) = (y1.min(y2), y1.max(y2));

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                self.set_char(x, y, ch);
            }
        }
    }

    /// Get a copy of a rectangular region.
    pub fn get_region(&self, x1: i32, y1: i32, x2: i32, y2: i32) -> SmallVec<[Cell; 256]> {
        let (min_x, max_x) = (x1.min(x2).max(0) as usize, x1.max(x2).min(self.width as i32 - 1) as usize);
        let (min_y, max_y) = (y1.min(y2).max(0) as usize, y1.max(y2).min(self.height as i32 - 1) as usize);

        let mut region = SmallVec::with_capacity((max_x - min_x + 1) * (max_y - min_y + 1));

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                region.push(self.cells[self.index_of(x, y)]);
            }
        }

        region
    }
}

impl Index<usize> for Grid {
    type Output = Cell;

    fn index(&self, index: usize) -> &Self::Output {
        &self.cells[index]
    }
}

impl IndexMut<usize> for Grid {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.cells[index]
    }
}

impl Index<(usize, usize)> for Grid {
    type Output = Cell;

    fn index(&self, (x, y): (usize, usize)) -> &Self::Output {
        &self.cells[self.index_of(x, y)]
    }
}

impl IndexMut<(usize, usize)> for Grid {
    fn index_mut(&mut self, (x, y): (usize, usize)) -> &mut Self::Output {
        let idx = self.index_of(x, y);
        &mut self.cells[idx]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_creation() {
        let grid = Grid::new(80, 40);
        assert_eq!(grid.width(), 80);
        assert_eq!(grid.height(), 40);
        assert_eq!(grid.len(), 80 * 40);
    }

    #[test]
    fn test_grid_index() {
        let mut grid = Grid::new(10, 10);
        grid.set_char(5, 3, 'X');
        assert_eq!(grid[(5, 3)].ch, 'X');
        assert_eq!(grid.index_of(5, 3), 35);
        assert_eq!(grid.coords_of(35), (5, 3));
    }

    #[test]
    fn test_bounds_check() {
        let grid = Grid::new(10, 10);
        assert!(grid.in_bounds(0, 0));
        assert!(grid.in_bounds(9, 9));
        assert!(!grid.in_bounds(-1, 0));
        assert!(!grid.in_bounds(10, 0));
    }

    #[test]
    fn test_fill_rect() {
        let mut grid = Grid::new(20, 20);
        grid.fill_rect(2, 2, 5, 5, '#');

        for y in 2..=5 {
            for x in 2..=5 {
                assert_eq!(grid.get(x, y).unwrap().ch, '#');
            }
        }

        assert!(grid.get(1, 2).unwrap().is_empty());
        assert!(grid.get(6, 2).unwrap().is_empty());
    }
}
