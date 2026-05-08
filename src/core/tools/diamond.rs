//! Diamond tool - draws diamond/rhombus shapes.

use super::{clamp_to_grid, DrawOp, Tool, ToolContext, ToolId, ToolResult};
use crate::utils::math::bresenham_line;
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

        let dx = (x2 - x1).abs();
        let dy = (y2 - y1).abs();

        if dx == 0 && dy == 0 {
            ops.push(DrawOp::new(x1, y1, '◆'));
            return ops;
        }

        let cx = (x1 + x2) / 2;
        let cy = (y1 + y2) / 2;

        // Ensure minimum size for visibility during drag
        let half_width = (dx / 2).max(if dx > 0 { 1 } else { 0 });
        let half_height = (dy / 2).max(if dy > 0 { 1 } else { 0 });

        // Draw the four sides of the diamond
        let sides = [
            (cx, cy - half_height, cx + half_width, cy, '╲'), // Top-Right
            (cx + half_width, cy, cx, cy + half_height, '╱'), // Bottom-Right
            (cx, cy + half_height, cx - half_width, cy, '╲'), // Bottom-Left
            (cx - half_width, cy, cx, cy - half_height, '╱'), // Top-Left
        ];

        for (px1, py1, px2, py2, ch) in sides {
            bresenham_line(px1, py1, px2, py2, |x, y| {
                ops.push(DrawOp::new(x, y, ch));
            });
        }

        // Remove duplicates and reorganize
        ops.sort_by_key(|op| (op.y, op.x));
        ops.dedup_by_key(|op| (op.x, op.y));

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
