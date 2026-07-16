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

export { BASE_URL };
