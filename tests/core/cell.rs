//! Cell tests.

use ascii_canvas::core::cell::{Cell, CellStyle};

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
fn test_cell_clear() {
    let mut cell = Cell::new('X');
    cell.clear();
    assert!(cell.is_empty());
}

#[test]
fn test_cell_style_flags() {
    let style = CellStyle::BOLD | CellStyle::UNDERLINE;
    
    assert!(style.contains(CellStyle::BOLD));
    assert!(style.contains(CellStyle::UNDERLINE));
    assert!(!style.contains(CellStyle::ITALIC));
}

#[test]
fn test_cell_with_style() {
    let style = CellStyle::BOLD;
    let cell = Cell::with_style('A', style);
    
    assert_eq!(cell.ch, 'A');
    assert!(cell.style.contains(CellStyle::BOLD));
}
