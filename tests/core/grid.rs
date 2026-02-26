//! Grid tests.

use ascii_canvas::core::grid::Grid;
use ascii_canvas::core::cell::Cell;

#[test]
fn test_grid_creation() {
    let grid = Grid::new(80, 40);
    assert_eq!(grid.width(), 80);
    assert_eq!(grid.height(), 40);
    assert_eq!(grid.len(), 80 * 40);
}

#[test]
fn test_grid_bounds() {
    let grid = Grid::new(10, 10);
    
    assert!(grid.in_bounds(0, 0));
    assert!(grid.in_bounds(9, 9));
    assert!(!grid.in_bounds(-1, 0));
    assert!(!grid.in_bounds(10, 0));
    assert!(!grid.in_bounds(0, 10));
}

#[test]
fn test_grid_set_get() {
    let mut grid = Grid::new(20, 20);
    
    assert!(grid.set_char(5, 5, 'X'));
    assert_eq!(grid.get(5, 5).unwrap().ch, 'X');
    
    assert!(grid.set(10, 10, Cell::new('A')));
    assert_eq!(grid.get(10, 10).unwrap().ch, 'A');
    
    assert!(!grid.set(-1, -1, Cell::new('B')));
}

#[test]
fn test_grid_clear() {
    let mut grid = Grid::new(10, 10);
    grid.set_char(5, 5, 'X');
    
    grid.clear();
    
    assert!(grid.get(5, 5).unwrap().is_empty());
}

#[test]
fn test_grid_resize() {
    let mut grid = Grid::new(10, 10);
    grid.set_char(5, 5, 'X');
    
    grid.resize(20, 20);
    
    assert_eq!(grid.width(), 20);
    assert_eq!(grid.height(), 20);
    assert_eq!(grid.get(5, 5).unwrap().ch, 'X');
}

#[test]
fn test_grid_fill_rect() {
    let mut grid = Grid::new(20, 20);
    grid.fill_rect(2, 2, 5, 5, '#');
    
    for y in 2..=5 {
        for x in 2..=5 {
            assert_eq!(grid.get(x, y).unwrap().ch, '#');
        }
    }
    
    assert!(grid.get(1, 2).unwrap().is_empty());
}

#[test]
fn test_grid_indexing() {
    let mut grid = Grid::new(10, 10);
    grid.set_char(5, 3, 'A');
    
    // Test tuple indexing
    assert_eq!(grid[(5, 3)].ch, 'A');
    
    // Test flat indexing
    let idx = grid.index_of(5, 3);
    assert_eq!(grid[idx].ch, 'A');
}
