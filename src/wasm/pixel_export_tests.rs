//! Clean export contract: same artwork, never editor overlays or state changes.

use super::AsciiEditor;
use crate::core::tools::DrawOp;

#[test]
fn clean_pixels_match_canvas_theme_atlas_and_visible_composite() {
    let mut editor = AsciiEditor::new(4, 2);
    editor.set_theme("Light".into());
    editor.update_font_atlas_glyph('A' as u32, vec![255; 160]);
    editor.commit_ops(&[DrawOp::new(0, 0, 'A')]);
    editor.add_layer();
    editor.commit_ops(&[DrawOp::new(1, 0, 'B')]);
    editor.add_layer();
    editor.commit_ops(&[DrawOp::new(0, 0, 'X')]);
    editor.set_layer_visible(2, false);
    editor.render_to_pixel_buffer();
    let expected = editor.pixel_buffer.clone();
    assert_eq!(editor.export_pixel_buffer(), expected);
    assert_eq!(expected.len(), 4 * 8 * 2 * 20 * 4);

    editor.set_selection_for_test(0, 0, 3, 1);
    editor.preview_ops = vec![DrawOp::new(2, 0, '#')];
    editor.request_redraw();
    editor.render_to_pixel_buffer();
    assert_ne!(editor.pixel_buffer, expected, "fixture must have overlays");
    let display_pixels = editor.pixel_buffer.clone();
    let document = editor.serialize_document();
    let metrics = (editor.full_render_count(), editor.dirty_render_count());
    let history = (editor.undo_count(), editor.redo_count());
    let selection = editor.current_selection.as_ref().map(|s| s.bounds());
    let needs_redraw = editor.needs_redraw();
    assert_eq!(editor.export_pixel_buffer(), expected);
    assert_eq!(editor.pixel_buffer, display_pixels);
    assert_eq!(editor.serialize_document(), document);
    assert_eq!(
        (editor.full_render_count(), editor.dirty_render_count()),
        metrics
    );
    assert_eq!((editor.undo_count(), editor.redo_count()), history);
    assert_eq!(
        editor.current_selection.as_ref().map(|s| s.bounds()),
        selection
    );
    assert_eq!(editor.preview_ops.len(), 1);
    assert_eq!(editor.needs_redraw(), needs_redraw);
}

#[test]
fn export_excludes_live_unfinished_stroke_without_cancelling_it() {
    let mut editor = AsciiEditor::new(4, 2);
    editor.commit_ops(&[DrawOp::new(0, 0, 'A')]);
    let expected = editor.export_pixel_buffer();
    editor.set_tool("freehand".into());
    editor.begin_stroke();
    editor.extend_stroke(&[DrawOp::new(0, 0, 'X'), DrawOp::new(1, 0, '#')]);
    assert_eq!(editor.export_pixel_buffer(), expected);
    assert_eq!(editor.state.grid.get(0, 0).unwrap().ch, 'X');
    assert!(editor.stroke.is_some());
    editor.finish_stroke();
    assert_ne!(editor.export_pixel_buffer(), expected);
}
