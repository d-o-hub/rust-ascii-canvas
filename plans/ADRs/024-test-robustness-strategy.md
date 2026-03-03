# ADR-024: Test Robustness Strategy

## Status
Proposed

## Date
2026-03-03

## Context

The E2E test suite has significant reliability and coverage problems:

1. **85 `waitForTimeout` calls** across `e2e/canvas.spec.ts` — the single biggest source of test flakiness. These are arbitrary sleeps (50ms-500ms) that either:
   - Are too short (causing intermittent failures)
   - Are too long (wasting CI time)
   - Mask underlying race conditions

2. **No Page Object Model** — all 800+ lines of E2E tests directly access page elements, making refactoring the UI extremely expensive.

3. **Single browser testing** — only Chromium is tested in CI. Firefox and WebKit are configured in `playwright.config.ts` but not executed.

4. **No vitest unit tests** — `vitest` is installed as a dev dependency but no test files exist. The `web/main.ts` file (781 lines) has zero unit test coverage.

5. **No ASCII output verification** — E2E tests verify UI interactions but don't assert the actual drawn output. Drawing a rectangle doesn't verify the ASCII characters produced.

## Decision

### 1. Page Object Model (POM)

Create `e2e/pages/EditorPage.ts`:

```typescript
export class EditorPage {
  constructor(private page: Page) {}

  async waitForReady() {
    await expect(this.page.locator('#canvas')).toBeVisible();
    await expect(this.page.evaluate(() => !!window.editor)).toBeTruthy();
  }

  async selectTool(name: string) {
    await this.page.getByRole('button', { name }).click();
    await expect(this.page.getByRole('button', { name })).toHaveClass(/active/);
  }

  async drawRectangle(x: number, y: number, w: number, h: number) { ... }
  async getExportedAscii(): Promise<string> { ... }
  async undo() { ... }
  async redo() { ... }
}
```

### 2. Replace `waitForTimeout`

Systematic replacement strategy:
- **After tool switch**: Replace with `await expect(button).toHaveClass(/active/)`
- **After drawing**: Replace with `await expect(undoButton).toBeEnabled()`
- **After undo/redo**: Replace with state assertions
- **After keyboard input**: Replace with `await expect(statusBar).toContainText(...)`
- **After zoom**: Replace with `await expect(zoomDisplay).toContainText(...)`

### 3. Cross-browser testing

Update CI to run all 3 browser projects:
```yaml
- run: npx playwright test --project=chromium --project=firefox --project=webkit
```

Install all browsers in CI: `npx playwright install --with-deps chromium firefox webkit`

### 4. Vitest frontend tests

Create `web/__tests__/` with tests for:
- Event handler registration
- Coordinate transformation
- Tool state management
- Toast notification logic
- Font measurement utilities

### 5. ASCII output verification

Add tests that:
1. Draw a shape (rectangle, line, etc.)
2. Call `editor.exportAscii()` via page.evaluate
3. Assert the exact ASCII output matches expected

## Consequences

### Positive
- Dramatically reduced flakiness (0 arbitrary sleeps)
- UI refactoring only requires POM updates, not test rewrites
- Cross-browser bugs caught before production
- Frontend logic tested independently of browser
- Actual drawing output verified, not just UI state

### Negative
- Significant upfront effort (~10 hours)
- POM adds indirection layer to understand test intent
- Cross-browser CI runs take 3x longer (mitigate with parallelism)
- Some `waitForTimeout` patterns may need investigation to find proper assertions

### Risks
- Some timing issues may be genuine race conditions in the WASM bridge that require code fixes, not just better assertions
- Firefox/WebKit may expose real bugs that need fixing before tests can pass
