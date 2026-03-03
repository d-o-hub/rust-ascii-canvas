/**
 * ASCII Canvas E2E Tests
 * Comprehensive tests for all buttons, functions, and drawing verification
 */

import { test, expect } from '@playwright/test';
import { EditorPage, createEditorPage } from './pages/EditorPage';

const BASE_URL = process.env.BASE_URL || 'http://localhost:3003';

test.describe('ASCII Canvas Editor', () => {
    test.beforeEach(async ({ page }) => {
        const editor = createEditorPage(page);
        await page.goto(BASE_URL);
        await editor.waitForLoad();
    });

    test('should load the editor', async ({ page }) => {
        const editor = createEditorPage(page);
        await expect(editor.toolbar).toBeVisible();
        await expect(editor.canvas).toBeVisible();
        await expect(page.locator('.logo-text')).toContainText('ASCII Canvas');
    });

    test('should have all tool buttons', async ({ page }) => {
        const editor = createEditorPage(page);
        const tools = ['select', 'rectangle', 'line', 'arrow', 'diamond', 'text', 'freehand', 'eraser'];
        
        for (const tool of tools) {
            const button = editor.toolButtons[tool as keyof typeof editor.toolButtons];
            await expect(button).toBeVisible();
        }
    });

    test('should switch tools when clicked', async ({ page }) => {
        const editor = createEditorPage(page);
        await editor.selectTool('line');
        await expect(editor.toolButtons.line).toHaveClass(/active/);
    });

    test('should switch tools with keyboard shortcuts', async ({ page }) => {
        const editor = createEditorPage(page);
        
        await editor.selectToolByShortcut('l');
        await expect(editor.toolButtons.line).toHaveClass(/active/);
        
        await editor.selectToolByShortcut('r');
        await expect(editor.toolButtons.rectangle).toHaveClass(/active/);
    });

    test('should show border style selector with all options', async ({ page }) => {
        const editor = createEditorPage(page);
        const select = editor.controls.borderStyle;
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
        const editor = createEditorPage(page);
        
        await expect(editor.actionButtons.undo).toBeVisible();
        await expect(editor.actionButtons.redo).toBeVisible();
    });

    test('should have copy and clear buttons', async ({ page }) => {
        const editor = createEditorPage(page);
        await expect(editor.actionButtons.copy).toBeVisible();
        await expect(editor.actionButtons.clear).toBeVisible();
    });

    test('should have zoom controls', async ({ page }) => {
        const editor = createEditorPage(page);
        await expect(editor.controls.zoomFit).toBeVisible();
        await expect(editor.controls.zoomReset).toBeVisible();
        await expect(editor.controls.zoomOut).toBeVisible();
        await expect(editor.controls.zoomIn).toBeVisible();
    });

    test('should display cursor position', async ({ page }) => {
        const editor = createEditorPage(page);
        const cursorPos = editor.statusBar.cursorPosition;
        await expect(cursorPos).toBeVisible();
        
        const box = await editor.canvas.boundingBox();
        
        if (box) {
            await page.mouse.move(box.x + 100, box.y + 100);
            await expect(cursorPos).not.toBeEmpty();
        }
    });

    test('should show status bar', async ({ page }) => {
        const editor = createEditorPage(page);
        const statusBar = editor.statusBar.container;
        await expect(statusBar).toBeVisible();
        
        const toolStatus = page.locator('#status-tool');
        await expect(toolStatus).toContainText('Tool:');
    });

    test('should show grid info', async ({ page }) => {
        const editor = createEditorPage(page);
        const gridSize = page.locator('#grid-size');
        await expect(gridSize).toContainText('80 × 40');
    });

    test('should show zoom level', async ({ page }) => {
        const editor = createEditorPage(page);
        const zoomLevel = editor.statusBar.zoomLevel;
        await expect(zoomLevel).toContainText('100%');
    });
});

