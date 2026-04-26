//! ASCII Export module - exports grid content to clean ASCII text.

use crate::core::grid::Grid;

/// Options for ASCII export.
#[derive(Clone, Debug)]
pub struct ExportOptions {
    /// Trim empty borders
    pub trim_borders: bool,
    /// Include line numbers
    pub line_numbers: bool,
    /// Maximum width (0 = no limit)
    pub max_width: usize,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            trim_borders: true,
            line_numbers: false,
            max_width: 0,
        }
    }
}

/// Export the grid to an ASCII string.
pub fn export_grid(grid: &Grid, options: &ExportOptions) -> String {
    if options.trim_borders {
        export_trimmed(grid, options)
    } else {
        export_full(grid, options)
    }
}

/// Export the full grid without trimming.
fn export_full(grid: &Grid, _options: &ExportOptions) -> String {
    let mut result = String::with_capacity(grid.len());

    for y in 0..grid.height() {
        for x in 0..grid.width() {
            let ch = grid.get(x as i32, y as i32).map(|c| c.ch).unwrap_or(' ');
            result.push(ch);
        }
        if y < grid.height() - 1 {
            result.push('\n');
        }
    }

    result
}

/// Export with trimmed empty borders.
fn export_trimmed(grid: &Grid, options: &ExportOptions) -> String {
    // Find content bounds
    let bounds = find_content_bounds(grid);

    // If no content, return empty string
    let (min_x, min_y, max_x, max_y) = match bounds {
        Some(b) => b,
        None => return String::new(),
    };

    // Calculate approximate capacity
    let cols = ((max_x - min_x + 1) as usize).min(if options.max_width > 0 {
        options.max_width
    } else {
        usize::MAX
    });
    let rows = (max_y - min_y + 1) as usize;
    let mut result = String::with_capacity(rows * (cols + 1));

    for y in min_y..=max_y {
        let mut line_chars_count = 0;

        if options.line_numbers {
            let prefix = format!("{:4} | ", y + 1);
            for ch in prefix.chars() {
                if options.max_width > 0 && line_chars_count >= options.max_width {
                    break;
                }
                result.push(ch);
                line_chars_count += 1;
            }
        }

        for x in min_x..=max_x {
            if options.max_width > 0 && line_chars_count >= options.max_width {
                break;
            }
            let ch = grid.get(x, y).map(|c| c.ch).unwrap_or(' ');
            result.push(ch);
            line_chars_count += 1;
        }

        if y < max_y {
            result.push('\n');
        }
    }

    result
}

/// Find the bounding box of non-empty content.
pub fn find_content_bounds(grid: &Grid) -> Option<(i32, i32, i32, i32)> {
    let mut min_x = grid.width() as i32;
    let mut min_y = grid.height() as i32;
    let mut max_x = -1i32;
    let mut max_y = -1i32;

    for (x, y, cell) in grid.iter_with_coords() {
        if cell.is_visible() {
            min_x = min_x.min(x);
            min_y = min_y.min(y);
            max_x = max_x.max(x);
            max_y = max_y.max(y);
        }
    }

    if max_x < 0 {
        None
    } else {
        Some((min_x, min_y, max_x, max_y))
    }
}

/// Export a rectangular region of the grid.
pub fn export_region(grid: &Grid, x1: i32, y1: i32, x2: i32, y2: i32) -> String {
    let min_x = x1.min(x2).max(0) as usize;
    let min_y = y1.min(y2).max(0) as usize;
    let max_x = (x1.max(x2).min(grid.width() as i32 - 1)) as usize;
    let max_y = (y1.max(y2).min(grid.height() as i32 - 1)) as usize;

    let mut result = String::new();

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let ch = grid.get(x as i32, y as i32).map(|c| c.ch).unwrap_or(' ');
            result.push(ch);
        }
        if y < max_y {
            result.push('\n');
        }
    }

    result
}

/// Count non-empty cells in the grid.
pub fn count_content(grid: &Grid) -> usize {
    grid.cells().iter().filter(|c| c.is_visible()).count()
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_export_padding_preserves_geometry() {
        let mut grid = Grid::new(10, 10);
        // Create a diamond-like shape
        //   .
        //  / \
        //  \ /
        //   '
        grid.set_char(5, 2, '.');
        grid.set_char(4, 3, '/');
        grid.set_char(6, 3, '\\');
        grid.set_char(4, 4, '\\');
        grid.set_char(6, 4, '/');
        grid.set_char(5, 5, '\'');

        let options = ExportOptions::default();
        let result = export_grid(&grid, &options);

        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines.len(), 4);
        // Every line should have the same length (3 characters: index 4 to 6)
        for line in &lines {
            assert_eq!(
                line.chars().count(),
                3,
                "Line '{}' should be padded to length 3",
                line
            );
        }

        assert_eq!(lines[0], " . ");
        assert_eq!(lines[1], "/ \\");
        assert_eq!(lines[2], "\\ /");
        assert_eq!(lines[3], " ' ");
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
    fn test_export_region() {
        let mut grid = Grid::new(20, 20);
        grid.set_char(5, 5, 'A');
        grid.set_char(6, 5, 'B');
        grid.set_char(5, 6, 'C');
        grid.set_char(6, 6, 'D');

        let result = export_region(&grid, 5, 5, 6, 6);

        assert!(result.contains('A'));
        assert!(result.contains('D'));
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
        // First line should be "X    " (X followed by 4 spaces)
        assert!(result.starts_with('X'));
    }

    #[test]
    fn test_export_max_width() {
        let mut grid = Grid::new(20, 20);
        grid.fill_rect(0, 0, 9, 0, 'X'); // 10 Xs

        let mut options = ExportOptions::default();
        options.max_width = 5;

        let result = export_grid(&grid, &options);
        assert_eq!(result, "XXXXX");
    }

    #[test]
    fn test_export_max_width_multibyte() {
        let mut grid = Grid::new(20, 20);
        grid.fill_rect(0, 0, 9, 0, '🦀'); // 10 crabs

        let mut options = ExportOptions::default();
        options.max_width = 3;

        let result = export_grid(&grid, &options);
        assert_eq!(result.chars().count(), 3);
        assert_eq!(result, "🦀🦀🦀");
    }
}
