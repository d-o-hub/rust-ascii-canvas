import type { Page } from '@playwright/test';

const BASE_URL = process.env.BASE_URL || 'http://localhost:3003';

/** Clear autosave so tests start from a blank editor. */
export async function clearAutosave(page: Page): Promise<void> {
    await page.addInitScript(() => {
        try {
            localStorage.removeItem('ascii-canvas-autosave');
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
    await page.goto(BASE_URL);
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

export { BASE_URL };
