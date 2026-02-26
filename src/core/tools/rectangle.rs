//! Rectangle tool - draws rectangular ASCII boxes with various border styles.

use super::{clamp_to_grid, BorderStyle, DrawOp, Tool, ToolContext, ToolId, ToolResult};

/// Rectangle drawing tool.
#[derive(Default)]
pub struct RectangleTool {
    /// Start point of drag
    start: Option<(i32, i32)>,
    /// Current end point during drag
    end: Option<(i32, i32)>,
    /// Border style for the rectangle
    border_style: BorderStyle,
}

impl RectangleTool {
    /// Create a new rectangle tool.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the border style for this tool.
    pub fn with_border_style(mut self, style: BorderStyle) -> Self {
        self.border_style = style;
        self
    }

    /// Update the border style.
    pub fn set_border_style(&mut self, style: BorderStyle) {
        self.border_style = style;
    }

    /// Generate draw operations for a rectangle.
    fn draw_rectangle(&self, x1: i32, y1: i32, x2: i32, y2: i32) -> Vec<DrawOp> {
        let mut ops = Vec::new();

        let (min_x, max_x) = (x1.min(x2), x1.max(x2));
        let (min_y, max_y) = (y1.min(y2), y1.max(y2));

        // Handle single-point case
        if min_x == max_x && min_y == max_y {
            ops.push(DrawOp::new(min_x, min_y, self.border_style.corners()[0]));
            return ops;
        }

        // Handle single-line horizontal
        if min_y == max_y {
            let h = self.border_style.horizontal();
            for x in min_x..=max_x {
                ops.push(DrawOp::new(x, min_y, h));
            }
            return ops;
        }

        // Handle single-line vertical
        if min_x == max_x {
            let v = self.border_style.vertical();
            for y in min_y..=max_y {
                ops.push(DrawOp::new(min_x, y, v));
            }
            return ops;
        }

        let corners = self.border_style.corners();
        let h = self.border_style.horizontal();
        let v = self.border_style.vertical();

        // Draw corners
        ops.push(DrawOp::new(min_x, min_y, corners[0])); // top-left
        ops.push(DrawOp::new(max_x, min_y, corners[1])); // top-right
        ops.push(DrawOp::new(min_x, max_y, corners[2])); // bottom-left
        ops.push(DrawOp::new(max_x, max_y, corners[3])); // bottom-right

        // Draw horizontal lines
        for x in (min_x + 1)..max_x {
            ops.push(DrawOp::new(x, min_y, h));
            ops.push(DrawOp::new(x, max_y, h));
        }

        // Draw vertical lines
        for y in (min_y + 1)..max_y {
            ops.push(DrawOp::new(min_x, y, v));
            ops.push(DrawOp::new(max_x, y, v));
        }

        ops
    }
}

impl Tool for RectangleTool {
    fn id(&self) -> ToolId {
        ToolId::Rectangle
    }

    fn on_pointer_down(&mut self, x: i32, y: i32, _ctx: &ToolContext) -> ToolResult {
        self.start = Some((x, y));
        self.end = Some((x, y));
        ToolResult::new()
    }

    fn on_pointer_move(&mut self, x: i32, y: i32, ctx: &ToolContext) -> ToolResult {
        if let Some(start) = self.start {
            let (x, y) = clamp_to_grid(x, y, ctx.grid_width, ctx.grid_height);
            self.end = Some((x, y));
            let ops = self.draw_rectangle(start.0, start.1, x, y);
            ToolResult::new().with_ops(ops)
        } else {
            ToolResult::new()
        }
    }

    fn on_pointer_up(&mut self, x: i32, y: i32, ctx: &ToolContext) -> ToolResult {
        if let Some(start) = self.start {
            let (x, y) = clamp_to_grid(x, y, ctx.grid_width, ctx.grid_height);
            let ops = self.draw_rectangle(start.0, start.1, x, y);
            self.start = None;
            self.end = None;
            ToolResult::new().with_ops(ops).finish()
        } else {
            ToolResult::new()
        }
    }

    fn reset(&mut self) {
        self.start = None;
        self.end = None;
    }

    fn is_active(&self) -> bool {
        self.start.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rectangle_tool() {
        let tool = RectangleTool::new();
        assert_eq!(tool.id(), ToolId::Rectangle);
        assert!(!tool.is_active());
    }

    #[test]
    fn test_draw_rectangle() {
        let tool = RectangleTool::new();
        let ops = tool.draw_rectangle(0, 0, 4, 2);

        // Check corners
        assert_eq!(
            ops.iter().find(|o| o.x == 0 && o.y == 0).unwrap().cell.ch,
            '┌'
        );
        assert_eq!(
            ops.iter().find(|o| o.x == 4 && o.y == 0).unwrap().cell.ch,
            '┐'
        );
        assert_eq!(
            ops.iter().find(|o| o.x == 0 && o.y == 2).unwrap().cell.ch,
            '└'
        );
        assert_eq!(
            ops.iter().find(|o| o.x == 4 && o.y == 2).unwrap().cell.ch,
            '┘'
        );
    }

    #[test]
    fn test_single_point() {
        let tool = RectangleTool::new();
        let ops = tool.draw_rectangle(5, 5, 5, 5);
        assert_eq!(ops.len(), 1);
        assert_eq!(ops[0].x, 5);
        assert_eq!(ops[0].y, 5);
    }
}
