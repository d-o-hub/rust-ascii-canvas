//! Selection module - represents a rectangular selection on the canvas.

use serde::{Deserialize, Serialize};

/// A rectangular selection region.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Selection {
    /// Top-left X coordinate
    pub x1: i32,
    /// Top-left Y coordinate
    pub y1: i32,
    /// Bottom-right X coordinate
    pub x2: i32,
    /// Bottom-right Y coordinate
    pub y2: i32,
}

impl Selection {
    /// Create a new selection from two corner points.
    pub fn new(x1: i32, y1: i32, x2: i32, y2: i32) -> Self {
        Self { x1, y1, x2, y2 }
    }

    /// Get normalized (min, max) bounds.
    pub fn bounds(&self) -> (i32, i32, i32, i32) {
        (
            self.x1.min(self.x2),
            self.y1.min(self.y2),
            self.x1.max(self.x2),
            self.y1.max(self.y2),
        )
    }

    /// Get the width of the selection.
    pub fn width(&self) -> i32 {
        (self.x2 - self.x1).abs() + 1
    }

    /// Get the height of the selection.
    pub fn height(&self) -> i32 {
        (self.y2 - self.y1).abs() + 1
    }

    /// Check if a point is within the selection.
    pub fn contains(&self, x: i32, y: i32) -> bool {
        let (min_x, min_y, max_x, max_y) = self.bounds();
        x >= min_x && x <= max_x && y >= min_y && y <= max_y
    }

    /// Check if the selection is empty (zero size).
    pub fn is_empty(&self) -> bool {
        self.x1 == self.x2 && self.y1 == self.y2
    }

    /// Get the area of the selection.
    pub fn area(&self) -> i32 {
        self.width() * self.height()
    }

    /// Translate the selection by a delta.
    pub fn translate(&mut self, dx: i32, dy: i32) {
        self.x1 += dx;
        self.y1 += dy;
        self.x2 += dx;
        self.y2 += dy;
    }

    /// Create a new selection translated by delta.
    pub fn translated(&self, dx: i32, dy: i32) -> Self {
        Self {
            x1: self.x1 + dx,
            y1: self.y1 + dy,
            x2: self.x2 + dx,
            y2: self.y2 + dy,
        }
    }
}

impl Default for Selection {
    fn default() -> Self {
        Self {
            x1: 0,
            y1: 0,
            x2: 0,
            y2: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selection_bounds() {
        let sel = Selection::new(5, 2, 10, 8);
        let (min_x, min_y, max_x, max_y) = sel.bounds();

        assert_eq!(min_x, 5);
        assert_eq!(min_y, 2);
        assert_eq!(max_x, 10);
        assert_eq!(max_y, 8);
    }

    #[test]
    fn test_selection_dimensions() {
        let sel = Selection::new(0, 0, 10, 5);
        assert_eq!(sel.width(), 11);
        assert_eq!(sel.height(), 6);
        assert_eq!(sel.area(), 66);
    }

    #[test]
    fn test_selection_contains() {
        let sel = Selection::new(5, 5, 10, 10);

        assert!(sel.contains(5, 5));
        assert!(sel.contains(7, 7));
        assert!(sel.contains(10, 10));
        assert!(!sel.contains(4, 5));
        assert!(!sel.contains(11, 10));
    }

    #[test]
    fn test_selection_translate() {
        let mut sel = Selection::new(5, 5, 10, 10);
        sel.translate(5, -2);

        assert_eq!(sel.x1, 10);
        assert_eq!(sel.y1, 3);
        assert_eq!(sel.x2, 15);
        assert_eq!(sel.y2, 8);
    }
}
