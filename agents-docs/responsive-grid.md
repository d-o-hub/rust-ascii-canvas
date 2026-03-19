# Responsive Canvas Grid Learnings (2026-03-19)

## Never Hardcode Grid Dimensions

The WASM `AsciiEditor` grid (`width` × `height`) must **never** be hardcoded as constants. They must always be derived from the container's pixel dimensions at runtime.

**Pattern to avoid:**
```ts
// ❌ WRONG — drawing will clip at the hardcoded column on all screen sizes
const GRID_WIDTH = 80;
const GRID_HEIGHT = 40;
editor = new AsciiEditor(GRID_WIDTH, GRID_HEIGHT);
```

**Correct pattern:**
```ts
// ✅ CORRECT — compute from container dimensions
// measureFont() MUST run before this so charWidth/lineHeight are ready
measureFont();
const { width, height } = computeGridDimensions();
editor = new AsciiEditor(width, height);
```

## resizeCanvas() Must Also Resize the Editor Grid

Resizing only the HTML `<canvas>` element leaves the WASM grid at its original dimensions. Every window.resize must also call `editor.resize()`:

```ts
const { width, height } = computeGridDimensions();
if (editor.width !== width || editor.height !== height) {
    editor.resize(width, height);
}
offscreenCanvas = null; // force offscreen canvas recreation
offscreenCtx    = null;
```

## WASM Must Expose a resize() Method

The Rust `AsciiEditor` must expose a `#[wasm_bindgen]` `resize()` method that reallocates the grid and pixel buffer while preserving existing content.

## Responsive Breakpoints for Grid Caps

| Viewport | maxCols | maxRows |
| :--- | :--- | :--- |
| < 600 px (mobile) | 60 | 30 |
| < 1024 px (tablet) | 120 | 50 |
| ≥ 1024 px (desktop) | 240 | 80 |

## Agent Checklist — Canvas Grid Changes

When any agent modifies grid initialization, canvas sizing, or WASM editor construction, verify:
* [ ] `GRID_WIDTH` / `GRID_HEIGHT` constants removed — use `computeGridDimensions()`
* [ ] `measureFont()` is called before `computeGridDimensions()`
* [ ] `resizeCanvas()` calls `editor.resize()` when dimensions change
* [ ] `offscreenCanvas` is nullified after every grid resize
* [ ] Rust `AsciiEditor` has a `pub fn resize()` wasm_bindgen method
* [ ] `AsciiEditorInterface` TypeScript interface includes `resize()`
* [ ] E2E tests cover drawing near the right/bottom edge on mobile + desktop viewports
* [ ] `playwright.config.ts` includes mobile + tablet + desktop device projects
