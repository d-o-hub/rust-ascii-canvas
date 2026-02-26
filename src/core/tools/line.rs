//! Line tool - draws ASCII lines using Bresenham's algorithm.

use super::{DrawOp, Tool, ToolContext, ToolId, ToolResult, clamp_to_grid};

/// Line drawing tool using Bresenham's algorithm with ASCII characters.
pub struct LineTool {
    /// Start point of drag
    start: Option<(i32, i32)>,
}

impl Default for LineTool {
    fn default() -> Self {
        Self { start: None }
    }
}

impl LineTool {
    /// Create a new line tool.
    pub fn new() -> Self {
        Self::default()
    }

    /// Get appropriate line character based on line direction.
    /// sx, sy: direction of line movement (sign of delta)
    /// dx, dy: absolute distances
    fn get_line_char(sx: i32, sy: i32, dx: i32, dy: i32) -> char {
        if dx == 0 {
            // Pure vertical
            '│'
        } else if dy == 0 {
            // Pure horizontal
            '─'
        } else {
            // Diagonal - determine character based on direction
            if sx > 0 {
                if sy > 0 {
                    '\\' // moving down-right
                } else {
                    '/'  // moving up-right
                }
            } else {
                if sy > 0 {
                    '/'  // moving down-left
                } else {
                    '\\' // moving up-left
                }
            }
        }
    }

    /// Draw a line using Bresenham's algorithm.
    fn draw_line(&self, x1: i32, y1: i32, x2: i32, y2: i32) -> Vec<DrawOp> {
        let mut ops = Vec::new();

        let dx = (x2 - x1).abs();
        let dy = (y2 - y1).abs();
        let sx = if x1 < x2 { 1 } else { -1 };
        let sy = if y1 < y2 { 1 } else { -1 };

        // Get the character for this line's primary direction
        let ch = Self::get_line_char(sx, sy, dx, dy);

        let mut err = dx - dy;
        let mut x = x1;
        let mut y = y1;

        loop {
            ops.push(DrawOp::new(x, y, ch));

            if x == x2 && y == y2 {
                break;
            }

            let e2 = 2 * err;
            if e2 > -dy {
                err -= dy;
                x += sx;
            }
            if e2 < dx {
                err += dx;
                y += sy;
            }
        }

        ops
    }
}

impl Tool for LineTool {
    fn id(&self) -> ToolId {
        ToolId::Line
    }

    fn on_pointer_down(&mut self, x: i32, y: i32, _ctx: &ToolContext) -> ToolResult {
        self.start = Some((x, y));
        ToolResult::new()
    }

    fn on_pointer_move(&mut self, x: i32, y: i32, ctx: &ToolContext) -> ToolResult {
        if let Some(start) = self.start {
            let (x, y) = clamp_to_grid(x, y, ctx.grid_width, ctx.grid_height);
            let ops = self.draw_line(start.0, start.1, x, y);
            ToolResult::new().with_ops(ops)
        } else {
            ToolResult::new()
        }
    }

    fn on_pointer_up(&mut self, x: i32, y: i32, ctx: &ToolContext) -> ToolResult {
        if let Some(start) = self.start {
            let (x, y) = clamp_to_grid(x, y, ctx.grid_width, ctx.grid_height);
            let ops = self.draw_line(start.0, start.1, x, y);
            self.start = None;
            ToolResult::new().with_ops(ops).finish()
        } else {
            ToolResult::new()
        }
    }

    fn reset(&mut self) {
        self.start = None;
    }

    fn is_active(&self) -> bool {
        self.start.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_tool() {
        let tool = LineTool::new();
        assert_eq!(tool.id(), ToolId::Line);
    }

    #[test]
    fn test_horizontal_line() {
        let tool = LineTool::new();
        let ops = tool.draw_line(0, 0, 5, 0);

        assert_eq!(ops.len(), 6);
        for op in &ops {
            assert_eq!(op.cell.ch, '─');
        }
    }

    #[test]
    fn test_vertical_line() {
        let tool = LineTool::new();
        let ops = tool.draw_line(0, 0, 0, 5);

        assert_eq!(ops.len(), 6);
        for op in &ops {
            assert_eq!(op.cell.ch, '│');
        }
    }

    #[test]
    fn test_diagonal_line() {
        let tool = LineTool::new();

        // Down-right diagonal
        let ops = tool.draw_line(0, 0, 3, 3);
        for op in &ops {
            assert_eq!(op.cell.ch, '\\');
        }

        // Down-left diagonal
        let ops = tool.draw_line(3, 0, 0, 3);
        for op in &ops {
            assert_eq!(op.cell.ch, '/');
        }
    }
}
