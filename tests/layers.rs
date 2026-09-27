//! Layer model tests (ADR-043).
//!
//! These live in an integration test rather than beside `src/core/layer.rs` so
//! the source file stays inside the 500-line budget. Every assertion goes
//! through the public API, which is what an integration test is for.

use ascii_canvas::core::cell::Cell;
use ascii_canvas::core::commands::{AddLayerCommand, Command, LayerCommand, SetCellCommand};
use ascii_canvas::core::grid::Grid;
use ascii_canvas::core::history::HistoryOutcome;
use ascii_canvas::core::layer::{
    active_after_insert, active_after_move, active_after_remove, Layer, LayerStack,
};

fn three_layers() -> LayerStack {
    let mut stack = LayerStack::new(4, 2);
    stack.add_layer();
    stack.add_layer();
    stack
}

fn push_set(stack: &mut LayerStack, x: i32, y: i32, ch: char) {
    stack.push_grid(Box::new(SetCellCommand::new(x, y, Cell::new(ch))));
}

/// A stack whose second layer is active, as after adding a layer.
fn two_layers() -> LayerStack {
    let mut stack = LayerStack::new(4, 2);
    stack.add_layer();
    stack
}

fn names(stack: &LayerStack) -> Vec<String> {
    stack
        .layers()
        .iter()
        .map(|l| l.name().to_string())
        .collect()
}

fn char_at(stack: &LayerStack, layer: usize, x: i32, y: i32) -> Option<char> {
    stack
        .get(layer)?
        .grid()
        .get(x, y)
        .filter(|cell| cell.is_visible())
        .map(|c| c.ch)
}

#[test]
fn new_stack_has_one_active_layer() {
    let stack = LayerStack::new(8, 3);
    assert_eq!(stack.len(), 1);
    assert_eq!(stack.active_index(), 0);
    assert_eq!(stack.active().name(), "Layer 1");
    assert_eq!(stack.active().grid().width(), 8);
    assert!(stack.active().is_visible());
    assert!(!stack.active().is_locked());
    assert!(!stack.is_empty());
}

#[test]
fn add_layer_appends_and_activates() {
    let mut stack = three_layers();
    assert_eq!(stack.add_layer(), 3);
    assert_eq!(stack.len(), 4);
    assert_eq!(stack.active_index(), 3);
    assert_eq!(stack.active().name(), "Layer 4");
}

#[test]
fn from_layers_clamps_active_index() {
    let layers = vec![Layer::new("A", 2, 2), Layer::new("B", 2, 2)];
    assert_eq!(LayerStack::from_layers(layers, 7).active_index(), 1);
}

#[test]
fn metadata_setters_report_out_of_range() {
    let mut stack = LayerStack::new(2, 2);
    assert!(stack.set_name(0, "Renamed".to_string()));
    assert!(stack.set_visible(0, false));
    assert!(stack.set_locked(0, true));
    assert!(!stack.set_name(5, "Nope".to_string()));
    assert!(!stack.set_visible(5, false));
    assert!(!stack.set_locked(5, true));
    assert!(!stack.set_active(5));
    assert_eq!(stack.active().name(), "Renamed");
    assert!(!stack.active().is_visible());
    assert!(stack.is_active_locked());
}

#[test]
fn remove_and_insert_round_trip() {
    let mut stack = three_layers();
    let removed = stack.remove_layer(1).expect("layer 1 exists");
    assert_eq!(removed.name(), "Layer 2");
    assert_eq!(stack.len(), 2);
    assert!(stack.remove_layer(9).is_none());

    stack.insert_layer(1, removed);
    assert_eq!(names(&stack), vec!["Layer 1", "Layer 2", "Layer 3"]);

    // Out-of-range inserts clamp to the end instead of panicking.
    stack.insert_layer(99, Layer::new("Z", 2, 2));
    assert_eq!(stack.layers()[3].name(), "Z");
}

