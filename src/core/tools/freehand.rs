//! Freehand tool - draws free-form ASCII characters.

use super::{clamp_to_grid, DrawOp, Tool, ToolContext, ToolId, ToolResult};
use smallvec::SmallVec;

/// Freehand drawing tool.
pub struct FreehandTool {
    /// Whether currently drawing
    drawing: bool,
    /// Last position for interpolation
    last_pos: Option<(i32, i32)>,
    /// Character to draw with
    draw_char: char,
    /// Buffer for undo
    ops_buffer: SmallVec<[DrawOp; 128]>,
}

impl Default for FreehandTool {
    fn default() -> Self {
        Self {
            drawing: false,
            last_pos: None,
            draw_char: '*',
            ops_buffer: SmallVec::new(),
        }
    }
}

impl FreehandTool {
    /// Create a new freehand tool.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the character to draw with.
    pub fn set_char(&mut self, ch: char) {
        self.draw_char = ch;
    }

    /// Get the current draw character.
    pub fn get_char(&self) -> char {
        self.draw_char
    }

    /// Draw a line from last position to current (interpolation).
    fn interpolate_to(&self, x: i32, y: i32) -> Vec<DrawOp> {
        let mut ops = Vec::new();

        if let Some((lx, ly)) = self.last_pos {
            // Use Bresenham to interpolate
            let dx = (x - lx).abs();
            let dy = (y - ly).abs();
            let sx = if lx < x { 1 } else { -1 };
            let sy = if ly < y { 1 } else { -1 };
            let mut err = dx - dy;

            let mut cx = lx;
            let mut cy = ly;

            loop {
                ops.push(DrawOp::new(cx, cy, self.draw_char));

                if cx == x && cy == y {
                    break;
                }

                let e2 = 2 * err;
                if e2 > -dy {
                    err -= dy;
                    cx += sx;
                }
                if e2 < dx {
                    err += dx;
                    cy += sy;
                }
            }
        } else {
            ops.push(DrawOp::new(x, y, self.draw_char));
        }

        ops
    }
}

impl Tool for FreehandTool {
    fn id(&self) -> ToolId {
        ToolId::Freehand
    }

    fn on_pointer_down(&mut self, x: i32, y: i32, ctx: &ToolContext) -> ToolResult {
        let (x, y) = clamp_to_grid(x, y, ctx.grid_width, ctx.grid_height);

        self.drawing = true;
        self.last_pos = Some((x, y));
        self.ops_buffer.clear();

        let op = DrawOp::new(x, y, self.draw_char);
        self.ops_buffer.push(op.clone());

        ToolResult::new().with_op(op)
    }

    fn on_pointer_move(&mut self, x: i32, y: i32, ctx: &ToolContext) -> ToolResult {
        if !self.drawing {
            return ToolResult::new();
        }

        let (x, y) = clamp_to_grid(x, y, ctx.grid_width, ctx.grid_height);

        // Skip if same position
        if self.last_pos == Some((x, y)) {
            return ToolResult::new();
        }

        let ops = self.interpolate_to(x, y);
        self.last_pos = Some((x, y));

        // Store for undo
        for op in &ops {
            self.ops_buffer.push(op.clone());
        }

        ToolResult::new().with_ops(ops)
    }

    fn on_pointer_up(&mut self, _x: i32, _y: i32, _ctx: &ToolContext) -> ToolResult {
        self.drawing = false;
        self.last_pos = None;

        // Return finished result with all ops for undo
        let ops: Vec<DrawOp> = self.ops_buffer.to_vec();
        self.ops_buffer.clear();

        ToolResult::new().with_ops(ops).finish()
    }

    fn reset(&mut self) {
        self.drawing = false;
        self.last_pos = None;
        self.ops_buffer.clear();
    }

    fn is_active(&self) -> bool {
        self.drawing
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_freehand_tool() {
        let tool = FreehandTool::new();
        assert_eq!(tool.id(), ToolId::Freehand);
        assert!(!tool.is_active());
    }

    #[test]
    fn test_freehand_draw() {
        let mut tool = FreehandTool::new();
        let ctx = ToolContext {
            grid_width: 80,
            grid_height: 40,
            border_style: Default::default(),
        };

        let result = tool.on_pointer_down(10, 10, &ctx);
        assert!(result.modified);
        assert!(tool.is_active());

        let result = tool.on_pointer_move(15, 10, &ctx);
        assert!(result.modified);
    }
}
