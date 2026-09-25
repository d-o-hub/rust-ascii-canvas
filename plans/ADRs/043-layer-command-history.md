# ADR-043: Unified Command History for Layer Operations

## Status
Proposed — 2026-09-25 (F-13 cycle; implementation starts with the core model)

## Context

F-11 shipped the full layer editor (rename, visibility, lock, reorder, delete, merge) and issue #111 was closed, but the history half of that issue (F-13) was never implemented. Every structural layer mutation edits state directly and never enters the command history:

| Operation | Binding | History today |
|---|---|---|
| `setLayerVisible` | `src/wasm/render_api.rs:136` | direct mutation |
| `addLayer` | `:151` | direct |
| `renameLayer` | `:157` | direct |
| `setLayerLocked` | `:171` | direct |
| `moveLayer` | `:180` | direct |
| `deleteLayer` | `:186` | direct |
| `mergeLayerDown` | `:192` | direct |
| `setActiveLayer` | `:145` | navigation (correctly not undoable) |

`History` is grid-only: `History::undo(&mut Grid)` replays `Box<dyn Command>` values recorded for grid edits (`src/core/history.rs:13-16, 69-83`). Drawing history survives layer switches, so undo today is *partial* — pixels are protected, structure is not — and `mergeLayerDown` / `deleteLayer` are destructive with no way back. Tracked by issue #207.

## Decision

1. **Pure-core layer model.** Introduce `src/core/layers.rs` with a `Layer` / `LayerStack` value type owned by the editor. Structural state moves out of `src/wasm` so layer commands remain pure data transformations under the architecture rules (`core` must not import `wasm`).
2. **Layer commands implement the existing core `Command` trait** (`src/core/commands/layer.rs`). Each command carries only what it needs to invert itself; no command reaches back into the editor.
3. **One global chronological history.** `History` gains a `Layer(LayerCommand)` variant alongside `Grid { target_layer, command }`. History is not scoped per layer: interleaved grid and layer edits are the only ordering a user can reason about.
4. **Bounded snapshots, not inverse logic.** Metadata ops capture old/new values; `move` captures indices plus the active index; `add` / `delete` capture the whole inserted/removed `Layer`; `mergeLayerDown` captures the upper layer, the overwritten lower-cell deltas, and the active index. Undo is a data restore, so it cannot drift from the apply path.
5. **Semantics.** Undoable: add, delete, merge, move, rename, visibility, lock. Navigation: `setActiveLayer` (not undoable). No-op mutations (e.g. setting a layer visible when it already is) push nothing.
6. **Lock rule.** The "locked layer rejects edits" guard must accept commands originating from undo, otherwise restoring a lock change can deadlock history.
7. **Compatibility.** The `.asc` document format (v1) and autosave payloads are unchanged. The web keeps a single history entry point; no web-level undo is introduced.

## Consequences

- Layer edits become undoable/redoable with exact restoration, including the overlap semantics of merge.
- Undo expectations change: today "undo affects the layer I was drawing on"; after this, undo is chronological across layers. Documented in the runbook and the shortcuts modal.
- `src/wasm/helpers.rs` shrinks (layer logic moves to core) and stays on the LOC allowlist; new files stay under the 500-line budget.
- Estimated +550–750 LOC including tests and the E2E spec.
- ADR-026 (layer system) must be amended to "partial" once the core model lands, verified against real commits.
- Web `undo`/`redo` should surface the affected layer so the new semantics are legible.

## Alternatives considered

- **Inverse commands** (e.g. re-running a move backwards): rejected — brittle for merge, where the inverse depends on the overwritten cell state at apply time.
- **Per-layer histories**: rejected — cannot preserve interleaving of grid and layer edits; switching layers would silently change what undo means.
- **Web-side undo stack**: rejected — breaks the core/render/ui layering rule, cannot be persisted, and would drift from the Rust history.
- **Leave it as a documented gap**: rejected — delete and merge are irreversible for the user today, and the gap was closed by accident in #111.
