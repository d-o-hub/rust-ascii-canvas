import type { Page } from '@playwright/test';

/** Clear browser-local documents so tests start from a blank editor. */
export async function clearAutosave(page: Page): Promise<void> {
    await page.addInitScript(() => {
        try {
            localStorage.removeItem('ascii-canvas-autosave');
            localStorage.removeItem('ascii-canvas-drafts-v1');
        } catch {
            /* ignore */
        }
    });
}

/**
 * Navigate and wait for the editor to finish loading.
 * Note: `#loading.hidden` uses `display: none`, so we must use `state: 'attached'`
 * (not the default `visible`).
 */
export async function openEditor(page: Page): Promise<void> {
    await clearAutosave(page);
    // Relative: Playwright resolves it against `use.baseURL` in
    // playwright.config.ts, which is the single source of truth for the host.
    // A literal here would be a second copy to keep in sync — and the two had
    // already drifted apart (localhost vs 127.0.0.1).
    await page.goto('/');
    await page.waitForSelector('#loading.hidden', { state: 'attached', timeout: 30000 });
    await page.waitForSelector('#canvas', { timeout: 15000 });
    await page.waitForFunction(() => window.editor !== null, null, { timeout: 15000 });
}

/**
 * Read the canvas as plain ASCII from inside the page.
 *
 * `window.editor` is typed `| null` (see `e2e/globals.d.ts`), so a direct
 * `window.editor.exportAscii()` needs either a non-null assertion — which this
 * repo forbids via `@typescript-eslint/no-non-null-assertion` — or an `as any`,
 * which hides real type errors. Returning `null` when the editor is missing
 * keeps the call sites free of both casts and lets the assertion in the spec
 * produce the actual failure message.
 */
export async function getAsciiContent(page: Page): Promise<string | null> {
    return page.evaluate(() => window.editor?.exportAscii() ?? null);
}

/**
 * Same as {@link getAsciiContent} but narrows the result to `string`.
 *
 * A Playwright `expect(...).not.toBeNull()` does not narrow the type for
 * `tsc`, so call sites that immediately use the value would still need a cast.
 * Throwing here keeps the failure message in one place and leaves every spec
 * free of non-null assertions.
 */
export async function requireAsciiContent(page: Page): Promise<string> {
    const ascii = await getAsciiContent(page);
    if (ascii === null) {
        throw new Error('window.editor is not available; call openEditor(page) first');
    }
    return ascii;
}

/**
 * Grid metrics the editor lays out with, in CSS pixels.
 *
 * The app measures these itself: `USE_PIXEL_BUFFER` is on, so
 * `measureFont()` in `web/render.ts` assigns `GLYPH_WIDTH` / `GLYPH_HEIGHT` from
 * `web/constants.ts` and publishes them as `window.charWidth` /
 * `window.lineHeight`. Hard-coding them in a spec means a metrics change
 * silently moves every click, which is exactly the kind of drift a test that
 * skips its own assertion cannot notice — see the anchor cell in
 * `canvas.spec.ts` "should insert text at exact clicked grid position".
 */
export const GLYPH_WIDTH = 8;
export const GLYPH_HEIGHT = 20;

/**
 * Click the centre of a grid cell and wait for the render that follows.
 *
 * Returns the cell that was clicked so a caller can assert against it. Clicking
 * the centre rather than the top-left corner keeps the maths unambiguous: the
 * half-cell offset can never carry a click into the neighbouring cell when the
 * metrics are fractional.
 */
export async function clickGridCell(
    page: Page,
    cellX: number,
    cellY: number
): Promise<{ x: number; y: number }> {
    const canvas = page.locator('#canvas');
    const box = await canvas.boundingBox();
    if (!box) {
        throw new Error('clickGridCell: #canvas has no bounding box');
    }
    await page.mouse.click(
        box.x + (cellX * GLYPH_WIDTH) + (GLYPH_WIDTH / 2),
        box.y + (cellY * GLYPH_HEIGHT) + (GLYPH_HEIGHT / 2)
    );
    return { x: cellX, y: cellY };
}
