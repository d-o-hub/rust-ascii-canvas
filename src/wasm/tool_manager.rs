//! Tool management - tool creation, switching, and context.

use crate::core::tools::{
    ArrowTool, BorderStyle, DiamondTool, EraserTool, FreehandTool, LineDirection, LineTool,
    RectangleTool, SelectTool, TextTool, Tool, ToolId,
};
use crate::core::EditorState;
use std::str::FromStr;

pub(crate) fn parse_tool_id(s: &str) -> Option<ToolId> {
    match s.to_lowercase().as_str() {
        "rectangle" | "rect" | "r" => Some(ToolId::Rectangle),
        "line" | "l" => Some(ToolId::Line),
        "arrow" | "a" => Some(ToolId::Arrow),
        "diamond" | "d" => Some(ToolId::Diamond),
        "text" | "t" => Some(ToolId::Text),
        "freehand" | "f" => Some(ToolId::Freehand),
        "select" | "v" => Some(ToolId::Select),
        "eraser" | "e" => Some(ToolId::Eraser),
        _ => None,
    }
}

pub(crate) fn set_tool_by_id(
    active_tool: &mut Box<dyn Tool>,
    tool_id: &mut ToolId,
    preview_ops: &mut Vec<crate::core::tools::DrawOp>,
    state: &mut EditorState,
    current_selection: &mut Option<crate::core::selection::Selection>,
) {
    active_tool.reset();
    preview_ops.clear();

    if *tool_id != ToolId::Select {
        *current_selection = None;
    }

    state.tool = *tool_id;

    match *tool_id {
        ToolId::Rectangle => {
            *active_tool = Box::new(RectangleTool::new());
        }
        ToolId::Line => {
            *active_tool = Box::new(LineTool::new());
        }
        ToolId::Arrow => {
            *active_tool = Box::new(ArrowTool::new());
        }
        ToolId::Diamond => {
            *active_tool = Box::new(DiamondTool::new());
        }
        ToolId::Text => {
            *active_tool = Box::new(TextTool::new());
        }
        ToolId::Freehand => {
            *active_tool = Box::new(FreehandTool::new());
        }
        ToolId::Select => {
            *active_tool = Box::new(SelectTool::new());
        }
        ToolId::Eraser => {
            *active_tool = Box::new(EraserTool::new());
        }
    }
}

pub(crate) fn set_border_style(state: &mut EditorState, style: String) {
    let style = match style.to_lowercase().as_str() {
        "single" => BorderStyle::Single,
        "double" => BorderStyle::Double,
        "heavy" => BorderStyle::Heavy,
        "rounded" => BorderStyle::Rounded,
        "ascii" => BorderStyle::Ascii,
        "dotted" => BorderStyle::Dotted,
        _ => BorderStyle::Single,
    };
    state.border_style = style;
}

pub(crate) fn set_line_direction(
    tool_id: ToolId,
    active_tool: &mut Box<dyn Tool>,
    direction: String,
) {
    if tool_id == ToolId::Line {
        let dir = LineDirection::from_str(&direction).unwrap_or_default();
        if let Some(line_tool) = active_tool.as_any_mut().downcast_mut::<LineTool>() {
            line_tool.set_direction(dir);
        }
    }
}