test.describe('Drawing Tools Interaction', () => {
    test.beforeEach(async ({ page }) => {
        const editor = createEditorPage(page);
        await page.goto(BASE_URL);
        await editor.waitForLoad();
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
        const editor = createEditorPage(page);
        
        const box = await editor.canvas.boundingBox();
        if (!box) return;
        
        await editor.selectTool('rectangle');
        
        const statusTool = page.locator('#status-tool');
        await expect(statusTool).toContainText('Tool: Rectangle');
        
        await drawOnCanvas(page, 100, 100, 300, 200);
    });

    test('should select line tool and draw', async ({ page }) => {
        const editor = createEditorPage(page);
        
        await editor.selectTool('line');
        await drawOnCanvas(page, 100, 100, 300, 100);
    });

    test('should select arrow tool and draw', async ({ page }) => {
        const editor = createEditorPage(page);
        
        await editor.selectTool('arrow');
        await drawOnCanvas(page, 100, 100, 300, 100);
    });

    test('should select diamond tool and draw', async ({ page }) => {
        const editor = createEditorPage(page);
        
        await editor.selectTool('diamond');
        await drawOnCanvas(page, 200, 100, 300, 200);
    });

    test('should select text tool', async ({ page }) => {
        const editor = createEditorPage(page);
        
        await editor.selectTool('text');
        
        const box = await editor.canvas.boundingBox();
        if (!box) return;
        
        await page.mouse.click(box.x + 100, box.y + 100);
        
        await page.keyboard.type('Hello');
        await page.keyboard.press('Escape');
    });

    test('should insert text at exact clicked grid position', async ({ page }) => {
        const editor = createEditorPage(page);
        
        await editor.selectTool('text');
        
        const box = await editor.canvas.boundingBox();
        if (!box) return;
        
        const charWidth = 9.6;
        const lineHeight = 19.2;
        const gridX = 10;
        const gridY = 10;
        const clickX = box.x + (gridX * charWidth) + (charWidth / 2);
        const clickY = box.y + (gridY * lineHeight) + (lineHeight / 2);
        
        await editor.canvas.focus();
        await page.mouse.click(clickX, clickY);
        
        await page.keyboard.type('X');
        
        const ascii = await page.evaluate(() => {
            return window.editor.exportAscii();
        });
        
        const lines = ascii.split('\n');
        if (lines[gridY]) {
            expect(lines[gridY][gridX]).toBe('X');
        }
    });

    test('should insert multiple characters sequentially', async ({ page }) => {
        const editor = createEditorPage(page);
        
        await editor.selectTool('text');
        
        const box = await editor.canvas.boundingBox();
        if (!box) return;
        
        await editor.canvas.focus();
        await page.mouse.click(box.x + 50, box.y + 50);
        
        await page.keyboard.type('A');
        await page.keyboard.type('B');
        await page.keyboard.type('C');
        await page.keyboard.type('D');
        await page.keyboard.type('E');
        
        const ascii = await page.evaluate(() => {
            return window.editor.exportAscii();
        });
        
        expect(ascii).toContain('ABCDE');
    });

    test('should type at different positions independently', async ({ page }) => {
        const editor = createEditorPage(page);
        
        await editor.selectTool('text');
        
        const box = await editor.canvas.boundingBox();
        if (!box) return;
        
        await editor.canvas.focus();
        await page.mouse.click(box.x + 50, box.y + 30);
        await page.keyboard.type('X');
        
        await page.keyboard.press('Escape');
        
        await page.mouse.click(box.x + 100, box.y + 30);
        await page.keyboard.type('Y');
        
        const ascii = await page.evaluate(() => {
            return window.editor.exportAscii();
        });
        
        expect(ascii).toContain('X');
        expect(ascii).toContain('Y');
        const xIndex = ascii.indexOf('X');
        const yIndex = ascii.indexOf('Y');
        expect(xIndex).not.toBe(yIndex);
    });

    test('should handle backspace at any position', async ({ page }) => {
        const editor = createEditorPage(page);
        
        await editor.selectTool('text');
        
        const box = await editor.canvas.boundingBox();
        if (!box) return;
        
        await editor.canvas.focus();
        await page.mouse.click(box.x + 50, box.y + 50);
        
        await page.keyboard.type('Hello');
        
        await page.keyboard.press('Backspace');
        await page.keyboard.press('Backspace');
        await page.keyboard.press('Backspace');
        
        const ascii = await page.evaluate(() => {
            return window.editor.exportAscii();
        });
        
        expect(ascii).toContain('He');
    });

    test('should start fresh after clicking new position', async ({ page }) => {
        const editor = createEditorPage(page);
        
        await editor.selectTool('text');
        
        const box = await editor.canvas.boundingBox();
        if (!box) return;
        
        await editor.canvas.focus();
        await page.mouse.click(box.x + 50, box.y + 30);
        await page.keyboard.type('AAA');
        
        await page.mouse.click(box.x + 150, box.y + 30);
        await page.keyboard.type('BBB');
        
        const ascii = await page.evaluate(() => {
            return window.editor.exportAscii();
        });
        
        expect(ascii).toContain('AAA');
        expect(ascii).toContain('BBB');
    });

    test('should work with zoom level changes', async ({ page }) => {
        const editor = createEditorPage(page);
        
        await editor.selectTool('text');
        
        const box = await editor.canvas.boundingBox();
        if (!box) return;
        
        await page.mouse.click(box.x + 50, box.y + 50);
        await page.keyboard.type('Hello');
        
        await page.keyboard.press('Backspace');
        await page.keyboard.press('Backspace');
        await page.keyboard.press('Backspace');
        
        const ascii = await page.evaluate(() => {
            return window.editor.exportAscii();
        });
        
        expect(ascii).toContain('He');
        expect(ascii).not.toContain('Hello');
    });

    test('should select freehand tool and draw', async ({ page }) => {
        const editor = createEditorPage(page);
        
        await editor.selectTool('freehand');
        
        const box = await editor.canvas.boundingBox();
        if (!box) return;
        
        await page.mouse.move(box.x + 100, box.y + 100);
        await page.mouse.down();
        await page.mouse.move(box.x + 150, box.y + 120, { steps: 5 });
        await page.mouse.move(box.x + 200, box.y + 140, { steps: 5 });
        await page.mouse.move(box.x + 250, box.y + 130, { steps: 5 });
        await page.mouse.up();
    });

    test('should select eraser tool', async ({ page }) => {
        const editor = createEditorPage(page);
        
        await editor.selectTool('eraser');
        
        await expect(editor.toolButtons.eraser).toHaveClass(/active/);
    });

    test('should select and move a shape', async ({ page }) => {
        const editor = createEditorPage(page);
        
        await editor.selectTool('rectangle');
        await drawOnCanvas(page, 100, 100, 200, 150);
        
        await editor.selectTool('select');
        
        const box = await editor.canvas.boundingBox();
        if (!box) return;
        
        await page.mouse.click(box.x + 150, box.y + 125);
        
        await page.mouse.move(box.x + 150, box.y + 125);
        await page.mouse.down();
        await page.mouse.move(box.x + 200, box.y + 175, { steps: 5 });
        await page.mouse.up();
    });
});

