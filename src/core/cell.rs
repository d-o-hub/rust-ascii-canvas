//! Cell module - represents a single character cell in the ASCII grid.
//!
//! Each cell contains a character and optional styling metadata.
//! Cells are the fundamental unit of the ASCII canvas.

use serde::{Deserialize, Serialize};

/// A single cell in the ASCII grid.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cell {
    /// The character in this cell (default: space)
    pub ch: char,
    /// Cell style flags for rendering
    pub style: CellStyle,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            ch: ' ',
            style: CellStyle::empty(),
        }
    }
}

impl Cell {
    /// Create a new cell with the given character.
    #[inline]
    pub fn new(ch: char) -> Self {
        Self {
            ch,
            style: CellStyle::empty(),
        }
    }

    /// Create a cell with character and style.
    #[inline]
    pub fn with_style(ch: char, style: CellStyle) -> Self {
        Self { ch, style }
    }

    /// Check if this cell is empty (space character).
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.ch == ' '
    }

    /// Check if this cell contains a visible character.
    #[inline]
    pub fn is_visible(&self) -> bool {
        !self.ch.is_whitespace()
    }

    /// Clear the cell to default state.
    #[inline]
    pub fn clear(&mut self) {
        *self = Self::default();
    }

    /// Set the character.
    #[inline]
    pub fn set_char(&mut self, ch: char) {
        self.ch = ch;
    }
}

bitflags::bitflags! {
    /// Style flags for cell rendering.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(transparent)]
    pub struct CellStyle: u8 {
        /// No special styling
        const NONE = 0;
        /// Bold text
        const BOLD = 1 << 0;
        /// Italic text
        const ITALIC = 1 << 1;
        /// Underlined
        const UNDERLINE = 1 << 2;
        /// Highlighted/selected
        const HIGHLIGHT = 1 << 3;
    }
}

impl Default for CellStyle {
    fn default() -> Self {
        Self::empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cell_creation() {
        let cell = Cell::new('A');
        assert_eq!(cell.ch, 'A');
        assert!(cell.style.is_empty());
        assert!(!cell.is_empty());
        assert!(cell.is_visible());
    }

    #[test]
    fn test_cell_default() {
        let cell = Cell::default();
        assert_eq!(cell.ch, ' ');
        assert!(cell.is_empty());
        assert!(!cell.is_visible());
    }

    #[test]
    fn test_cell_style() {
        let style = CellStyle::BOLD | CellStyle::UNDERLINE;
        assert!(style.contains(CellStyle::BOLD));
        assert!(style.contains(CellStyle::UNDERLINE));
        assert!(!style.contains(CellStyle::ITALIC));
    }

    #[test]
    fn test_cell_clear() {
        let mut cell = Cell::new('X');
        cell.clear();
        assert!(cell.is_empty());
    }
}
