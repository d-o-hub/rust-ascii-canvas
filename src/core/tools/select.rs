//! Select tool - selects and moves regions of the canvas.

use super::{clamp_to_grid, DrawOp, Tool, ToolContext, ToolId, ToolResult};
use crate::core::cell::Cell;
use crate::core::selection::Selection;
use smallvec::SmallVec;
use std::any::Any;

/// Character used for the transient marquee drawn while dragging a selection.
///
/// It must be one of the glyphs the web build uploads to the pixel atlas
/// (`web/render.ts`), otherwise the default renderer would draw nothing.
const MARQUEE_CHAR: char = '·';

/// Select and move tool.
pub struct SelectTool {
    /// Current selection
    selection: Option<Selection>,
    /// Whether we're dragging to select
    selecting: bool,
    /// Whether we're moving a selection
    moving: bool,
    /// Start of drag operation
    drag_start: Option<(i32, i32)>,
    /// Offset from selection corner when moving
    move_offset: Option<(i32, i32)>,
    /// Original content before move (for undo)
    original_content: SmallVec<[Cell; 256]>,
    /// Original bounds (for clearing during move)
    original_bounds: Option<(i32, i32, i32, i32)>,
}

impl Default for SelectTool {
    fn default() -> Self {
        Self {
            selection: None,
            selecting: false,
            moving: false,
            drag_start: None,
            move_offset: None,
            original_content: SmallVec::new(),
            original_bounds: None,
        }
    }
}

impl SelectTool {
    /// Create a new select tool.
    pub fn new() -> Self {
        Self::default()
    }

    /// Get current selection bounds ( clones the selection).
    pub fn get_selection(&self) -> Option<Selection> {
        self.selection.clone()
    }

    /// Set the current selection.
    pub fn set_selection(&mut self, selection: Selection) {
        self.selection = Some(selection);
    }

    /// Clear the current selection.
    pub fn clear_selection(&mut self) {
        self.selection = None;
        self.selecting = false;
        self.moving = false;
        self.drag_start = None;
        self.move_offset = None;
        self.original_content.clear();
        self.original_bounds = None;
    }

    /// Check if point is within current selection.
    fn point_in_selection(&self, x: i32, y: i32) -> bool {
        if let Some(ref sel) = self.selection {
            sel.contains(x, y)
        } else {
            false
        }
    }

    /// Check if we should start moving (clicked inside selection).
    fn should_start_move(&self, x: i32, y: i32) -> bool {
        self.selection.is_some() && self.point_in_selection(x, y)
    }

    /// Get the move offset if currently moving.
    pub fn get_move_offset(&self) -> Option<(i32, i32)> {
        if self.moving {
            self.move_offset
        } else {
            None
        }
    }

    /// Check if currently moving a selection.
    pub fn is_moving(&self) -> bool {
        self.moving
    }

    /// Check if currently dragging out a new selection.
    pub fn is_selecting(&self) -> bool {
        self.selecting
    }

    /// Preview operations for the pending selection: a dashed rectangle.
    ///
    /// The overlay is the only feedback on screen until pointer-up, so it is
    /// emitted from the press itself and refreshed on every drag step.
    fn marquee_preview(&self, ctx: &ToolContext) -> Vec<DrawOp> {
        match self.selection {
            Some(ref sel) => selection_marquee(sel, ctx.grid_width, ctx.grid_height),
            None => Vec::new(),
        }
    }
}

/// Build a dashed outline for `sel`, clipped to the grid.
fn selection_marquee(sel: &Selection, grid_width: usize, grid_height: usize) -> Vec<DrawOp> {
    if grid_width == 0 || grid_height == 0 {
        return Vec::new();
    }

    let (bounds_x1, bounds_y1, bounds_x2, bounds_y2) = sel.bounds();
    let max_x = grid_width as i32 - 1;
    let max_y = grid_height as i32 - 1;
    let min_x = bounds_x1.clamp(0, max_x);
    let min_y = bounds_y1.clamp(0, max_y);
    let max_x = bounds_x2.clamp(0, max_x);
    let max_y = bounds_y2.clamp(0, max_y);

    let mut points: Vec<(i32, i32)> = Vec::new();
    for x in min_x..=max_x {
        // Every other cell, plus the far corner, so the outline reads as dashed.
        if (x - min_x) % 2 == 0 || x == max_x {
            points.push((x, min_y));
            points.push((x, max_y));
        }
    }
    for y in min_y..=max_y {
        if (y - min_y) % 2 == 0 || y == max_y {
            points.push((min_x, y));
            points.push((max_x, y));
        }
    }
    points.sort_unstable();
    points.dedup();

    points
        .into_iter()
        .map(|(x, y)| DrawOp::new(x, y, MARQUEE_CHAR))
        .collect()
}

impl Tool for SelectTool {
    fn id(&self) -> ToolId {
        ToolId::Select
    }

    fn get_selection(&self) -> Option<Selection> {
        self.selection.clone()
    }

    fn on_pointer_down(&mut self, x: i32, y: i32, ctx: &ToolContext) -> ToolResult {
        let (x, y) = clamp_to_grid(x, y, ctx.grid_width, ctx.grid_height);

        if self.should_start_move(x, y) {
            self.moving = true;
            self.move_offset = self.selection.as_ref().map(|sel| (x - sel.x1, y - sel.y1));
            return ToolResult::new();
        }

        // Start a fresh selection. Seed it with the pressed cell so the marquee
        // is on screen before the first pointer-move arrives.
        self.selecting = true;
        self.moving = false;
        self.move_offset = None;
        self.drag_start = Some((x, y));
        self.selection = Some(Selection::new(x, y, x, y));

        ToolResult::new().with_ops(self.marquee_preview(ctx))
    }

