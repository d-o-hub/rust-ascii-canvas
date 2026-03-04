# ADR-034: Eliminate waitForTimeout from E2E Tests

## Status
Proposed

## Date
2026-03-03

## Context

The E2E test suite contains **116 `waitForTimeout` calls** across two test files:

```bash
$ grep -c "waitForTimeout" e2e/canvas.spec.ts
85

$ grep -c "waitForTimeout" e2e/tools-drawing.spec.ts
31
```

`waitForTimeout` is a **fixed-duration sleep** — it pauses the test for a hardcoded number of milliseconds regardless of whether the application is ready. This is the primary cause of flaky tests in CI environments where timing differs from local development.

### Evidence from the Files

**Pattern 1: `beforeEach` initialization sleep** (appears in every `describe` block in `canvas.spec.ts`):
```typescript
test.beforeEach(async ({ page }) => {
    await page.goto(BASE_URL);
    await page.waitForSelector('#loading.hidden', { timeout: 15000 });
    await page.waitForSelector('#canvas', { timeout: 10000 });
    await page.waitForTimeout(500);  // ← arbitrary 500ms sleep after load
});
```
This pattern appears **6 times** (once per `describe` block). The 500ms sleep is meant to ensure WASM is initialized, but it's a guess — on a slow CI machine it may not be enough; on a fast machine it wastes 500ms per test.

**Pattern 2: Post-draw settlement sleep**:
```typescript
test('should select rectangle tool and draw', async ({ page }) => {
    await page.click('[data-tool="rectangle"]');
    // ...
    await drawOnCanvas(page, 100, 100, 300, 200);
    await page.waitForTimeout(300);  // ← wait for render to settle
});
```
This appears after nearly every drawing operation. The 300ms is meant to wait for the `requestAnimationFrame` render cycle, but `rAF` completes in ~16ms — the 300ms is 18× too conservative.

**Pattern 3: Post-keyboard-event sleep**:
```typescript
await page.keyboard.press('l');
await page.waitForTimeout(100);  // ← wait for tool switch
const lineButton = page.locator('[data-tool="line"]');
await expect(lineButton).toHaveClass(/active/);
```
The `waitForTimeout(100)` before the assertion is unnecessary because `expect(...).toHaveClass()` already has a built-in retry timeout (configured as 10 seconds in `playwright.config.ts`).

**Pattern 4: Sequential character typing delays**:
```typescript
await page.keyboard.type('A');
await page.waitForTimeout(50);
await page.keyboard.type('B');
await page.waitForTimeout(50);
// ... repeated 5 times
```
These 50ms delays between keystrokes are meant to prevent event queue overflow, but Playwright's `keyboard.type()` already handles this correctly.

### Why `waitForTimeout` Causes Flakiness

1. **CI machines are slower**: A 100ms sleep that works locally may be insufficient on a GitHub Actions runner under load.
2. **CI machines are faster**: A 500ms sleep that was needed on a slow machine wastes time on fast runners, increasing total test time.
3. **No retry logic**: If the condition isn't met when the sleep ends, the test fails immediately. Playwright's built-in `expect` retries for up to `expect.timeout` (10s).
4. **ADR-031 evidence**: The CI failure rate of 18.4% (14/76 tests) is directly attributable to timing-sensitive `waitForTimeout` calls.

### The `EditorPage` POM Exists But Is Not Used

`e2e/pages/EditorPage.ts` (284 LOC) was created as part of ADR-024 but **neither `canvas.spec.ts` nor `tools-drawing.spec.ts` imports it**:

```bash
$ grep -n "EditorPage" e2e/canvas.spec.ts
(no output)

$ grep -n "EditorPage" e2e/tools-drawing.spec.ts
(no output)
```

The POM provides proper `waitForLoad()` without `waitForTimeout`, but the test files bypass it entirely. Additionally, the POM itself contains one `waitForTimeout(50)` in `selectToolByShortcut()` and one `waitForTimeout(100)` that should be replaced.

### WASM Initialization Detection

The root cause of the `waitForTimeout(500)` in `beforeEach` is that there is no reliable signal for "WASM is fully initialized and the editor is ready." The current approach:

1. Wait for `#loading.hidden` — the loading overlay is hidden
2. Wait for `#canvas` to be visible
3. Sleep 500ms hoping WASM is done

A proper signal would be: wait until `window.editor !== null`, which is set synchronously after `new AsciiEditor()` succeeds in `initialize()`.

## Decision

### Strategy: Replace all `waitForTimeout` with deterministic assertions

#### Fix 1: WASM Ready Signal

Add a `data-ready` attribute to the canvas element after initialization completes:

```typescript
// In web/main.ts, at the end of initialize():
canvas.setAttribute('data-ready', 'true');
```

Then in E2E tests:
```typescript
// BEFORE
await page.waitForSelector('#loading.hidden', { timeout: 15000 });
await page.waitForSelector('#canvas', { timeout: 10000 });
await page.waitForTimeout(500);

// AFTER
await page.waitForSelector('#canvas[data-ready="true"]', { timeout: 15000 });
```

This is a single, deterministic selector that only resolves when the editor is fully initialized.

#### Fix 2: Post-draw assertions instead of sleeps

```typescript
// BEFORE
await drawOnCanvas(page, 100, 100, 300, 200);
await page.waitForTimeout(300);

// AFTER
await drawOnCanvas(page, 100, 100, 300, 200);
// No sleep needed — subsequent assertions use Playwright's built-in retry
await expect(page.locator('#undo-btn')).toBeEnabled();
// OR for output verification:
const ascii = await page.evaluate(() => window.editor?.exportAscii() ?? '');
expect(ascii).not.toMatch(/^(\s*\n)*$/);
```