#[test]
fn move_layer_tracks_active_layer() {
    let mut stack = three_layers();
    // Active is 2; moving 0 -> 2 must leave it on the same layer ("Layer 3").
    assert!(stack.move_layer(0, 2));
    assert_eq!(stack.active().name(), "Layer 3");
    assert!(!stack.move_layer(0, 0));
    assert!(!stack.move_layer(0, 9));
}

#[test]
fn move_layer_from_active_follows_it() {
    let mut stack = three_layers();
    assert!(stack.move_layer(2, 0));
    assert_eq!(stack.active_index(), 0);
    assert_eq!(stack.active().name(), "Layer 3");
}

#[test]
fn resize_touches_every_layer_and_keeps_content() {
    let mut stack = three_layers();
    stack.active_mut().grid_mut().set_char(0, 0, 'X');
    stack.resize(6, 5);
    assert_eq!(stack.active().grid().width(), 6);
    assert_eq!(stack.active().grid().get(0, 0).map(|c| c.ch), Some('X'));
    for layer in stack.layers() {
        assert_eq!(layer.grid().height(), 5);
        assert_eq!(layer.grid().width(), 6);
    }
}

#[test]
fn clear_all_histories_empties_every_layer() {
    let mut stack = three_layers();
    push_set(&mut stack, 0, 0, 'A');
    stack.add_layer();
    push_set(&mut stack, 0, 0, 'B');
    assert!(stack.can_undo());
    stack.clear_all_histories();
    assert!(!stack.can_undo());
    for layer in stack.layers() {
        assert!(!layer.history().can_undo(), "history survived on a layer");
    }
}

#[test]
fn history_is_per_layer() {
    let mut stack = three_layers();
    push_set(&mut stack, 0, 0, 'A');
    assert!(stack.can_undo());
    stack.add_layer();
    assert!(!stack.can_undo(), "a new layer starts with empty history");
    assert!(stack.layers()[2].history().can_undo());
}

#[test]
fn merge_overlap_lists_only_what_the_upper_layer_writes() {
    let mut stack = three_layers();
    stack.active_mut().grid_mut().set_char(0, 0, 'A');
    stack.active_mut().grid_mut().set_char(1, 0, ' ');
    let upper = stack.active().grid().clone();
    let overlap = stack.merge_overlap(stack.active_index(), &upper);
    assert_eq!(overlap.len(), 1);
    assert_eq!(overlap[0], (0, 0, Some(Cell::new('A'))));
}

#[test]
fn merge_overlap_marks_empty_lower_cells_as_none() {
    let stack = three_layers();
    let mut upper = Grid::new(4, 2);
    upper.set_char(2, 0, 'U');
    assert_eq!(stack.merge_overlap(0, &upper), vec![(2, 0, None)]);
}

#[test]
fn blit_and_restore_round_trip() {
    let mut stack = three_layers();
    let mut upper = Grid::new(4, 2);
    upper.set_char(1, 1, 'U');
    *stack.get_mut(1).expect("layer 1").grid_mut() = upper;

    let snapshot = stack.merge_overlap(0, stack.get(1).expect("layer 1").grid());
    let source = stack.get(1).expect("layer 1").grid().clone();
    stack.blit_visible_into(0, &source);
    assert_eq!(char_at(&stack, 0, 1, 1), Some('U'));

    stack.restore_optional_cells(0, &snapshot);
    assert_eq!(char_at(&stack, 0, 1, 1), None);
}

#[test]
fn blit_to_missing_layer_is_a_no_op() {
    let mut stack = three_layers();
    let source = Grid::new(2, 2);
    stack.blit_visible_into(42, &source);
    assert!(stack.merge_overlap(42, &source).is_empty());
    stack.restore_optional_cells(42, &[(0, 0, Some(Cell::new('Z')))]);
    assert_eq!(stack.len(), 3);
}

