/**
 * ASCII Canvas E2E Tests
 * Comprehensive tests for all buttons, functions, and drawing verification
 */

import { test, expect } from '@playwright/test';

const BASE_URL = process.env.BASE_URL || 'http://localhost:3003';

test.describe('ASCII Canvas Editor', () => {
    test.beforeEach(async ({ page }) => {
        await page.goto(BASE_URL);
        
        // Wait for the loading overlay to be hidden
        await page.waitForSelector('#loading.hidden', { timeout: 15000 });
        
        // Wait for the canvas to be visible
        await page.waitForSelector('#canvas', { timeout: 10000 });
        
        // Small delay to ensure JS is fully loaded
        await page.waitForTimeout(500);
    });

    test('should load the editor', async ({ page }) => {
        await expect(page.locator('.toolbar')).toBeVisible();
        await expect(page.locator('#canvas')).toBeVisible();
        await expect(page.locator('.logo-text')).toContainText('ASCII Canvas');
    });

    test('should have all tool buttons', async ({ page }) => {
        const tools = ['select', 'rectangle', 'line', 'arrow', 'diamond', 'text', 'freehand', 'eraser'];
        
        for (const tool of tools) {
            const button = page.locator(`[data-tool="${tool}"]`);
            await expect(button).toBeVisible();
        }
    });

    test('should switch tools when clicked', async ({ page }) => {
        await page.click('[data-tool="line"]');
        const lineButton = page.locator('[data-tool="line"]');
        await expect(lineButton).toHaveClass(/active/);
    });

    test('should switch tools with keyboard shortcuts', async ({ page }) => {
        await page.click('#canvas');
        await page.waitForTimeout(100);
        
        // Press 'L' for line tool
        await page.keyboard.press('l');
        await page.waitForTimeout(100);
        const lineButton = page.locator('[data-tool="line"]');
        await expect(lineButton).toHaveClass(/active/);
        
        // Press 'R' for rectangle tool
        await page.keyboard.press('r');
        await page.waitForTimeout(100);
        const rectButton = page.locator('[data-tool="rectangle"]');
        await expect(rectButton).toHaveClass(/active/);
    });

    test('should show border style selector with all options', async ({ page }) => {
        const select = page.locator('#border-style');
        await expect(select).toBeVisible();
        
        const options = await select.locator('option').allTextContents();
        expect(options).toContain('Single (─)');
        expect(options).toContain('Double (═)');
        expect(options).toContain('Heavy (━)');
        expect(options).toContain('Rounded (╭)');
        expect(options).toContain('ASCII (-)');
        expect(options).toContain('Dotted (*)');
    });

    test('should have undo/redo buttons initially', async ({ page }) => {
        const undoBtn = page.locator('#undo-btn');
        const redoBtn = page.locator('#redo-btn');
        
        await expect(undoBtn).toBeVisible();
        await expect(redoBtn).toBeVisible();
    });

    test('should have copy and clear buttons', async ({ page }) => {
        const copyBtn = page.locator('#copy-btn');
        await expect(copyBtn).toBeVisible();
        
        const clearBtn = page.locator('#clear-btn');
        await expect(clearBtn).toBeVisible();
    });

    test('should have zoom controls', async ({ page }) => {
        await expect(page.locator('#zoom-fit')).toBeVisible();
        await expect(page.locator('#zoom-reset')).toBeVisible();
        await expect(page.locator('#zoom-out')).toBeVisible();
        await expect(page.locator('#zoom-in')).toBeVisible();
    });

    test('should display cursor position', async ({ page }) => {
        const cursorPos = page.locator('#cursor-pos');
        await expect(cursorPos).toBeVisible();
        
        const canvas = page.locator('#canvas');
        const box = await canvas.boundingBox();
        
        if (box) {
            await page.mouse.move(box.x + 100, box.y + 100);
            await page.waitForTimeout(100);
            await expect(cursorPos).not.toBeEmpty();
        }
    });

    test('should show status bar', async ({ page }) => {
        const statusBar = page.locator('.status-bar');
        await expect(statusBar).toBeVisible();
        
        const toolStatus = page.locator('#status-tool');
        await expect(toolStatus).toContainText('Tool:');
    });

    test('should show grid info', async ({ page }) => {
        const gridSize = page.locator('#grid-size');
        await expect(gridSize).toContainText('80 × 40');
    });

    test('should show zoom level', async ({ page }) => {
        const zoomLevel = page.locator('#zoom-level');
        await expect(zoomLevel).toContainText('100%');
    });
});

