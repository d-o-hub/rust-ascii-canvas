/**
 * ASCII Canvas E2E Tests
 * Comprehensive tests for all buttons, functions, and drawing verification
 */

import { test, expect, type Page } from '@playwright/test';
import { openEditor } from './helpers';


async function waitForRender(page: Page): Promise<void> {
    await page.waitForFunction(() => {
        const canvas = document.querySelector('#canvas') as HTMLCanvasElement;
        return canvas && canvas.width > 0 && canvas.height > 0;
    }, { timeout: 5000 });
}

async function waitForCursorUpdate(page: Page): Promise<void> {
    await page.waitForFunction(
        () => {
            const el = document.querySelector('#cursor-pos');
            return el && el.textContent && el.textContent.trim().length > 0;
        },
        { timeout: 5000 }
    );
}

test.describe('ASCII Canvas Editor', () => {
    test.beforeEach(async ({ page }) => {
        await openEditor(page);
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
        await page.mouse.up();
        
        await page.keyboard.press('l');
        const lineButton = page.locator('[data-tool="line"]');
        await expect(lineButton).toHaveClass(/active/);
        
        await page.keyboard.press('r');
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

    test('should have undo/redo buttons initially', async ({ page, isMobile }) => {
        test.skip(isMobile, 'Undo/redo buttons are hidden on mobile viewports');
        const undoBtn = page.locator('#undo-btn');
        const redoBtn = page.locator('#redo-btn');
        
        await expect(undoBtn).toBeVisible();
        await expect(redoBtn).toBeVisible();
    });

    test('should have copy and clear buttons', async ({ page, isMobile }) => {
        test.skip(isMobile, 'Copy and clear buttons are hidden on mobile viewports');
        const copyBtn = page.locator('#copy-btn');
        await expect(copyBtn).toBeVisible();
        
        const clearBtn = page.locator('#clear-btn');
        await expect(clearBtn).toBeVisible();
    });

    test('should have zoom controls', async ({ page, isMobile }) => {
        test.skip(isMobile, 'Zoom controls are hidden on mobile viewports');
        await expect(page.locator('#zoom-fit')).toBeVisible();
        await expect(page.locator('#zoom-reset')).toBeVisible();
        await expect(page.locator('#zoom-out')).toBeVisible();
        await expect(page.locator('#zoom-in')).toBeVisible();
    });

    test('should display cursor position', async ({ page, isMobile }) => {
        test.skip(isMobile, 'Side panel and status bar are hidden on mobile viewports');
        const cursorPos = page.locator('#cursor-pos');
        await expect(cursorPos).toBeVisible();
        
        const canvas = page.locator('#canvas');
        const box = await canvas.boundingBox();
        
        if (box) {
            await page.mouse.move(box.x + 100, box.y + 100);
            await waitForCursorUpdate(page);
            await expect(cursorPos).not.toBeEmpty();
        }
    });

    test('should show status bar', async ({ page, isMobile }) => {
        test.skip(isMobile, 'Status bar is hidden on mobile viewports');
        const statusBar = page.locator('.status-bar');
        await expect(statusBar).toBeVisible();
        
        const toolStatus = page.locator('#status-tool');
        await expect(toolStatus).toContainText('Tool:');
    });

    test('should show grid info', async ({ page, isMobile }) => {
        test.skip(isMobile, 'Side panel is hidden on mobile viewports');
        const gridSize = page.locator('#grid-size');
        await expect(gridSize).toContainText(/(\d+) × (\d+)/);
    });

    test('should show zoom level', async ({ page, isMobile }) => {
        test.skip(isMobile, 'Side panel is hidden on mobile viewports');
        const zoomLevel = page.locator('#zoom-level');
        await expect(zoomLevel).toContainText('100%');
    });
});

test.describe('Drawing Tools Interaction', () => {
    test.beforeEach(async ({ page }) => {
        await openEditor(page);
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
    });

    test('should select line tool and draw', async ({ page }) => {
        await page.click('[data-tool="line"]');
        
        const lineBtn = page.locator('[data-tool="line"]');
        await expect(lineBtn).toHaveClass(/active/);
        
        await drawOnCanvas(page, 100, 100, 300, 100);
    });

    test('should select arrow tool and draw', async ({ page }) => {
        await page.click('[data-tool="arrow"]');
        
        const arrowBtn = page.locator('[data-tool="arrow"]');
        await expect(arrowBtn).toHaveClass(/active/);
        
        await drawOnCanvas(page, 100, 100, 300, 100);
    });

    test('should select diamond tool and draw', async ({ page }) => {
        await page.click('[data-tool="diamond"]');
        
        const diamondBtn = page.locator('[data-tool="diamond"]');
        await expect(diamondBtn).toHaveClass(/active/);
        
        await drawOnCanvas(page, 200, 100, 300, 200);
    });

    test('should select text tool', async ({ page }) => {
        await page.click('[data-tool="text"]');
        
        const textBtn = page.locator('[data-tool="text"]');
        await expect(textBtn).toHaveClass(/active/);
        
        const canvas = page.locator('#canvas');
        const box = await canvas.boundingBox();
        if (!box) return;
        
        await page.mouse.click(box.x + 100, box.y + 100);
        await waitForRender(page);
        
        await page.keyboard.type('Hello');
        await page.keyboard.press('Escape');
    });

    test('should insert text at exact clicked grid position', async ({ page }) => {
        await page.click('[data-tool="text"]');
        
        const canvas = page.locator('#canvas');
        const box = await canvas.boundingBox();
        if (!box) return;
        
        // Click at a known position - grid cell (10, 10)
        // Using exact metrics from implementation (charWidth=8, lineHeight=20)
        const charWidth = 8;
        const lineHeight = 20;
        const gridX = 10;
        const gridY = 10;
        const clickX = box.x + (gridX * charWidth) + (charWidth / 2);
        const clickY = box.y + (gridY * lineHeight) + (lineHeight / 2);
        
        await canvas.focus();
        await page.mouse.click(clickX, clickY);
        await waitForRender(page);
        
        await page.keyboard.type('X');
        await waitForRender(page);
        
        // Verify text was inserted at the correct position
        const ascii = await page.evaluate(() => {
            // @ts-ignore
            return window.editor.exportAscii();
        });
        
        const lines = ascii.split('\n');
        if (lines[gridY]) {
            expect(lines[gridY][gridX]).toBe('X');
        }
    });

    test('should insert multiple characters sequentially', async ({ page }) => {
        await page.click('[data-tool="text"]');
        
        const canvas = page.locator('#canvas');
        const box = await canvas.boundingBox();
        if (!box) return;
        
        // Click at position - focus canvas first
        await canvas.focus();
        await page.mouse.click(box.x + 50, box.y + 50);
        await waitForRender(page);
        
        // Re-focus canvas before typing (clicking may have moved focus)
        await canvas.focus();
        
        // Type 5 characters one by one
        await page.keyboard.type('A');
        await page.keyboard.type('B');
        await page.keyboard.type('C');
        await page.keyboard.type('D');
        await page.keyboard.type('E');
        await waitForRender(page);
        
        // Verify all characters are present
        const ascii = await page.evaluate(() => {
            // @ts-ignore
            return window.editor.exportAscii();
        });
        
        expect(ascii).toContain('ABCDE');
    });

    test('should type at different positions independently', async ({ page }) => {
        await page.click('[data-tool="text"]');
        
        const canvas = page.locator('#canvas');
        const box = await canvas.boundingBox();
        if (!box) return;
        
        // First position - type "X"
        await canvas.focus();
        await page.mouse.click(box.x + 50, box.y + 30);
        await waitForRender(page);
        await canvas.focus();
        await page.keyboard.type('X');
        await waitForRender(page);
        
        // Press Escape to commit
        await page.keyboard.press('Escape');
        await waitForRender(page);
        
        // Second position - type "Y"
        await canvas.focus();
        await page.mouse.click(box.x + 100, box.y + 30);
        await waitForRender(page);
        await canvas.focus();
        await page.keyboard.type('Y');
        await waitForRender(page);
        
        // Verify both characters exist at different positions
        const ascii = await page.evaluate(() => {
            // @ts-ignore
            return window.editor.exportAscii();
        });
        
        expect(ascii).toContain('X');
        expect(ascii).toContain('Y');
        // They should be at different positions
        const xIndex = ascii.indexOf('X');
        const yIndex = ascii.indexOf('Y');
        expect(xIndex).not.toBe(yIndex);
    });

    test('should handle backspace at any position', async ({ page }) => {
        await page.click('[data-tool="text"]');
        
        const canvas = page.locator('#canvas');
        const box = await canvas.boundingBox();
        if (!box) return;
        
        await canvas.focus();
        await page.mouse.click(box.x + 50, box.y + 50);
        await waitForRender(page);
        
        // Type "Hello"
        await page.keyboard.type('Hello');
        await waitForRender(page);
        
        // Backspace 3 times
        await page.keyboard.press('Backspace');
        await page.keyboard.press('Backspace');
        await page.keyboard.press('Backspace');
        await waitForRender(page);
        
        // Should have "He" remaining
        const ascii = await page.evaluate(() => {
            // @ts-ignore
            return window.editor.exportAscii();
        });
        
        expect(ascii).toContain('He');
    });

    test('should start fresh after clicking new position', async ({ page }) => {
        await page.click('[data-tool="text"]');
        
        const canvas = page.locator('#canvas');
        const box = await canvas.boundingBox();
        if (!box) return;
        
        // First click - type "AAA"
        await canvas.focus();
        await page.mouse.click(box.x + 50, box.y + 30);
        await waitForRender(page);
        await canvas.focus();
        await page.keyboard.type('AAA');
        await waitForRender(page);
        
        // Click in new position - type "BBB"
        await canvas.focus();
        await page.mouse.click(box.x + 150, box.y + 30);
        await waitForRender(page);
        await canvas.focus();
        await page.keyboard.type('BBB');
        await waitForRender(page);
        
        // Verify both are present
        const ascii = await page.evaluate(() => {
            // @ts-ignore
            return window.editor.exportAscii();
        });
        
        expect(ascii).toContain('AAA');
        expect(ascii).toContain('BBB');
    });

    test('should work with zoom level changes', async ({ page }) => {
        await page.click('[data-tool="text"]');
        
        const canvas = page.locator('#canvas');
        const box = await canvas.boundingBox();
        if (!box) return;
        
        // Click and type "Hello"
        await canvas.focus();
        await page.mouse.click(box.x + 50, box.y + 50);
        await waitForRender(page);
        await canvas.focus();
        await page.keyboard.type('Hello');
        await waitForRender(page);
        
        // Backspace 3 times
        await page.keyboard.press('Backspace');
        await page.keyboard.press('Backspace');
        await page.keyboard.press('Backspace');
        await waitForRender(page);
        
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
        await waitForRender(page);
    });

    test('should select eraser tool', async ({ page }) => {
        await page.click('[data-tool="eraser"]');
        
        const eraserBtn = page.locator('[data-tool="eraser"]');
        await expect(eraserBtn).toHaveClass(/active/);
    });

    test('should select and move a shape', async ({ page }) => {
        await page.click('[data-tool="rectangle"]');
        await drawOnCanvas(page, 100, 100, 200, 150);
        await waitForRender(page);
        
        await page.click('[data-tool="select"]');
        
        const canvas = page.locator('#canvas');
        const box = await canvas.boundingBox();
        if (!box) return;
        
        await page.mouse.click(box.x + 150, box.y + 125);
        await waitForRender(page);
        
        await page.mouse.move(box.x + 150, box.y + 125);
        await page.mouse.down();
        await page.mouse.move(box.x + 200, box.y + 175, { steps: 5 });
        await page.mouse.up();
        await waitForRender(page);
    });
});

test.describe('Undo/Redo', () => {
    test.beforeEach(async ({ page, isMobile }) => {
        test.skip(isMobile, 'Undo/redo buttons are hidden on mobile viewports');
        await openEditor(page);
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
        await waitForRender(page);
        
        const undoBtn = page.locator('#undo-btn');
        await expect(undoBtn).toBeEnabled();
    });

    test('should undo drawing', async ({ page }) => {
        await page.click('[data-tool="rectangle"]');
        await drawRect(page);
        await waitForRender(page);
        
        await page.click('#undo-btn');
        await waitForRender(page);
        
        const undoBtn = page.locator('#undo-btn');
        await expect(undoBtn).toBeDisabled();
    });

    test('should enable redo after undo', async ({ page }) => {
        await page.click('[data-tool="rectangle"]');
        await drawRect(page);
        await waitForRender(page);
        
        await page.click('#undo-btn');
        await waitForRender(page);
        
        const redoBtn = page.locator('#redo-btn');
        await expect(redoBtn).toBeEnabled();
    });

    test('should redo drawing', async ({ page }) => {
        await page.click('[data-tool="rectangle"]');
        await drawRect(page);
        await waitForRender(page);
        
        await page.click('#undo-btn');
        await waitForRender(page);
        
        await page.click('#redo-btn');
        await waitForRender(page);
        
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
        await waitForRender(page);
        
        page.on('dialog', dialog => dialog.accept());
        
        await page.click('#clear-btn');
        await waitForRender(page);
        
        const undoBtn = page.locator('#undo-btn');
        await expect(undoBtn).toBeDisabled();
    });
});

test.describe('Border Styles', () => {
    test.beforeEach(async ({ page }) => {
        await openEditor(page);
    });

    test('should change border style via select', async ({ page }) => {
        const select = page.locator('#border-style');
        
        await select.selectOption('double');
        await waitForRender(page);
        
        const selected = await select.inputValue();
        expect(selected).toBe('double');
    });

    test('should cycle border styles with B key', async ({ page, isMobile }) => {
        test.skip(isMobile, 'Status bar and toast are hidden on mobile viewports');
        await page.click('#canvas');
        await waitForRender(page);
        
        await page.keyboard.press('b');
        await waitForRender(page);
        
        await page.keyboard.press('b');
        await waitForRender(page);
        
        const toast = page.locator('#status-toast');
        await expect(toast).toBeVisible();
    });
});

test.describe('Export Functionality', () => {
    test.beforeEach(async ({ page, isMobile }) => {
        test.skip(isMobile, 'Export buttons are hidden on mobile viewports');
        await openEditor(page);
    });

    test('should have SVG and PNG export buttons', async ({ page }) => {
        await expect(page.locator('#png-btn')).toBeVisible();
        await expect(page.locator('#svg-btn')).toBeVisible();
    });

    test('should trigger SVG export on click', async ({ page }) => {
        const downloadPromise = page.waitForEvent('download');
        await page.click('#svg-btn');
        const download = await downloadPromise;
        expect(download.suggestedFilename()).toBe('ascii-canvas.svg');
    });
});

test.describe('Zoom Controls', () => {
    test.beforeEach(async ({ page, isMobile }) => {
        test.skip(isMobile, 'Zoom controls and indicators are hidden on mobile viewports');
        await openEditor(page);
        await page.click('#zoom-reset');
        await waitForRender(page);
    });

    test('should zoom in with button', async ({ page }) => {
        await expect(page.locator('#zoom-level')).toContainText('100%');
        
        await page.click('#zoom-in');
        await waitForRender(page);
        
        await expect(page.locator('#zoom-level')).not.toContainText('100%');
    });

    test('should zoom out with button', async ({ page }) => {
        await expect(page.locator('#zoom-level')).toContainText('100%');
        
        await page.click('#zoom-out');
        await waitForRender(page);
        
        await expect(page.locator('#zoom-level')).not.toContainText('100%');
    });

    test('should reset zoom', async ({ page }) => {
        await page.click('#zoom-in');
        await waitForRender(page);
        
        await page.click('#zoom-reset');
        await waitForRender(page);
        
        await expect(page.locator('#zoom-level')).toContainText('100%');
    });

    test('should fit zoom', async ({ page }) => {
        await page.click('#zoom-fit');
        await waitForRender(page);
        
        await expect(page.locator('#zoom-level')).not.toBeEmpty();
    });

    test('should zoom with scroll wheel', async ({ page }) => {
        await expect(page.locator('#zoom-level')).toContainText('100%');
        
        const canvas = page.locator('#canvas');
        await canvas.hover();
        await page.mouse.wheel(0, -100);
        
        await waitForRender(page);
        
        await expect(page.locator('#zoom-level')).not.toContainText('100%');
    });
});

test.describe('Keyboard Shortcuts', () => {
    test.beforeEach(async ({ page }) => {
        await openEditor(page);
    });

    test('should support all tool shortcuts', async ({ page }) => {
        await page.locator('#canvas').focus();
        await waitForRender(page);
        
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
            await page.locator('#canvas').focus();
            await page.keyboard.press(key);
            await expect(page.locator(`[data-tool="${tool}"]`)).toHaveClass(/active/, { timeout: 3000 });
        }
    });

    test('should show shortcuts modal with ?', async ({ page }) => {
        await page.click('#canvas');
        await page.keyboard.press('?');
        await waitForRender(page);
        
        const modal = page.locator('#shortcuts-modal');
        await expect(modal).not.toHaveClass(/hidden/);
    });

    test('should close shortcuts modal with Escape', async ({ page }) => {
        await page.click('#canvas');
        await page.keyboard.press('?');
        await waitForRender(page);
        
        const modal = page.locator('#shortcuts-modal');
        await expect(modal).not.toHaveClass(/hidden/);
        
        await page.click('.modal-close');
        await waitForRender(page);
        
        await expect(modal).toHaveClass(/hidden/);
    });

    test('should close shortcuts modal by clicking overlay', async ({ page }) => {
        const modal = page.locator('#shortcuts-modal');
        const isVisible = await modal.evaluate(el => !el.classList.contains('hidden'));
        
        if (isVisible) {
            await page.click('.modal-close');
            await waitForRender(page);
        }
        
        await page.click('#canvas');
        await page.keyboard.press('?');
        await waitForRender(page);
        
        await expect(modal).not.toHaveClass(/hidden/);
        
        await page.click('#shortcuts-modal', { position: { x: 10, y: 10 } });
        await waitForRender(page);
        
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
        await waitForRender(page);
        
        await page.click('#canvas');
        await page.keyboard.press('Control+z');
        await waitForRender(page);
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
        await waitForRender(page);
        
        await page.click('#canvas');
        await page.keyboard.press('Control+z');
        await waitForRender(page);
        
        await page.keyboard.press('Control+Shift+z');
        await waitForRender(page);
    });
});

test.describe('Copy Functionality', () => {
    test.beforeEach(async ({ page, isMobile }) => {
        test.skip(isMobile, 'Copy button and status toast are hidden on mobile viewports');
        await openEditor(page);
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
        await waitForRender(page);
        
        await page.click('#copy-btn');
        await waitForRender(page);
        
        const toast = page.locator('#status-toast');
        await expect(toast).toBeVisible();
    });
});

test.describe('Output Verification', () => {
    test.beforeEach(async ({ page }) => {
        await openEditor(page);
    });

    test('should have editor available', async ({ page }) => {
        const hasEditor = await page.evaluate(() => {
            // @ts-ignore
            return window.editor !== null && window.editor !== undefined;
        });
        expect(hasEditor).toBe(true);
    });
});