#[test]
fn active_index_mappings() {
    assert_eq!(active_after_remove(0, 0, 2), 0);
    assert_eq!(active_after_remove(2, 0, 2), 1);
    assert_eq!(active_after_remove(3, 3, 2), 1);
    assert_eq!(active_after_remove(1, 3, 2), 1);
    assert_eq!(active_after_insert(2, 1), 3);
    assert_eq!(active_after_insert(0, 1), 0);
    // [A,B,C] with C active; moving A to index 2 leaves C at index 1.
    assert_eq!(active_after_move(2, 0, 2), 1);
    assert_eq!(active_after_move(2, 2, 0), 0);
    assert_eq!(active_after_move(1, 0, 2), 0);
    assert_eq!(active_after_move(0, 2, 0), 1);
    assert_eq!(active_after_move(3, 0, 2), 3);
}

#[test]
fn add_command_scaffolding_is_public() {
    // The add command derives its layer from the stack, so callers cannot
    // record an add that differs from what `add_layer` would do.
    let stack = three_layers();
    let cmd = AddLayerCommand::adding_next(&stack);
    assert_eq!(cmd.index(), 3);
    assert_eq!(cmd.description(), "Add layer");
}

// ------------------------------------------------- structural command matrix
//
// Each operation runs as apply -> undo -> redo through the same entry points
// `src/wasm/layer_api.rs` uses. A layer command is recorded on the *active*
// layer's history, so these tests keep the target layer active and mirror the
// editor by keeping the drawing surface in a separate grid.

use ascii_canvas::core::commands::{
    DeleteLayerCommand, MergeLayerDownCommand, MoveLayerCommand, SetLayerLockedCommand,
    SetLayerNameCommand, SetLayerVisibleCommand,
};

/// Apply a structural command and record it on the active layer's history.
fn record(stack: &mut LayerStack, mut cmd: Box<dyn LayerCommand>) {
    cmd.apply(stack);
    stack.push_layer_command(cmd);
}

fn undo_with(stack: &mut LayerStack, surface: &mut Grid) -> HistoryOutcome {
    stack.undo_active(surface)
}

fn redo_with(stack: &mut LayerStack, surface: &mut Grid) -> HistoryOutcome {
    stack.redo_active(surface)
}

fn undo(stack: &mut LayerStack) -> HistoryOutcome {
    let mut surface = Grid::new(4, 2);
    undo_with(stack, &mut surface)
}

fn redo(stack: &mut LayerStack) -> HistoryOutcome {
    let mut surface = Grid::new(4, 2);
    redo_with(stack, &mut surface)
}

/// Three layers with `index` active, so a command on that layer is undoable.
fn stack_active_at(index: usize) -> LayerStack {
    let mut stack = three_layers();
    stack.set_active(index);
    stack
}

#[test]
fn rename_undoes_and_redoes() {
    let mut stack = stack_active_at(0);
    let old = stack.layers()[0].name().to_string();
    let cmd = SetLayerNameCommand::new(0, old, "Renamed".to_string());
    record(&mut stack, Box::new(cmd));
    assert_eq!(stack.layers()[0].name(), "Renamed");
    assert_eq!(stack.undo_description(), Some("Rename layer"));

    assert_eq!(undo(&mut stack), HistoryOutcome::Layer);
    assert_eq!(stack.layers()[0].name(), "Layer 1");
    assert_eq!(redo(&mut stack), HistoryOutcome::Layer);
    assert_eq!(stack.layers()[0].name(), "Renamed");
}

#[test]
fn visibility_undoes_and_redoes() {
    let mut stack = stack_active_at(1);
    record(
        &mut stack,
        Box::new(SetLayerVisibleCommand::new(1, true, false)),
    );
    assert!(!stack.layers()[1].is_visible());
    assert_eq!(undo(&mut stack), HistoryOutcome::Layer);
    assert!(stack.layers()[1].is_visible());
    assert_eq!(redo(&mut stack), HistoryOutcome::Layer);
    assert!(!stack.layers()[1].is_visible());
}

