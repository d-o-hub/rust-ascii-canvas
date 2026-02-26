//! Dirty rectangle tracking for efficient rendering.

use serde::{Deserialize, Serialize};

/// A rectangular region that needs to be redrawn.
#[derive(Clone, Debug, Copy, Serialize, Deserialize)]
pub struct DirtyRect {
    /// Minimum X coordinate (inclusive)
    pub x1: i32,
    /// Minimum Y coordinate (inclusive)
    pub y1: i32,
    /// Maximum X coordinate (inclusive)
    pub x2: i32,
    /// Maximum Y coordinate (inclusive)
    pub y2: i32,
}

impl Default for DirtyRect {
    fn default() -> Self {
        Self::empty()
    }
}

impl DirtyRect {
    /// Create an empty dirty rect (nothing to redraw).
    pub fn empty() -> Self {
        Self {
            x1: i32::MAX,
            y1: i32::MAX,
            x2: i32::MIN,
            y2: i32::MIN,
        }
    }

    /// Create a dirty rect for a single cell.
    pub fn single(x: i32, y: i32) -> Self {
        Self {
            x1: x,
            y1: y,
            x2: x,
            y2: y,
        }
    }

    /// Create a dirty rect from two corner points.
    pub fn from_points(x1: i32, y1: i32, x2: i32, y2: i32) -> Self {
        Self {
            x1: x1.min(x2),
            y1: y1.min(y2),
            x2: x1.max(x2),
            y2: y1.max(y2),
        }
    }

    /// Create a dirty rect covering the entire grid.
    pub fn full(width: usize, height: usize) -> Self {
        Self {
            x1: 0,
            y1: 0,
            x2: (width as i32).saturating_sub(1).max(0),
            y2: (height as i32).saturating_sub(1).max(0),
        }
    }

    /// Check if the rect is empty (nothing to redraw).
    pub fn is_empty(&self) -> bool {
        self.x1 > self.x2 || self.y1 > self.y2
    }

    /// Check if the rect covers the entire grid.
    pub fn is_full(&self, width: usize, height: usize) -> bool {
        self.x1 == 0
            && self.y1 == 0
            && self.x2 == (width as i32).saturating_sub(1)
            && self.y2 == (height as i32).saturating_sub(1)
    }

    /// Get the width of the dirty rect.
    pub fn width(&self) -> i32 {
        if self.is_empty() {
            0
        } else {
            self.x2 - self.x1 + 1
        }
    }

    /// Get the height of the dirty rect.
    pub fn height(&self) -> i32 {
        if self.is_empty() {
            0
        } else {
            self.y2 - self.y1 + 1
        }
    }

    /// Get the area (number of cells).
    pub fn area(&self) -> i32 {
        self.width() * self.height()
    }

    /// Expand the rect to include a point.
    pub fn include(&mut self, x: i32, y: i32) {
        self.x1 = self.x1.min(x);
        self.y1 = self.y1.min(y);
        self.x2 = self.x2.max(x);
        self.y2 = self.y2.max(y);
    }

    /// Expand the rect to include another rect.
    pub fn union(&mut self, other: &DirtyRect) {
        if other.is_empty() {
            return;
        }
        if self.is_empty() {
            *self = *other;
            return;
        }
        self.x1 = self.x1.min(other.x1);
        self.y1 = self.y1.min(other.y1);
        self.x2 = self.x2.max(other.x2);
        self.y2 = self.y2.max(other.y2);
    }

    /// Clamp the rect to grid bounds.
    pub fn clamp(&mut self, width: usize, height: usize) {
        self.x1 = self.x1.max(0).min(width as i32 - 1);
        self.y1 = self.y1.max(0).min(height as i32 - 1);
        self.x2 = self.x2.max(0).min(width as i32 - 1);
        self.y2 = self.y2.max(0).min(height as i32 - 1);
    }

    /// Check if a point is inside the rect.
    pub fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.x1 && x <= self.x2 && y >= self.y1 && y <= self.y2
    }

    /// Iterate over all cells in the rect.
    pub fn iter(&self) -> impl Iterator<Item = (i32, i32)> + '_ {
        (self.y1..=self.y2).flat_map(move |y| (self.x1..=self.x2).map(move |x| (x, y)))
    }
}

/// Tracker for dirty regions.
#[derive(Clone, Debug, Default)]
pub struct DirtyTracker {
    /// Current dirty rect (union of all dirty regions)
    dirty: DirtyRect,
    /// Whether a full redraw is needed
    needs_full_redraw: bool,
}

impl DirtyTracker {
    /// Create a new dirty tracker.
    pub fn new() -> Self {
        Self::default()
    }

    /// Mark a single cell as dirty.
    pub fn mark_dirty(&mut self, x: i32, y: i32) {
        self.dirty.include(x, y);
    }

    /// Mark a region as dirty.
    pub fn mark_region_dirty(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) {
        self.dirty.union(&DirtyRect::from_points(x1, y1, x2, y2));
    }

    /// Request a full redraw.
    pub fn request_full_redraw(&mut self) {
        self.needs_full_redraw = true;
    }

    /// Get the current dirty rect.
    pub fn dirty_rect(&self) -> &DirtyRect {
        &self.dirty
    }

    /// Check if full redraw is needed.
    pub fn needs_full_redraw(&self) -> bool {
        self.needs_full_redraw || self.dirty.is_empty()
    }

    /// Clear the dirty state (after rendering).
    pub fn clear(&mut self) {
        self.dirty = DirtyRect::empty();
        self.needs_full_redraw = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_dirty_rect() {
        let rect = DirtyRect::empty();
        assert!(rect.is_empty());
        assert_eq!(rect.width(), 0);
        assert_eq!(rect.height(), 0);
    }

    #[test]
    fn test_single_cell() {
        let rect = DirtyRect::single(5, 10);
        assert!(!rect.is_empty());
        assert_eq!(rect.width(), 1);
        assert_eq!(rect.height(), 1);
        assert!(rect.contains(5, 10));
        assert!(!rect.contains(4, 10));
    }

    #[test]
    fn test_union() {
        let mut rect = DirtyRect::single(5, 5);
        rect.union(&DirtyRect::single(10, 10));

        assert_eq!(rect.x1, 5);
        assert_eq!(rect.y1, 5);
        assert_eq!(rect.x2, 10);
        assert_eq!(rect.y2, 10);
    }

    #[test]
    fn test_dirty_tracker() {
        let mut tracker = DirtyTracker::new();

        tracker.mark_dirty(5, 5);
        tracker.mark_dirty(10, 10);

        let rect = tracker.dirty_rect();
        assert_eq!(rect.x1, 5);
        assert_eq!(rect.x2, 10);

        tracker.clear();
        assert!(tracker.dirty_rect().is_empty());
    }
}