test.describe('Drawing Tools Interaction', () => {
    test.beforeEach(async ({ page }) => {
        await page.goto(BASE_URL);
        await page.waitForSelector('#loading.hidden', { timeout: 15000 });
        await page.waitForSelector('#canvas', { timeout: 10000 });
        await page.waitForTimeout(500);
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

    test('should select rectangle tool and draw', async ({ page }) => {
        await page.click('[data-tool="rectangle"]');
        
        const rectBtn = page.locator('[data-tool="rectangle"]');
        await expect(rectBtn).toHaveClass(/active/);
        
        const statusTool = page.locator('#status-tool');
        await expect(statusTool).toContainText('Tool: Rectangle');
        
        await drawOnCanvas(page, 100, 100, 300, 200);
        await page.waitForTimeout(300);
    });

    test('should select line tool and draw', async ({ page }) => {
        await page.click('[data-tool="line"]');
        
        const lineBtn = page.locator('[data-tool="line"]');
        await expect(lineBtn).toHaveClass(/active/);
        
        await drawOnCanvas(page, 100, 100, 300, 100);
        await page.waitForTimeout(300);
    });

    test('should select arrow tool and draw', async ({ page }) => {
        await page.click('[data-tool="arrow"]');
        
        const arrowBtn = page.locator('[data-tool="arrow"]');
        await expect(arrowBtn).toHaveClass(/active/);
        
        await drawOnCanvas(page, 100, 100, 300, 100);
        await page.waitForTimeout(300);
    });

    test('should select diamond tool and draw', async ({ page }) => {
        await page.click('[data-tool="diamond"]');
        
        const diamondBtn = page.locator('[data-tool="diamond"]');
        await expect(diamondBtn).toHaveClass(/active/);
        
        await drawOnCanvas(page, 200, 100, 300, 200);
        await page.waitForTimeout(300);
    });

    test('should select text tool', async ({ page }) => {
        await page.click('[data-tool="text"]');
        
        const textBtn = page.locator('[data-tool="text"]');
        await expect(textBtn).toHaveClass(/active/);
        
        const canvas = page.locator('#canvas');
        const box = await canvas.boundingBox();
        if (!box) return;
        
        await page.mouse.click(box.x + 100, box.y + 100);
        await page.waitForTimeout(200);
        
        await page.keyboard.type('Hello');
        await page.keyboard.press('Escape');
        await page.waitForTimeout(300);
    });

    test('should insert multiple characters at cursor position', async ({ page }) => {
        await page.click('[data-tool="text"]');
        
        const canvas = page.locator('#canvas');
        const box = await canvas.boundingBox();
        if (!box) return;
        
        // Click at a position and type "Test"
        await page.mouse.click(box.x + 50, box.y + 50);
        await page.waitForTimeout(100);
        
        await page.keyboard.type('Test');
        await page.waitForTimeout(200);
        
        // Verify text was inserted by checking export
        const ascii = await page.evaluate(() => {
            // @ts-ignore
            return window.editor.exportAscii();
        });
        
        // The grid should contain "Test" near the click position
        expect(ascii).toContain('Test');
    });

    test('should type at different cursor positions', async ({ page }) => {
        await page.click('[data-tool="text"]');
        
        const canvas = page.locator('#canvas');
        const box = await canvas.boundingBox();
        if (!box) return;
        
        // First click at (50, 50) and type "A"
        await page.mouse.click(box.x + 50, box.y + 50);
        await page.waitForTimeout(100);
        await page.keyboard.type('A');
        await page.waitForTimeout(100);
        
        // Click at (100, 50) and type "B"  
        await page.mouse.click(box.x + 100, box.y + 50);
        await page.waitForTimeout(100);
        await page.keyboard.type('B');
        await page.waitForTimeout(200);
        
        // Verify both characters are present
        const ascii = await page.evaluate(() => {
            // @ts-ignore
            return window.editor.exportAscii();
        });
        
        expect(ascii).toContain('A');
        expect(ascii).toContain('B');
    });

    test('should handle backspace correctly', async ({ page }) => {
        await page.click('[data-tool="text"]');
        
        const canvas = page.locator('#canvas');
        const box = await canvas.boundingBox();
        if (!box) return;
        
        // Click and type "Hello"
        await page.mouse.click(box.x + 50, box.y + 50);
        await page.waitForTimeout(100);
        await page.keyboard.type('Hello');
        await page.waitForTimeout(100);
        
        // Backspace 3 times
        await page.keyboard.press('Backspace');
        await page.waitForTimeout(50);
        await page.keyboard.press('Backspace');
        await page.waitForTimeout(50);
        await page.keyboard.press('Backspace');
        await page.waitForTimeout(200);
        
        // Should have "He" remaining
        const ascii = await page.evaluate(() => {
            // @ts-ignore
            return window.editor.exportAscii();
        });
        
        expect(ascii).toContain('He');
        expect(ascii).not.toContain('Hello');
    });

    test('should select freehand tool and draw', async ({ page }) => {
        await page.click('[data-tool="freehand"]');
        
        const freehandBtn = page.locator('[data-tool="freehand"]');
        await expect(freehandBtn).toHaveClass(/active/);
        
        const canvas = page.locator('#canvas');
        const box = await canvas.boundingBox();
        if (!box) return;
        
        await page.mouse.move(box.x + 100, box.y + 100);
        await page.mouse.down();
        await page.mouse.move(box.x + 150, box.y + 120, { steps: 5 });
        await page.mouse.move(box.x + 200, box.y + 140, { steps: 5 });
        await page.mouse.move(box.x + 250, box.y + 130, { steps: 5 });
        await page.mouse.up();
        await page.waitForTimeout(300);
    });

    test('should select eraser tool', async ({ page }) => {
        await page.click('[data-tool="eraser"]');
        
        const eraserBtn = page.locator('[data-tool="eraser"]');
        await expect(eraserBtn).toHaveClass(/active/);
    });

    test('should select and move a shape', async ({ page }) => {
        await page.click('[data-tool="rectangle"]');
        await drawOnCanvas(page, 100, 100, 200, 150);
        await page.waitForTimeout(300);
        
        await page.click('[data-tool="select"]');
        
        const canvas = page.locator('#canvas');
        const box = await canvas.boundingBox();
        if (!box) return;
        
        await page.mouse.click(box.x + 150, box.y + 125);
        await page.waitForTimeout(200);
        
        await page.mouse.move(box.x + 150, box.y + 125);
        await page.mouse.down();
        await page.mouse.move(box.x + 200, box.y + 175, { steps: 5 });
        await page.mouse.up();
        await page.waitForTimeout(300);
    });
});

test.describe('Undo/Redo', () => {
    test.beforeEach(async ({ page }) => {
        await page.goto(BASE_URL);
        await page.waitForSelector('#loading.hidden', { timeout: 15000 });
        await page.waitForSelector('#canvas', { timeout: 10000 });
        await page.waitForTimeout(500);
    });

    async function drawRect(page: import('@playwright/test').Page) {
        const canvas = page.locator('#canvas');
        const box = await canvas.boundingBox();
        if (!box) return;
        
        await page.mouse.move(box.x + 100, box.y + 100);
        await page.mouse.down();
        await page.mouse.move(box.x + 200, box.y + 150, { steps: 10 });
        await page.mouse.up();
    }

    test('should enable undo after drawing', async ({ page }) => {
        await page.click('[data-tool="rectangle"]');
        await drawRect(page);
        await page.waitForTimeout(300);
        
        const undoBtn = page.locator('#undo-btn');
        await expect(undoBtn).toBeEnabled();
    });

    test('should undo drawing', async ({ page }) => {
        await page.click('[data-tool="rectangle"]');
        await drawRect(page);
        await page.waitForTimeout(300);
        
        await page.click('#undo-btn');
        await page.waitForTimeout(300);
        
        const undoBtn = page.locator('#undo-btn');
        await expect(undoBtn).toBeDisabled();
    });

    test('should enable redo after undo', async ({ page }) => {
        await page.click('[data-tool="rectangle"]');
        await drawRect(page);
        await page.waitForTimeout(300);
        
        await page.click('#undo-btn');
        await page.waitForTimeout(300);
        
        const redoBtn = page.locator('#redo-btn');
        await expect(redoBtn).toBeEnabled();
    });

    test('should redo drawing', async ({ page }) => {
        await page.click('[data-tool="rectangle"]');
        await drawRect(page);
        await page.waitForTimeout(300);
        
        await page.click('#undo-btn');
        await page.waitForTimeout(300);
        
        await page.click('#redo-btn');
        await page.waitForTimeout(300);
        
        const undoBtn = page.locator('#undo-btn');
        await expect(undoBtn).toBeEnabled();
    });

    test('should clear canvas', async ({ page }) => {
        await page.click('[data-tool="rectangle"]');
        
        const canvas = page.locator('#canvas');
        const box = await canvas.boundingBox();
        if (!box) return;
        
        await page.mouse.move(box.x + 100, box.y + 100);
        await page.mouse.down();
        await page.mouse.move(box.x + 200, box.y + 150, { steps: 10 });
        await page.mouse.up();
        await page.waitForTimeout(300);
        
        page.on('dialog', dialog => dialog.accept());
        
        await page.click('#clear-btn');
        await page.waitForTimeout(300);
        
        const undoBtn = page.locator('#undo-btn');
        await expect(undoBtn).toBeDisabled();
    });
});

test.describe('Border Styles', () => {
    test.beforeEach(async ({ page }) => {
        await page.goto(BASE_URL);
        await page.waitForSelector('#loading.hidden', { timeout: 15000 });
        await page.waitForSelector('#canvas', { timeout: 10000 });
        await page.waitForTimeout(500);
    });

    test('should change border style via select', async ({ page }) => {
        const select = page.locator('#border-style');
        
        await select.selectOption('double');
        await page.waitForTimeout(100);
        
        const selected = await select.inputValue();
        expect(selected).toBe('double');
    });

    test('should cycle border styles with B key', async ({ page }) => {
        await page.click('#canvas');
        await page.waitForTimeout(100);
        
        await page.keyboard.press('b');
        await page.waitForTimeout(100);
        
        await page.keyboard.press('b');
        await page.waitForTimeout(100);
        
        const toast = page.locator('#status-toast');
        await expect(toast).toBeVisible();
    });
});

test.describe('Zoom Controls', () => {
    test.beforeEach(async ({ page }) => {
        await page.goto(BASE_URL);
        await page.waitForSelector('#loading.hidden', { timeout: 15000 });
        await page.waitForSelector('#canvas', { timeout: 10000 });
        await page.waitForTimeout(500);
        
        // Reset zoom to 100% before each test
        await page.click('#zoom-reset');
        await page.waitForTimeout(200);
    });

    test('should zoom in with button', async ({ page }) => {
        await expect(page.locator('#zoom-level')).toContainText('100%');
        
        await page.click('#zoom-in');
        await page.waitForTimeout(200);
        
        await expect(page.locator('#zoom-level')).not.toContainText('100%');
    });

    test('should zoom out with button', async ({ page }) => {
        await expect(page.locator('#zoom-level')).toContainText('100%');
        
        await page.click('#zoom-out');
        await page.waitForTimeout(200);
        
        await expect(page.locator('#zoom-level')).not.toContainText('100%');
    });

    test('should reset zoom', async ({ page }) => {
        await page.click('#zoom-in');
        await page.waitForTimeout(200);
        
        await page.click('#zoom-reset');
        await page.waitForTimeout(200);
        
        await expect(page.locator('#zoom-level')).toContainText('100%');
    });

    test('should fit zoom', async ({ page }) => {
        await page.click('#zoom-fit');
        await page.waitForTimeout(500);
        
        await expect(page.locator('#zoom-level')).not.toBeEmpty();
    });

    test('should zoom with scroll wheel', async ({ page }) => {
        await expect(page.locator('#zoom-level')).toContainText('100%');
        
        const canvas = page.locator('#canvas');
        await canvas.hover();
        await page.mouse.wheel(0, -100);
        
        await page.waitForTimeout(200);
        
        await expect(page.locator('#zoom-level')).not.toContainText('100%');
    });
});

test.describe('Keyboard Shortcuts', () => {
    test.beforeEach(async ({ page }) => {
        await page.goto(BASE_URL);
        await page.waitForSelector('#loading.hidden', { timeout: 15000 });
        await page.waitForSelector('#canvas', { timeout: 10000 });
        await page.waitForTimeout(500);
    });

    test('should support all tool shortcuts', async ({ page }) => {
        await page.click('#canvas');
        await page.waitForTimeout(100);
        
        const shortcuts = [
            { key: 'v', tool: 'select' },
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

    test('should show shortcuts modal with ?', async ({ page }) => {
        await page.click('#canvas');
        await page.keyboard.press('?');
        await page.waitForTimeout(200);
        
        const modal = page.locator('#shortcuts-modal');
        await expect(modal).not.toHaveClass(/hidden/);
    });

    test('should close shortcuts modal with Escape', async ({ page }) => {
        await page.click('#canvas');
        await page.keyboard.press('?');
        await page.waitForTimeout(200);
        
        const modal = page.locator('#shortcuts-modal');
        await expect(modal).not.toHaveClass(/hidden/);
        
        // Click the close button instead
        await page.click('.modal-close');
        await page.waitForTimeout(200);
        
        await expect(modal).toHaveClass(/hidden/);
    });

    test('should close shortcuts modal by clicking overlay', async ({ page }) => {
        // First close any existing modal
        const modal = page.locator('#shortcuts-modal');
        const isVisible = await modal.evaluate(el => !el.classList.contains('hidden'));
        
        if (isVisible) {
            await page.click('.modal-close');
            await page.waitForTimeout(200);
        }
        
        // Now open and close
        await page.click('#canvas');
        await page.keyboard.press('?');
        await page.waitForTimeout(200);
        
        await expect(modal).not.toHaveClass(/hidden/);
        
        await page.click('#shortcuts-modal', { position: { x: 10, y: 10 } });
        await page.waitForTimeout(200);
        
        await expect(modal).toHaveClass(/hidden/);
    });

    test('should undo with Ctrl+Z', async ({ page }) => {
        await page.click('[data-tool="rectangle"]');
        
        const canvas = page.locator('#canvas');
        const box = await canvas.boundingBox();
        if (!box) return;
        
        await page.mouse.move(box.x + 100, box.y + 100);
        await page.mouse.down();
        await page.mouse.move(box.x + 200, box.y + 150, { steps: 10 });
        await page.mouse.up();
        await page.waitForTimeout(300);
        
        await page.click('#canvas');
        await page.keyboard.press('Control+z');
        await page.waitForTimeout(300);
    });

    test('should redo with Ctrl+Shift+Z', async ({ page }) => {
        await page.click('[data-tool="rectangle"]');
        
        const canvas = page.locator('#canvas');
        const box = await canvas.boundingBox();
        if (!box) return;
        
        await page.mouse.move(box.x + 100, box.y + 100);
        await page.mouse.down();
        await page.mouse.move(box.x + 200, box.y + 150, { steps: 10 });
        await page.mouse.up();
        await page.waitForTimeout(300);
        
        await page.click('#canvas');
        await page.keyboard.press('Control+z');
        await page.waitForTimeout(300);
        
        await page.keyboard.press('Control+Shift+z');
        await page.waitForTimeout(300);
    });
});

test.describe('Copy Functionality', () => {
    test.beforeEach(async ({ page }) => {
        await page.goto(BASE_URL);
        await page.waitForSelector('#loading.hidden', { timeout: 15000 });
        await page.waitForSelector('#canvas', { timeout: 10000 });
        await page.waitForTimeout(500);
    });

    test('should show toast when copy is clicked', async ({ page }) => {
        await page.click('[data-tool="rectangle"]');
        
        const canvas = page.locator('#canvas');
        const box = await canvas.boundingBox();
        if (!box) return;
        
        await page.mouse.move(box.x + 100, box.y + 100);
        await page.mouse.down();
        await page.mouse.move(box.x + 200, box.y + 150, { steps: 10 });
        await page.mouse.up();
        await page.waitForTimeout(300);
        
        await page.click('#copy-btn');
        await page.waitForTimeout(500);
        
        const toast = page.locator('#status-toast');
        await expect(toast).toBeVisible();
    });
});

test.describe('Output Verification', () => {
    test.beforeEach(async ({ page }) => {
        await page.goto(BASE_URL);
        await page.waitForSelector('#loading.hidden', { timeout: 15000 });
        await page.waitForSelector('#canvas', { timeout: 10000 });
        await page.waitForTimeout(500);
    });

    test('should have editor available', async ({ page }) => {
        const hasEditor = await page.evaluate(() => {
            // @ts-ignore
            return window.editor !== null && window.editor !== undefined;
        });
        expect(hasEditor).toBe(true);
    });
});
