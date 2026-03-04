# ADR-035: Fix Page Object Model Selector Mismatches

## Status
Proposed

## Date
2026-03-03

## Context

`e2e/pages/EditorPage.ts` (284 LOC) was created as the Page Object Model for the ASCII Canvas Editor. However, a cross-reference audit between the POM's selectors and the actual HTML in `web/index.html` reveals **two broken selectors** that will cause test failures when the POM is actually used.

### Mismatch 1: `#grid-info` Does Not Exist

**In `EditorPage.ts` (line 89)**:
```typescript
this.statusBar = {
    container: page.locator('.status-bar'),
    cursorPosition: page.locator('#cursor-pos'),
    gridInfo: page.locator('#grid-info'),      // ← BROKEN
    zoomLevel: page.locator('#zoom-level'),
};
```

**In `web/index.html`** — searching for `grid-info`:
```bash
$ grep -n "grid-info" web/index.html
(no output)
```

The actual element ID in `index.html` is `grid-size` (line 122):
```html
<div class="info-item">
    <span class="info-label">Grid</span>
    <span class="info-value" id="grid-size">80 × 40</span>
</div>
```

And in `web/main.ts` (line 74):
```typescript
gridSizeEl = getElement('grid-size');
```

The POM uses `#grid-info` but the real ID is `#grid-size`. Any test calling `editorPage.statusBar.gridInfo` will fail with a timeout because the element doesn't exist.

### Mismatch 2: `.modal .close` Does Not Exist

**In `EditorPage.ts` (line 95)**:
```typescript
this.modal = {
    container: page.locator('.modal'),
    closeButton: page.locator('.modal .close'),  // ← BROKEN
};
```

**In `web/index.html` (line 210)**:
```html
<button class="modal-close" aria-label="Close modal">&times;</button>
```

The actual close button class is `modal-close`, not `close`. The correct selector is `.modal-close` or `.modal-overlay .modal-close`.

Additionally, the modal container selector `.modal` is also wrong — the actual class is `modal-overlay`:
```html
<div id="shortcuts-modal" class="modal-overlay hidden" role="dialog" ...>
    <div class="modal-content">
```

So both selectors in the `modal` object are broken:
- `page.locator('.modal')` → should be `page.locator('.modal-overlay')` or `page.locator('#shortcuts-modal')`
- `page.locator('.modal .close')` → should be `page.locator('.modal-close')`

### Impact Assessment

These mismatches are currently **latent bugs** because neither `canvas.spec.ts` nor `tools-drawing.spec.ts` imports `EditorPage` (confirmed by grep — zero imports). However:

1. When ADR-034 is implemented (migrating tests to use the POM), these broken selectors will cause immediate test failures.
2. The `getGridInfo()` method in the POM will always return `{ width: 0, height: 0 }` because `#grid-info` never resolves.
3. The `hideShortcutsModal()` method will fail because `.modal .close` doesn't exist.

### Additional POM Issues Found

**`getCursorPosition()` regex mismatch** (line 196):
```typescript
async getCursorPosition(): Promise<{ x: number; y: number }> {
    const text = await this.statusBar.cursorPosition.textContent();
    const match = text?.match(/X:\s*(\d+)\s*Y:\s*(\d+)/);  // ← expects "X: 5 Y: 10"
    if (match) {
        return { x: parseInt(match[1], 10), y: parseInt(match[2], 10) };
    }
    return { x: 0, y: 0 };
}
```

But `main.ts` sets cursor position as (line 258):
```typescript
cursorPosEl.textContent = `${gridX}, ${gridY}`;  // ← format: "5, 10"
```

The regex expects `X: 5 Y: 10` but the actual format is `5, 10`. The method will always return `{ x: 0, y: 0 }`.

**`getGridInfo()` regex mismatch** (line 204):
```typescript
async getGridInfo(): Promise<{ width: number; height: number }> {
    const text = await this.statusBar.gridInfo.textContent();
    const match = text?.match(/(\d+)\s*x\s*(\d+)/);  // ← expects "80 x 40"
```

But `main.ts` sets grid size as (line 571):
```typescript
gridSizeEl.textContent = `${editor.width} × ${editor.height}`;  // ← uses × (U+00D7), not x
```

The regex uses lowercase `x` but the actual separator is `×` (Unicode multiplication sign). The method will always return `{ width: 0, height: 0 }`.

### Summary of All POM Issues

