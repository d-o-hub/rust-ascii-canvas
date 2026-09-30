# Responsive Viewport and Document Dimensions

**Updated 2026-09-30 — ADR-046.** This supersedes the 2026-03-19 instruction to
resize the document on every window resize: that policy cropped existing cells
and cleared every layer's history when a viewport narrowed.

## Initial sizing only

A fresh editor chooses useful defaults only after stylesheets and initial layout
are ready, then font measurement. A positive container height is not proof that
layout is ready: WebKit can expose the unstyled canvas's intrinsic 150px height.
Wait for page load and a completed paint **before creating the editor**, never
correct a provisional size later by resizing a live document.

The final sizing sequence is:

```ts
measureFont();
const { width, height } = computeGridDimensions();
state.editor = new AsciiEditor(width, height);
```

| Initial container width | Maximum columns | Maximum rows |
|-------------------------|-----------------|--------------|
| < 600 px | 60 | 30 |
| < 1024 px | 120 | 50 |
| ≥ 1024 px | 240 | 80 |

Restored/imported documents retain their saved dimensions, regardless of the
current viewport. Initial responsive defaults are not a document resize policy.

## Viewport changes are presentation changes

`web/render.ts:resizeCanvas()` updates the HTML canvas backing dimensions, pixel
ratio and font metrics, then requests rendering. **It must not call
`editor.resize()`**. A smaller window clips the view, not document data; pan/zoom
make the remaining diagram accessible.

## Explicit document resize

The Grid Size Apply action in `web/document-events.ts` is the normal UI path that
changes document dimensions. A shrink requires confirmation because cells outside
the new bounds are cropped and coordinate-based undo histories are cleared.
Cancellation changes neither the document nor its history. Actual resizing
invalidates the offscreen canvas, updates the UI, and schedules persistence.

The WASM API remains `resize(width, height)`, bounded by shared core document
limits (400×200). Invalid or unchanged sizes are no-ops. Generated TypeScript
declarations, not a hand-maintained duplicate interface, describe the binding.

## Verification checklist

- Wait for stylesheet/layout readiness, then measure font metrics before sizing.
- Preserve saved/imported dimensions across desktop/mobile views.
- Draw at an edge, narrow/restore the viewport, then verify content and undo/redo.
- Cancel manual cropping and assert serialized document and history unchanged.
- Invalidate the offscreen canvas when document dimensions actually change.
- Run `e2e/initial-layout.spec.ts`, `e2e/document-safety.spec.ts` and
  `e2e/responsive.spec.ts`; the initial-layout sensor injects a nonzero provisional
  height in every engine, so Chromium-only local full checks catch this race too.
- Viewport-specific tests set their viewport explicitly rather than assuming
  extra project names.
