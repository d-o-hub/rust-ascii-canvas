import { test, expect } from '@playwright/test';
import { openEditor } from './helpers';

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
        await openEditor(page);
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
            const ascii = window.editor?.exportAscii() ?? '';
            return ascii.length > 0;
        }, null, { timeout: 10000 });

        const ascii = await page.evaluate(() => window.editor?.exportAscii() ?? '');
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

        await page.waitForFunction(() => (window.editor?.exportAscii() ?? '').length > 0);

        const { full, forCopy } = await page.evaluate(() => {
            const ed = window.editor;
            if (!ed) return { full: '', forCopy: '' };
            return {
                full: ed.exportAscii(),
                forCopy: typeof ed.exportForCopy === 'function' ? ed.exportForCopy() : ed.exportAscii(),
            };
        });

        expect(forCopy).toBe(full);
    });

    test('copy button shows toast', async ({ page, isMobile }) => {
        test.skip(isMobile, 'Copy button and status toast are hidden on mobile viewports');
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

        await page.waitForFunction(() => (window.editor?.exportAscii() ?? '').length > 0);

        const ok = await page.evaluate(() => {
            const ed = window.editor;
            if (!ed) {
                return { loaded: false, before: '', afterClear: '', after: '', same: false };
            }
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

    test('external paste at cursor origin respects spaces and boundaries', async ({ page }) => {
        // Draw a Rectangle at upper-left to have some content underneath
        await page.click('[data-tool="rectangle"]');
        const canvasLocator = page.locator('#canvas');
        const box = await canvasLocator.boundingBox();
        expect(box).toBeTruthy();
        if (!box) return;

        // Draw a small solid structure (or simple lines) using rectangle
        await page.mouse.move(box.x + 10, box.y + 10);
        await page.mouse.down();
        await page.mouse.move(box.x + 80, box.y + 40);
        await page.mouse.up();

        // Wait for it to render
        await page.waitForFunction(() => (window.editor?.exportAscii() ?? '').length > 0);

        // Now dispatch a paste event with text "P S" over the drawn content.
        await page.click('[data-tool="select"]');
        await page.mouse.move(box.x + 30, box.y + 30);
        await page.mouse.down();
        await page.mouse.move(box.x + 30, box.y + 30);
        await page.mouse.up();

        // Dispatch Paste Event
        await page.evaluate(() => {
            const pasteEvent = new ClipboardEvent('paste', {
                bubbles: true,
                cancelable: true,
                clipboardData: new DataTransfer(),
            });
            pasteEvent.clipboardData?.setData('text/plain', 'P S');
            window.dispatchEvent(pasteEvent);
        });

        // Verify characters were pasted
        const asciiAfter = await page.evaluate(() => window.editor?.exportAscii() ?? '');
        expect(asciiAfter).toContain('P');
        expect(asciiAfter).toContain('S');
    });

    test('navigator.clipboard.readText() matches export geometry where permissions allow (F-27)', async ({ page, context, browserName, isMobile }) => {
        // Idiomatically skip browsers and viewports upfront
        test.skip(isMobile, 'Copy button and status toast are hidden on mobile viewports');
        test.skip(browserName !== 'chromium', 'Permissions and reading from system clipboard is only fully supported/queryable under Chromium');

        // Grant permissions explicitly in Chromium context to guarantee API access
        await context.grantPermissions(['clipboard-read', 'clipboard-write']);

        // Draw a rectangle to produce non-empty ASCII content
        await page.click('[data-tool="rectangle"]');
        const canvas = page.locator('#canvas');
        const box = await canvas.boundingBox();
        expect(box).toBeTruthy();

        // Use safe inset coordinates relative to bounding box to draw inside the grid canvas bounds
        const x1 = box!.x + 40;
        const y1 = box!.y + 40;
        const x2 = box!.x + 160;
        const y2 = box!.y + 120;

        await page.mouse.move(x1, y1);
        await page.mouse.down();
        await page.mouse.move(x2, y2);
        await page.mouse.up();

        // Wait up to 3 seconds for content to render and be ready for export
        await page.waitForFunction(() => {
            const ascii = window.editor?.exportAscii() ?? '';
            return ascii.length > 0;
        }, null, { timeout: 3000 });

        const exportedAscii = await page.evaluate(() => window.editor?.exportAscii() ?? '');

        // Click the copy button to write ASCII content to the system clipboard
        await page.click('#copy-btn');

        // Retrieve clipboard content directly using browser's native API in the page context
        const clipboardText = await page.evaluate(async () => navigator.clipboard.readText());

        // Normalize line endings to LF for unified geometry comparison
        const normExported = exportedAscii.replace(/\r\n/g, '\n');
        const normClipboard = clipboardText.replace(/\r\n/g, '\n');

        const exportedLines = normExported.split('\n').filter(l => l.length > 0);
        const clipboardLines = normClipboard.split('\n').filter(l => l.length > 0);

        // Ensure we actually drew and captured lines to prevent silent falsy passes
        expect(exportedLines.length).toBeGreaterThan(0);

        // Perform standard deep array comparison of the line contents
        expect(clipboardLines).toEqual(exportedLines);
    });
});
