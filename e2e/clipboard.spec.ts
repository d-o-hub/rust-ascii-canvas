import { test, expect } from '@playwright/test';

/**
 * Clipboard / export fidelity tests for issue #21.
 * Verifies selection-aware export geometry (right borders, uniform line widths).
 */

declare global {
    interface Window {
        editor: {
            exportAscii(): string;
            exportForCopy?: () => string;
            serializeDocument(): string;
            loadDocument(json: string): boolean;
            clear(): void;
        } | null;
    }
}

test.describe('Copy / export fidelity', () => {
    test.beforeEach(async ({ page }) => {
                await page.addInitScript(() => {
            try { localStorage.removeItem('ascii-canvas-autosave'); } catch { /* ignore */ }
        });
        await page.goto('/');
        await page.waitForFunction(() => window.editor !== null, null, { timeout: 30000 });
        // Grant clipboard permissions where supported
        await page.context().grantPermissions(['clipboard-read', 'clipboard-write']).catch(() => {});
    });

    test('exportAscii preserves box right borders and uniform line widths', async ({ page }) => {
        // Draw a rectangle roughly in the upper-left
        await page.click('[data-tool="rectangle"]');
        const canvas = page.locator('#canvas');
        const box = await canvas.boundingBox();
        expect(box).toBeTruthy();
        if (!box) return;

        const x1 = box.x + 40;
        const y1 = box.y + 40;
        const x2 = box.x + 160;
        const y2 = box.y + 120;

        await page.mouse.move(x1, y1);
        await page.mouse.down();
        await page.mouse.move(x2, y2);
        await page.mouse.up();

        // Wait for content to be non-empty
        await page.waitForFunction(() => {
            const ascii = window.editor?.exportAscii?.() ?? '';
            return ascii.length > 0;
        }, null, { timeout: 10000 });

        const ascii = await page.evaluate(() => window.editor!.exportAscii());
        const lines = ascii.split('\n').filter((l: string) => l.length > 0);
        expect(lines.length).toBeGreaterThanOrEqual(2);

        // All content lines same width (uniform column export)
        const widths = lines.map((l: string) => [...l].length);
        expect(widths.every((w: number) => w === widths[0])).toBeTruthy();

        // Right-side box characters should survive (┐ ┘ │ or ascii equivalents)
        const rightChars = lines.map((l: string) => l[l.length - 1] ?? '');
        const joined = rightChars.join('');
        // At least one non-space on the right edge of some line
        expect(joined.trim().length).toBeGreaterThan(0);
    });

    test('exportForCopy matches exportAscii when no selection', async ({ page }) => {
        await page.click('[data-tool="rectangle"]');
        const canvas = page.locator('#canvas');
        const box = await canvas.boundingBox();
        if (!box) return;

        await page.mouse.move(box.x + 50, box.y + 50);
        await page.mouse.down();
        await page.mouse.move(box.x + 120, box.y + 90);
        await page.mouse.up();

        await page.waitForFunction(() => (window.editor?.exportAscii?.() ?? '').length > 0);

        const { full, forCopy } = await page.evaluate(() => {
            const ed = window.editor!;
            return {
                full: ed.exportAscii(),
                forCopy: typeof ed.exportForCopy === 'function' ? ed.exportForCopy() : ed.exportAscii(),
            };
        });

        expect(forCopy).toBe(full);
    });

    test('copy button shows toast', async ({ page }) => {
        await page.click('#copy-btn');
        const toast = page.locator('#status-toast');
        await expect(toast).not.toHaveClass(/hidden/);
    });

    test('serializeDocument / loadDocument round-trip', async ({ page }) => {
        await page.click('[data-tool="rectangle"]');
        const canvas = page.locator('#canvas');
        const box = await canvas.boundingBox();
        if (!box) return;

        await page.mouse.move(box.x + 60, box.y + 60);
        await page.mouse.down();
        await page.mouse.move(box.x + 140, box.y + 100);
        await page.mouse.up();

        await page.waitForFunction(() => (window.editor?.exportAscii?.() ?? '').length > 0);

        const ok = await page.evaluate(() => {
            const ed = window.editor!;
            const before = ed.exportAscii();
            const json = ed.serializeDocument();
            ed.clear();
            const afterClear = ed.exportAscii();
            const loaded = ed.loadDocument(json);
            const after = ed.exportAscii();
            return { loaded, before, afterClear, after, same: before === after };
        });

        expect(ok.loaded).toBe(true);
        expect(ok.afterClear).toBe('');
        expect(ok.same).toBe(true);
    });
});
