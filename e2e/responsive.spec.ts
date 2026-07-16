import { test, expect, type Page } from '@playwright/test';
import { clearAutosave, BASE_URL } from './helpers';

async function waitForRender(page: Page): Promise<void> {
    await page.waitForFunction(() => {
        const canvas = document.querySelector('#canvas') as HTMLCanvasElement;
        return canvas && canvas.width > 0 && canvas.height > 0;
    }, { timeout: 5000 });
}

async function openAtViewport(page: import('@playwright/test').Page, width: number, height: number) {
    await page.setViewportSize({ width, height });
    await clearAutosave(page);
    await page.goto(BASE_URL);
    await page.waitForSelector('#loading.hidden', { state: 'attached', timeout: 30000 });
    await page.waitForSelector('#canvas', { timeout: 15000 });
    await page.waitForFunction(() => window.editor !== null, null, { timeout: 15000 });
}

test.describe('Responsive Grid', () => {
    test('should have a smaller grid on mobile viewport', async ({ page }) => {
        await openAtViewport(page, 375, 667);

        const gridSize = page.locator('#grid-size');
        const text = await gridSize.textContent();
        const match = text?.match(/(\d+) × (\d+)/);
        if (match) {
            const cols = parseInt(match[1], 10);
            const rows = parseInt(match[2], 10);
            expect(cols).toBeLessThanOrEqual(60);
            expect(rows).toBeLessThanOrEqual(30);
            console.log(`Mobile grid size: ${cols}x${rows}`);
        } else {
            throw new Error(`Invalid grid size text: ${text}`);
        }
    });

    test('should have a medium grid on tablet viewport', async ({ page }) => {
        await openAtViewport(page, 768, 1024);

        const gridSize = page.locator('#grid-size');
        const text = await gridSize.textContent();
        const match = text?.match(/(\d+) × (\d+)/);
        if (match) {
            const cols = parseInt(match[1], 10);
            const rows = parseInt(match[2], 10);
            expect(cols).toBeGreaterThan(60);
            expect(cols).toBeLessThanOrEqual(120);
            expect(rows).toBeLessThanOrEqual(50);
            console.log(`Tablet grid size: ${cols}x${rows}`);
        } else {
            throw new Error(`Invalid grid size text: ${text}`);
        }
    });

    test('should have a large grid on desktop viewport', async ({ page }) => {
        await openAtViewport(page, 1600, 1200);

        const gridSize = page.locator('#grid-size');
        const text = await gridSize.textContent();
        const match = text?.match(/(\d+) × (\d+)/);
        if (match) {
            const cols = parseInt(match[1], 10);
            const rows = parseInt(match[2], 10);
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
        await openAtViewport(page, 600, 600);

        const gridSizeText = await page.locator('#grid-size').textContent();
        const match = gridSizeText?.match(/(\d+) × (\d+)/);
        if (!match) throw new Error('Could not get grid size');

        const cols = parseInt(match[1], 10);
        const rows = parseInt(match[2], 10);

        await page.click('[data-tool="rectangle"]');

        const canvas = page.locator('#canvas');
        const box = await canvas.boundingBox();
        if (!box) throw new Error('Could not get canvas bounding box');

        const charWidth = 8;
        const lineHeight = 20;

        await page.mouse.move(box.x + 1, box.y + 1);
        await page.mouse.down();
        await page.mouse.move(
            box.x + (cols - 1) * charWidth + 4,
            box.y + (rows - 1) * lineHeight + 10,
            { steps: 5 },
        );
        await page.mouse.up();

        await waitForRender(page);

        const ascii = await page.evaluate(() => {
            return window.editor?.exportAscii() ?? '';
        });

        const lines = ascii.trimEnd().split('\n');
        expect(lines.length).toBe(rows);
        const lastLine = lines[rows - 1];
        expect(lastLine.length).toBe(cols);
        expect(lastLine[cols - 1]).not.toBe(' ');
    });
});