#[test]
fn lock_undo_works_while_the_layer_stays_locked() {
    let mut stack = stack_active_at(0);
    record(
        &mut stack,
        Box::new(SetLayerLockedCommand::new(0, false, true)),
    );
    assert!(stack.is_active_locked());

    // The layer is locked, but undoing the lock must not be blocked: otherwise a
    // user can lock a layer and be unable to undo anything on it again.
    assert_eq!(undo(&mut stack), HistoryOutcome::Layer);
    assert!(!stack.is_active_locked());
    assert_eq!(redo(&mut stack), HistoryOutcome::Layer);
    assert!(stack.is_active_locked());
}

#[test]
fn move_undoes_and_redoes_with_the_active_index() {
    let mut stack = three_layers();
    // Active is 2 ("Layer 3"). Moving layer 0 up to index 2 reorders the stack
    // to [2, 3, 1] and must leave the active layer on "Layer 3" at index 1.
    record(&mut stack, Box::new(MoveLayerCommand::new(0, 2)));
    assert_eq!(names(&stack), vec!["Layer 2", "Layer 3", "Layer 1"]);
    assert_eq!(stack.active_index(), 1);

    assert_eq!(undo(&mut stack), HistoryOutcome::Layer);
    assert_eq!(names(&stack), vec!["Layer 1", "Layer 2", "Layer 3"]);
    assert_eq!(stack.active_index(), 2, "the active layer must follow back");
    assert_eq!(redo(&mut stack), HistoryOutcome::Layer);
    assert_eq!(names(&stack), vec!["Layer 2", "Layer 3", "Layer 1"]);
    assert_eq!(stack.active_index(), 1);
}

#[test]
fn add_undoes_once_and_does_not_toggle() {
    let mut stack = three_layers();
    let add = AddLayerCommand::adding_next(&stack);
    record(&mut stack, Box::new(add));
    assert_eq!(stack.len(), 4);
    assert_eq!(stack.active_index(), 3);

    assert_eq!(undo(&mut stack), HistoryOutcome::Layer);
    assert_eq!(stack.len(), 3);
    assert_eq!(stack.active_index(), 2);

    // The command lived in the layer it added, so it went away with that layer:
    // a second undo must not remove another layer.
    assert_eq!(undo(&mut stack), HistoryOutcome::None);
    assert_eq!(stack.len(), 3);
}

#[test]
fn delete_of_active_layer_undoes_and_redoes() {
    let mut stack = stack_active_at(1);
    stack
        .get_mut(1)
        .expect("layer 1")
        .grid_mut()
        .set_char(0, 0, 'B');
    let cmd = DeleteLayerCommand::deleting(&stack, 1).expect("deletable");
    record(&mut stack, Box::new(cmd));
    assert_eq!(names(&stack), vec!["Layer 1", "Layer 3"]);
    assert_eq!(stack.active_index(), 1);

    assert_eq!(undo(&mut stack), HistoryOutcome::Layer);
    assert_eq!(names(&stack), vec!["Layer 1", "Layer 2", "Layer 3"]);
    // Back on the restored layer, which keeps the index it was deleted from.
    assert_eq!(stack.active_index(), 1);
    assert_eq!(stack.active().name(), "Layer 2");
    assert_eq!(char_at(&stack, 1, 0, 0), Some('B'), "content is restored");

    // Redo of a delete is recorded on the layer that was active after the delete,
    // so after undoing (which returns focus to the restored layer) the redo is
    // only reachable from the other layer. Known limit of per-layer history.
    assert_eq!(redo(&mut stack), HistoryOutcome::None);
    stack.set_active(2); // "Layer 3" is the layer that recorded the delete
    assert_eq!(redo(&mut stack), HistoryOutcome::Layer);
    assert_eq!(names(&stack), vec!["Layer 1", "Layer 3"]);
    assert_eq!(stack.active_index(), 1);
}

