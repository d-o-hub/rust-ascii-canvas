//! Text tool - places and edits text on the canvas.

use super::{DrawOp, Tool, ToolContext, ToolId, ToolResult, clamp_to_grid};
use smallvec::SmallVec;

/// Text tool for typing characters on the canvas.
pub struct TextTool {
    /// Current cursor position
    cursor: Option<(i32, i32)>,
    /// Characters typed in current session
    buffer: SmallVec<[char; 64]>,
    /// Starting position for the text
    start_pos: Option<(i32, i32)>,
}

impl Default for TextTool {
    fn default() -> Self {
        Self {
            cursor: None,
            buffer: SmallVec::new(),
            start_pos: None,
        }
    }
}

impl TextTool {
    /// Create a new text tool.
    pub fn new() -> Self {
        Self::default()
    }

    /// Handle backspace - remove last character.
    pub fn backspace(&mut self) -> Option<DrawOp> {
        if self.buffer.pop().is_some() {
            if let Some((x, y)) = self.cursor {
                // Move cursor back
                if x > 0 {
                    self.cursor = Some((x - 1, y));
                    return Some(DrawOp::new(x - 1, y, ' '));
                }
            }
        }
        None
    }

    /// Get all operations to render current text buffer.
    fn get_text_ops(&self) -> Vec<DrawOp> {
        let mut ops = Vec::new();

        if let Some((start_x, start_y)) = self.start_pos {
            for (i, &ch) in self.buffer.iter().enumerate() {
                ops.push(DrawOp::new(start_x + i as i32, start_y, ch));
            }
        }

        ops
    }

    /// Get the cursor position.
    pub fn cursor_position(&self) -> Option<(i32, i32)> {
        self.cursor
    }
}

impl Tool for TextTool {
    fn id(&self) -> ToolId {
        ToolId::Text
    }

    fn on_pointer_down(&mut self, x: i32, y: i32, ctx: &ToolContext) -> ToolResult {
        let (x, y) = clamp_to_grid(x, y, ctx.grid_width, ctx.grid_height);

        // Commit any existing text and start fresh
        let prev_ops = self.get_text_ops();

        self.buffer.clear();
        self.cursor = Some((x, y));
        self.start_pos = Some((x, y));

        ToolResult::new().with_ops(prev_ops)
    }

    fn on_pointer_move(&mut self, _x: i32, _y: i32, _ctx: &ToolContext) -> ToolResult {
        // Text tool doesn't respond to drag
        ToolResult::new()
    }

    fn on_pointer_up(&mut self, _x: i32, _y: i32, _ctx: &ToolContext) -> ToolResult {
        // Text tool stays active after click
        ToolResult::new()
    }

    fn on_key(&mut self, ch: char, ctx: &ToolContext) -> ToolResult {
        if let Some((mut x, y)) = self.cursor {
            // Handle special characters
            if ch == '\n' || ch == '\r' {
                // Move to next line
                self.cursor = Some((self.start_pos.unwrap().0, y + 1));
                return ToolResult::new();
            }

            if ch == '\x08' {
                // Backspace
                if let Some(op) = self.backspace() {
                    return ToolResult::new().with_op(op);
                }
                return ToolResult::new();
            }

            // Skip control characters
            if ch.is_control() {
                return ToolResult::new();
            }

            // Check if we're at the right edge
            if x >= ctx.grid_width as i32 - 1 {
                return ToolResult::new();
            }

            // Add character to buffer and create draw op
            self.buffer.push(ch);

            // Update cursor
            x += 1;
            self.cursor = Some((x, y));

            return ToolResult::new().with_op(DrawOp::new(x - 1, y, ch));
        }

        ToolResult::new()
    }

    fn reset(&mut self) {
        self.cursor = None;
        self.buffer.clear();
        self.start_pos = None;
    }

    fn is_active(&self) -> bool {
        self.cursor.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_tool() {
        let tool = TextTool::new();
        assert_eq!(tool.id(), ToolId::Text);
        assert!(!tool.is_active());
    }

    #[test]
    fn test_text_input() {
        let mut tool = TextTool::new();
        let ctx = ToolContext {
            grid_width: 80,
            grid_height: 40,
            border_style: Default::default(),
        };

        tool.on_pointer_down(5, 5, &ctx);
        assert!(tool.is_active());

        let result = tool.on_key('H', &ctx);
        assert!(result.modified);
        assert_eq!(result.ops.len(), 1);
        assert_eq!(result.ops[0].cell.ch, 'H');
    }
}