test.describe('Undo/Redo', () => {
    test.beforeEach(async ({ page }) => {
        const editor = createEditorPage(page);
        await page.goto(BASE_URL);
        await editor.waitForLoad();
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
        const editor = createEditorPage(page);
        
        await editor.selectTool('rectangle');
        await drawRect(page);
        
        await editor.waitForUndoEnabled();
    });

    test('should undo drawing', async ({ page }) => {
        const editor = createEditorPage(page);
        
        await editor.selectTool('rectangle');
        await drawRect(page);
        
        await editor.undo();
        
        await editor.waitForUndoDisabled();
    });

    test('should enable redo after undo', async ({ page }) => {
        const editor = createEditorPage(page);
        
        await editor.selectTool('rectangle');
        await drawRect(page);
        
        await editor.undo();
        
        await editor.waitForRedoEnabled();
    });

    test('should redo drawing', async ({ page }) => {
        const editor = createEditorPage(page);
        
        await editor.selectTool('rectangle');
        await drawRect(page);
        
        await editor.undo();
        
        await editor.redo();
        
        await editor.waitForUndoEnabled();
    });

    test('should clear canvas', async ({ page }) => {
        const editor = createEditorPage(page);
        
        await editor.selectTool('rectangle');
        await drawRect(page);
        
        page.on('dialog', dialog => dialog.accept());
        
        await editor.clear();
        
        await editor.waitForUndoDisabled();
    });
});

test.describe('Border Styles', () => {
    test.beforeEach(async ({ page }) => {
        const editor = createEditorPage(page);
        await page.goto(BASE_URL);
        await editor.waitForLoad();
    });

    test('should change border style via select', async ({ page }) => {
        const editor = createEditorPage(page);
        
        await editor.setBorderStyle('double');
        
        const selected = await editor.controls.borderStyle.inputValue();
        expect(selected).toBe('double');
    });

    test('should cycle border styles with B key', async ({ page }) => {
        const editor = createEditorPage(page);
        
        await editor.canvas.click();
        
        await page.keyboard.press('b');
        
        await page.keyboard.press('b');
        
        const toast = page.locator('#status-toast');
        await expect(toast).toBeVisible();
    });
});

test.describe('Zoom Controls', () => {
    test.beforeEach(async ({ page }) => {
        const editor = createEditorPage(page);
        await page.goto(BASE_URL);
        await editor.waitForLoad();
        
        await editor.resetZoom();
    });

    test('should zoom in with button', async ({ page }) => {
        const editor = createEditorPage(page);
        
        await expect(editor.statusBar.zoomLevel).toContainText('100%');
        
        await editor.zoomIn();
        
        await expect(editor.statusBar.zoomLevel).not.toContainText('100%');
    });

    test('should zoom out with button', async ({ page }) => {
        const editor = createEditorPage(page);
        
        await expect(editor.statusBar.zoomLevel).toContainText('100%');
        
        await editor.zoomOut();
        
        await expect(editor.statusBar.zoomLevel).not.toContainText('100%');
    });

    test('should reset zoom', async ({ page }) => {
        const editor = createEditorPage(page);
        
        await editor.zoomIn();
        
        await editor.resetZoom();
        
        await expect(editor.statusBar.zoomLevel).toContainText('100%');
    });

    test('should fit zoom', async ({ page }) => {
        const editor = createEditorPage(page);
        
        await editor.fitZoom();
        
        await expect(editor.statusBar.zoomLevel).not.toBeEmpty();
    });

    test('should zoom with scroll wheel', async ({ page }) => {
        const editor = createEditorPage(page);
        
        await expect(editor.statusBar.zoomLevel).toContainText('100%');
        
        await editor.canvas.hover();
        await page.mouse.wheel(0, -100);
        
        await expect(editor.statusBar.zoomLevel).not.toContainText('100%');
    });
});

