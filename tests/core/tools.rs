//! Tools tests.

use ascii_canvas::core::tools::{
    ArrowTool, BorderStyle, DiamondTool, EraserTool, FreehandTool, LineTool, RectangleTool,
    SelectTool, TextTool, Tool, ToolContext, ToolId,
};

fn create_context() -> ToolContext {
    ToolContext {
        grid_width: 80,
        grid_height: 40,
        border_style: BorderStyle::Single,
    }
}

#[test]
fn test_tool_id_shortcuts() {
    assert_eq!(ToolId::from_shortcut('R'), Some(ToolId::Rectangle));
    assert_eq!(ToolId::from_shortcut('r'), Some(ToolId::Rectangle));
    assert_eq!(ToolId::from_shortcut('L'), Some(ToolId::Line));
    assert_eq!(ToolId::from_shortcut('A'), Some(ToolId::Arrow));
    assert_eq!(ToolId::from_shortcut('D'), Some(ToolId::Diamond));
    assert_eq!(ToolId::from_shortcut('T'), Some(ToolId::Text));
    assert_eq!(ToolId::from_shortcut('F'), Some(ToolId::Freehand));
    assert_eq!(ToolId::from_shortcut('V'), Some(ToolId::Select));
    assert_eq!(ToolId::from_shortcut('E'), Some(ToolId::Eraser));
    assert_eq!(ToolId::from_shortcut('X'), None);
}

#[test]
fn test_tool_names() {
    assert_eq!(ToolId::Rectangle.name(), "Rectangle");
    assert_eq!(ToolId::Line.name(), "Line");
    assert_eq!(ToolId::Arrow.name(), "Arrow");
}

#[test]
fn test_border_styles() {
    let single = BorderStyle::Single;
    assert_eq!(single.corners(), ['┌', '┐', '└', '┘']);
    assert_eq!(single.horizontal(), '─');
    assert_eq!(single.vertical(), '│');

    let double = BorderStyle::Double;
    assert_eq!(double.corners(), ['╔', '╗', '╚', '╝']);
    assert_eq!(double.horizontal(), '═');
    assert_eq!(double.vertical(), '║');
}

#[test]
fn test_rectangle_tool() {
    let mut tool = RectangleTool::new();
    let ctx = create_context();

    assert_eq!(tool.id(), ToolId::Rectangle);
    assert!(!tool.is_active());

    tool.on_pointer_down(0, 0, &ctx);
    assert!(tool.is_active());

    let result = tool.on_pointer_up(5, 3, &ctx);
    assert!(result.finished);
    assert!(result.modified);
    assert!(!result.ops.is_empty());

    assert!(!tool.is_active());
}

#[test]
fn test_line_tool() {
    let mut tool = LineTool::new();
    let ctx = create_context();

    assert_eq!(tool.id(), ToolId::Line);

    tool.on_pointer_down(0, 0, &ctx);
    let result = tool.on_pointer_up(5, 0, &ctx);

    assert!(result.finished);
    // Should have 6 points (0-5)
    assert_eq!(result.ops.len(), 6);
}

#[test]
fn test_arrow_tool() {
    let mut tool = ArrowTool::new();
    let ctx = create_context();

    assert_eq!(tool.id(), ToolId::Arrow);

    tool.on_pointer_down(0, 0, &ctx);
    let result = tool.on_pointer_up(5, 0, &ctx);

    assert!(result.finished);
    // Last op should be the arrowhead
    let last = result.ops.last().unwrap();
    assert_eq!(last.x, 5);
}

#[test]
fn test_diamond_tool() {
    let mut tool = DiamondTool::new();
    let ctx = create_context();

    assert_eq!(tool.id(), ToolId::Diamond);

    // Single point diamond
    tool.on_pointer_down(10, 10, &ctx);
    let result = tool.on_pointer_up(10, 10, &ctx);

    assert!(result.finished);
    assert_eq!(result.ops.len(), 1);
    assert_eq!(result.ops[0].cell.ch, '◆');
}

#[test]
fn test_text_tool() {
    let mut tool = TextTool::new();
    let ctx = create_context();

    assert_eq!(tool.id(), ToolId::Text);
    assert!(!tool.is_active());

    tool.on_pointer_down(5, 5, &ctx);
    assert!(tool.is_active());

    let result = tool.on_key('H', &ctx);
    assert!(result.modified);
    assert_eq!(result.ops.len(), 1);

    tool.reset();
    assert!(!tool.is_active());
}

#[test]
fn test_freehand_tool() {
    let mut tool = FreehandTool::new();
    let ctx = create_context();

    assert_eq!(tool.id(), ToolId::Freehand);
    assert!(!tool.is_active());

    tool.on_pointer_down(10, 10, &ctx);
    assert!(tool.is_active());

    let result = tool.on_pointer_up(10, 10, &ctx);
    assert!(result.finished);

    tool.reset();
    assert!(!tool.is_active());
}

#[test]
fn test_select_tool() {
    let mut tool = SelectTool::new();
    let ctx = create_context();

    assert_eq!(tool.id(), ToolId::Select);

    tool.on_pointer_down(5, 5, &ctx);
    tool.on_pointer_move(10, 10, &ctx);
    tool.on_pointer_up(10, 10, &ctx);

    let selection = tool.get_selection();
    assert!(selection.is_some());

    tool.clear_selection();
    assert!(tool.get_selection().is_none());
}

#[test]
fn test_eraser_tool() {
    let mut tool = EraserTool::new();
    let ctx = create_context();

    assert_eq!(tool.id(), ToolId::Eraser);
    assert!(!tool.is_active());

    tool.on_pointer_down(10, 10, &ctx);
    assert!(tool.is_active());

    let result = tool.on_pointer_up(10, 10, &ctx);
    assert!(result.finished);

    // Should have at least one clear operation
    assert!(!result.ops.is_empty());
    assert!(result.ops[0].cell.is_empty());
}
