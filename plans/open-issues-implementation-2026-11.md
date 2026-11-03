# Open issues implementation plan — 2026-11

## Scope

The live repository backlog contains issues #297–#303. This plan records the
implementation order and the evidence required before opening the PR.

## Dependency order

1. **#297 / #298 — tool regressions**: keep arrowhead placement and text-input
   focus/dispatch coverage green before changing persistence or rendering.
2. **#302 — Unicode rendering**: add the remaining box-drawing junctions to the
   shared geometry table and the browser atlas after the arrow path is stable.
3. **#299 / #300 — render/build performance**: retain the existing pixel-buffer
   bridge and complete release-profile checks; these are already implemented in
   the base tree and need regression evidence rather than a second bridge.
4. **#303 — durable history and tab synchronization**: persist bounded document
   snapshots with each local draft, restore the selected timeline on reload, and
   accept remote revisions only when the local tab has no unsaved work.
5. **#301 — distribution**: retain ADR-040's app-only decision. Package metadata
   and a registry upload are not introduced because this repository does not
   expose a stable library API and publication requires external credentials.

## Acceptance evidence

- Rust unit/WASM tests cover arrowhead terminal cells, text dispatch, Unicode
  junction atlas geometry, pixel-buffer dimensions, and release profile values.
- Vitest covers history cursor movement, bounded snapshots, shelf migration, and
  safe remote revision application/conflict handling.
- Playwright covers text entry, arrow visibility, reload undo/redo, and a second
  same-origin tab receiving a saved document revision.
- `npm run gate:fast` runs after edits; `npm run gate:full` runs before PR.

## Official references

- [Broadcast Channel API — MDN](https://developer.mozilla.org/en-US/docs/Web/API/Broadcast_Channel_API)
- [Window: storage event — MDN](https://developer.mozilla.org/en-US/docs/Web/API/Window/storage_event)
- [Canvas `tabindex` guidance — MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Global_attributes/tabindex)
- [Cargo release profiles — The Cargo Book](https://doc.rust-lang.org/cargo/reference/profiles.html)
- [WASM-bindgen guide](https://rustwasm.github.io/docs/wasm-bindgen/)
