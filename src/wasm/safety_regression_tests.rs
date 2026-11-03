//! Editing-boundary regressions that do not require a JavaScript runtime.

use super::AsciiEditor;
use crate::core::tools::DrawOp;

#[test]
fn added_layer_redo_restores_live_payload_and_both_histories() {
    let mut editor = AsciiEditor::new(10, 4);
    editor.commit_ops(&[DrawOp::new(0, 0, 'A')]);
    editor.add_layer();
    editor.commit_ops(&[DrawOp::new(1, 0, 'B')]);
    editor.rename_layer(1, "Sketch".into());
    editor.set_active_layer(0);
    assert!(editor.undo()); // Remove the added layer from its creator's history.
    assert_eq!(editor.layer_count(), 1);
    assert!(editor.redo());
    assert_eq!(editor.layer_name(1), "Sketch");
    assert_eq!(editor.export_ascii(), "AB");
    assert_eq!(editor.undo_count(), 2);
    assert!(editor.undo()); // Rename on restored layer.
    assert_eq!(editor.layer_name(1), "Layer 2");
    assert!(editor.undo()); // Draw on restored layer.
    assert_eq!(editor.export_ascii(), "A");
    assert!(editor.redo());
    assert_eq!(editor.export_ascii(), "AB");
    editor.set_active_layer(0);
    assert!(editor.undo()); // Creation remains on base history.
    assert!(editor.undo()); // Base drawing remains too.
    assert_eq!(editor.export_ascii(), "");
    assert!(editor.redo());
    assert!(editor.redo());
    assert_eq!(editor.export_ascii(), "AB");
    assert_eq!(
        editor.redo_count(),
        1,
        "restored layer keeps its rename redo"
    );
}

fn start_text(editor: &mut AsciiEditor) {
    editor.set_tool("text".into());
    let ctx = editor.create_tool_context();
    editor.active_tool.on_pointer_down(0, 0, &ctx);
    let ops = editor.active_tool.on_key('A', &ctx).ops;
    editor.commit_ops(&ops);
}

#[test]
fn load_clear_switch_and_resize_reset_text_sessions() {
    for boundary in ["load", "clear", "switch", "resize"] {
        let mut editor = AsciiEditor::new(10, 4);
        editor.add_layer();
        start_text(&mut editor);
        match boundary {
            "load" => assert!(editor.load_document(AsciiEditor::new(10, 4).serialize_document())),
            "clear" => editor.clear(),
            "switch" => assert!(editor.set_active_layer(0)),
            "resize" => editor.resize(11, 4),
            _ => unreachable!(),
        }
        assert!(
            !editor.active_tool.is_active(),
            "stale text after {boundary}"
        );
        assert_eq!(editor.text_cursor_position(), None);
        let ctx = editor.create_tool_context();
        let ops = editor.active_tool.on_pointer_down(4, 0, &ctx).ops;
        assert!(ops.is_empty(), "buffer replay after {boundary}");
    }
}

#[test]
fn layer_creation_is_bounded_and_every_created_document_roundtrips() {
    let mut editor = AsciiEditor::new(10, 4);
    for _ in 0..40 {
        editor.add_layer();
    }
    assert_eq!(editor.layer_count(), 32);
    let saved = editor.serialize_document();
    let mut restored = AsciiEditor::new(1, 1);
    assert!(restored.load_document(saved));
    assert_eq!(restored.layer_count(), 32);
}

#[test]
fn constructor_and_resize_cannot_create_unloadable_dimensions() {
    let mut editor = AsciiEditor::new(401, 201);
    let mut restored = AsciiEditor::new(1, 1);
    assert!(restored.load_document(editor.serialize_document()));
    editor.resize(0, 0);
    assert!(restored.load_document(editor.serialize_document()));
    editor.resize(401, 201);
    assert!(restored.load_document(editor.serialize_document()));
}

