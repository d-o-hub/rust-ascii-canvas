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
/// This method exports EXACTLY the region specified by the coordinates,
/// preserving internal spaces and alignment.
pub fn export_region(grid: &Grid, x1: i32, y1: i32, x2: i32, y2: i32) -> String {
    let min_x = x1.min(x2).max(0);
    let min_y = y1.min(y2).max(0);
    let max_x = x1.max(x2).min(grid.width() as i32 - 1);
    let max_y = y1.max(y2).min(grid.height() as i32 - 1);

    let mut result = String::new();
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let ch = grid.get(x, y).map(|c| c.ch).unwrap_or(' ');
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
        assert_eq!(result, "AB\nCD");
    }

    #[test]
    fn test_export_region_with_empty_space() {
        let mut grid = Grid::new(20, 20);
        grid.set_char(1, 1, 'X');
        // Region (0,0) to (2,2) should be 3x3 with 'X' at (1,1)
        let result = export_region(&grid, 0, 0, 2, 2);
        let expected = "   \n X \n   ";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_export_trimmed_preserves_right_border() {
        // Simulate a 5×3 rectangle drawn with box-drawing chars:
        // ┌───┐
        // │   │
        // └───┘
        let mut grid = Grid::new(10, 5);
        // Top border
        grid.set_char(0, 0, '┌');
        grid.set_char(1, 0, '─');
        grid.set_char(2, 0, '─');
        grid.set_char(3, 0, '─');
        grid.set_char(4, 0, '┐');
        // Middle row — right border only; interior is spaces (empty cells)
        grid.set_char(0, 1, '│');
        grid.set_char(4, 1, '│');
        // Bottom border
        grid.set_char(0, 2, '└');
        grid.set_char(1, 2, '─');
        grid.set_char(2, 2, '─');
        grid.set_char(3, 2, '─');
        grid.set_char(4, 2, '┘');

        let options = ExportOptions::default(); // trim_borders: true
        let result = export_grid(&grid, &options);
        let lines: Vec<&str> = result.lines().collect();

        assert_eq!(lines.len(), 3, "Should have 3 rows");
        // Top row must end with ┐
        assert!(
            lines[0].ends_with('┐'),
            "Top row must end with ┐, got: {:?}",
            lines[0]
        );
        // Middle row must end with │
        assert!(
            lines[1].ends_with('│'),
            "Middle row must end with │, got: {:?}",
            lines[1]
        );
        // Bottom row must end with ┘
        assert!(
            lines[2].ends_with('┘'),
            "Bottom row must end with ┘, got: {:?}",
            lines[2]
        );
        // All content lines must have the same width (uniform column trimming)
        let widths: Vec<usize> = lines.iter().map(|l| l.chars().count()).collect();
        assert!(
            widths.windows(2).all(|w| w[0] == w[1]),
            "All lines must be the same width: {:?}",
            widths
        );
    }

    #[test]
    fn test_export_region_preserves_right_border() {
        // Same box, exported via export_region — Bug 5
        let mut grid = Grid::new(10, 5);
        grid.set_char(0, 0, '┌');
        grid.set_char(1, 0, '─');
        grid.set_char(2, 0, '─');
        grid.set_char(3, 0, '─');
        grid.set_char(4, 0, '┐');
        grid.set_char(0, 1, '│');
        grid.set_char(4, 1, '│');
        grid.set_char(0, 2, '└');
        grid.set_char(1, 2, '─');
        grid.set_char(2, 2, '─');
        grid.set_char(3, 2, '─');
        grid.set_char(4, 2, '┘');

        // Export a slightly larger region (0,0) to (5,2)
        // Original was 5x3 (cols 0-4), we export 6x3 (cols 0-5)
        let result = export_region(&grid, 0, 0, 5, 2);
        let lines: Vec<&str> = result.lines().collect();

        assert_eq!(lines.len(), 3);
        // Each row should be 6 chars long, with a space at the end
        assert_eq!(lines[0], "┌───┐ ");
        assert_eq!(lines[1], "│   │ ");
        assert_eq!(lines[2], "└───┘ ");
    }

    #[test]
    fn test_export_trimmed_trailing_spaces_only_removed_globally() {
        // Grid with content only in col 0–4; col 5–9 are empty.
        // After global-column fix, lines should be trimmed to col 4 — not shorter.
        let mut grid = Grid::new(10, 3);
        grid.set_char(0, 0, 'A');
        grid.set_char(4, 0, 'B'); // rightmost content
        grid.set_char(0, 1, 'C');
        // Row 2: only col 0 occupied — must be padded to col 4 width by global trim rule
        grid.set_char(0, 2, 'D');
        grid.set_char(4, 2, 'E');

        let options = ExportOptions::default();
        let result = export_grid(&grid, &options);
        let lines: Vec<&str> = result.lines().collect();

        assert_eq!(lines.len(), 3);
        // Row 1 has content at col 4; all rows must be width 5 (cols 0–4)
        assert_eq!(lines[0].chars().count(), 5, "Row 0 width");
        // Row 1 has trailing spaces up to global max col
        assert_eq!(
            lines[1].chars().count(),
            5,
            "Row 1 must be padded to global width"
        );
        assert_eq!(lines[2].chars().count(), 5, "Row 2 width");
    }

    #[test]
    fn test_export_region_exact_content() {
        // export_region result must include AB on row 0 and CD on row 1
        let mut grid = Grid::new(10, 10);
        grid.set_char(2, 2, 'A');
        grid.set_char(3, 2, 'B');
        grid.set_char(2, 3, 'C');
        grid.set_char(3, 3, 'D');

        let result = export_region(&grid, 2, 2, 3, 3);
        let lines: Vec<&str> = result.lines().collect();

        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], "AB");
        assert_eq!(lines[1], "CD");
    }

    #[test]
    fn test_export_full_no_trim() {
        // Bug 1 regression: export_full must NOT trim anything
        let mut grid = Grid::new(5, 2);
        grid.set_char(0, 0, 'X');
        // All other cells are empty spaces

        let options = ExportOptions {
            trim_borders: false,
            ..Default::default()
        };
        let result = export_grid(&grid, &options);
        let lines: Vec<&str> = result.lines().collect();

        assert_eq!(lines.len(), 2, "Must export all rows");
        assert_eq!(lines[0].len(), 5, "Full row width must be preserved");
        assert_eq!(lines[1].len(), 5, "Full empty row must be preserved");
    }

    #[test]
    fn test_count_content() {
        let mut grid = Grid::new(10, 10);
        assert_eq!(count_content(&grid), 0);
        grid.set_char(1, 1, 'A');
        grid.set_char(2, 2, 'B');
        assert_eq!(count_content(&grid), 2);
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
