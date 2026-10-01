---
name: tool-validation
description: Behaviour harness for the 8 ASCII Canvas tools. Use when changing tools, keyboard/pointer routing, canvas interactions, border styles, history boundaries or viewport transforms.
---

# Tool Validation

Use focused Rust/Vitest tests while iterating, then Playwright for actual browser
routing. Passing one shape screenshot is not sufficient. Record failing-before
reproductions and the tests actually run; never mark an unchecked criterion green.
See [the harness](../../../agents-docs/harness.md) and [verify](../verify/SKILL.md).

## All tools: critical invariants

- Blank/unknown tool or style identifiers and unrecognized keys are non-mutating;
  no first-character truncation, panic, invented space or accidental tool switch.
- Text input owns printable keys, **including literal Space**, while Text is
  active. Space must not start panning or get swallowed by global shortcuts.
  Named browser keys (`Shift`, `Control`, `F1`, `Dead`, etc.) never insert text.
- Typing in an input, textarea, select or editable UI control does not draw,
  switch tools, pan, delete canvas cells, or trigger canvas undo/redo.
- A freehand/eraser drag is **one atomic undo step**, from down through up;
  undo restores the pre-stroke content, redo restores the entire stroke, including
  crossings/revisited cells. A no-op neither creates a history entry nor discards redo;
  interaction/chrome redraws are separate from document changes.
- Accepted paste during an unfinished stroke cancels its provisional cells before
  recording paste undo values. Both paste paths survive pointer-up/cancel and
  exact undo/redo; empty/rejected paste leaves the gesture alone.
- Locked-layer write rejection remains non-mutating across all entry points,
  including direct WASM calls; rejected actions must not consume history.
  Hidden layers stay excluded from composite rendering/export.
- Switching tools/layers/documents, successful load/clear/resize, cancellation
  and pointer loss reset transient state coherently. No preview, text cursor,
  selection or pending stroke leaks into a later operation. Failed loads leave
  the existing document and usable history intact. Test the chosen cancellation
  policy explicitly rather than guessing whether it commits or discards a stroke.
- Draft switching starts a fresh viewport. Window resize must preserve document
  dimensions/content/history; explicit manual cropping requires confirmation.
  Test mobile viewport and a heavily zoomed/panned draft switch, not just the
  default desktop view. Direct WASM load/resize retains the existing viewport;
  callers can reset pan/zoom explicitly when replacing a document.
- Export reflects committed visible content only: no preview, selection/cursor
  overlay, viewport clipping or zoom-dependent dimensions. Export itself must not
  alter editor state or history.

## Per-tool acceptance

| Tool | Required checks |
|------|-----------------|
| Select (V) | Drag highlight; move selected content; Delete/Backspace clear only selected cells; undo/redo exact content; reset selection at document boundaries. |
| Rectangle (R) | Preview uses current border style; release commits once; reverse-direction and edge drags clamp safely; cancellation removes preview. |
| Line (L) | Horizontal/vertical/diagonal; Auto/Horizontal/Vertical overrides; single-cell and reversed drags; preview is not committed data. |
| Arrow (A) | Correct directional head (`▲▼◄►`) at the endpoint; shaft never overwrites it; all directions and small drags. |
| Diamond (D) | Rhombus (`╱╲`); tiny/single-point fallback (`◆`); reversed drag and bounds. |
| Text (T) | Click sets cursor; printable characters/Space insert; Enter/newline, Backspace and Delete; non-text browser keys ignored; Escape releases text ownership; boundary reset. |
| Freehand (F) | Correct style character (below); fast interpolation remains continuous; one complete stroke per undo/redo, including revisits and same-value cells. |
| Erase (E) | Radius 1/3/5 clears intended cells to spaces; edge/corner operations stay in bounds; one complete stroke per undo/redo; blank erase is a no-op. |

## Freehand style oracle

The source of truth is `BorderStyle::freehand_char` in
[src/core/tools/mod.rs](../../../src/core/tools/mod.rs), not the constructor's
fallback character. Pointer-down selects the style character:

| Style | Character |
|-------|-----------|
| Single | `─` |
| Double | `═` |
| Heavy | `━` |
| Rounded | `─` |
| ASCII | `-` |
| Dotted | `*` |

## Automation

```bash
cargo test --lib
(cd web && pnpm test)
npx playwright test --project=chromium e2e/tools-drawing.spec.ts
```

Extend the relevant focused files for routing/history/document boundaries; the
suite is not limited to `tools-drawing.spec.ts`. Playwright owns local startup,
readiness and teardown. Before delivery, follow `verify` for full production-shaped
checks and the CI browser matrix. Record native, WASM and browser evidence
separately: native unit tests alone cannot prove browser keyboard behavior.
