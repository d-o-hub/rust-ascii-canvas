//! Regression tests for defects found reviewing PR #212.
//!
//! Each test reproduces a specific finding, so reintroducing the bug fails here.

use super::*;

/// Each test here reproduces a defect found in review of PR #212, so a
/// regression reintroduces the exact failure the review demonstrated.
fn editor() -> AsciiEditor {
    AsciiEditor::new(4, 2)
}

/// Undoing a metadata change must not discard drawing that the layer copy
/// has not caught up with yet (the CRITICAL finding).
#[test]
fn undoing_a_rename_keeps_drawing_made_since_the_last_sync() {
    let mut canvas = editor();
    assert!(canvas.paste_text_impl("Z"));
    assert!(canvas.export_ascii().contains('Z'));

    canvas.rename_layer(0, "Renamed".to_string());
    assert!(canvas.undo());
    assert_eq!(canvas.layer_name(0), "Layer 1");

    // The rename is undone, but the 'Z' the user drew must still be there:
    // reloading the surface from the layer copy would have dropped it.
    assert!(
        canvas.export_ascii().contains('Z'),
        "drawing was lost by a metadata undo: {:?}",
        canvas.export_ascii()
    );
}

#[test]
fn undoing_visibility_keeps_drawing_made_since_the_last_sync() {
    let mut canvas = editor();
    assert!(canvas.paste_text_impl("Z"));
    canvas.set_layer_visible(0, false);
    assert!(canvas.undo());
    assert!(canvas.layer_visible(0));
    assert!(canvas.export_ascii().contains('Z'));
}

/// A metadata undo must address the layer it was recorded against, even when
/// the stack was renumbered in between (the HIGH finding).
#[test]
fn metadata_undo_never_renames_a_different_layer() {
    let mut canvas = editor();
    canvas.add_layer();
    canvas.add_layer();
    // Active is layer 2. Rename layer 0 from here: the entry is recorded on
    // layer 2's history but targets layer 0 by identity.
    canvas.rename_layer(0, "Renamed".to_string());

    // Renumber the stack so index 0 refers to another layer: delete layer 0,
    // then reorder what is left.
    canvas.set_active_layer(0);
    assert!(canvas.delete_layer(0));
    canvas.move_layer(0, 1);
    let names_before: Vec<String> = (0..canvas.layer_count())
        .map(|i| canvas.layer_name(i))
        .collect();

    // Go back to whichever layer holds the rename entry (indices have moved,
    // which is the whole point) and undo it.
    let holder = (0..canvas.layer_count()).find(|i| {
        canvas.set_active_layer(*i);
        canvas.undo_label() == "Rename layer"
    });
    assert!(holder.is_some(), "the rename entry is still on some layer");
    assert!(canvas.undo());

    // The recorded layer is gone, so the undo is a no-op rather than applying
    // the rename to whichever layer now sits at that index.
    let names_after: Vec<String> = (0..canvas.layer_count())
        .map(|i| canvas.layer_name(i))
        .collect();
    assert_eq!(
        names_after, names_before,
        "the undo renamed a layer it was not recorded against"
    );
    assert!(!names_after.iter().any(|n| n == "Renamed"));
}

/// Redoing a merge must merge the layers it merged, not whichever layers
/// happen to sit at those indices now (the second HIGH finding).
#[test]
fn merge_redo_after_a_reorder_touches_the_right_layers() {
    let mut canvas = editor();
    canvas.add_layer();
    canvas.add_layer();
    canvas.set_active_layer(0);

    // Merge layer 1 into layer 0, recorded on layer 0.
    assert!(canvas.merge_layer_down(1));
    assert_eq!(canvas.layer_count(), 2);
    assert!(canvas.undo());
    assert_eq!(canvas.layer_count(), 3);

    // Reorder from a DIFFERENT layer, so the merge's redo entry on layer 0 is
    // not cleared. Reordering on layer 0 itself wipes the very entry this test
    // is about, which is what made the earlier version vacuous: its real
    // assertions sat behind a branch that never ran.
    //
    // "Layer 3" is the layer to move: the merged pair (Layer 1, Layer 2) stays
    // adjacent and in order, but the recorded index no longer points at it.
    assert!(canvas.set_active_layer(2));
    canvas.move_layer(2, 0);
    assert_eq!(canvas.layer_name(0), "Layer 3");
    assert_eq!(canvas.layer_count(), 3);

    // "Layer 1" is now at index 1; the merge was recorded with upper index 1.
    assert!(canvas.set_active_layer(1));
    assert_eq!(
        canvas.redo_label(),
        "Merge layer down",
        "reordering on another layer must not clear this layer's redo"
    );
    assert!(canvas.redo());
    assert_eq!(
        canvas.layer_count(),
        2,
        "the merge must fold exactly one layer"
    );
    assert!(
        (0..canvas.layer_count()).any(|i| canvas.layer_name(i) == "Layer 1"),
        "the lower layer of the pair must survive"
    );
    assert!(
        (0..canvas.layer_count()).any(|i| canvas.layer_name(i) == "Layer 3"),
        "the reordered layer must not be folded in by mistake"
    );
}

