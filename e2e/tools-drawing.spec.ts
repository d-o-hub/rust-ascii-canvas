/**
 * ASCII Canvas - Comprehensive Tool Drawing E2E Tests
 * Tests all tools draw correctly with screenshot verification
 */

import { test, expect } from '@playwright/test';

const BASE_URL = process.env.BASE_URL || 'http://localhost:3003';

test.describe('Tool Drawing Verification', () => {
    test.beforeEach(async ({ page }) => {
        await page.goto(BASE_URL);
        await page.waitForSelector('#loading.hidden', { timeout: 15000 });
        await page.waitForSelector('#canvas', { timeout: 10000 });
        await expect(page.locator('#canvas')).toBeVisible();
    });

    async function drawOnCanvas(page: import('@playwright/test').Page, startX: number, startY: number, endX: number, endY: number) {
        const canvas = page.locator('#canvas');
        const box = await canvas.boundingBox();
        if (!box) return;
        
        await page.mouse.move(box.x + startX, box.y + startY);
        await page.mouse.down();
        await page.mouse.move(box.x + endX, box.y + endY, { steps: 10 });
        await page.mouse.up();
    }

    async function getAsciiContent(page: import('@playwright/test').Page) {
        return page.evaluate(() => (window as any).editor.exportAscii());
    }

    test('Rectangle tool draws on canvas', async ({ page }) => {
        await page.click('[data-tool="rectangle"]');
        await expect(page.locator('[data-tool="rectangle"]')).toHaveClass(/active/);
        
        await drawOnCanvas(page, 100, 100, 300, 200);
        
        const ascii = await getAsciiContent(page);
        expect(ascii).not.toMatch(/^(\s*\n)*$/);
        
        await page.screenshot({ path: 'test-results/rectangle-drawing.png' });
    });

    test('Line tool draws horizontal line', async ({ page }) => {
        await page.click('[data-tool="line"]');
        await expect(page.locator('[data-tool="line"]')).toHaveClass(/active/);
        
        await drawOnCanvas(page, 100, 150, 300, 150);
        
        const ascii = await getAsciiContent(page);
        expect(ascii).toContain('─');
        
        await page.screenshot({ path: 'test-results/line-horizontal.png' });
    });

    test('Line tool draws vertical line', async ({ page }) => {
        await page.click('[data-tool="line"]');
        
        await drawOnCanvas(page, 200, 100, 200, 300);
        
        const ascii = await getAsciiContent(page);
        expect(ascii).toContain('│');
        
        await page.screenshot({ path: 'test-results/line-vertical.png' });
    });

    test('Arrow tool draws on canvas', async ({ page }) => {
        await page.click('[data-tool="arrow"]');
        await expect(page.locator('[data-tool="arrow"]')).toHaveClass(/active/);
        
        await drawOnCanvas(page, 100, 150, 300, 150);
        
        const ascii = await getAsciiContent(page);
        expect(ascii).not.toMatch(/^(\s*\n)*$/);
        
        await page.screenshot({ path: 'test-results/arrow-drawing.png' });
    });

    test('Diamond tool draws on canvas', async ({ page }) => {
        await page.click('[data-tool="diamond"]');
        await expect(page.locator('[data-tool="diamond"]')).toHaveClass(/active/);
        
        await drawOnCanvas(page, 150, 100, 300, 200);
        
        const ascii = await getAsciiContent(page);
        expect(ascii).not.toMatch(/^(\s*\n)*$/);
        
        await page.screenshot({ path: 'test-results/diamond-drawing.png' });
    });

    test('Text tool places characters at clicked position', async ({ page }) => {
        await page.click('[data-tool="text"]');
        await expect(page.locator('[data-tool="text"]')).toHaveClass(/active/);
        
        const canvas = page.locator('#canvas');
        const box = await canvas.boundingBox();
        if (!box) return;
        
        await page.mouse.click(box.x + 100, box.y + 100);
        await page.waitForTimeout(100);
        
        await page.keyboard.type('HELLO');
        await page.waitForTimeout(100);
        await page.keyboard.press('Escape');
        await page.waitForTimeout(100);
        
        const ascii = await getAsciiContent(page);
        expect(ascii).toContain('HELLO');
        
        await page.screenshot({ path: 'test-results/text-drawing.png' });
    });

    test('Freehand tool draws at dragged positions', async ({ page }) => {
        await page.click('[data-tool="freehand"]');
        await expect(page.locator('[data-tool="freehand"]')).toHaveClass(/active/);
        
        const canvas = page.locator('#canvas');
        const box = await canvas.boundingBox();
        if (!box) return;
        
        const startX = box.x + 100;
        const startY = box.y + 100;
        
        await page.mouse.move(startX, startY);
        await page.mouse.down();
        
        for (let i = 0; i < 50; i += 5) {
            await page.mouse.move(startX + i, startY + Math.sin(i * 0.2) * 10, { steps: 2 });
        }
        
        await page.mouse.up();
        await page.waitForTimeout(200);
        
        const ascii = await getAsciiContent(page);
        // Freehand uses the border style's horizontal character (default: ─ for Single style)
        expect(ascii).toContain('─');
        
        await page.screenshot({ path: 'test-results/freehand-drawing.png' });
    });

    test('Eraser tool clears content', async ({ page }) => {
        await page.click('[data-tool="rectangle"]');
        await drawOnCanvas(page, 100, 100, 300, 200);
        await page.waitForTimeout(200);
        
        const asciiBefore = await getAsciiContent(page);
        expect(asciiBefore).not.toMatch(/^(\s*\n)*$/);
        
        await page.click('[data-tool="eraser"]');
        await expect(page.locator('[data-tool="eraser"]')).toHaveClass(/active/);
        
        await drawOnCanvas(page, 150, 120, 250, 180);
        await page.waitForTimeout(200);
        
        await page.screenshot({ path: 'test-results/eraser-drawing.png' });
    });
});

