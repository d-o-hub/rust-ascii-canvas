//! Editor WASM tests.

#![cfg(target_arch = "wasm32")]

use wasm_bindgen_test::*;
use ascii_canvas::wasm::AsciiEditor;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_editor_creation() {
    let editor = AsciiEditor::new(80, 40);
    
    assert_eq!(editor.width(), 80);
    assert_eq!(editor.height(), 40);
}

#[wasm_bindgen_test]
fn test_editor_tool() {
    let mut editor = AsciiEditor::new(80, 40);
    
    assert_eq!(editor.tool(), "Rectangle");
    
    editor.set_tool("Line".to_string());
    assert_eq!(editor.tool(), "Line");
}

#[wasm_bindgen_test]
fn test_editor_tool_shortcut() {
    let mut editor = AsciiEditor::new(80, 40);
    
    let result = editor.set_tool_by_shortcut('L');
    assert!(result);
    assert_eq!(editor.tool(), "Line");
    
    let result = editor.set_tool_by_shortcut('x');
    assert!(!result);
}

#[wasm_bindgen_test]
fn test_editor_undo_redo() {
    let editor = AsciiEditor::new(80, 40);
    
    assert!(!editor.can_undo());
    assert!(!editor.can_redo());
}

#[wasm_bindgen_test]
fn test_editor_export() {
    let editor = AsciiEditor::new(80, 40);
    let ascii = editor.export_ascii();
    
    // Empty grid should export to empty string
    assert!(ascii.is_empty());
}

#[wasm_bindgen_test]
fn test_editor_zoom() {
    let mut editor = AsciiEditor::new(80, 40);
    
    assert_eq!(editor.zoom(), 1.0);
    
    editor.set_zoom(2.0);
    assert_eq!(editor.zoom(), 2.0);
    
    // Zoom should be clamped
    editor.set_zoom(10.0);
    assert_eq!(editor.zoom(), 4.0);
    
    editor.set_zoom(0.1);
    assert_eq!(editor.zoom(), 0.25);
}

#[wasm_bindgen_test]
fn test_editor_pan() {
    let mut editor = AsciiEditor::new(80, 40);
    
    editor.set_pan(100.0, 50.0);
    let pan = editor.get_pan();
    
    assert_eq!(pan[0], 100.0);
    assert_eq!(pan[1], 50.0);
}

#[wasm_bindgen_test]
fn test_editor_clear() {
    let mut editor = AsciiEditor::new(80, 40);
    
    // Clear should not panic
    editor.clear();
    
    assert!(!editor.can_undo());
    assert!(!editor.can_redo());
}

#[wasm_bindgen_test]
fn test_editor_needs_redraw() {
    let mut editor = AsciiEditor::new(80, 40);
    
    // Initially needs redraw after creation
    assert!(editor.needs_redraw());
    
    // Get render commands clears redraw flag
    let _ = editor.get_render_commands();
    
    // After requesting redraw
    editor.request_redraw();
    assert!(editor.needs_redraw());
}
