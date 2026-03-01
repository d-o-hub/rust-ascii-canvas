//! Diamond tool - draws diamond/rhombus shapes.

use super::{clamp_to_grid, DrawOp, Tool, ToolContext, ToolId, ToolResult};
use std::any::Any;

/// Diamond drawing tool.
#[derive(Default)]
pub struct DiamondTool {
    /// Start point of drag
    start: Option<(i32, i32)>,
}

impl DiamondTool {
    /// Create a new diamond tool.
    pub fn new() -> Self {
        Self::default()
    }

    /// Draw a diamond shape.
    fn draw_diamond(&self, x1: i32, y1: i32, x2: i32, y2: i32) -> Vec<DrawOp> {
        let mut ops = Vec::new();

        let cx = (x1 + x2) / 2;
        let cy = (y1 + y2) / 2;
        let half_width = (x2 - x1).abs() / 2;
        let half_height = (y2 - y1).abs() / 2;

        if half_width == 0 && half_height == 0 {
            ops.push(DrawOp::new(cx, cy, '◆'));
            return ops;
        }

        // Draw four diagonal lines from center to corners
        // Top corner
        ops.append(&mut Self::draw_diagonal_line(
            cx,
            cy,
            cx,
            cy - half_height,
            '/',
            '\\',
        ));

        // Right corner
        ops.append(&mut Self::draw_diagonal_line(
            cx,
            cy,
            cx + half_width,
            cy,
            '/',
            '\\',
        ));

        // Bottom corner
        ops.append(&mut Self::draw_diagonal_line(
            cx,
            cy,
            cx,
            cy + half_height,
            '\\',
            '/',
        ));

        // Left corner
        ops.append(&mut Self::draw_diagonal_line(
            cx,
            cy,
            cx - half_width,
            cy,
            '\\',
            '/',
        ));

        // Remove duplicates and reorganize
        ops.sort_by_key(|op| (op.y, op.x));
        ops.dedup_by_key(|op| (op.x, op.y));

        ops
    }

    /// Draw a diagonal line segment.
    fn draw_diagonal_line(x1: i32, y1: i32, x2: i32, y2: i32, ch1: char, ch2: char) -> Vec<DrawOp> {
        let mut ops = Vec::new();

        let sx = if x1 < x2 { 1 } else { -1 };
        let sy = if y1 < y2 { 1 } else { -1 };

        let mut x = x1;
        let mut y = y1;

        // Use the appropriate character based on direction
        let ch = if (sx > 0 && sy < 0) || (sx < 0 && sy > 0) {
            ch1
        } else {
            ch2
        };

        loop {
            ops.push(DrawOp::new(x, y, ch));

            if x == x2 && y == y2 {
                break;
            }

            if x != x2 {
                x += sx;
            }
            if y != y2 {
                y += sy;
            }
        }

        ops
    }
}

impl Tool for DiamondTool {
    fn id(&self) -> ToolId {
        ToolId::Diamond
    }

    fn on_pointer_down(&mut self, x: i32, y: i32, _ctx: &ToolContext) -> ToolResult {
        self.start = Some((x, y));
        ToolResult::new()
    }

    fn on_pointer_move(&mut self, x: i32, y: i32, ctx: &ToolContext) -> ToolResult {
        if let Some(start) = self.start {
            let (x, y) = clamp_to_grid(x, y, ctx.grid_width, ctx.grid_height);
            let ops = self.draw_diamond(start.0, start.1, x, y);
            ToolResult::new().with_ops(ops)
        } else {
            ToolResult::new()
        }
    }

    fn on_pointer_up(&mut self, x: i32, y: i32, ctx: &ToolContext) -> ToolResult {
        if let Some(start) = self.start {
            let (x, y) = clamp_to_grid(x, y, ctx.grid_width, ctx.grid_height);
            let ops = self.draw_diamond(start.0, start.1, x, y);
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

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diamond_tool() {
        let tool = DiamondTool::new();
        assert_eq!(tool.id(), ToolId::Diamond);
    }

    #[test]
    fn test_single_point_diamond() {
        let tool = DiamondTool::new();
        let ops = tool.draw_diamond(5, 5, 5, 5);
        assert_eq!(ops.len(), 1);
        assert_eq!(ops[0].cell.ch, '◆');
    }
}