#[test]
fn delete_keeps_the_removed_layers_own_history() {
    let mut stack = stack_active_at(0);
    push_set(&mut stack, 0, 0, 'A');
    let cmd = DeleteLayerCommand::deleting(&stack, 0).expect("deletable");
    record(&mut stack, Box::new(cmd));
    assert_eq!(stack.len(), 2);

    assert_eq!(undo(&mut stack), HistoryOutcome::Layer);
    assert_eq!(stack.len(), 3);
    // The layer came back with the undo history it had, not an empty one.
    assert!(stack.layers()[0].history().can_undo());
}

#[test]
fn delete_of_last_layer_is_refused() {
    let stack = LayerStack::new(4, 2);
    assert!(DeleteLayerCommand::deleting(&stack, 0).is_none());
    assert!(DeleteLayerCommand::deleting(&stack, 7).is_none());
}

#[test]
fn merge_undo_restores_both_layers_and_the_overlap() {
    let mut stack = two_layers();
    // Lower layer: A at (0,0), B at (1,0). Upper layer overwrites (0,0) with C
    // and adds D at (2,0), so one undo has to restore all of it.
    let lower = stack.get_mut(0).expect("layer 0");
    lower.grid_mut().set_char(0, 0, 'A');
    lower.grid_mut().set_char(1, 0, 'B');
    let upper = stack.get_mut(1).expect("layer 1");
    upper.grid_mut().set_char(0, 0, 'C');
    upper.grid_mut().set_char(2, 0, 'D');

    let cmd = MergeLayerDownCommand::merging(&stack, 1).expect("mergeable");
    record(&mut stack, Box::new(cmd));
    assert_eq!(stack.len(), 1);
    assert_eq!(
        char_at(&stack, 0, 0, 0),
        Some('C'),
        "upper wins the overlap"
    );
    assert_eq!(char_at(&stack, 0, 1, 0), Some('B'), "untouched cell stays");
    assert_eq!(char_at(&stack, 0, 2, 0), Some('D'));

    assert_eq!(undo(&mut stack), HistoryOutcome::Layer);
    assert_eq!(stack.len(), 2);
    assert_eq!(char_at(&stack, 0, 0, 0), Some('A'), "overlap is restored");
    assert_eq!(char_at(&stack, 0, 1, 0), Some('B'));
    assert_eq!(char_at(&stack, 1, 0, 0), Some('C'));
    assert_eq!(char_at(&stack, 1, 2, 0), Some('D'));
    assert_eq!(stack.active_index(), 1, "the merged layer is active again");

    // Redo is recorded on the layer that was active after the merge (the lower
    // one), so it is only reachable from there - a known limit of per-layer
    // history, not a lost entry.
    assert_eq!(redo(&mut stack), HistoryOutcome::None);
    stack.set_active(0);
    assert_eq!(redo(&mut stack), HistoryOutcome::Layer);
    assert_eq!(stack.len(), 1);
    assert_eq!(char_at(&stack, 0, 0, 0), Some('C'));
}

#[test]
fn merge_undo_clears_cells_the_merge_created() {
    let mut stack = two_layers();
    // The lower cell at (2,0) is empty, so the merge writes into empty space and
    // undo has to clear it again rather than leave a stale glyph behind.
    stack
        .get_mut(1)
        .expect("layer 1")
        .grid_mut()
        .set_char(2, 0, 'D');

    let cmd = MergeLayerDownCommand::merging(&stack, 1).expect("mergeable");
    record(&mut stack, Box::new(cmd));
    assert_eq!(char_at(&stack, 0, 2, 0), Some('D'));

    assert_eq!(undo(&mut stack), HistoryOutcome::Layer);
    assert_eq!(
        char_at(&stack, 0, 2, 0),
        None,
        "the written cell is cleared"
    );
    assert_eq!(char_at(&stack, 1, 2, 0), Some('D'));
}

#[test]
fn merge_of_bottom_layer_is_refused() {
    let stack = three_layers();
    assert!(MergeLayerDownCommand::merging(&stack, 0).is_none());
    assert!(MergeLayerDownCommand::merging(&stack, 9).is_none());
}

