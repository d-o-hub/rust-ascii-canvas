//! Arrow tool - draws lines with arrowheads.

use super::{DrawOp, Tool, ToolContext, ToolId, ToolResult, clamp_to_grid};

/// Arrow drawing tool.
pub struct ArrowTool {
    /// Start point of drag
    start: Option<(i32, i32)>,
}

impl Default for ArrowTool {
    fn default() -> Self {
        Self { start: None }
    }
}

impl ArrowTool {
    /// Create a new arrow tool.
    pub fn new() -> Self {
        Self::default()
    }

    /// Get arrowhead character based on direction.
    fn get_arrowhead(dx: i32, dy: i32) -> char {
        // Determine the primary direction
        if dx == 0 && dy == 0 {
            return '•';
        }

        let abs_dx = dx.abs();
        let abs_dy = dy.abs();

        // Determine based on angle
        if abs_dx == 0 {
            // Pure vertical
            if dy > 0 { '▼' } else { '▲' }
        } else if abs_dy == 0 {
            // Pure horizontal
            if dx > 0 { '►' } else { '◄' }
        } else {
            // Diagonal - use angle to determine
            let ratio = abs_dx * 10 / abs_dy.max(1);

            if ratio < 3 {
                // More vertical
                if dx > 0 {
                    if dy > 0 { '╲' } else { '╱' }
                } else {
                    if dy > 0 { '╱' } else { '╲' }
                }
            } else if ratio > 7 {
                // More horizontal
                if dx > 0 { '►' } else { '◄' }
            } else {
                // True diagonal - use '>' and '<' for compatibility
                if dx > 0 { '>' } else { '<' }
            }
        }
    }

    /// Draw an arrow line.
    fn draw_arrow(&self, x1: i32, y1: i32, x2: i32, y2: i32) -> Vec<DrawOp> {
        let mut ops = Vec::new();

        let dx = (x2 - x1).abs();
        let dy = -(y2 - y1).abs();
        let sx = if x1 < x2 { 1 } else { -1 };
        let sy = if y1 < y2 { 1 } else { -1 };
        let mut err = dx + dy;

        let mut x = x1;
        let mut y = y1;

        // Draw line using Bresenham's
        loop {
            let ch = Self::get_line_char(x, y, x2, y2);
            ops.push(DrawOp::new(x, y, ch));

            if x == x2 && y == y2 {
                break;
            }

            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }

        // Draw arrowhead at the end
        let arrowhead = Self::get_arrowhead(x2 - x1, y2 - y1);
        ops.push(DrawOp::new(x2, y2, arrowhead));

        ops
    }

    /// Get line character for direction.
    fn get_line_char(current_x: i32, current_y: i32, target_x: i32, target_y: i32) -> char {
        let dx = target_x - current_x;
        let dy = target_y - current_y;

        if dx == 0 {
            '│'
        } else if dy == 0 {
            '─'
        } else {
            let ratio = dx.abs() * 10 / dy.abs().max(1);
            if ratio < 3 {
                '│'
            } else if ratio > 7 {
                '─'
            } else {
                if (dx > 0 && dy < 0) || (dx < 0 && dy > 0) {
                    '/'
                } else {
                    '\\'
                }
            }
        }
    }
}

impl Tool for ArrowTool {
    fn id(&self) -> ToolId {
        ToolId::Arrow
    }

    fn on_pointer_down(&mut self, x: i32, y: i32, _ctx: &ToolContext) -> ToolResult {
        self.start = Some((x, y));
        ToolResult::new()
    }

    fn on_pointer_move(&mut self, x: i32, y: i32, ctx: &ToolContext) -> ToolResult {
        if let Some(start) = self.start {
            let (x, y) = clamp_to_grid(x, y, ctx.grid_width, ctx.grid_height);
            let ops = self.draw_arrow(start.0, start.1, x, y);
            ToolResult::new().with_ops(ops)
        } else {
            ToolResult::new()
        }
    }

    fn on_pointer_up(&mut self, x: i32, y: i32, ctx: &ToolContext) -> ToolResult {
        if let Some(start) = self.start {
            let (x, y) = clamp_to_grid(x, y, ctx.grid_width, ctx.grid_height);
            let ops = self.draw_arrow(start.0, start.1, x, y);
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
    fn test_arrow_tool() {
        let tool = ArrowTool::new();
        assert_eq!(tool.id(), ToolId::Arrow);
    }

    #[test]
    fn test_horizontal_arrow() {
        let tool = ArrowTool::new();
        let ops = tool.draw_arrow(0, 0, 5, 0);

        assert!(!ops.is_empty());
        // Last op should be arrowhead
        let last = ops.last().unwrap();
        assert_eq!(last.x, 5);
        assert_eq!(last.y, 0);
    }

    #[test]
    fn test_arrowhead_directions() {
        assert_eq!(ArrowTool::get_arrowhead(0, -1), '▲');  // up
        assert_eq!(ArrowTool::get_arrowhead(0, 1), '▼');   // down
        assert_eq!(ArrowTool::get_arrowhead(1, 0), '►');   // right
        assert_eq!(ArrowTool::get_arrowhead(-1, 0), '◄');  // left
    }
}
