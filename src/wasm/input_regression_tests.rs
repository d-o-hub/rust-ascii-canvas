//! Binding-level event regressions: executed by wasm-bindgen-test in Node.

use super::AsciiEditor;
use wasm_bindgen_test::wasm_bindgen_test;

fn editor(tool: &str, width: usize) -> AsciiEditor {
    let mut editor = AsciiEditor::new(width, 4);
    editor.set_font_metrics(8.0, 20.0, 16.0);
    editor.set_tool(tool.into());
    editor
}

fn key(editor: &mut AsciiEditor, key: &str) {
    editor.on_key_down(key.into(), false, false);
}

#[wasm_bindgen_test]
fn text_accepts_space_and_printable_unicode_scalars() {
    let mut editor = editor("text", 20);
    editor.on_pointer_down(1.0, 1.0);
    for input in ["A", " ", "B", "é", "界", "😀"] {
        key(&mut editor, input);
    }
    assert_eq!(editor.export_ascii(), "A Bé界😀");
    assert!(!editor.space_held);
}

#[wasm_bindgen_test]
fn unknown_keys_and_control_chords_never_delete_or_type() {
    let mut editor = editor("text", 20);
    editor.state.grid.set_char(0, 0, 'X');
    editor.on_pointer_down(1.0, 1.0);
    let before = editor.serialize_document();
    for input in [
        "ArrowRight",
        "Shift",
        "Dead",
        "F1",
        "Unidentified",
        "\0",
        "\u{1b}",
        "",
    ] {
        key(&mut editor, input);
        assert_eq!(
            editor.serialize_document(),
            before,
            "destructive key {input:?}"
        );
    }
    editor.on_key_down("s".into(), true, false);
    assert_eq!(editor.serialize_document(), before);
    assert_eq!(editor.undo_count(), 0);
    key(&mut editor, "Delete");
    assert_eq!(editor.export_ascii(), "");
    key(&mut editor, "A");
    key(&mut editor, "Backspace");
    assert_eq!(editor.export_ascii(), "");
}

fn stroke(editor: &mut AsciiEditor, y: f64, cells: usize) {
    editor.on_pointer_down(1.0, y);
    for x in 1..cells {
        editor.on_pointer_move(x as f64 * 8.0 + 1.0, y);
    }
    editor.on_pointer_up((cells - 1) as f64 * 8.0 + 1.0, y);
}

#[wasm_bindgen_test]
fn long_freehand_and_eraser_gestures_are_single_exact_transactions() {
    for tool in ["freehand", "eraser"] {
        let mut editor = editor(tool, 320);
        for x in 0..300 {
            editor.state.grid.set_char(x, 0, 'X');
            // Both gestures must change cells, including the second eraser pass.
            editor.state.grid.set_char(x, 1, 'X');
        }
        let before = editor.serialize_document();
        stroke(&mut editor, 1.0, 300);
        let after = editor.serialize_document();
        assert_ne!(after, before);
        assert_eq!(
            editor.undo_count(),
            1,
            "{tool} must consume one history entry"
        );
        assert!(editor.undo());
        assert_eq!(
            editor.serialize_document(),
            before,
            "{tool} undo restores originals"
        );
        assert!(!editor.undo());
        assert!(editor.redo());
        assert_eq!(editor.serialize_document(), after);
        stroke(&mut editor, 21.0, 300);
        assert_eq!(editor.undo_count(), 2, "independent gestures cannot merge");
        assert!(editor.undo());
        assert_eq!(editor.serialize_document(), after);
    }
}

#[wasm_bindgen_test]
fn escape_rolls_back_an_unfinished_stroke_without_history() {
    let mut editor = editor("freehand", 20);
    editor.on_pointer_down(1.0, 1.0);
    editor.on_pointer_move(41.0, 1.0);
    key(&mut editor, "Escape");
    editor.on_pointer_up(41.0, 1.0);
    assert_eq!(editor.export_ascii(), "");
    assert_eq!(editor.undo_count(), 0);
}

#[wasm_bindgen_test]
fn stroke_cancellation_cannot_leak_to_another_layer_or_tool() {
    for boundary in ["switch", "tool", "lock", "resize", "clear", "cancel"] {
        let mut editor = editor("freehand", 20);
        editor.add_layer();
        editor.on_pointer_down(1.0, 1.0);
        editor.on_pointer_move(41.0, 1.0);
        match boundary {
            "switch" => {
                editor.set_active_layer(0);
            }
            "tool" => editor.set_tool("rectangle".into()),
            "lock" => editor.set_layer_locked(1, true),
            "resize" => editor.resize(21, 4),
            "clear" => editor.clear(),
            "cancel" => editor.on_pointer_cancel(),
            _ => unreachable!(),
        }
        editor.on_pointer_up(41.0, 1.0);
        assert_eq!(
            editor.export_ascii(),
            "",
            "unfinished stroke after {boundary}"
        );
        editor.set_active_layer(1);
        assert_eq!(
            editor.export_ascii(),
            "",
            "old layer retained stroke after {boundary}"
        );
    }
}

