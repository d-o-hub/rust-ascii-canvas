/**
 * ASCII Canvas Editor Page Object Model
 * Encapsulates all interactions with the ASCII Canvas Editor
 */

import { type Locator, type Page, expect } from '@playwright/test';

export class EditorPage {
    readonly page: Page;

    readonly toolbar: Locator;
    readonly canvas: Locator;
    readonly loadingOverlay: Locator;

    readonly toolButtons: {
        select: Locator;
        rectangle: Locator;
        line: Locator;
        arrow: Locator;
        diamond: Locator;
        text: Locator;
        freehand: Locator;
        eraser: Locator;
    };

    readonly actionButtons: {
        undo: Locator;
        redo: Locator;
        copy: Locator;
        clear: Locator;
    };

    readonly controls: {
        borderStyle: Locator;
        zoomIn: Locator;
        zoomOut: Locator;
        zoomReset: Locator;
        zoomFit: Locator;
    };

    readonly statusBar: {
        container: Locator;
        cursorPosition: Locator;
        gridInfo: Locator;
        zoomLevel: Locator;
    };

    readonly modal: {
        container: Locator;
        closeButton: Locator;
    };

    constructor(page: Page) {
        this.page = page;

        this.toolbar = page.locator('.toolbar');
        this.canvas = page.locator('#canvas');
        this.loadingOverlay = page.locator('#loading.hidden');

        this.toolButtons = {
            select: page.locator('[data-tool="select"]'),
            rectangle: page.locator('[data-tool="rectangle"]'),
            line: page.locator('[data-tool="line"]'),
            arrow: page.locator('[data-tool="arrow"]'),
            diamond: page.locator('[data-tool="diamond"]'),
            text: page.locator('[data-tool="text"]'),
            freehand: page.locator('[data-tool="freehand"]'),
            eraser: page.locator('[data-tool="eraser"]'),
        };

        this.actionButtons = {
            undo: page.locator('#undo-btn'),
            redo: page.locator('#redo-btn'),
            copy: page.locator('#copy-btn'),
            clear: page.locator('#clear-btn'),
        };

        this.controls = {
            borderStyle: page.locator('#border-style'),
            zoomIn: page.locator('#zoom-in'),
            zoomOut: page.locator('#zoom-out'),
            zoomReset: page.locator('#zoom-reset'),
            zoomFit: page.locator('#zoom-fit'),
        };

        this.statusBar = {
            container: page.locator('.status-bar'),
            cursorPosition: page.locator('#cursor-pos'),
            gridInfo: page.locator('#grid-info'),
            zoomLevel: page.locator('#zoom-level'),
        };

        this.modal = {
            container: page.locator('.modal'),
            closeButton: page.locator('.modal .close'),
        };
    }

    async waitForLoad(): Promise<void> {
        await this.page.waitForSelector('#loading.hidden', { timeout: 15000 });
        await this.page.waitForSelector('#canvas', { timeout: 10000 });
    }

    async selectTool(toolName: keyof typeof this.toolButtons): Promise<void> {
        const button = this.toolButtons[toolName];
        await button.click();
        await expect(button).toHaveClass(/active/);
    }

    async selectToolByShortcut(shortcut: string): Promise<void> {
        // Focus the canvas properly
        await this.canvas.focus();
        await this.page.waitForTimeout(50);
        await this.page.keyboard.press(shortcut.toLowerCase());
        await this.page.waitForTimeout(100);
    }

    async drawWithTool(
        toolName: keyof typeof this.toolButtons,
        startX: number,
        startY: number,
        endX: number,
        endY: number
    ): Promise<void> {
        await this.selectTool(toolName);
        await this.canvas.click();
        await this.page.mouse.move(startX, startY);
        await this.page.mouse.down();
        await this.page.mouse.move(endX, endY);
        await this.page.mouse.up();
    }

    async drawRectangle(x1: number, y1: number, x2: number, y2: number): Promise<void> {
        await this.drawWithTool('rectangle', x1, y1, x2, y2);
    }

    async drawLine(x1: number, y1: number, x2: number, y2: number): Promise<void> {
        await this.drawWithTool('line', x1, y1, x2, y2);
    }

    async drawArrow(x1: number, y1: number, x2: number, y2: number): Promise<void> {
        await this.drawWithTool('arrow', x1, y1, x2, y2);
    }

