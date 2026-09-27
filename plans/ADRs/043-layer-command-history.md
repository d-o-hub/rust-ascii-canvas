# ADR-043: Undoable Layer Operations

## Status
Proposed — 2026-09-25, **corrected after a design review** (the first draft asserted a seam that does not exist in the code and a history model with a prohibitive cost). Not yet implemented; see "Open decisions" before any code lands. Tracked by issue #207.

## Context

F-11 shipped the layer editor (rename, visibility, lock, reorder, delete, merge) and issue #111 was closed, but its history half (F-13) was never implemented. Every structural layer mutation edits state directly and never enters a command history:

| Operation | Binding | Undoable today |
|---|---|---|
| `setLayerVisible` | `src/wasm/render_api.rs:136` | no |
| `addLayer` | `:151` | no |
| `renameLayer` | `:157` | no |
| `setLayerLocked` | `:171` | no |
| `moveLayer` | `:180` | no |
| `deleteLayer` | `:186` | no |
| `mergeLayerDown` | `:192` | no |
| `setActiveLayer` | `:145` | n/a — navigation |

Drawing history is per layer and already works: `LayerData { name, visible, locked, grid, history }` (`src/wasm/bindings.rs:48-56`), with the active layer mirrored into `state.grid` (`bindings.rs:41-43`) so a restore is a grid swap. What is missing is structural undo.

Two facts from the code shape the decision, and both contradict the first draft of this ADR:

1. **`Command` is Grid-only.** `apply`/`undo` take `&mut Grid` (`src/core/commands/mod.rs:13-18`), with `can_merge`/`merge`/`as_any` alongside. A layer command **cannot** implement it as written; the trait is not generic today.
2. **A single global timeline is prohibitively expensive here.** Because history is per layer and only the active grid is mirrored, a global entry would have to snapshot the affected layer wholesale — roughly 400 × 200 × 32 ≈ 2.5M cells per entry — against today's cell-list draw commands (`src/core/commands/draw.rs`). Composite render also reads two sources (`src/wasm/helpers.rs:580-593`), and delete/merge have no inverse to recompute from, so divergence would be silent.

## Decision

1. **Per-layer history stays; it is extended, not replaced.** `LayerData.history` (`bindings.rs:55`) keeps owning that layer's history, and layer commands push into the **active layer's** history. Undo/redo therefore applies to the layer you are on, matching today's behaviour for drawing.
2. **A separate core trait, not a modified `Command`.** `src/core/commands/layer.rs` defines `LayerCommand { apply/undo(&mut self, stack: &mut LayerStack), description, as_any, as_any_mut }`; `draw.rs` and `composite.rs` are untouched. The history side turned out differently than first written: instead of a second `LayerHistory` type, a layer's existing `History` now holds **both** kinds of entry behind one `HistoryEntry` enum (`src/core/history.rs`). A separate stack would have made "draw, rename, draw, undo, undo" ambiguous, and per-layer history is only useful if it is one timeline.
3. **Pure core layer model.** `src/core/layer.rs` holds `Layer` (metadata, grid, its own history) and `src/core/layer_stack.rs` holds `LayerStack`; the split keeps both files inside the 500-line budget. The `wasm::LayerData` struct and the `std::mem::take`/`swap` dance that swapped the active layer's history in and out of the editor on every switch are gone. The `state.grid` mirror and every existing JS method signature stay unchanged; `undoLabel`/`redoLabel` are additive getters.
4. **Bounded snapshots, not inverse logic.** Metadata ops carry old/new values; `move` carries indices plus the active index; `add`/`delete` carry the whole inserted/removed `Layer`; `mergeLayerDown` carries the upper layer, the overwritten lower-cell deltas, and the active index. Undo is a data restore, so it cannot drift from apply.
5. **Semantics.** Undoable: add, delete, merge, move, rename, visibility, lock. Navigation (`setActiveLayer`) is not undoable. No-op mutations push nothing.
6. **Lock guard.** The early return in `undo`/`redo` is gone: `History::undo`/`redo` consult the lock themselves, and only when the top entry is a grid command. A blocked undo is refused **without consuming the entry**, so unlocking frees it again. Every other guard site keeps blocking: `event_handlers.rs:18,69,107,196,224`, `helpers.rs` draw/paste/clipboard sites, `clear` (`bindings.rs`).
7. **Persistence is frozen at v1.** Serialization stays byte-compatible: `format`, `version:1`, `canvas{width,height}`, `active_layer`, `layers[]{name,visible,locked,cells[{x,y,ch}]}` (`src/wasm/helpers.rs:605-618`, `:650-659`), with the `#[serde(default)]` attributes on `active_layer` (`:701`) and `visible`/`locked` (`:679-682`) preserved. Do **not** bump `version`: the gate rejects `version == 0` and nothing else (`helpers.rs:710`), so a bump to 2 would silently keep accepting v1.
8. **Placement respects the sensors.** Architecture: core may only import core — `scripts/check-architecture.sh:55-59` bans `crate::(wasm|render|ui)` inside `src/core`, and `:83-90` bans `wasm_bindgen`/`web_sys`/`js_sys`; do not drag `FontAtlas` or `Theme` (`bindings.rs:38,45`) into core. LOC: `src/wasm/render_api.rs` is 489/500, so new `#[wasm_bindgen]` wiring goes in a new `src/wasm/layer_api.rs`; `src/wasm/helpers.rs` (1120, allowlisted) must not grow, and `quality-gates.sh:79` forbids widening `.loc-allowlist` without an ADR. New core files each stay under 500 lines.

