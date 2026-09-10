//! ASCII export tests.

use ascii_canvas::core::ascii_export::{
    ascii_fallback_char, count_content, export_grid, export_region, find_content_bounds,
    ExportOptions,
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
fn test_copy_options_preserve_rectangular_box_and_unicode() {
    let mut grid = Grid::new(12, 4);
    grid.set_char(0, 0, '┌');
    grid.set_char(1, 0, '─');
    grid.set_char(2, 0, '┐');
    grid.set_char(0, 1, '│');
    grid.set_char(2, 1, '│');
    grid.set_char(0, 2, '└');
    grid.set_char(1, 2, '─');
    grid.set_char(2, 2, '┘');
    grid.set_char(10, 0, 'B');

    let result = export_grid(&grid, &ExportOptions::default());
    let widths: Vec<usize> = result.lines().map(|line| line.chars().count()).collect();
    assert_eq!(widths, vec![11, 11, 11]);
    assert_eq!(result.lines().nth(1), Some("│ │        "));

    let ascii = export_grid(
        &grid,
        &ExportOptions {
            convert_unicode_to_ascii: true,
            ..Default::default()
        },
    );
    assert_eq!(ascii.lines().next(), Some("+-+       B"));
    assert!(ascii.chars().all(|ch| ch.is_ascii() || ch == '\n'));
}

#[test]
fn test_copy_options_can_trim_trailing_whitespace() {
    let mut grid = Grid::new(4, 2);
    grid.set_char(0, 0, 'A');
    grid.set_char(2, 1, 'B');

    let result = export_grid(
        &grid,
        &ExportOptions {
            trim_trailing_whitespace: true,
            enforce_bounding_box: false,
            ..Default::default()
        },
    );
    assert_eq!(result, "A\n  B");
}

#[test]
fn test_ascii_fallback_maps_box_drawing_glyphs() {
    assert_eq!(ascii_fallback_char('│'), '|');
    assert_eq!(ascii_fallback_char('─'), '-');
    assert_eq!(ascii_fallback_char('┌'), '+');
    assert_eq!(ascii_fallback_char('┘'), '+');
    assert_eq!(ascii_fallback_char('🦀'), '?');
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