#[test]
fn history_restoration_at_layer_limit_is_blocked_without_consuming_payload() {
    for remove in ["delete", "merge"] {
        let mut editor = AsciiEditor::new(10, 4);
        for _ in 1..32 {
            editor.add_layer();
        }
        editor.commit_ops(&[DrawOp::new(2, 0, 'X')]);
        editor.set_active_layer(0);
        if remove == "delete" {
            assert!(editor.delete_layer(31));
        } else {
            assert!(editor.merge_layer_down(31));
        }
        editor.set_active_layer(1);
        editor.add_layer(); // Fill the slot on a different timeline.
        editor.set_active_layer(0);
        let undo_count = editor.undo_count();
        assert!(!editor.undo(), "{remove} undo would exceed 32 layers");
        assert_eq!(editor.undo_count(), undo_count);
        assert_eq!(editor.layer_count(), 32);
        assert!(AsciiEditor::new(1, 1).load_document(editor.serialize_document()));
        editor.set_active_layer(1);
        editor.delete_layer(31);
        editor.set_active_layer(0);
        assert!(editor.undo()); // The blocked payload is still available.
        assert_eq!(editor.layer_count(), 32);
        assert_eq!(
            editor
                .layer_stack
                .get(31)
                .unwrap()
                .grid()
                .get(2, 0)
                .unwrap()
                .ch,
            'X'
        );
    }
}

#[test]
fn added_layer_redo_cannot_exceed_limit_after_other_timeline_fills_slot() {
    let mut editor = AsciiEditor::new(10, 4);
    for _ in 1..31 {
        editor.add_layer();
    }
    editor.set_active_layer(0);
    editor.add_layer();
    editor.set_active_layer(0);
    assert!(editor.undo());
    editor.set_active_layer(1);
    editor.add_layer();
    editor.set_active_layer(0);
    assert!(!editor.redo());
    assert_eq!(editor.redo_count(), 1);
    assert_eq!(editor.layer_count(), 32);
}

/// Issue #296: a select drag must preview before pointer-up, and the tool must
/// own the "new selection or move" decision the editor acts on.
#[test]
fn select_drag_previews_a_marquee_and_moves_committed_content() {
    let mut editor = AsciiEditor::new(10, 4);
    editor.commit_ops(&[DrawOp::new(1, 0, 'A')]);
    editor.set_tool("select".into());
    let ctx = editor.create_tool_context();

    let pressed = editor.active_tool.on_pointer_down(1, 0, &ctx);
    assert_eq!(pressed.ops.len(), 1, "no feedback on press");
    let dragged = editor.active_tool.on_pointer_move(2, 1, &ctx);
    assert!(!dragged.ops.is_empty(), "no marquee while dragging");
    editor.active_tool.on_pointer_up(2, 1, &ctx);
    editor.update_select_tool_selection();
    assert_eq!(
        editor.current_selection.as_ref().map(|s| s.bounds()),
        Some((1, 0, 2, 1))
    );

    editor.active_tool.on_pointer_down(1, 0, &ctx);
    editor.begin_select_gesture();
    assert!(
        editor.is_moving_selection,
        "click inside did not arm a move"
    );
    editor.active_tool.on_pointer_move(3, 0, &ctx);
    editor.update_select_tool_selection();
    assert!(!editor.generate_move_preview_ops().is_empty());
    editor.commit_selection_move();
    assert_eq!(editor.state.grid.get(3, 0).map(|c| c.ch), Some('A'));
    assert!(editor.state.grid.get(1, 0).unwrap().is_empty());
}

/// Issue #296: delete used to clear only the editor's selection, so the next
/// press inside the tool's invisible copy armed a move with nothing to move.
#[test]
fn delete_clears_the_tool_selection_so_the_next_press_is_not_a_ghost_move() {
    let mut editor = AsciiEditor::new(10, 4);
    editor.commit_ops(&[DrawOp::new(1, 0, 'A')]);
    editor.set_tool("select".into());
    let ctx = editor.create_tool_context();
    editor.active_tool.on_pointer_down(1, 0, &ctx);
    editor.active_tool.on_pointer_up(2, 1, &ctx);
    editor.update_select_tool_selection();

    assert!(editor.delete_selection_impl());
    assert!(editor.current_selection.is_none());

    let pressed = editor.active_tool.on_pointer_down(1, 0, &ctx);
    editor.begin_select_gesture();
    assert!(!editor.is_moving_selection, "ghost move after delete");
    assert!(!pressed.ops.is_empty(), "a new drag must still preview");
}

#[test]
fn invalid_load_does_not_reset_the_current_session() {
    let mut editor = AsciiEditor::new(10, 4);
    start_text(&mut editor);
    assert!(!editor.load_document("{}".into()));
    assert!(editor.active_tool.is_active());
    assert_eq!(editor.export_ascii(), "A");
}
