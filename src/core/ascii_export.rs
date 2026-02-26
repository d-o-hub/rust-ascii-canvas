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

    // Build output
    let mut result = String::new();

    for y in min_y..=max_y {
        if options.line_numbers {
            result.push_str(&format!("{:4} | ", y + 1));
        }

        for x in min_x..=max_x {
            let ch = grid.get(x, y).map(|c| c.ch).unwrap_or(' ');
            result.push(ch);
        }

        // Trim trailing spaces from this line
        let trimmed_end = result.trim_end_matches(' ').len();
        result.truncate(trimmed_end);

        if y < max_y {
            result.push('\n');
        }
    }

    // Apply max width if set
    if options.max_width > 0 {
        let lines: Vec<&str> = result.lines().collect();
        let mut limited = String::new();
        for (i, line) in lines.iter().enumerate() {
            let line = if line.len() > options.max_width {
                &line[..options.max_width]
            } else {
                line
            };
            limited.push_str(line);
            if i < lines.len() - 1 {
                limited.push('\n');
            }
        }
        return limited;
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
        // Trim trailing spaces
        let trimmed = result.trim_end_matches(' ');
        result.truncate(trimmed.len());

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
}
