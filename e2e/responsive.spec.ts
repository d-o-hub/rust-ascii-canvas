import { test, expect } from '@playwright/test';

const BASE_URL = process.env.BASE_URL || 'http://localhost:3003';

test.describe('Responsive Grid', () => {
    test.beforeEach(async ({ page }) => {
        await page.goto(BASE_URL);
        await page.waitForSelector('#loading.hidden', { timeout: 15000 });
        await page.waitForSelector('#canvas', { timeout: 10000 });
        await page.waitForTimeout(500);
    });

    test('should have a smaller grid on mobile viewport', async ({ page }) => {
        await page.setViewportSize({ width: 375, height: 667 });
        await page.waitForTimeout(500);

        const gridSize = page.locator('#grid-size');
        const text = await gridSize.textContent();
        // Mobile max is 60x30, but it should be even smaller based on viewport
        // 375 / 8 = 46.875 -> 46 cols
        // (667 - toolbar/header) / 20 -> around 25-30 rows
        const match = text?.match(/(\d+) × (\d+)/);
        if (match) {
            const cols = parseInt(match[1]);
            const rows = parseInt(match[2]);
            expect(cols).toBeLessThanOrEqual(60);
            expect(rows).toBeLessThanOrEqual(30);
            console.log(`Mobile grid size: ${cols}x${rows}`);
        } else {
            throw new Error(`Invalid grid size text: ${text}`);
        }
    });

    test('should have a medium grid on tablet viewport', async ({ page }) => {
        await page.setViewportSize({ width: 768, height: 1024 });
        await page.waitForTimeout(500);

        const gridSize = page.locator('#grid-size');
        const text = await gridSize.textContent();
        // Tablet max is 120x50
        const match = text?.match(/(\d+) × (\d+)/);
        if (match) {
            const cols = parseInt(match[1]);
            const rows = parseInt(match[2]);
            expect(cols).toBeGreaterThan(60);
            expect(cols).toBeLessThanOrEqual(120);
            expect(rows).toBeLessThanOrEqual(50);
            console.log(`Tablet grid size: ${cols}x${rows}`);
        } else {
            throw new Error(`Invalid grid size text: ${text}`);
        }
    });

    test('should have a large grid on desktop viewport', async ({ page }) => {
        await page.setViewportSize({ width: 1600, height: 1200 });
        await page.waitForTimeout(500);

        const gridSize = page.locator('#grid-size');
        const text = await gridSize.textContent();
        // Desktop max is 240x80
        const match = text?.match(/(\d+) × (\d+)/);
        if (match) {
            const cols = parseInt(match[1]);
            const rows = parseInt(match[2]);
            expect(cols).toBeGreaterThan(120);
            expect(cols).toBeLessThanOrEqual(240);
            expect(rows).toBeGreaterThan(50);
            expect(rows).toBeLessThanOrEqual(80);
            console.log(`Desktop grid size: ${cols}x${rows}`);
        } else {
            throw new Error(`Invalid grid size text: ${text}`);
        }
    });

    test('should allow drawing at the edges of the responsive grid', async ({ page }) => {
        await page.setViewportSize({ width: 600, height: 600 });
        await page.waitForTimeout(500);

        const gridSizeText = await page.locator('#grid-size').textContent();
        const match = gridSizeText?.match(/(\d+) × (\d+)/);
        if (!match) throw new Error('Could not get grid size');

        const cols = parseInt(match[1]);
        const rows = parseInt(match[2]);

        await page.click('[data-tool="rectangle"]');

        const canvas = page.locator('#canvas');
        const box = await canvas.boundingBox();
        if (!box) throw new Error('Could not get canvas bounding box');

        // Draw from (0,0) to (cols-1, rows-1)
        // We need to translate grid coords to screen coords
        // These are available via window.editor.pan and zoom which we assume are 0 and 1 here
        const charWidth = 8;
        const lineHeight = 20;

        await page.mouse.move(box.x + 1, box.y + 1);
        await page.mouse.down();
        await page.mouse.move(box.x + (cols - 1) * charWidth + 4, box.y + (rows - 1) * lineHeight + 10, { steps: 5 });
        await page.mouse.up();

        await page.waitForTimeout(300);

        // Verify drawing at bottom-right corner
        const ascii = await page.evaluate(() => {
            // @ts-ignore
            return window.editor.exportAscii();
        });

        const lines = ascii.trimEnd().split('\n');
        expect(lines.length).toBe(rows);
        const lastLine = lines[rows - 1];
        expect(lastLine.length).toBe(cols);
        // Expect some box drawing character or border at the corner
        expect(lastLine[cols - 1]).not.toBe(' ');
    });
});
