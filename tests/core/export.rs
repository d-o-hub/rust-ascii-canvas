//! ASCII export tests.

use ascii_canvas::core::ascii_export::{
    count_content, export_grid, export_region, find_content_bounds, ExportOptions,
};
use ascii_canvas::core::grid::Grid;

#[test]
fn test_export_empty_grid() {
    let grid = Grid::new(10, 10);
    let options = ExportOptions::default();
    let result = export_grid(&grid, &options);

    assert!(result.is_empty());
}

#[test]
fn test_export_with_content() {
    let mut grid = Grid::new(10, 10);
    grid.set_char(2, 2, 'H');
    grid.set_char(3, 2, 'i');

    let options = ExportOptions::default();
    let result = export_grid(&grid, &options);

    assert_eq!(result, "Hi");
}

#[test]
fn test_export_multiline() {
    let mut grid = Grid::new(10, 10);
    grid.set_char(0, 0, 'A');
    grid.set_char(0, 1, 'B');

    let options = ExportOptions::default();
    let result = export_grid(&grid, &options);

    assert_eq!(result, "A\nB");
}

#[test]
fn test_export_no_trim() {
    let mut grid = Grid::new(5, 3);
    grid.set_char(0, 0, 'X');

    let options = ExportOptions {
        trim_borders: false,
        ..Default::default()
    };
    let result = export_grid(&grid, &options);

    // Should have all rows
    assert_eq!(result.lines().count(), 3);
}

#[test]
fn test_find_content_bounds() {
    let mut grid = Grid::new(20, 20);
    grid.set_char(5, 5, 'X');
    grid.set_char(10, 10, 'Y');

    let bounds = find_content_bounds(&grid);

    assert_eq!(bounds, Some((5, 5, 10, 10)));
}

#[test]
fn test_find_content_bounds_empty() {
    let grid = Grid::new(20, 20);
    let bounds = find_content_bounds(&grid);

    assert!(bounds.is_none());
}

#[test]
fn test_export_region() {
    let mut grid = Grid::new(20, 20);
    grid.set_char(5, 5, 'A');
    grid.set_char(6, 5, 'B');
    grid.set_char(5, 6, 'C');
    grid.set_char(6, 6, 'D');

    let result = export_region(&grid, 5, 5, 6, 6);

    assert!(result.contains('A'));
    assert!(result.contains('B'));
    assert!(result.contains('C'));
    assert!(result.contains('D'));
}

#[test]
fn test_count_content() {
    let mut grid = Grid::new(10, 10);

    assert_eq!(count_content(&grid), 0);

    grid.set_char(0, 0, 'A');
    grid.set_char(1, 0, 'B');

    assert_eq!(count_content(&grid), 2);

    grid.set_char(2, 0, ' '); // Space doesn't count as visible

    assert_eq!(count_content(&grid), 2);
}

#[test]
fn test_export_with_line_numbers() {
    let mut grid = Grid::new(10, 10);
    grid.set_char(0, 0, 'A');
    grid.set_char(0, 2, 'B');

    let options = ExportOptions {
        line_numbers: true,
        ..Default::default()
    };
    let result = export_grid(&grid, &options);

    assert!(result.contains("1 | A"));
    assert!(result.contains("3 | B"));
}

#[test]
fn test_export_max_width() {
    let mut grid = Grid::new(100, 10);
    for i in 0..50 {
        grid.set_char(i, 0, 'X');
    }

    let options = ExportOptions {
        max_width: 20,
        ..Default::default()
    };
    let result = export_grid(&grid, &options);

    assert!(result.lines().next().unwrap().len() <= 20);
}
