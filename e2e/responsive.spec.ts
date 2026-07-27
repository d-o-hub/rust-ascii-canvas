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

    test('should support mobile-specific sliding panel, touch targets, and action triggers', async ({ page }) => {
        await openAtViewport(page, 375, 667);

        // Mobile menu button should be visible on phone viewports
        const mobileMenuBtn = page.locator('#mobile-menu-btn');
        await expect(mobileMenuBtn).toBeVisible();

        // Initially side panel should not be open
        const sidePanel = page.locator('#side-panel');
        await expect(sidePanel).not.toHaveClass(/open/);

        // Click the mobile menu button to slide in the side panel drawer
        await mobileMenuBtn.click();
        await expect(sidePanel).toHaveClass(/open/);

        // Close button inside drawer should be visible on mobile
        const closeDrawerBtn = page.locator('#close-drawer-btn');
        await expect(closeDrawerBtn).toBeVisible();

        // Drawer overlay should be active and visible
        const drawerOverlay = page.locator('#drawer-overlay');
        await expect(drawerOverlay).toHaveClass(/open/);

        // Mobile theme toggle button should be visible in the actions grid
        const mobileThemeBtn = page.locator('#mobile-theme-btn');
        await expect(mobileThemeBtn).toBeVisible();

        // Click mobile theme button to toggle theme
        await mobileThemeBtn.click();

        // Drawer should be dismissed on action click
        await expect(sidePanel).not.toHaveClass(/open/);
        await expect(drawerOverlay).not.toHaveClass(/open/);

        // Re-open and close with drawer overlay click
        await mobileMenuBtn.click();
        await expect(sidePanel).toHaveClass(/open/);
        await drawerOverlay.click({ position: { x: 10, y: 10 } }); // click outside
        await expect(sidePanel).not.toHaveClass(/open/);
    });
});
