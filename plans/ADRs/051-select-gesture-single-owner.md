# ADR-051: Select Gesture State Has One Owner

## Status

Accepted — bug fix for #296 (`bug(tools): Select Tool lacks drag feedback and
has a race condition preventing move-to-click`). **Implementation:**
`src/core/tools/select.rs`, `src/wasm/helpers.rs`, `src/wasm/event_handlers.rs`,
`src/wasm/clipboard_helpers.rs`. No document format, WASM public surface,
ruleset, release pin or required check changes.

## Context

Two copies of the selection existed and each made the same decision
independently:

- `SelectTool::on_pointer_down` decided *move vs. new drag* from the tool's own
  `selection` field (`should_start_move`);
- `AsciiEditor::on_pointer_down` decided the same thing again from
  `self.current_selection`, and armed `is_moving_selection` + captured the move
  clipboard when the two disagreed.

`current_selection` is only refreshed from the tool on a select pointer-move or
pointer-up, so it can be stale or empty while the tool holds a live selection.
Two reachable failures followed:

1. **Ghost move.** `delete_selection_impl` / `cut_selection_impl` cleared
   `current_selection` but not the tool's copy. The next press inside the
   now-invisible rectangle made the tool enter `moving` while the editor never
   armed `is_moving_selection`: the highlight slid with the pointer, the content
   never moved, and pointer-up committed nothing — "click inside to move is
   unreliable".
2. **Phantom arm.** Any state where the cache outlived the tool's decision
   produced the inverse: `is_moving_selection = true` with no tool-side move, so
   `generate_move_preview_ops` built a preview from a selection the tool had
   already discarded.

Separately, a drag emitted no overlay at all: `SelectTool::on_pointer_move`
returned an empty `ToolResult`, and the editor only requested a redraw once
`current_selection` existed — after the first move event. A press showed
nothing.

## Decision

- **The tool answers "is this a move?" — once.** The editor's press handler now
  calls `begin_select_gesture()`, which reads `SelectTool::is_moving()` and
  adopts the tool's selection, instead of re-testing `contains(x, y)` against a
  cache. `current_selection` is a projection of the tool, never an input to the
  decision.
- **Selection teardown is one operation.** `clear_selection_state()` clears the
  editor's selection and the tool's copy together, and delete/cut use it. A
  selection the user cannot see cannot be moved.
- **The drag previews through the existing overlay channel.** While
  `selecting`, `on_pointer_down` seeds a one-cell selection and both down and
  move return dashed marquee `DrawOp`s, which the editor already routes into
  `preview_ops` (rendered in both the canvas-command and pixel-buffer paths).
  The marquee is transient by construction: preview ops are skipped by
  `exportAscii`, PNG export and document serialisation, and cleared on
  pointer-up, so it can never be committed into the document.
- **Pointer-up re-reads the tool before committing a move**, so a release
  without a final move event still lands the selection the tool last computed.
- **Marquee glyphs come from the pixel atlas.** The marquee uses `·`, which
  `web/render.ts` already uploads; a box-drawing stroke would render empty in
  the pixel path.

## Alternatives

- **Fix only the delete/cut desync.** Rejected: it removes one symptom of the
  two-owner bug and leaves the class intact for the next state that forgets to
  mirror `current_selection`.
- **Remove `current_selection` entirely and read the tool on demand.** Rejected
  as a larger refactor of clipboard, export and render call sites that is not
  needed to fix the defect; the projection is documented here instead.
- **Draw the marquee as a native `DrawRect` with a dashed stroke.** Rejected:
  `RenderCommand::DrawRect` is a filled rectangle with no stroke style, and the
  pixel-buffer path renders no rect stroke at all, so it would need a new
  render-command shape in both paths. The overlay `DrawOp` channel already
  exists, is theme-tinted, and works in both.

## Consequences

`current_selection` is still a field, so a future writer must keep using
`update_select_tool_selection()` / `clear_selection_state()` rather than
assigning it directly — the invariant is stated at both helpers. The marquee is
a character overlay, so it is visible only while the selection is being dragged
and does not follow the committed selection afterwards (the existing filled
highlight does that).

## Verification

- `src/core/tools/select.rs` — 5 tests: a press previews a one-cell marquee, a
  drag previews a dashed outline inside the selection bounds, the marquee clips
  to the grid, a press inside a committed selection moves instead of
  reselecting (bounds preserved), and a press outside starts a new drag.
- `src/wasm/safety_regression_tests.rs` — 2 regressions: the full
  press → marquee → commit-selection → press-inside → move → commit path lands
  the content, and delete followed by a press inside the old rectangle does not
  arm a ghost move.
