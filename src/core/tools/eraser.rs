//! Eraser tool - clears cells on the canvas.

use super::{clamp_to_grid, DrawOp, Tool, ToolContext, ToolId, ToolResult};
use smallvec::SmallVec;

/// Eraser tool for clearing cells.
pub struct EraserTool {
    /// Whether currently erasing
    erasing: bool,
    /// Last position for interpolation
    last_pos: Option<(i32, i32)>,
    /// Eraser size (radius in cells)
    size: i32,
    /// Buffer for undo
    ops_buffer: SmallVec<[DrawOp; 128]>,
}

impl Default for EraserTool {
    fn default() -> Self {
        Self {
            erasing: false,
            last_pos: None,
            size: 1,
            ops_buffer: SmallVec::new(),
        }
    }
}

impl EraserTool {
    /// Create a new eraser tool.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set eraser size (radius).
    pub fn set_size(&mut self, size: i32) {
        self.size = size.max(1);
    }

    /// Get current eraser size.
    pub fn size(&self) -> i32 {
        self.size
    }

    /// Generate clear operations for eraser at position.
    fn erase_at(&self, x: i32, y: i32) -> Vec<DrawOp> {
        let mut ops = Vec::new();

        for dy in -self.size + 1..self.size {
            for dx in -self.size + 1..self.size {
                // Optional: make circular eraser
                // if dx * dx + dy * dy < self.size * self.size {
                let ex = x + dx;
                let ey = y + dy;
                ops.push(DrawOp::new(ex, ey, ' '));
                // }
            }
        }

        ops
    }

    /// Interpolate erase path.
    fn interpolate_to(&self, x: i32, y: i32) -> Vec<DrawOp> {
        let mut ops = Vec::new();

        if let Some((lx, ly)) = self.last_pos {
            // Bresenham interpolation
            let dx = (x - lx).abs();
            let dy = (y - ly).abs();
            let sx = if lx < x { 1 } else { -1 };
            let sy = if ly < y { 1 } else { -1 };
            let mut err = dx - dy;

            let mut cx = lx;
            let mut cy = ly;

            loop {
                ops.extend(self.erase_at(cx, cy));

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
            ops.extend(self.erase_at(x, y));
        }

        ops
    }
}

impl Tool for EraserTool {
    fn id(&self) -> ToolId {
        ToolId::Eraser
    }

    fn on_pointer_down(&mut self, x: i32, y: i32, ctx: &ToolContext) -> ToolResult {
        let (x, y) = clamp_to_grid(x, y, ctx.grid_width, ctx.grid_height);

        self.erasing = true;
        self.last_pos = Some((x, y));
        self.ops_buffer.clear();

        let ops = self.erase_at(x, y);
        for op in &ops {
            self.ops_buffer.push(op.clone());
        }

        ToolResult::new().with_ops(ops)
    }

    fn on_pointer_move(&mut self, x: i32, y: i32, ctx: &ToolContext) -> ToolResult {
        if !self.erasing {
            return ToolResult::new();
        }

        let (x, y) = clamp_to_grid(x, y, ctx.grid_width, ctx.grid_height);

        if self.last_pos == Some((x, y)) {
            return ToolResult::new();
        }

        let ops = self.interpolate_to(x, y);
        self.last_pos = Some((x, y));

        for op in &ops {
            self.ops_buffer.push(op.clone());
        }

        ToolResult::new().with_ops(ops)
    }

    fn on_pointer_up(&mut self, _x: i32, _y: i32, _ctx: &ToolContext) -> ToolResult {
        self.erasing = false;
        self.last_pos = None;

        let ops: Vec<DrawOp> = self.ops_buffer.to_vec();
        self.ops_buffer.clear();

        ToolResult::new().with_ops(ops).finish()
    }

    fn reset(&mut self) {
        self.erasing = false;
        self.last_pos = None;
        self.ops_buffer.clear();
    }

    fn is_active(&self) -> bool {
        self.erasing
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eraser_tool() {
        let tool = EraserTool::new();
        assert_eq!(tool.id(), ToolId::Eraser);
        assert!(!tool.is_active());
    }

    #[test]
    fn test_eraser_at() {
        let tool = EraserTool::new();
        let ops = tool.erase_at(10, 10);

        // Size 1 eraser should clear 1 cell
        assert_eq!(ops.len(), 1);
        assert_eq!(ops[0].x, 10);
        assert_eq!(ops[0].y, 10);
        assert!(ops[0].cell.is_empty());
    }

    #[test]
    fn test_larger_eraser() {
        let mut tool = EraserTool::new();
        tool.set_size(2);
        let ops = tool.erase_at(10, 10);

        // Size 2 eraser clears a 3x3 area (from -size+1 to size-1 = -1 to 1)
        // dx: -1, 0, 1 (3 values)
        // dy: -1, 0, 1 (3 values)
        // Total: 3 x 3 = 9 cells
        assert_eq!(ops.len(), 9);
    }
}