/// A merge whose pair is no longer adjacent in the right order must be refused
/// outright, leaving the stack untouched. Swapping the pair puts the lower layer
/// above the upper one, which is no longer a merge; silently folding the wrong
/// two layers would be data loss.
#[test]
fn merge_redo_is_refused_when_the_pair_is_no_longer_ordered() {
    let mut canvas = editor();
    canvas.add_layer();
    canvas.add_layer();
    assert!(canvas.set_active_layer(0));
    assert!(canvas.merge_layer_down(1));
    assert!(canvas.undo());
    assert_eq!(canvas.layer_count(), 3);

    // Move "Layer 2" (the recorded upper layer) above "Layer 1" (the lower one).
    assert!(canvas.set_active_layer(1));
    canvas.move_layer(1, 0);
    assert_eq!(canvas.layer_name(0), "Layer 2");

    // The redo must not fold anything: the pair's relative order is inverted.
    assert!(canvas.set_active_layer(1));
    let before: Vec<String> = (0..canvas.layer_count())
        .map(|i| canvas.layer_name(i))
        .collect();
    let _ = canvas.redo();
    let after: Vec<String> = (0..canvas.layer_count())
        .map(|i| canvas.layer_name(i))
        .collect();
    assert_eq!(
        after, before,
        "a broken pair must leave the stack untouched"
    );
}

/// A locked layer blocks drawing history but must say so through the public
/// surface, and the entry survives (the silent-block finding).
#[test]
fn blocked_undo_is_visible_through_can_undo() {
    let mut canvas = editor();
    canvas.add_layer();
    // Draw on layer 0, so its newest entry is a drawing...
    canvas.set_active_layer(0);
    assert!(canvas.paste_text_impl("Z"));
    // ...then lock it from layer 1, so the lock lands on layer 1's history and
    // the drawing stays on top of layer 0's.
    canvas.set_active_layer(1);
    canvas.set_layer_locked(0, true);

    canvas.set_active_layer(0);
    assert!(
        canvas.can_undo(),
        "the caller can see there is something to undo"
    );
    let depth = canvas.undo_count();
    assert!(!canvas.undo(), "but the locked layer refuses it");
    assert_eq!(
        canvas.undo_count(),
        depth,
        "the entry is kept, not consumed"
    );
    assert!(canvas.export_ascii().contains('Z'));

    // Unlock from the layer that recorded the lock, so layer 0's timeline is
    // untouched, then come back: the drawing that was refused a moment ago is
    // now the newest step on it.
    canvas.set_active_layer(1);
    canvas.set_layer_locked(0, false);
    canvas.set_active_layer(0);
    assert!(canvas.undo());
    assert!(
        !canvas.export_ascii().contains('Z'),
        "the drawing should undo once the layer is unlocked"
    );
}

/// v1 documents may omit the optional per-layer keys; the defaults must hold
/// and the cell content must survive the round trip.
#[test]
fn v1_document_without_optional_keys_uses_defaults() {
    let json = r#"{"format":"ascii-canvas","version":1,"canvas":{"width":4,"height":2},
"layers":[{"name":"Sparse","cells":[{"x":0,"y":0,"ch":"Q"}]}]}"#;
    let mut canvas = editor();
    assert!(canvas.load_document(json.to_string()));
    assert_eq!(canvas.layer_count(), 1);
    assert!(canvas.layer_visible(0), "visible defaults to true");
    assert!(!canvas.layer_locked(0), "locked defaults to false");
    assert!(
        canvas.export_ascii().contains('Q'),
        "cells survive the load"
    );
    assert_eq!(canvas.active_layer_index(), 0, "active_layer defaults to 0");
}

#[test]
fn v1_document_preserves_cell_content_through_a_round_trip() {
    let json = r#"{"format":"ascii-canvas","version":1,"canvas":{"width":4,"height":2},
"active_layer":0,"layers":[
{"name":"Base","visible":true,"locked":false,"cells":[{"x":0,"y":0,"ch":"A"}]},
{"name":"Sketch","visible":true,"locked":true,"cells":[{"x":1,"y":1,"ch":"B"}]}
]}"#;
    let mut canvas = editor();
    assert!(canvas.load_document(json.to_string()));
    assert!(canvas.export_ascii().contains('A'));
    assert!(canvas.export_ascii().contains('B'));

    let again = canvas.serialize_document();
    let mut reloaded = editor();
    assert!(reloaded.load_document(again));
    assert!(reloaded.export_ascii().contains('A'));
    assert!(reloaded.export_ascii().contains('B'));
    assert!(reloaded.layer_locked(1));
}

/// Resize invalidates every layer's history, not only the active one.
#[test]
fn resize_clears_history_on_an_inactive_layer_too() {
    let mut canvas = editor();
    canvas.add_layer();
    // Put entries on layer 0, then leave it.
    canvas.set_active_layer(0);
    canvas.rename_layer(0, "Recorded".to_string());
    let before = canvas.undo_count();
    assert!(before >= 1, "layer 0 should have history to lose");
    canvas.set_active_layer(1);
    assert!(!canvas.can_undo(), "layer 1 has none of its own");

    canvas.resize(6, 4);
    // Layer 0's entries are gone: its grid changed size too.
    canvas.set_active_layer(0);
    assert_eq!(canvas.undo_count(), 0);
    assert_eq!(
        canvas.layer_name(0),
        "Recorded",
        "content and names survive"
    );
}

/// Out-of-range and no-op mutations must not record anything.
#[test]
fn refused_mutations_record_nothing() {
    let mut canvas = editor();
    canvas.add_layer();
    canvas.rename_layer(1, "Named".to_string());
    let depth = canvas.undo_count();

    // Same value, unknown index, and a no-op move.
    canvas.rename_layer(1, "Named".to_string());
    canvas.set_layer_visible(9, true);
    canvas.set_layer_locked(9, true);
    canvas.move_layer(1, 1);
    canvas.move_layer(1, 9);
    assert_eq!(canvas.undo_count(), depth, "a refused change was recorded");
    assert_eq!(canvas.layer_name(1), "Named");
}
