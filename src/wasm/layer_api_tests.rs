//! Editor-level tests for the layer API (ADR-043).
//!
//! Kept in their own file so `layer_api.rs` stays inside the 500-line budget;
//! they are child modules of it, so `super::*` still reaches the API under test.

use super::*;

fn editor() -> AsciiEditor {
    AsciiEditor::new(4, 2)
}

/// A v1 document with two layers, the second hidden and locked.
fn v1_document() -> String {
    r#"{"format":"ascii-canvas","version":1,"canvas":{"width":4,"height":2},
"active_layer":0,"layers":[
{"name":"Base","visible":true,"locked":false,"cells":[{"x":0,"y":0,"ch":"A"}]},
{"name":"Sketch","visible":false,"locked":true,"cells":[{"x":1,"y":1,"ch":"B"}]}
]}"#
    .to_string()
}

#[test]
fn rename_is_undoable_and_no_op_rename_records_nothing() {
    let mut canvas = editor();
    canvas.rename_layer(0, "Renamed".to_string());
    assert_eq!(canvas.layer_name(0), "Renamed");
    assert_eq!(canvas.undo_label(), "Rename layer");
    assert!(canvas.can_undo());

    assert!(canvas.undo());
    assert_eq!(canvas.layer_name(0), "Layer 1");
    assert!(canvas.redo());
    assert_eq!(canvas.layer_name(0), "Renamed");

    // Renaming to the same name changes nothing, so it records nothing.
    let depth_before = canvas.undo_label();
    canvas.rename_layer(0, "Renamed".to_string());
    assert_eq!(canvas.undo_label(), depth_before);
    // An out-of-range index is ignored.
    canvas.rename_layer(9, "Nope".to_string());
    assert_eq!(canvas.undo_label(), depth_before);
}

#[test]
fn visibility_and_lock_are_undoable() {
    let mut canvas = editor();
    canvas.set_layer_visible(0, false);
    assert!(!canvas.layer_visible(0));
    assert_eq!(canvas.undo_label(), "Hide layer");
    assert!(canvas.undo());
    assert!(canvas.layer_visible(0));
    assert!(canvas.redo());
    assert!(!canvas.layer_visible(0));

    // Setting the same value again records nothing, and the redo that was
    // already consumed stays consumed.
    canvas.set_layer_visible(0, false);
    assert_eq!(canvas.redo_label(), "");
    assert_eq!(canvas.undo_label(), "Hide layer");
    assert!(!canvas.redo());

    canvas.set_layer_locked(0, true);
    assert!(canvas.layer_locked(0));
    assert_eq!(canvas.undo_label(), "Lock layer");
    // Undoing a lock must work even though the layer is locked.
    assert!(canvas.undo());
    assert!(!canvas.layer_locked(0));
}

#[test]
fn add_and_delete_layers_are_undoable() {
    let mut canvas = editor();
    let index = canvas.add_layer();
    assert_eq!(index, 1);
    assert_eq!(canvas.layer_count(), 2);
    assert_eq!(canvas.active_layer_index(), 1);

    // The add is recorded on the layer the user was on, so both undo and redo
    // are reachable from there - and undo brings focus back to it.
    assert_eq!(canvas.undo_count(), 0, "the new layer starts empty");
    assert!(canvas.set_active_layer(0));
    assert_eq!(canvas.undo_label(), "Add layer");
    assert_eq!(canvas.undo_count(), 1);

    assert!(canvas.undo());
    assert_eq!(canvas.layer_count(), 1);
    assert_eq!(canvas.active_layer_index(), 0);
    assert_eq!(canvas.redo_count(), 1);
    // A second undo must not remove another layer: there is only one entry.
    assert!(!canvas.undo());
    assert_eq!(canvas.layer_count(), 1);

    // Redo puts the added layer back (it used to be unreachable).
    assert!(canvas.redo());
    assert_eq!(canvas.layer_count(), 2);
    assert_eq!(canvas.layer_name(1), "Layer 2");

    canvas.set_layer_visible(1, true);
    assert!(canvas.delete_layer(0));
    assert_eq!(canvas.layer_count(), 1);
    assert_eq!(canvas.undo_label(), "Delete layer");
    assert!(canvas.undo());
    assert_eq!(canvas.layer_count(), 2);
    assert_eq!(canvas.layer_name(0), "Layer 1");
}

#[test]
fn the_last_layer_cannot_be_deleted() {
    let mut canvas = editor();
    assert!(!canvas.delete_layer(0), "one layer is not deletable");
    assert!(!canvas.delete_layer(7), "an unknown index is not deletable");
    assert_eq!(canvas.layer_count(), 1);
    assert!(!canvas.can_undo(), "a refused delete records nothing");
}

