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
    /// Remove spaces at the end of each exported line when the bounding box is not enforced.
    pub trim_trailing_whitespace: bool,
    /// Keep every exported line padded to the selected rectangular width.
    pub enforce_bounding_box: bool,
    /// Convert box-drawing and other supported Unicode glyphs to 7-bit ASCII.
    pub convert_unicode_to_ascii: bool,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            trim_borders: true,
            line_numbers: false,
            max_width: 0,
            trim_trailing_whitespace: false,
            enforce_bounding_box: true,
            convert_unicode_to_ascii: false,
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
fn export_full(grid: &Grid, options: &ExportOptions) -> String {
    if grid.is_empty() {
        return String::new();
    }

    export_in_bounds(
        grid,
        options,
        0,
        0,
        grid.width() as i32 - 1,
        grid.height() as i32 - 1,
    )
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

    export_in_bounds(grid, options, min_x, min_y, max_x, max_y)
}

/// Common implementation for exporting a specific range with options.
fn export_in_bounds(
    grid: &Grid,
    options: &ExportOptions,
    min_x: i32,
    min_y: i32,
    max_x: i32,
    max_y: i32,
) -> String {
    // Width is measured in grid columns, not UTF-8 bytes.
    let width = (max_x - min_x + 1).max(0) as usize;
    let height = (max_y - min_y + 1).max(0) as usize;
    let prefix_width = if options.line_numbers { 7 } else { 0 };
    let max_line_width = if options.max_width > 0 {
        options.max_width
    } else {
        usize::MAX
    };
    let mut result = String::with_capacity(height * (prefix_width + width + 1));

    for y in min_y..=max_y {
        let mut line = String::new();

        if options.line_numbers {
            for ch in format!("{:4} | ", y + 1).chars() {
                if line.chars().count() >= max_line_width {
                    break;
                }
                line.push(ch);
            }
        }

        for x in min_x..=max_x {
            if line.chars().count() >= max_line_width {
                break;
            }
            let ch = grid.get(x, y).map(|c| c.ch).unwrap_or(' ');
            line.push(if options.convert_unicode_to_ascii {
                ascii_fallback_char(ch)
            } else {
                ch
            });
        }

        // A rectangular export deliberately retains trailing spaces. Trimming is only
        // meaningful when callers opt out of the geometry guarantee.
        if options.trim_trailing_whitespace && !options.enforce_bounding_box {
            while line.ends_with(' ') {
                line.pop();
            }
        }

        result.push_str(&line);
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

/// Export a rectangular region of the grid using the default fidelity options.
pub fn export_region(grid: &Grid, x1: i32, y1: i32, x2: i32, y2: i32) -> String {
    export_region_with_options(grid, x1, y1, x2, y2, &ExportOptions::default())
}

/// Export a rectangular region of the grid with explicit fidelity options.
pub fn export_region_with_options(
    grid: &Grid,
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
    options: &ExportOptions,
) -> String {
    let min_x = x1.min(x2);
    let min_y = y1.min(y2);
    let max_x = x1.max(x2);
    let max_y = y1.max(y2);

    export_in_bounds(grid, options, min_x, min_y, max_x, max_y)
}

/// Convert a supported Unicode drawing glyph to its 7-bit ASCII equivalent.
///
/// Box corners and junctions become `+`, horizontal strokes become `-`, and
/// vertical strokes become `|`. Other non-ASCII glyphs become `?` so callers
/// requesting a pure ASCII export receive only 7-bit characters.
pub fn ascii_fallback_char(ch: char) -> char {
    match ch {
        '─' | '━' | '═' | '╌' | '╍' | '┄' | '┅' | '┈' | '┉' | '╴' | '╶' | '╸' | '╺' => {
            '-'
        }
        '│' | '┃' | '║' | '┆' | '┇' | '┊' | '┋' | '╎' | '╏' | '╵' | '╷' | '╹' | '╻' => {
            '|'
        }
        '┌' | '┐' | '└' | '┘' | '┏' | '┓' | '┗' | '┛' | '╔' | '╗' | '╚' | '╝' | '╭' | '╮' | '╰'
        | '╯' | '┼' | '╋' | '╬' | '╁' | '╂' | '╃' | '╄' | '╅' | '╆' | '╇' | '╈' | '╉' | '╊'
        | '╳' | '┬' | '┴' | '├' | '┤' | '╞' | '╡' | '╥' | '╨' | '╪' | '╫' | '╀' => {
            '+'
        }
        '╱' => '/',
        '╲' => '\\',
        '◆' | '◇' | '●' | '•' | '·' => '*',
        '▲' | '△' => '^',
        '▼' | '▽' => 'v',
        '◀' | '◁' => '<',
        '▶' | '▷' => '>',
        ch if ch.is_ascii() => ch,
        _ => '?',
    }
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

        assert_eq!(result, "AB\nCD");
    }

    #[test]
    fn test_export_region_out_of_bounds() {
        let mut grid = Grid::new(10, 10);
        grid.set_char(0, 0, 'X');

        // Region partially outside (top-left)
        let result = export_region(&grid, -1, -1, 1, 1);
        assert_eq!(result, "   \n X \n   ");

        // Region entirely outside
        let result = export_region(&grid, 20, 20, 21, 21);
        assert_eq!(result, "  \n  ");
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
        assert_eq!(result.lines().next().unwrap(), "X    ");
    }

    #[test]
    fn test_export_full_max_width() {
        let mut grid = Grid::new(10, 2);
        grid.fill_rect(0, 0, 9, 1, 'X');

        let options = ExportOptions {
            trim_borders: false,
            max_width: 5,
            ..Default::default()
        };

        let result = export_grid(&grid, &options);
        assert_eq!(result, "XXXXX\nXXXXX");
    }

    #[test]
    fn test_export_max_width() {
        let mut grid = Grid::new(20, 20);
        grid.fill_rect(0, 0, 9, 0, 'X'); // 10 Xs

        let options = ExportOptions {
            max_width: 5,
            ..Default::default()
        };

        let result = export_grid(&grid, &options);
        assert_eq!(result, "XXXXX");

        // Multi-line max width
        grid.fill_rect(0, 1, 9, 1, 'Y');
        let result = export_grid(&grid, &options);
        assert_eq!(result, "XXXXX\nYYYYY");
    }

    #[test]
    fn test_export_max_width_multibyte() {
        let mut grid = Grid::new(20, 20);
        grid.fill_rect(0, 0, 9, 0, '🦀'); // 10 crabs

        let options = ExportOptions {
            max_width: 3,
            ..Default::default()
        };

        let result = export_grid(&grid, &options);
        assert_eq!(result.chars().count(), 3);
        assert_eq!(result, "🦀🦀🦀");
    }

    #[test]
    fn test_export_line_numbers_max_width() {
        let mut grid = Grid::new(20, 20);
        grid.set_char(0, 0, 'A');

        let options = ExportOptions {
            line_numbers: true,
            max_width: 5,
            ..Default::default()
        };

        let result = export_grid(&grid, &options);
        // "   1 | A" is 8 chars, max_width 5 should truncate it
        assert_eq!(result, "   1 ");
    }

    #[test]
    fn test_export_trimmed_preserves_right_border() {
        // ┌───┐
        // │   │
        // └───┘
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

        let options = ExportOptions::default();
        let result = export_grid(&grid, &options);
        let lines: Vec<&str> = result.lines().collect();

        assert_eq!(lines.len(), 3);
        assert!(
            lines[0].ends_with('┐'),
            "Top row must end with ┐, got: {:?}",
            lines[0]
        );
        assert!(
            lines[1].ends_with('│'),
            "Middle row must end with │, got: {:?}",
            lines[1]
        );
        assert!(
            lines[2].ends_with('┘'),
            "Bottom row must end with ┘, got: {:?}",
            lines[2]
        );
        let widths: Vec<usize> = lines.iter().map(|l| l.chars().count()).collect();
        assert!(
            widths.windows(2).all(|w| w[0] == w[1]),
            "All lines must be the same width: {:?}",
            widths
        );
    }

    #[test]
    fn test_export_region_preserves_right_border() {
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

        let result = export_region(&grid, 0, 0, 4, 2);
        let lines: Vec<&str> = result.lines().collect();

        assert_eq!(lines.len(), 3);
        assert!(lines[0].ends_with('┐'));
        assert!(lines[1].ends_with('│'));
        assert!(lines[2].ends_with('┘'));
        let widths: Vec<usize> = lines.iter().map(|l| l.chars().count()).collect();
        assert!(widths.windows(2).all(|w| w[0] == w[1]));
    }
}