    async drawDiamond(x1: number, y1: number, x2: number, y2: number): Promise<void> {
        await this.drawWithTool('diamond', x1, y1, x2, y2);
    }

    async drawFreehand(points: Array<{ x: number; y: number }>): Promise<void> {
        await this.selectTool('freehand');
        await this.canvas.click();
        await this.page.mouse.down();
        for (const point of points) {
            await this.page.mouse.move(point.x, point.y);
        }
        await this.page.mouse.up();
    }

    async eraseAt(x: number, y: number): Promise<void> {
        await this.selectTool('eraser');
        await this.canvas.click();
        await this.page.mouse.click(x, y);
    }

    async insertText(x: number, y: number, text: string): Promise<void> {
        await this.selectTool('text');
        await this.canvas.click({ position: { x, y } });
        await this.page.keyboard.type(text);
    }

    async setBorderStyle(style: string): Promise<void> {
        await this.controls.borderStyle.selectOption(style);
    }

    async undo(): Promise<void> {
        await this.actionButtons.undo.click();
    }

    async redo(): Promise<void> {
        await this.actionButtons.redo.click();
    }

    async clear(): Promise<void> {
        await this.actionButtons.clear.click();
    }

    async copyToClipboard(): Promise<void> {
        await this.actionButtons.copy.click();
    }

    async zoomIn(): Promise<void> {
        await this.controls.zoomIn.click();
    }

    async zoomOut(): Promise<void> {
        await this.controls.zoomOut.click();
    }

    async resetZoom(): Promise<void> {
        await this.controls.zoomReset.click();
    }

    async fitZoom(): Promise<void> {
        await this.controls.zoomFit.click();
    }

    async zoomWithWheel(delta: number): Promise<void> {
        await this.canvas.click();
        await this.page.mouse.wheel(0, delta);
    }

    async panWithSpaceDrag(startX: number, startY: number, endX: number, endY: number): Promise<void> {
        await this.canvas.click();
        await this.page.keyboard.down(' ');
        await this.page.mouse.move(startX, startY);
        await this.page.mouse.down();
        await this.page.mouse.move(endX, endY);
        await this.page.mouse.up();
        await this.page.keyboard.up(' ');
    }

    async showShortcutsModal(): Promise<void> {
        await this.canvas.click();
        await this.page.keyboard.press('?');
    }

    async hideShortcutsModal(): Promise<void> {
        await this.page.keyboard.press('Escape');
    }

    async getCursorPosition(): Promise<{ x: number; y: number }> {
        const text = await this.statusBar.cursorPosition.textContent();
        const match = text?.match(/X:\s*(\d+)\s*Y:\s*(\d+)/);
        if (match) {
            return { x: parseInt(match[1], 10), y: parseInt(match[2], 10) };
        }
        return { x: 0, y: 0 };
    }

    async getZoomLevel(): Promise<number> {
        const text = await this.statusBar.zoomLevel.textContent();
        const match = text?.match(/(\d+)%/);
        return match ? parseInt(match[1], 10) : 100;
    }

    async getGridInfo(): Promise<{ width: number; height: number }> {
        const text = await this.statusBar.gridInfo.textContent();
        const match = text?.match(/(\d+)\s*x\s*(\d+)/);
        if (match) {
            return { width: parseInt(match[1], 10), height: parseInt(match[2], 10) };
        }
        return { width: 0, height: 0 };
    }

    async isUndoEnabled(): Promise<boolean> {
        const isDisabled = await this.actionButtons.undo.getAttribute('disabled');
        return isDisabled === null;
    }

    async isRedoEnabled(): Promise<boolean> {
        const isDisabled = await this.actionButtons.redo.getAttribute('disabled');
        return isDisabled === null;
    }

    async waitForUndoEnabled(): Promise<void> {
        await expect(this.actionButtons.undo).toBeEnabled();
    }

    async waitForRedoEnabled(): Promise<void> {
        await expect(this.actionButtons.redo).toBeEnabled();
    }

    async waitForUndoDisabled(): Promise<void> {
        await expect(this.actionButtons.undo).toBeDisabled();
    }

    async waitForRedoDisabled(): Promise<void> {
        await expect(this.actionButtons.redo).toBeDisabled();
    }
}

export function createEditorPage(page: Page): EditorPage {
    return new EditorPage(page);
}