#[test]
fn move_and_merge_are_undoable() {
    let mut canvas = editor();
    canvas.add_layer();
    canvas.add_layer();
    assert_eq!(canvas.layer_count(), 3);
    canvas.set_active_layer(0);

    canvas.move_layer(0, 2);
    assert_eq!(canvas.layer_name(2), "Layer 1");
    assert_eq!(canvas.undo_label(), "Move layer up");
    assert!(canvas.undo());
    assert_eq!(canvas.layer_name(0), "Layer 1");

    // A no-op move records nothing.
    let label = canvas.undo_label();
    canvas.move_layer(1, 1);
    canvas.move_layer(0, 9);
    assert_eq!(canvas.undo_label(), label);

    assert!(canvas.merge_layer_down(1));
    assert_eq!(canvas.layer_count(), 2);
    assert_eq!(canvas.undo_label(), "Merge layer down");
    assert!(canvas.undo());
    assert_eq!(canvas.layer_count(), 3);

    // The bottom layer has nothing to merge into.
    assert!(!canvas.merge_layer_down(0));
}

#[test]
fn switching_layers_does_not_record_history() {
    let mut canvas = editor();
    canvas.add_layer();
    // The new layer has no history of its own: the add went on the old one.
    assert!(!canvas.can_undo());
    assert_eq!(canvas.undo_count(), 0);

    // Navigation is not undoable and must not create an entry: the depth on
    // each side of a switch is unchanged.
    assert!(canvas.set_active_layer(0));
    assert_eq!(canvas.active_layer_index(), 0);
    let before = canvas.undo_count();
    assert!(canvas.set_active_layer(1));
    assert_eq!(canvas.undo_count(), 0);
    assert!(canvas.set_active_layer(0));
    assert_eq!(canvas.undo_count(), before);
    // Re-selecting the current layer is a no-op that still reports success.
    assert!(canvas.set_active_layer(0));
    assert!(!canvas.set_active_layer(9));
}

#[test]
fn undo_is_scoped_to_the_active_layer() {
    let mut canvas = editor();
    canvas.rename_layer(0, "First".to_string());
    // The add is recorded on layer 0 (the layer the user was on), so layer 0
    // now holds [rename, add] and the new layer holds nothing.
    canvas.add_layer();
    assert_eq!(canvas.undo_count(), 0, "the new layer starts empty");
    canvas.rename_layer(1, "Second".to_string());
    assert_eq!(canvas.undo_label(), "Rename layer");

    // Undo walks the new layer's own history, and stops when it is empty.
    assert!(canvas.undo());
    assert_eq!(canvas.layer_name(1), "Layer 2");
    assert!(!canvas.undo(), "layer 1 has nothing left");

    // Layer 0's entries were never touched by the other layer's undo.
    assert!(canvas.set_active_layer(0));
    assert_eq!(canvas.undo_label(), "Add layer");
    assert_eq!(canvas.undo_count(), 2);
    assert!(canvas.undo());
    assert_eq!(canvas.layer_count(), 1);
    assert_eq!(canvas.active_layer_index(), 0);
    assert_eq!(canvas.layer_name(0), "First");
    assert!(canvas.undo());
    assert_eq!(canvas.layer_name(0), "Layer 1");
    assert!(!canvas.undo(), "nothing left on layer 0");
}

#[test]
fn resize_drops_history_on_every_layer() {
    let mut canvas = editor();
    canvas.add_layer();
    canvas.rename_layer(0, "Kept".to_string());
    assert!(canvas.can_undo());

    canvas.resize(8, 4);
    assert_eq!(canvas.width(), 8);
    assert_eq!(canvas.height(), 4);
    // Recorded draw commands carry coordinates and sizes that no longer match
    // the resized grids, so no layer may keep its history.
    assert!(!canvas.can_undo());
    assert_eq!(canvas.undo_label(), "");
    assert_eq!(canvas.layer_name(0), "Kept", "content and names survive");
}

#[test]
fn v1_document_loads_with_its_layer_flags() {
    let mut canvas = editor();
    assert!(canvas.load_document(v1_document()));
    assert_eq!(canvas.layer_count(), 2);
    assert_eq!(canvas.active_layer_index(), 0);
    assert_eq!(canvas.layer_name(0), "Base");
    assert_eq!(canvas.layer_name(1), "Sketch");
    // The frozen v1 format carries per-layer visibility and lock state.
    assert!(canvas.layer_visible(0));
    assert!(!canvas.layer_visible(1));
    assert!(!canvas.layer_locked(0));
    assert!(canvas.layer_locked(1));

    // Round-tripping keeps both flags, so nothing is silently dropped.
    let again = canvas.serialize_document();
    let mut reloaded = editor();
    assert!(reloaded.load_document(again));
    assert!(!reloaded.layer_visible(1));
    assert!(reloaded.layer_locked(1));
    assert_eq!(reloaded.layer_name(1), "Sketch");
}

#[test]
fn loading_a_document_clears_history() {
    let mut canvas = editor();
    canvas.rename_layer(0, "Before".to_string());
    assert!(canvas.can_undo());
    assert!(canvas.load_document(v1_document()));
    assert!(!canvas.can_undo());
}