## Identity: why layers carry an id

Structural undo has to answer "which layer was active before this change?" and indices cannot answer it, because inserting and removing layers renumbers them. Every `Layer` therefore carries a stable `id`, and the stack owns the remapping in one place (`remove_layer_tracking`, `insert_layer_tracking`, `move_layer`, `set_active_id`). Two bugs found by the test matrix came straight from getting this wrong, and both would have shipped as "the wrong layer became active after undo":

- commands that derived the active index from a pre-apply snapshot, and
- handing a layer's history back by index, which overwrote the history of a *different* layer that undo had just restored at that index.

The history is now returned to the layer it came from **by id**; if that layer is gone (undoing "add layer"), the history dies with it instead of leaking onto another layer.

## Known limits (deliberate, from per-layer history)

- Redo of a structural change is reachable only from the layer that recorded it. Undoing a merge or a delete returns focus to the restored layer, while the redo entry sits on the layer that was active afterwards, so redo needs a layer switch first. Recording on the pre-change layer instead would move the same wart onto "add layer then Ctrl+Z", which is the more common action.
- A structural command is recorded on the layer that is active *after* it runs, so "add layer" is undoable immediately.
- Undo of a restored layer does not restore that layer's *redo* stack, only its undo history.

## Resize and history

`resize` resizes every layer and then clears **every** layer's history. The first draft of this section said "the active layer only", which is wrong: `Grid::resize` keeps content, so recorded draw commands (coordinates, and a width/height for clear-canvas) in *every* layer would replay onto grids they no longer describe.

## Evidence and cost

Sizes before: `history.rs` 200, `commands/mod.rs` 245, `bindings.rs` 330, `render_api.rs` 489, `helpers.rs` 1120 (allowlisted). After: `core/layer.rs` 163, `core/layer_stack.rs` 396, `core/commands/layer.rs` 398, `core/history.rs` 464, `wasm/layer_api.rs` 467, `bindings.rs` 288, `render_api.rs` 396, `helpers.rs` 939 (shrunk: the layer mutation logic moved to core). `helpers.rs` did not grow and `.loc-allowlist` is unchanged.

## Test matrix (each claim needs an observable test)

| Claim | Test |
|---|---|
| each mutation is one step with exact undo **and** redo | per-op unit tests in `core/commands/layer.rs` + `e2e/layers.spec.ts` |
| merge undo is atomic and restores the overlap | merge test in `layer.rs` (one undo restores both layers and overlapped cells) + spec |
| no-ops and navigation add no history | history length unchanged; `web/ux.test.ts` asserts `undo() === false` |
| undo of a lock change works and does not deadlock | `SetLocked` unit test + spec exercising undo while locked |
| undo after a layer switch hits that layer's stack | interleaving unit test + spec |
| v1 documents still load | **new** golden-fixture test loading the literal v1 JSON (shape at `helpers.rs:856-862`, `:877`); the existing `test_serialize_load_round_trip` (`helpers.rs:836-843`) only proves round-trip |

## Definition of done for #207

`core/layer.rs`, `core/commands/layer.rs`, `wasm/layer_api.rs` landed; all seven ops with exact undo/redo; JS `.d.ts` unchanged; `helpers.rs` not grown; `gate:full` green including the new E2E spec; PR merged. **Only then** verify against `git log` and move this ADR Proposed → Accepted, close #207, and amend ADR-026 to "partial" (or "full" if web parity is proven).

## Definition of not done

A demo or CLI-only proof; one op unit-tested; docs or shortcuts-modal wording updated without the core model; ADR status flipped before merge; E2E green without unit tests; weakened or skipped tests; per-layer history replaced by a global stack (see Open decision 1); `.loc-allowlist` widened to fit the change.

## Decisions (answered 2026-09-25)

1. **Cross-layer undo: no.** Per-layer history. A single global stack would cost ~2.5M cells per entry at current grid sizes. Revisit only if cross-layer undo is shown to be needed, or with a cheaper command representation.
2. **Seam shape: a separate `LayerCommand` trait**, not a genericised `Command<T>`, which would touch every grid command and the history internals for no user-visible gain.
3. **Blocked-grid behaviour: skip and keep.** When the top entry is a grid command and the active layer is locked, undo is refused and the entry stays; unlocking frees it again. A lock must never dead-end undo, and `canUndo` stays honest about there being something to undo.
4. **Labelling: additive `undoLabel`/`redoLabel` getters**, surfaced in the undo/redo tooltips and `aria-label`s (`web/ui.ts`), e.g. "Undo Hide layer (Ctrl+Z)". No new modal.
5. **Resize/history invalidation: resize clears every layer's history** (see above).

## Alternatives considered

- **Global chronological history**: rejected on cost (2.5M cells/entry) and on composite-render divergence for delete/merge.
- **Inverse commands**: rejected — merge's inverse depends on the overwritten cell state at apply time.
- **Web-side undo stack**: rejected — breaks the core/render/ui layering rule, cannot be persisted, drifts from Rust history.
- **Document as a known gap**: rejected — delete and merge are irreversible for the user today, and #111 closed the issue without them.