#[wasm_bindgen_test]
fn stroke_pointer_up_includes_the_final_segment_and_duplicate_up_is_harmless() {
    for tool in ["freehand", "eraser"] {
        let mut editor = editor(tool, 20);
        for x in 0..8 {
            editor.state.grid.set_char(x, 0, 'X');
        }
        let before = editor.serialize_document();
        editor.on_pointer_down(1.0, 1.0);
        editor.on_pointer_up(57.0, 1.0); // No intervening move event.
        let after = editor.serialize_document();
        assert_eq!(editor.undo_count(), 1);
        if tool == "eraser" {
            assert_eq!(editor.export_ascii(), "");
        } else {
            assert_eq!(editor.export_ascii().chars().count(), 8);
            assert!(!editor.export_ascii().contains('X'));
        }
        editor.on_pointer_up(57.0, 1.0);
        assert_eq!(editor.undo_count(), 1);
        assert!(editor.undo());
        assert_eq!(editor.serialize_document(), before);
        assert!(editor.redo());
        assert_eq!(editor.serialize_document(), after);
    }
}

#[wasm_bindgen_test]
fn pan_shortcuts_selection_delete_and_ctrl_history_still_work() {
    let mut editor = editor("rectangle", 20);
    key(&mut editor, " ");
    editor.on_pointer_down(1.0, 1.0);
    editor.on_pointer_move(9.0, 21.0);
    editor.on_pointer_up(9.0, 21.0);
    assert_eq!(editor.get_pan(), vec![8.0, 20.0]);
    editor.on_key_up(" ".into());
    editor.set_pan(0.0, 0.0);
    key(&mut editor, "t");
    editor.on_pointer_down(1.0, 1.0);
    key(&mut editor, "A");
    key(&mut editor, "Enter");
    key(&mut editor, "B");
    assert_eq!(editor.export_ascii(), "A\nB");
    editor.on_key_down("a".into(), true, false);
    key(&mut editor, "Delete");
    assert_eq!(editor.export_ascii(), "");
    editor.on_key_down("z".into(), true, false);
    assert_eq!(editor.export_ascii(), "A\nB");
    editor.on_key_down("z".into(), true, true);
    assert_eq!(editor.export_ascii(), "");
}

#[wasm_bindgen_test]
fn paste_during_stroke_is_preserved_through_cancel_completion_and_history() {
    for tool in ["freehand", "eraser"] {
        for internal in [false, true] {
            for cancel in [false, true] {
                let mut editor = editor(tool, 20);
                editor.state.grid.set_char(0, 0, 'P');
                assert!(editor.copy_selection());
                editor.state.grid.set_char(0, 0, ' ');
                editor.state.grid.set_char(3, 1, 'X');
                let before = editor.serialize_document();
                editor.on_pointer_down(25.0, 21.0);
                let accepted = if internal {
                    editor.paste()
                } else {
                    editor.paste_text("P".into())
                };
                assert!(accepted);
                assert!(
                    editor.stroke.is_none(),
                    "paste must close {tool} transaction"
                );
                assert_eq!(editor.state.grid.get(3, 1).unwrap().ch, 'P');
                let after = editor.serialize_document();
                assert_ne!(after, before);
                if cancel {
                    editor.on_pointer_cancel();
                } else {
                    editor.on_pointer_move(49.0, 21.0);
                    editor.on_pointer_up(49.0, 21.0);
                }
                assert_eq!(editor.serialize_document(), after);
                assert_eq!(editor.undo_count(), 1);
                assert!(editor.undo());
                assert_eq!(editor.serialize_document(), before);
                assert!(!editor.undo());
                assert!(editor.redo());
                assert_eq!(editor.serialize_document(), after);
                let mut restored = AsciiEditor::new(20, 4);
                assert!(restored.load_document(after));
                assert_eq!(restored.state.grid.get(3, 1).unwrap().ch, 'P');
                assert_eq!(editor.export_pixel_buffer(), restored.export_pixel_buffer());
            }
        }
    }
}

#[wasm_bindgen_test]
fn unchanged_strokes_do_not_consume_undo_or_discard_redo() {
    for tool in ["freehand", "eraser"] {
        let mut editor = editor(tool, 20);
        stroke(&mut editor, 1.0, 2);
        if tool == "freehand" {
            assert_eq!(editor.undo_count(), 1);
            stroke(&mut editor, 1.0, 2);
            assert_eq!(editor.undo_count(), 1);
            assert!(editor.undo());
            editor.set_tool("eraser".into());
        }
        let undo = editor.undo_count();
        let redo = editor.redo_count();
        stroke(&mut editor, 1.0, 2);
        assert_eq!(editor.undo_count(), undo);
        assert_eq!(editor.redo_count(), redo);
    }
}

#[wasm_bindgen_test]
fn rejected_paste_does_not_cancel_an_unfinished_stroke() {
    let mut editor = editor("freehand", 20);
    editor.on_pointer_down(1.0, 1.0);
    assert!(!editor.paste_text(" \t\n".into()));
    assert!(!editor.paste());
    assert!(editor.stroke.is_some());
    editor.on_pointer_up(9.0, 1.0);
    assert_eq!(editor.undo_count(), 1);
    assert_eq!(editor.export_ascii(), "──");
}

#[wasm_bindgen_test]
fn unfinished_shape_cannot_cross_document_or_layer_boundaries() {
    for boundary in ["load", "clear", "switch", "resize"] {
        let mut editor = editor("rectangle", 20);
        editor.add_layer();
        editor.on_pointer_down(1.0, 1.0);
        editor.on_pointer_move(41.0, 41.0);
        match boundary {
            "load" => assert!(editor.load_document(AsciiEditor::new(20, 4).serialize_document())),
            "clear" => editor.clear(),
            "switch" => assert!(editor.set_active_layer(0)),
            "resize" => editor.resize(21, 4),
            _ => unreachable!(),
        }
        editor.on_pointer_up(41.0, 41.0);
        assert_eq!(editor.export_ascii(), "", "stale shape after {boundary}");
    }
}