| Issue | POM Location | Actual HTML/JS | Severity |
|---|---|---|---|
| `#grid-info` selector | line 89 | `#grid-size` | **High** — element not found |
| `.modal .close` selector | line 95 | `.modal-close` | **High** — element not found |
| `.modal` container | line 94 | `.modal-overlay` or `#shortcuts-modal` | **Medium** — wrong element |
| `getCursorPosition()` regex | line 196 | format is `"5, 10"` not `"X: 5 Y: 10"` | **High** — always returns 0,0 |
| `getGridInfo()` regex | line 204 | uses `×` not `x` | **High** — always returns 0,0 |

## Decision

Fix all five POM issues to match the actual HTML structure and JavaScript output format.

### Fix 1: `#grid-info` → `#grid-size`

```typescript
// BEFORE
gridInfo: page.locator('#grid-info'),

// AFTER
gridSize: page.locator('#grid-size'),
```

Note: rename the property from `gridInfo` to `gridSize` to match the actual ID and avoid confusion.

### Fix 2: Modal selectors

```typescript
// BEFORE
this.modal = {
    container: page.locator('.modal'),
    closeButton: page.locator('.modal .close'),
};

// AFTER
this.modal = {
    container: page.locator('#shortcuts-modal'),
    closeButton: page.locator('.modal-close'),
};
```

### Fix 3: `getCursorPosition()` regex

```typescript
// BEFORE
const match = text?.match(/X:\s*(\d+)\s*Y:\s*(\d+)/);

// AFTER — matches "5, 10" format from main.ts line 258
const match = text?.match(/(-?\d+),\s*(-?\d+)/);
```

Note: use `-?\d+` to handle negative coordinates (when cursor is outside the grid).

### Fix 4: `getGridInfo()` regex

```typescript
// BEFORE
const match = text?.match(/(\d+)\s*x\s*(\d+)/);

// AFTER — matches "80 × 40" format (Unicode × U+00D7)
const match = text?.match(/(\d+)\s*×\s*(\d+)/);
```

### Fix 5: Update `getGridInfo()` to use renamed property

```typescript
// BEFORE
async getGridInfo(): Promise<{ width: number; height: number }> {
    const text = await this.statusBar.gridInfo.textContent();

// AFTER
async getGridSize(): Promise<{ width: number; height: number }> {
    const text = await this.statusBar.gridSize.textContent();
```

## Consequences

### Positive
- POM selectors match actual DOM — tests using the POM will work correctly
- `getCursorPosition()` and `getGridSize()` return real values instead of always `{ x: 0, y: 0 }`
- Prevents a class of silent test failures where assertions pass vacuously (e.g., `expect(0).toBe(0)`)
- Establishes a pattern: POM selectors must be verified against actual HTML before merging

### Negative
- Any existing code calling `editorPage.statusBar.gridInfo` must be updated to `gridSize`
- Any existing code calling `editorPage.getGridInfo()` must be updated to `getGridSize()`
- Currently zero tests use the POM, so the rename impact is zero today

## Implementation Plan

1. **Fix `#grid-info` → `#grid-size`** in `EditorPage.ts` line 89. Rename property `gridInfo` → `gridSize`.
2. **Fix modal selectors** in `EditorPage.ts` lines 94–95: use `#shortcuts-modal` and `.modal-close`.
3. **Fix `getCursorPosition()` regex** in `EditorPage.ts` line 196: use `/(-?\d+),\s*(-?\d+)/`.
4. **Fix `getGridInfo()` regex and rename** in `EditorPage.ts` line 204: use `/(\d+)\s*×\s*(\d+)/`, rename to `getGridSize()`.
5. **Add a POM validation test**: create `e2e/pom-validation.spec.ts` that exercises every POM method and asserts non-null/non-zero results.
6. **Verify**: `npx playwright test e2e/pom-validation.spec.ts` passes.

## Prevention

Add a CI check: after any change to `web/index.html`, run a selector audit script that verifies all POM selectors exist in the DOM:

```typescript
// scripts/validate-pom-selectors.ts
// Reads EditorPage.ts, extracts all locator() calls, 
// checks each selector against index.html
```

## Alternatives Considered

### A. Delete the POM and use raw Playwright selectors
- **Rejected**: The POM provides valuable abstraction. The fix is to correct the selectors, not abandon the pattern.

### B. Auto-generate the POM from index.html
- **Considered for future**: A code generator that reads `index.html` and produces typed locators would prevent this class of bug. Deferred to a future ADR.

## References
- [ADR-024: Test Robustness Strategy](./024-test-robustness-strategy.md)
- [ADR-034: Eliminate waitForTimeout](./034-e2e-waitfortimeout-elimination.md)
- `e2e/pages/EditorPage.ts` — the POM file
- `web/index.html` — the actual DOM structure
- `web/main.ts` lines 258, 571 — cursor and grid size format strings