#### Fix 3: Remove post-keyboard-event sleeps

```typescript
// BEFORE
await page.keyboard.press('l');
await page.waitForTimeout(100);
await expect(lineButton).toHaveClass(/active/);

// AFTER
await page.keyboard.press('l');
await expect(lineButton).toHaveClass(/active/);  // retries for up to 10s
```

#### Fix 4: Use `keyboard.type()` without inter-character delays

```typescript
// BEFORE
await page.keyboard.type('A');
await page.waitForTimeout(50);
await page.keyboard.type('B');
await page.waitForTimeout(50);

// AFTER
await page.keyboard.type('ABCDE');  // Playwright handles timing internally
```

#### Fix 5: Migrate all tests to use `EditorPage` POM

```typescript
// BEFORE (canvas.spec.ts)
import { test, expect } from '@playwright/test';

test.beforeEach(async ({ page }) => {
    await page.goto(BASE_URL);
    await page.waitForSelector('#loading.hidden', { timeout: 15000 });
    await page.waitForSelector('#canvas', { timeout: 10000 });
    await page.waitForTimeout(500);
});

// AFTER
import { test, expect } from '@playwright/test';
import { EditorPage } from './pages/EditorPage.js';

test.beforeEach(async ({ page }) => {
    const editor = new EditorPage(page);
    await page.goto(BASE_URL);
    await editor.waitForLoad();  // uses data-ready selector, no sleep
});
```

#### Fix 6: Remove `waitForTimeout` from `EditorPage.ts` itself

```typescript
// EditorPage.ts line ~120 - selectToolByShortcut
async selectToolByShortcut(shortcut: string): Promise<void> {
    await this.canvas.focus();
    await this.page.waitForTimeout(50);  // ← REMOVE THIS
    await this.page.keyboard.press(shortcut.toLowerCase());
    await this.page.waitForTimeout(100); // ← REMOVE THIS
}

// AFTER
async selectToolByShortcut(shortcut: string): Promise<void> {
    await this.canvas.focus();
    await this.page.keyboard.press(shortcut.toLowerCase());
    // Caller asserts the expected state change
}
```

### Target State

| Metric | Current | Target |
|---|---|---|
| `waitForTimeout` in `canvas.spec.ts` | 85 | 0 |
| `waitForTimeout` in `tools-drawing.spec.ts` | 31 | 0 |
| `waitForTimeout` in `EditorPage.ts` | 2 | 0 |
| Tests using `EditorPage` POM | 0 | 100% |
| WASM ready detection | 500ms sleep | `data-ready` attribute |

## Consequences

### Positive
- Eliminates the primary source of CI flakiness (ADR-031 failure rate: 18.4%)
- Tests run faster: removing 116 × avg 200ms = ~23 seconds of unnecessary sleep per full run
- Tests are more reliable: Playwright's retry logic handles transient slowness
- `EditorPage` POM is actually used, justifying its existence
- `data-ready` attribute provides a testable initialization contract

### Negative
- Requires modifying `web/main.ts` to add `data-ready` attribute (small change)
- All 116 `waitForTimeout` calls must be individually reviewed and replaced (~6 hours)
- Some tests may need new intermediate assertions to replace the implicit "wait for render" that `waitForTimeout` provided

## Implementation Plan

1. **Add `data-ready` attribute** to `web/main.ts` at end of `initialize()`: `canvas.setAttribute('data-ready', 'true')`.
2. **Update `EditorPage.waitForLoad()`** to use `#canvas[data-ready="true"]` selector.
3. **Remove `waitForTimeout` from `EditorPage.ts`** (2 occurrences in `selectToolByShortcut`).
4. **Migrate `canvas.spec.ts` `beforeEach`** blocks (6 occurrences) to use `EditorPage.waitForLoad()`.
5. **Replace post-draw `waitForTimeout(300)`** calls with `expect(undoBtn).toBeEnabled()` or ASCII content assertions.
6. **Replace post-keyboard `waitForTimeout(100)`** calls — remove entirely (Playwright retries handle this).
7. **Replace inter-character `waitForTimeout(50)`** calls — use `keyboard.type('ABCDE')` directly.
8. **Run full E2E suite** 3 times to verify zero flakiness: `npx playwright test --repeat-each=3`.

## Alternatives Considered

### A. Increase `waitForTimeout` values to reduce flakiness
- **Rejected**: Makes tests slower and still flaky on very slow machines. Does not fix the root cause.

### B. Use `page.waitForFunction()` for each operation
- **Partially accepted**: `waitForFunction(() => window.editor !== null)` is used for the WASM ready check. For post-draw assertions, Playwright's built-in `expect` retry is preferred as it's more readable.

### C. Add `--retries=2` to CI configuration
- **Rejected**: Already present (`retries: process.env.CI ? 1 : 0` in `playwright.config.ts`). Retries mask flakiness rather than fixing it.

## References
- [ADR-024: Test Robustness Strategy](./024-test-robustness-strategy.md)
- [ADR-031: GitHub Monitoring and CI Fix Strategy](./031-github-monitoring-ci-fixes.md)
- [Playwright Best Practices: Avoid waitForTimeout](https://playwright.dev/docs/best-practices#avoid-using-waitfortimeout)
- `playwright.config.ts`: `expect.timeout: 10000` (10s retry window)