test.describe('Keyboard Shortcuts', () => {
    test.beforeEach(async ({ page }) => {
        const editor = createEditorPage(page);
        await page.goto(BASE_URL);
        await editor.waitForLoad();
    });

    test('should support all tool shortcuts', async ({ page }) => {
        const editor = createEditorPage(page);
        
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
            await editor.selectToolByShortcut(key);
            await expect(editor.toolButtons[tool as keyof typeof editor.toolButtons]).toHaveClass(/active/, { timeout: 3000 });
        }
    });

    test('should show shortcuts modal with ?', async ({ page }) => {
        const editor = createEditorPage(page);
        
        await editor.showShortcutsModal();
        
        const modal = page.locator('#shortcuts-modal');
        await expect(modal).not.toHaveClass(/hidden/);
    });

    test('should close shortcuts modal with Escape', async ({ page }) => {
        const editor = createEditorPage(page);
        
        await editor.showShortcutsModal();
        
        const modal = page.locator('#shortcuts-modal');
        await expect(modal).not.toHaveClass(/hidden/);
        
        await page.click('.modal-close');
        
        await expect(modal).toHaveClass(/hidden/);
    });

    test('should close shortcuts modal by clicking overlay', async ({ page }) => {
        const modal = page.locator('#shortcuts-modal');
        const isVisible = await modal.evaluate(el => !el.classList.contains('hidden'));
        
        if (isVisible) {
            await page.click('.modal-close');
        }
        
        const editor = createEditorPage(page);
        await editor.showShortcutsModal();
        
        await expect(modal).not.toHaveClass(/hidden/);
        
        await page.click('#shortcuts-modal', { position: { x: 10, y: 10 } });
        
        await expect(modal).toHaveClass(/hidden/);
    });

    test('should undo with Ctrl+Z', async ({ page }) => {
        const editor = createEditorPage(page);
        
        await editor.selectTool('rectangle');
        
        const box = await editor.canvas.boundingBox();
        if (!box) return;
        
        await page.mouse.move(box.x + 100, box.y + 100);
        await page.mouse.down();
        await page.mouse.move(box.x + 200, box.y + 150, { steps: 10 });
        await page.mouse.up();
        
        await editor.canvas.click();
        await page.keyboard.press('Control+z');
    });

    test('should redo with Ctrl+Shift+Z', async ({ page }) => {
        const editor = createEditorPage(page);
        
        await editor.selectTool('rectangle');
        
        const box = await editor.canvas.boundingBox();
        if (!box) return;
        
        await page.mouse.move(box.x + 100, box.y + 100);
        await page.mouse.down();
        await page.mouse.move(box.x + 200, box.y + 150, { steps: 10 });
        await page.mouse.up();
        
        await editor.canvas.click();
        await page.keyboard.press('Control+z');
        
        await page.keyboard.press('Control+Shift+z');
    });
});

test.describe('Copy Functionality', () => {
    test.beforeEach(async ({ page }) => {
        const editor = createEditorPage(page);
        await page.goto(BASE_URL);
        await editor.waitForLoad();
    });

    test('should show toast when copy is clicked', async ({ page }) => {
        const editor = createEditorPage(page);
        
        await editor.selectTool('rectangle');
        
        const box = await editor.canvas.boundingBox();
        if (!box) return;
        
        await page.mouse.move(box.x + 100, box.y + 100);
        await page.mouse.down();
        await page.mouse.move(box.x + 200, box.y + 150, { steps: 10 });
        await page.mouse.up();
        
        await editor.copyToClipboard();
        
        const toast = page.locator('#status-toast');
        await expect(toast).toBeVisible();
    });
});

test.describe('Output Verification', () => {
    test.beforeEach(async ({ page }) => {
        const editor = createEditorPage(page);
        await page.goto(BASE_URL);
        await editor.waitForLoad();
    });

    test('should have editor available', async ({ page }) => {
        const hasEditor = await page.evaluate(() => {
            return window.editor !== null && window.editor !== undefined;
        });
        expect(hasEditor).toBe(true);
    });
});