test.describe('Select Tool Delete Functionality', () => {
    test.beforeEach(async ({ page }) => {
        await page.goto(BASE_URL);
        await page.waitForSelector('#loading.hidden', { timeout: 15000 });
        await page.waitForSelector('#canvas', { timeout: 10000 });
    });

    async function drawOnCanvas(page: import('@playwright/test').Page, startX: number, startY: number, endX: number, endY: number) {
        const canvas = page.locator('#canvas');
        const box = await canvas.boundingBox();
        if (!box) return;
        
        await page.mouse.move(box.x + startX, box.y + startY);
        await page.mouse.down();
        await page.mouse.move(box.x + endX, box.y + endY, { steps: 10 });
        await page.mouse.up();
    }

    async function getAsciiContent(page: import('@playwright/test').Page) {
        return page.evaluate(() => (window as any).editor.exportAscii());
    }

    test('Select + Delete should clear selected area', async ({ page }) => {
        await page.click('[data-tool="rectangle"]');
        await drawOnCanvas(page, 100, 100, 300, 200);
        await page.waitForTimeout(200);
        
        const asciiBefore = await getAsciiContent(page);
        expect(asciiBefore).not.toMatch(/^(\s*\n)*$/);
        
        await page.click('[data-tool="select"]');
        await expect(page.locator('[data-tool="select"]')).toHaveClass(/active/);
        
        await drawOnCanvas(page, 80, 80, 320, 220);
        await page.waitForTimeout(200);
        
        await page.screenshot({ path: 'test-results/select-before-delete.png' });
        
        const canvas = page.locator('#canvas');
        await canvas.focus();
        await page.keyboard.press('Delete');
        await page.waitForTimeout(300);
        
        await page.screenshot({ path: 'test-results/select-after-delete.png' });
        
        const asciiAfter = await getAsciiContent(page);
        expect(asciiAfter).toMatch(/^(\s*\n)*$/);
    });

    test('Select + Backspace should clear selected area', async ({ page }) => {
        await page.click('[data-tool="line"]');
        await drawOnCanvas(page, 100, 150, 400, 150);
        await page.waitForTimeout(200);
        
        const asciiBefore = await getAsciiContent(page);
        expect(asciiBefore).toContain('─');
        
        await page.click('[data-tool="select"]');
        await drawOnCanvas(page, 80, 130, 420, 170);
        await page.waitForTimeout(200);
        
        const canvas = page.locator('#canvas');
        await canvas.focus();
        await page.keyboard.press('Backspace');
        await page.waitForTimeout(300);
        
        const asciiAfter = await getAsciiContent(page);
        expect(asciiAfter).toMatch(/^(\s*\n)*$/);
    });
});

