//! Tools module - drawing tools for the ASCII canvas.
//!
//! Each tool implements the Tool trait for handling input events
//! and generating draw operations.

mod arrow;
mod diamond;
mod eraser;
mod freehand;
mod line;
mod rectangle;
mod select;
mod text;

pub use arrow::ArrowTool;
pub use diamond::DiamondTool;
pub use eraser::EraserTool;
pub use freehand::FreehandTool;
pub use line::{LineDirection, LineTool};
pub use rectangle::RectangleTool;
pub use select::SelectTool;
pub use text::TextTool;

use crate::core::cell::Cell;
use crate::core::selection::Selection;
use serde::{Deserialize, Serialize};

/// Unique identifier for a tool.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ToolId {
    /// Rectangle tool for drawing boxes (shortcut: R).
    #[default]
    Rectangle,
    /// Line tool for drawing straight lines (shortcut: L).
    Line,
    /// Arrow tool for drawing arrows (shortcut: A).
    Arrow,
    /// Diamond tool for drawing diamond shapes (shortcut: D).
    Diamond,
    /// Text tool for typing characters (shortcut: T).
    Text,
    /// Freehand drawing tool (shortcut: F).
    Freehand,
    /// Selection tool (shortcut: V).
    Select,
    /// Eraser tool for clearing cells (shortcut: E).
    Eraser,
}

impl ToolId {
    /// Get keyboard shortcut for this tool.
    pub fn shortcut(&self) -> char {
        match self {
            ToolId::Rectangle => 'R',
            ToolId::Line => 'L',
            ToolId::Arrow => 'A',
            ToolId::Diamond => 'D',
            ToolId::Text => 'T',
            ToolId::Freehand => 'F',
            ToolId::Select => 'V',
            ToolId::Eraser => 'E',
        }
    }

    /// Get tool from keyboard shortcut.
    pub fn from_shortcut(ch: char) -> Option<Self> {
        match ch.to_uppercase().next().unwrap() {
            'R' => Some(Self::Rectangle),
            'L' => Some(Self::Line),
            'A' => Some(Self::Arrow),
            'D' => Some(Self::Diamond),
            'T' => Some(Self::Text),
            'F' => Some(Self::Freehand),
            'V' => Some(Self::Select),
            'E' => Some(Self::Eraser),
            _ => None,
        }
    }

    /// Get display name for this tool.
    pub fn name(&self) -> &'static str {
        match self {
            ToolId::Rectangle => "Rectangle",
            ToolId::Line => "Line",
            ToolId::Arrow => "Arrow",
            ToolId::Diamond => "Diamond",
            ToolId::Text => "Text",
            ToolId::Freehand => "Freehand",
            ToolId::Select => "Select",
            ToolId::Eraser => "Eraser",
        }
    }
}

/// Border style for shapes like rectangles.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum BorderStyle {
    /// Single line border: ┌─┐│└┘
    #[default]
    Single,
    /// Double line border: ╔═╗║╚╝
    Double,
    /// Heavy line border: ┏━┓┃┗┛
    Heavy,
    /// Rounded corners: ╭─╮│╰╯
    Rounded,
    /// ASCII stars: +-+|+-+
    Ascii,
    /// Hash/dotted style
    Dotted,
}

impl BorderStyle {
    /// Get corner characters for this style.
    pub fn corners(&self) -> [char; 4] {
        match self {
            BorderStyle::Single => ['┌', '┐', '└', '┘'],
            BorderStyle::Double => ['╔', '╗', '╚', '╝'],
            BorderStyle::Heavy => ['┏', '┓', '┗', '┛'],
            BorderStyle::Rounded => ['╭', '╮', '╰', '╯'],
            BorderStyle::Ascii => ['+', '+', '+', '+'],
            BorderStyle::Dotted => ['*', '*', '*', '*'],
        }
    }

    /// Get horizontal line character.
    pub fn horizontal(&self) -> char {
        match self {
            BorderStyle::Single => '─',
            BorderStyle::Double => '═',
            BorderStyle::Heavy => '━',
            BorderStyle::Rounded => '─',
            BorderStyle::Ascii => '-',
            BorderStyle::Dotted => '*',
        }
    }

    /// Get vertical line character.
    pub fn vertical(&self) -> char {
        match self {
            BorderStyle::Single => '│',
            BorderStyle::Double => '║',
            BorderStyle::Heavy => '┃',
            BorderStyle::Rounded => '│',
            BorderStyle::Ascii => '|',
            BorderStyle::Dotted => '*',
        }
    }
}

/// A single draw operation to be applied to the grid.
#[derive(Clone, Debug)]
pub struct DrawOp {
    /// X coordinate in grid space.
    pub x: i32,
    /// Y coordinate in grid space.
    pub y: i32,
    /// The cell content to set.
    pub cell: Cell,
}

impl DrawOp {
    /// Create a new draw operation at the given coordinates.
    pub fn new(x: i32, y: i32, ch: char) -> Self {
        Self {
            x,
            y,
            cell: Cell::new(ch),
        }
    }
}

/// Result of a tool operation.
#[derive(Clone, Debug, Default)]
pub struct ToolResult {
    /// Draw operations to apply
    pub ops: Vec<DrawOp>,
    /// Whether the tool has finished (e.g., after click-drag-release)
    pub finished: bool,
    /// Whether the grid was modified
    pub modified: bool,
}

impl ToolResult {
    /// Create a new empty tool result.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a single draw operation to the result.
    pub fn with_op(mut self, op: DrawOp) -> Self {
        self.ops.push(op);
        self.modified = true;
        self
    }

    /// Set all draw operations for the result.
    pub fn with_ops(mut self, ops: Vec<DrawOp>) -> Self {
        if !ops.is_empty() {
            self.modified = true;
        }
        self.ops = ops;
        self
    }

    /// Mark the result as finished.
    pub fn finish(mut self) -> Self {
        self.finished = true;
        self
    }
}

/// Context provided to tools during operations.
pub struct ToolContext {
    /// Current grid width
    pub grid_width: usize,
    /// Current grid height
    pub grid_height: usize,
    /// Current border style preference
    pub border_style: BorderStyle,
}

/// Tool trait - implemented by all drawing tools.
pub trait Tool {
    /// Get the tool ID.
    fn id(&self) -> ToolId;

    /// Handle pointer down event.
    fn on_pointer_down(&mut self, x: i32, y: i32, ctx: &ToolContext) -> ToolResult;

    /// Handle pointer move event.
    fn on_pointer_move(&mut self, x: i32, y: i32, ctx: &ToolContext) -> ToolResult;

    /// Handle pointer up event.
    fn on_pointer_up(&mut self, x: i32, y: i32, ctx: &ToolContext) -> ToolResult;

    /// Handle keyboard input (for text tool).
    fn on_key(&mut self, _ch: char, _ctx: &ToolContext) -> ToolResult {
        ToolResult::default()
    }

    /// Get selection (for select tool).
    fn get_selection(&self) -> Option<Selection> {
        None
    }

    /// Reset tool state.
    fn reset(&mut self);

    /// Check if tool is currently active (dragging/typing).
    fn is_active(&self) -> bool;
}

/// Helper to clamp coordinates to grid bounds.
#[inline]
pub fn clamp_to_grid(x: i32, y: i32, width: usize, height: usize) -> (i32, i32) {
    (
        x.clamp(0, (width as i32).saturating_sub(1)),
        y.clamp(0, (height as i32).saturating_sub(1)),
    )
}
