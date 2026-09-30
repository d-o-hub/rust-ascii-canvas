#[cfg(test)]
mod clipboard_tests {
    use crate::core::tools::DrawOp;
    use crate::wasm::bindings::AsciiEditor;

    fn make_canvas_with_box() -> AsciiEditor {
        let mut canvas = AsciiEditor::new(10, 10);
        // ┌───┐
        // │   │
        // └───┘
        canvas.state.grid.set_char(0, 0, '┌');
        canvas.state.grid.set_char(1, 0, '─');
        canvas.state.grid.set_char(2, 0, '─');
        canvas.state.grid.set_char(3, 0, '─');
        canvas.state.grid.set_char(4, 0, '┐');
        canvas.state.grid.set_char(0, 1, '│');
        canvas.state.grid.set_char(4, 1, '│');
        canvas.state.grid.set_char(0, 2, '└');
        canvas.state.grid.set_char(1, 2, '─');
        canvas.state.grid.set_char(2, 2, '─');
        canvas.state.grid.set_char(3, 2, '─');
        canvas.state.grid.set_char(4, 2, '┘');
        canvas
    }

    #[test]
    fn test_export_for_copy_preserves_box() {
        let canvas = make_canvas_with_box();
        let ascii = canvas.export_for_copy();
        let lines: Vec<&str> = ascii.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[0].ends_with('┐'));
        assert!(lines[1].ends_with('│'));
        assert!(lines[2].ends_with('┘'));
    }

    #[test]
    fn test_export_svg() {
        let canvas = make_canvas_with_box();
        let svg = canvas.export_svg();
        assert!(svg.starts_with("<svg"));
        assert!(svg.contains("<rect"));
        assert!(svg.contains("<g fill="));
        assert!(svg.contains("dominant-baseline=\"hanging\""));
        assert!(svg.contains("┌"));
        assert!(svg.contains("┐"));
        assert!(svg.contains("┘"));
        assert!(svg.contains("└"));
        assert!(svg.ends_with("</g></svg>"));
    }

    #[test]
    fn test_copy_and_paste_at_selection_origin() {
        let mut canvas = make_canvas_with_box();
        canvas.set_selection_for_test(0, 0, 4, 2);
        assert!(canvas.copy_selection_impl());
        // Move selection to paste origin
        canvas.set_selection_for_test(5, 5, 5, 5);
        assert!(canvas.paste_impl());
        assert_eq!(canvas.state.grid.get(5, 5).map(|c| c.ch), Some('┌'));
        assert_eq!(canvas.state.grid.get(9, 5).map(|c| c.ch), Some('┐'));
        // Origin box still present
        assert_eq!(canvas.state.grid.get(0, 0).map(|c| c.ch), Some('┌'));
    }

    #[test]
    fn test_paste_does_not_clobber_with_spaces() {
        let mut canvas = AsciiEditor::new(10, 10);
        canvas.state.grid.set_char(5, 5, 'Z');
        canvas.state.grid.set_char(0, 0, 'A');
        canvas.set_selection_for_test(0, 0, 0, 0);
        assert!(canvas.copy_selection_impl());
        canvas.set_selection_for_test(5, 5, 5, 5);
        assert!(canvas.paste_impl());
        // Only A is in clipboard; A overwrites Z at paste target
        assert_eq!(canvas.state.grid.get(5, 5).map(|c| c.ch), Some('A'));
    }

    #[test]
    fn test_serialize_load_round_trip() {
        let canvas = make_canvas_with_box();
        let json = canvas.serialize_document_impl();
        let mut other = AsciiEditor::new(10, 10);
        assert!(other.load_document_impl(&json));
        assert_eq!(other.export_for_copy(), canvas.export_for_copy());
    }

    #[test]
    fn test_add_layer() {
        let mut canvas = AsciiEditor::new(8, 8);
        let idx = canvas.add_layer_impl();
        assert_eq!(idx, 1);
        assert_eq!(canvas.layer_stack.len(), 2);
    }

    #[test]
    fn test_load_document_rejects_oversized_canvas() {
        let mut canvas = AsciiEditor::new(10, 10);
        let json = r#"{
            "format":"ascii-canvas",
            "version":1,
            "canvas":{"width":50000,"height":50000},
            "active_layer":0,
            "layers":[{"name":"Layer 1","visible":true,"cells":[]}]
        }"#;
        assert!(!canvas.load_document_impl(json));
        // Original canvas unchanged
        assert_eq!(canvas.state.grid.width(), 10);
        assert_eq!(canvas.state.grid.height(), 10);
    }

    #[test]
    fn test_load_document_rejects_too_many_layers() {
        let mut canvas = AsciiEditor::new(10, 10);
        let layers: String = (0..40)
            .map(|i| format!(r#"{{"name":"L{i}","visible":true,"cells":[]}}"#))
            .collect::<Vec<_>>()
            .join(",");
        let json = format!(
            r#"{{"format":"ascii-canvas","version":1,"canvas":{{"width":10,"height":10}},"active_layer":0,"layers":[{layers}]}}"#
        );
        assert!(!canvas.load_document_impl(&json));
    }

    #[test]
    fn test_export_and_copy_use_composite_layers() {
        let mut canvas = AsciiEditor::new(10, 10);
        // Layer 0: 'A' at (0,0)
        canvas.state.grid.set_char(0, 0, 'A');
        // Sync layer 0 snapshot then add layer 1 with 'B' at (1,0)
        let idx = canvas.add_layer_impl();
        assert_eq!(idx, 1);
        canvas.state.grid.set_char(1, 0, 'B');
        // Both layers visible — export/copy must include A and B
        let ascii = canvas.export_for_copy();
        assert!(
            ascii.contains('A'),
            "composite export missing layer 0: {ascii:?}"
        );
        assert!(
            ascii.contains('B'),
            "composite export missing layer 1: {ascii:?}"
        );

        assert!(canvas.copy_selection_impl());
        // Paste onto a clean area and verify composite was captured
        canvas.set_selection_for_test(5, 5, 5, 5);
        assert!(canvas.paste_impl());
        assert_eq!(canvas.state.grid.get(5, 5).map(|c| c.ch), Some('A'));
        assert_eq!(canvas.state.grid.get(6, 5).map(|c| c.ch), Some('B'));
    }

    #[test]
    fn test_selection_export_uses_composite() {
        let mut canvas = AsciiEditor::new(10, 10);
        canvas.state.grid.set_char(0, 0, 'A');
        let _ = canvas.add_layer_impl();
        canvas.state.grid.set_char(1, 0, 'B');
        // Active layer is 1; selection covers both cells
        canvas.set_selection_for_test(0, 0, 1, 0);
        let ascii = canvas.export_for_copy();
        assert!(
            ascii.contains('A') && ascii.contains('B'),
            "selection export must composite layers: {ascii:?}"
        );
    }

    #[test]
    fn test_layer_lock_prevents_draw_ops() {
        let mut canvas = AsciiEditor::new(10, 10);
        // Pre-populate a cell
        use crate::core::tools::DrawOp;
        canvas.commit_ops(&[DrawOp::new(1, 1, 'A')]);
        assert_eq!(canvas.state.grid.get(1, 1).unwrap().ch, 'A');

        // Lock the layer
        canvas.set_layer_locked(0, true);
        assert!(canvas.layer_locked(0));

        // Try to write a character (e.g. via commit_ops) while locked
        canvas.commit_ops(&[DrawOp::new(2, 2, 'X')]);
        // Cell (2, 2) should remain empty
        assert_eq!(canvas.state.grid.get(2, 2).unwrap().ch, ' ');

        // Try to clear while locked
        canvas.clear();
        // Cell (1, 1) should STILL be 'A' because clear was blocked by lock
        assert_eq!(canvas.state.grid.get(1, 1).unwrap().ch, 'A');

        // Unlock the layer
        canvas.set_layer_locked(0, false);
        assert!(!canvas.layer_locked(0));

        // Try clearing now that it is unlocked
        canvas.clear();
        // Cell should now be cleared
        assert!(!canvas.state.grid.get(1, 1).unwrap().is_visible());
    }

    #[test]
    fn test_reorder_layers_preserves_indices() {
        let mut canvas = AsciiEditor::new(10, 10);
        canvas.state.grid.set_char(0, 0, 'A');

        let _idx1 = canvas.add_layer_impl();
        canvas.state.grid.set_char(1, 1, 'B');

        let _idx2 = canvas.add_layer_impl();
        canvas.state.grid.set_char(2, 2, 'C');

        assert_eq!(canvas.layer_stack.len(), 3);
        assert_eq!(canvas.layer_stack.active_index(), 2);

        // Move active layer (2) down to index 1
        canvas.move_layer(2, 1);
        assert_eq!(canvas.layer_stack.active_index(), 1);
        assert_eq!(canvas.layer_stack.layers()[1].name(), "Layer 3"); // C should now be at index 1
        assert_eq!(canvas.layer_stack.layers()[2].name(), "Layer 2"); // B should now be at index 2
    }

    #[test]
    fn test_delete_layer_prevents_deleting_last_layer() {
        let mut canvas = AsciiEditor::new(10, 10);
        assert_eq!(canvas.layer_stack.len(), 1);

        // Try deleting layer 0 (the only layer)
        assert!(!canvas.delete_layer(0));
        assert_eq!(canvas.layer_stack.len(), 1);

        // Add a layer and delete it
        canvas.add_layer_impl();
        assert_eq!(canvas.layer_stack.len(), 2);
        assert!(canvas.delete_layer(1));
        assert_eq!(canvas.layer_stack.len(), 1);
        assert_eq!(canvas.layer_stack.active_index(), 0);
    }

    #[test]
    fn test_merge_layer_down_composites_cells() {
        let mut canvas = AsciiEditor::new(10, 10);
        canvas.state.grid.set_char(0, 0, 'A');

        canvas.add_layer_impl();
        canvas.state.grid.set_char(1, 1, 'B');

        // Merge layer 1 down to layer 0
        assert!(canvas.merge_layer_down(1));
        assert_eq!(canvas.layer_stack.len(), 1);
        assert_eq!(canvas.layer_stack.active_index(), 0);

        // Cell 'A' from bottom and 'B' from top should now both be in the bottom grid
        assert_eq!(canvas.state.grid.get(0, 0).unwrap().ch, 'A');
        assert_eq!(canvas.state.grid.get(1, 1).unwrap().ch, 'B');
    }

    #[test]
    fn test_layer_history_preservation_across_switches() {
        // Two layers, loaded rather than added, so neither history is seeded with
        // a structural entry and the undo order stays purely about drawing.
        let json = r#"{"format":"ascii-canvas","version":1,"canvas":{"width":10,"height":10},
"active_layer":0,"layers":[{"name":"Base","visible":true,"locked":false,
"cells":[{"x":0,"y":0,"ch":"A"}]},
{"name":"Sketch","visible":true,"locked":false,
"cells":[{"x":1,"y":1,"ch":"B"}]}]}"#;
        let mut canvas = AsciiEditor::new(10, 10);
        assert!(canvas.load_document_impl(json));

        // Layer 0 draws 'C'; layer 1 already holds 'B' and gets 'D'.
        canvas.commit_ops(&[DrawOp::new(2, 2, 'C')]);
        assert_eq!(canvas.state.grid.get(2, 2).unwrap().ch, 'C');
        assert!(canvas.set_active_layer(1));
        assert_eq!(canvas.state.grid.get(1, 1).unwrap().ch, 'B');
        canvas.commit_ops(&[DrawOp::new(3, 3, 'D')]);
        assert_eq!(canvas.state.grid.get(3, 3).unwrap().ch, 'D');

        // Undo on layer 1 touches only layer 1: 'D' goes, the loaded 'B' stays
        // (it came from the document, not from a draw, so it has no history entry).
        assert!(canvas.undo());
        assert!(canvas.state.grid.get(3, 3).unwrap().is_empty());
        assert_eq!(canvas.state.grid.get(1, 1).unwrap().ch, 'B');
        assert!(!canvas.undo(), "layer 1 has nothing left");

        // Layer 0's own history is untouched: 'A' and 'C' are still there.
        assert!(canvas.set_active_layer(0));
        assert_eq!(canvas.state.grid.get(0, 0).unwrap().ch, 'A');
        assert_eq!(canvas.state.grid.get(2, 2).unwrap().ch, 'C');
        assert!(canvas.undo());
        assert!(canvas.state.grid.get(2, 2).unwrap().is_empty());
        assert_eq!(canvas.state.grid.get(0, 0).unwrap().ch, 'A');
    }

    #[test]
    fn test_paste_text_basic() {
        let mut canvas = AsciiEditor::new(10, 10);
        // Paste plain text at default origin (0, 0)
        assert!(canvas.paste_text_impl("HELLO"));
        assert_eq!(canvas.state.grid.get(0, 0).unwrap().ch, 'H');
        assert_eq!(canvas.state.grid.get(1, 0).unwrap().ch, 'E');
        assert_eq!(canvas.state.grid.get(2, 0).unwrap().ch, 'L');
        assert_eq!(canvas.state.grid.get(3, 0).unwrap().ch, 'L');
        assert_eq!(canvas.state.grid.get(4, 0).unwrap().ch, 'O');
    }

    #[test]
    fn test_paste_text_multiline_normalization() {
        let mut canvas = AsciiEditor::new(10, 10);
        // Paste multiline text with mixed LF/CRLF
        let content = "AB\nCD\r\nEF";
        assert!(canvas.paste_text_impl(content));

        // Row 0
        assert_eq!(canvas.state.grid.get(0, 0).unwrap().ch, 'A');
        assert_eq!(canvas.state.grid.get(1, 0).unwrap().ch, 'B');

        // Row 1
        assert_eq!(canvas.state.grid.get(0, 1).unwrap().ch, 'C');
        assert_eq!(canvas.state.grid.get(1, 1).unwrap().ch, 'D');

        // Row 2
        assert_eq!(canvas.state.grid.get(0, 2).unwrap().ch, 'E');
        assert_eq!(canvas.state.grid.get(1, 2).unwrap().ch, 'F');
    }

    #[test]
    fn test_paste_text_respect_bounds() {
        let mut canvas = AsciiEditor::new(10, 10);
        // Set selection at (8, 8) to set paste origin
        canvas.set_selection_for_test(8, 8, 8, 8);

        // Paste text that exceeds right and bottom boundaries
        assert!(canvas.paste_text_impl("XYZ\n123"));

        // At (8,8) and (9,8)
        assert_eq!(canvas.state.grid.get(8, 8).unwrap().ch, 'X');
        assert_eq!(canvas.state.grid.get(9, 8).unwrap().ch, 'Y');
        // 'Z' should be out of bounds at (10, 8), grid width is 10
        assert!(canvas.state.grid.get(10, 8).is_none());

        // At (8,9) and (9,9)
        assert_eq!(canvas.state.grid.get(8, 9).unwrap().ch, '1');
        assert_eq!(canvas.state.grid.get(9, 9).unwrap().ch, '2');
        // '3' should be out of bounds at (10, 9)
        assert!(canvas.state.grid.get(10, 9).is_none());
    }

    #[test]
    fn test_paste_text_do_not_clobber_with_spaces() {
        let mut canvas = AsciiEditor::new(10, 10);
        // Pre-fill canvas
        canvas.state.grid.set_char(0, 0, '1');
        canvas.state.grid.set_char(1, 0, '2');
        canvas.state.grid.set_char(2, 0, '3');
        canvas.state.grid.set_char(0, 1, '4');
        canvas.state.grid.set_char(1, 1, '5');
        canvas.state.grid.set_char(2, 1, '6');

        // Paste text with spaces: "A C" and a line with leading/trailing spaces
        let content = "A C\n B ";
        assert!(canvas.paste_text_impl(content));

        // Row 0: '1' is overwritten by 'A', '2' is NOT overwritten by space, '3' is overwritten by 'C'
        assert_eq!(canvas.state.grid.get(0, 0).unwrap().ch, 'A');
        assert_eq!(canvas.state.grid.get(1, 0).unwrap().ch, '2');
        assert_eq!(canvas.state.grid.get(2, 0).unwrap().ch, 'C');

        // Row 1: '4' is NOT overwritten by space, '5' is overwritten by 'B', '6' is NOT overwritten by space
        assert_eq!(canvas.state.grid.get(0, 1).unwrap().ch, '4');
        assert_eq!(canvas.state.grid.get(1, 1).unwrap().ch, 'B');
        assert_eq!(canvas.state.grid.get(2, 1).unwrap().ch, '6');
    }
}