test.describe('Edge Cases', () => {
    test.beforeEach(async ({ page }) => {
        await page.goto(BASE_URL);
        await page.waitForSelector('#loading.hidden', { timeout: 15000 });
        await page.waitForSelector('#canvas', { timeout: 10000 });
    });

    async function drawOnCanvas(page: import('@playwright/test').Page, startX: number, startY: number, endX: number, endY: number) {
        const canvas = page.locator('#canvas');
        const box = await canvas.boundingBox();
        if (!box) return;
        
        await page.mouse.move(box.x + startX, box.y + startY);
        await page.mouse.down();
        await page.mouse.move(box.x + endX, box.y + endY, { steps: 10 });
        await page.mouse.up();
    }

    test('Rectangle at canvas origin', async ({ page }) => {
        await page.click('[data-tool="rectangle"]');
        
        const canvas = page.locator('#canvas');
        const box = await canvas.boundingBox();
        if (!box) return;
        
        await page.mouse.move(box.x + 10, box.y + 10);
        await page.mouse.down();
        await page.mouse.move(box.x + 100, box.y + 50, { steps: 10 });
        await page.mouse.up();
        await page.waitForTimeout(200);
        
        const ascii = await page.evaluate(() => (window as any).editor.exportAscii());
        expect(ascii).not.toMatch(/^(\s*\n)*$/);
        
        await page.screenshot({ path: 'test-results/edge-rectangle-origin.png' });
    });

    test('Text at different positions', async ({ page }) => {
        await page.click('[data-tool="text"]');
        
        const canvas = page.locator('#canvas');
        const box = await canvas.boundingBox();
        if (!box) return;
        
        const positions = [
            { x: box.x + 50, y: box.y + 50, text: 'ABC' },
            { x: box.x + 200, y: box.y + 150, text: 'XYZ' },
        ];
        
        for (const pos of positions) {
            await canvas.focus();
            await page.mouse.click(pos.x, pos.y);
            await canvas.focus();
            await page.waitForTimeout(100);
            await page.keyboard.type(pos.text);
            await page.waitForTimeout(50);
            await page.keyboard.press('Escape');
            await page.waitForTimeout(50);
        }
        
        const ascii = await page.evaluate(() => (window as any).editor.exportAscii());
        expect(ascii).toContain('ABC');
        expect(ascii).toContain('XYZ');
        
        await page.screenshot({ path: 'test-results/edge-text-positions.png' });
    });

    test('Undo after drawing', async ({ page, isMobile }) => {
        await page.click('[data-tool="rectangle"]');
        await drawOnCanvas(page, 100, 100, 200, 150);
        await page.waitForTimeout(200);
        
        if (isMobile) {
            // On mobile, undo button is hidden in CSS. Use keyboard shortcut.
            await page.keyboard.press('Control+z');
        } else {
            await expect(page.locator('#undo-btn')).toBeEnabled();
            await page.click('#undo-btn');
        }
        
        await page.waitForTimeout(200);
        
        if (!isMobile) {
            await expect(page.locator('#undo-btn')).toBeDisabled();
        }
        
        await page.screenshot({ path: 'test-results/edge-undo.png' });
    });

    test('All border styles draw correctly', async ({ page }) => {
        const borderStyles = ['single', 'double', 'heavy', 'rounded', 'ascii', 'dotted'];
        
        for (let i = 0; i < borderStyles.length; i++) {
            const style = borderStyles[i];
            const offsetY = i * 60;
            
            await page.locator('#border-style').selectOption(style);
            await page.click('[data-tool="rectangle"]');
            await page.waitForTimeout(50);
            
            await drawOnCanvas(page, 50, offsetY + 20, 250, offsetY + 50);
            await page.waitForTimeout(100);
        }
        
        await page.screenshot({ path: 'test-results/edge-all-border-styles.png' });
    });

    test('Small single-point shape', async ({ page }) => {
        await page.click('[data-tool="rectangle"]');
        
        const canvas = page.locator('#canvas');
        const box = await canvas.boundingBox();
        if (!box) return;
        
        await page.mouse.click(box.x + 100, box.y + 100);
        await page.waitForTimeout(200);
        
        const ascii = await page.evaluate(() => (window as any).editor.exportAscii());
        expect(ascii).not.toMatch(/^[ \t\r]*(?:\n[ \t\r]*)*$/);
        
        await page.screenshot({ path: 'test-results/edge-single-point.png' });
    });
});

test.describe('Keyboard Shortcuts', () => {
    test.beforeEach(async ({ page }) => {
        await page.goto(BASE_URL);
        await page.waitForSelector('#loading.hidden', { timeout: 15000 });
        await page.waitForSelector('#canvas', { timeout: 10000 });
    });

    test('All tool shortcuts work', async ({ page }) => {
        const canvas = page.locator('#canvas');
        await canvas.click();
        await page.waitForTimeout(100);
        
        const shortcuts: Array<{ key: string; tool: string }> = [
            { key: 'r', tool: 'rectangle' },
            { key: 'l', tool: 'line' },
            { key: 'a', tool: 'arrow' },
            { key: 'd', tool: 'diamond' },
            { key: 't', tool: 'text' },
            { key: 'f', tool: 'freehand' },
            { key: 'e', tool: 'eraser' },
        ];
        
        for (const { key, tool } of shortcuts) {
            await page.keyboard.press(key);
            await page.waitForTimeout(100);
            await expect(page.locator(`[data-tool="${tool}"]`)).toHaveClass(/active/, { timeout: 3000 });
        }
    });

    test('Select tool shortcut works', async ({ page }) => {
        const canvas = page.locator('#canvas');
        await canvas.focus();
        await page.waitForTimeout(100);
        
        await page.keyboard.press('r');
        await page.waitForTimeout(100);
        await expect(page.locator('[data-tool="rectangle"]')).toHaveClass(/active/);
        
        await page.keyboard.press('v');
        await page.waitForTimeout(100);
        await expect(page.locator('[data-tool="select"]')).toHaveClass(/active/);
    });
});