#[test]
fn structural_and_draw_entries_interleave_in_order() {
    let mut stack = stack_active_at(0);
    let mut surface = Grid::new(4, 2);

    // Draw, then rename: undo walks back through both kinds of entry.
    let mut draw = SetCellCommand::new(0, 0, Cell::new('Z'));
    draw.apply(&mut surface);
    stack.push_grid(Box::new(draw));
    let old = stack.layers()[0].name().to_string();
    let cmd = SetLayerNameCommand::new(0, old, "Renamed".to_string());
    record(&mut stack, Box::new(cmd));
    assert_eq!(stack.layers()[0].name(), "Renamed");

    assert_eq!(undo_with(&mut stack, &mut surface), HistoryOutcome::Layer);
    assert_eq!(stack.layers()[0].name(), "Layer 1");
    assert_eq!(undo_with(&mut stack, &mut surface), HistoryOutcome::Grid);
    assert!(surface.get(0, 0).expect("cell").is_empty());
    assert_eq!(redo_with(&mut stack, &mut surface), HistoryOutcome::Grid);
    assert_eq!(surface.get(0, 0).expect("cell").ch, 'Z');
    assert_eq!(redo_with(&mut stack, &mut surface), HistoryOutcome::Layer);
    assert_eq!(stack.layers()[0].name(), "Renamed");
}

#[test]
fn a_locked_layer_blocks_draw_undo() {
    let mut stack = three_layers();
    let mut surface = Grid::new(4, 2);

    // Record a draw on layer 0...
    stack.set_active(0);
    let mut draw = SetCellCommand::new(0, 0, Cell::new('Z'));
    draw.apply(&mut surface);
    stack.push_grid(Box::new(draw));

    // ...then lock layer 0 from layer 1, so the lock lands on layer 1's history.
    stack.set_active(1);
    let lock = SetLayerLockedCommand::new(0, false, true);
    record(&mut stack, Box::new(lock));
    assert!(stack.layers()[0].is_locked());

    // On the locked layer, undo is refused and the entry is kept, not consumed.
    stack.set_active(0);
    assert_eq!(undo_with(&mut stack, &mut surface), HistoryOutcome::None);
    assert!(stack.can_undo());
    assert_eq!(surface.get(0, 0).expect("cell").ch, 'Z');

    // Unlocking the layer (done directly here; the lock command's own
    // undoability is covered above) frees the very same entry.
    stack.set_locked(0, false);
    stack.set_active(0);
    assert_eq!(undo_with(&mut stack, &mut surface), HistoryOutcome::Grid);
    assert!(surface.get(0, 0).expect("cell").is_empty());
}

#[test]
fn undo_does_not_cross_layers() {
    let mut stack = stack_active_at(0);
    let lock = SetLayerLockedCommand::new(0, false, true);
    record(&mut stack, Box::new(lock));
    assert!(stack.layers()[0].is_locked());

    // Another layer has its own (empty) history, so undo is a no-op there and
    // the first layer's entry is untouched.
    stack.set_active(1);
    assert_eq!(undo(&mut stack), HistoryOutcome::None);
    assert!(stack.layers()[0].is_locked());

    stack.set_active(0);
    assert_eq!(undo(&mut stack), HistoryOutcome::Layer);
    assert!(!stack.layers()[0].is_locked());
}

#[test]
fn history_reports_descriptions_for_both_kinds_of_entry() {
    let mut stack = stack_active_at(0);
    let old = stack.layers()[0].name().to_string();
    let cmd = SetLayerNameCommand::new(0, old, "Renamed".to_string());
    record(&mut stack, Box::new(cmd));
    assert_eq!(stack.undo_description(), Some("Rename layer"));
    assert_eq!(stack.redo_description(), None);
    undo(&mut stack);
    assert_eq!(stack.redo_description(), Some("Rename layer"));
}