    fn on_pointer_move(&mut self, x: i32, y: i32, ctx: &ToolContext) -> ToolResult {
        let (x, y) = clamp_to_grid(x, y, ctx.grid_width, ctx.grid_height);

        if self.selecting {
            if let Some((sx, sy)) = self.drag_start {
                self.selection = Some(Selection::new(sx, sy, x, y));
            }
            // Real-time dashed rectangle; the selection is committed on pointer-up.
            return ToolResult::new().with_ops(self.marquee_preview(ctx));
        }

        if self.moving {
            if let Some((offset_x, offset_y)) = self.move_offset {
                if let Some(ref mut sel) = self.selection {
                    // Calculate new position based on drag point minus offset
                    let new_x1 = x - offset_x;
                    let new_y1 = y - offset_y;
                    let width = sel.width() - 1;
                    let height = sel.height() - 1;

                    // Update selection bounds to new position
                    sel.x1 = new_x1;
                    sel.y1 = new_y1;
                    sel.x2 = new_x1 + width;
                    sel.y2 = new_y1 + height;
                }
            }
        }

        ToolResult::new()
    }

    fn on_pointer_up(&mut self, x: i32, y: i32, ctx: &ToolContext) -> ToolResult {
        let (x, y) = clamp_to_grid(x, y, ctx.grid_width, ctx.grid_height);

        if self.selecting {
            if let Some((sx, sy)) = self.drag_start {
                self.selection = Some(Selection::new(sx, sy, x, y));
            }
            self.selecting = false;
            self.drag_start = None;
            return ToolResult::new();
        }

        if self.moving {
            self.moving = false;
            self.move_offset = None;
            return ToolResult::new().finish();
        }

        ToolResult::new()
    }

    fn reset(&mut self) {
        self.clear_selection();
    }

    fn is_active(&self) -> bool {
        self.selecting || self.moving
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::tools::BorderStyle;

    fn ctx() -> ToolContext {
        ToolContext {
            grid_width: 80,
            grid_height: 40,
            border_style: BorderStyle::Single,
        }
    }

    #[test]
    fn test_select_tool() {
        let tool = SelectTool::new();
        assert_eq!(tool.id(), ToolId::Select);
        assert!(!tool.is_active());
    }

    #[test]
    fn test_selection_creation() {
        let mut tool = SelectTool::new();
        let ctx = ctx();

        tool.on_pointer_down(5, 5, &ctx);
        tool.on_pointer_move(10, 10, &ctx);
        tool.on_pointer_up(10, 10, &ctx);

        assert!(tool.get_selection().is_some());
        let sel = tool.get_selection().unwrap();
        assert_eq!(sel.x1, 5);
        assert_eq!(sel.y1, 5);
        assert_eq!(sel.x2, 10);
        assert_eq!(sel.y2, 10);
    }

    /// Issue #296: pressing must already preview, or the tool looks dead until
    /// pointer-up.
    #[test]
    fn drag_previews_a_dashed_marquee() {
        let mut tool = SelectTool::new();
        let ctx = ctx();

        let pressed = tool.on_pointer_down(5, 5, &ctx);
        assert_eq!(pressed.ops.len(), 1);
        assert_eq!(pressed.ops[0].cell.ch, MARQUEE_CHAR);

        let dragged = tool.on_pointer_move(9, 7, &ctx);
        assert!(dragged.ops.iter().all(|op| op.cell.ch == MARQUEE_CHAR));
        // Dashed: the outline is not a solid ring of cells.
        assert!(
            dragged.ops.len() < 2 * (5 + 3),
            "outline is not dashed: {}",
            dragged.ops.len()
        );
        // Every marquee cell sits on the selection border.
        for op in &dragged.ops {
            assert!(
                (5..=9).contains(&op.x) && (5..=7).contains(&op.y),
                "marquee escaped the selection at ({}, {})",
                op.x,
                op.y
            );
        }
    }

    #[test]
    fn marquee_is_clipped_to_the_grid() {
        let mut tool = SelectTool::new();
        let ctx = ToolContext {
            grid_width: 4,
            grid_height: 3,
            border_style: BorderStyle::Single,
        };

        tool.on_pointer_down(2, 1, &ctx);
        let dragged = tool.on_pointer_move(99, 99, &ctx);
        assert!(!dragged.ops.is_empty());
        for op in &dragged.ops {
            assert!((0..4).contains(&op.x) && (0..3).contains(&op.y));
        }
    }

    #[test]
    fn click_inside_a_committed_selection_moves_instead_of_reselecting() {
        let mut tool = SelectTool::new();
        let ctx = ctx();

        tool.on_pointer_down(2, 2, &ctx);
        tool.on_pointer_up(6, 6, &ctx);
        assert!(!tool.is_selecting());

        let pressed = tool.on_pointer_down(3, 3, &ctx);
        assert!(tool.is_moving());
        assert!(!tool.is_selecting());
        assert!(pressed.ops.is_empty(), "a move draws no marquee");

        tool.on_pointer_move(5, 3, &ctx);
        let sel = tool.get_selection().unwrap();
        assert_eq!(sel.bounds(), (4, 2, 8, 6));
        assert_eq!(sel.width(), 5);
        assert_eq!(sel.height(), 5);
    }

    #[test]
    fn click_outside_a_committed_selection_starts_a_new_drag() {
        let mut tool = SelectTool::new();
        let ctx = ctx();

        tool.on_pointer_down(2, 2, &ctx);
        tool.on_pointer_up(6, 6, &ctx);

        let pressed = tool.on_pointer_down(20, 20, &ctx);
        assert!(tool.is_selecting());
        assert!(!tool.is_moving());
        assert!(!pressed.ops.is_empty());
    }
}
