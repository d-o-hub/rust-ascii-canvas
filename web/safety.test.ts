import { beforeEach, describe, expect, it, vi } from 'vitest';
import { state } from './state.js';
import { resizeCanvas } from './render.js';
import { applyCustomGridSize, handlePasteEvent } from './document-events.js';

beforeEach(() => {
    document.body.innerHTML = '<canvas id="canvas"></canvas><div id="container"></div><input id="grid-width" value="40"><input id="grid-height" value="20">';
    state.canvas = document.querySelector('canvas');
    state.canvasContainer = document.querySelector('#container');
    state.ctx = { scale: vi.fn() } as unknown as CanvasRenderingContext2D;
    state.editor = {
        width: 100, height: 50, resize: vi.fn(), setFontMetrics: vi.fn(),
        pasteText: vi.fn(), paste: vi.fn(),
    } as unknown as typeof state.editor;
    state.gridSizeLocked = false;
    state.animationFrameId = 1;
});

describe('document safety at browser boundaries', () => {
    it('never resizes an existing document with the viewport, even if unlocked', () => {
        resizeCanvas();
        expect(state.editor?.resize).not.toHaveBeenCalled();
    });

    it('cancelling a destructive crop leaves dimensions, caches, history and save state untouched', () => {
        vi.stubGlobal('confirm', vi.fn().mockReturnValue(false));
        const cache = document.createElement('canvas');
        state.offscreenCanvas = cache;
        applyCustomGridSize();
        expect(window.confirm).toHaveBeenCalled();
        expect(state.editor?.resize).not.toHaveBeenCalled();
        expect(state.offscreenCanvas).toBe(cache);
        expect(state.gridSizeLocked).toBe(false);
        expect(document.querySelector<HTMLInputElement>('#grid-width')?.value).toBe('100');
    });

    it.each(['input', 'textarea', 'div'])('leaves native paste into a focused %s alone', (tag) => {
        const field = document.createElement(tag);
        if (tag === 'div') {
            field.contentEditable = 'true';
            field.tabIndex = 0;
        }
        document.body.append(field);
        field.focus();
        const event = new Event('paste', { cancelable: true }) as ClipboardEvent;
        Object.defineProperty(event, 'clipboardData', { value: { getData: () => 'pasted text' } });
        handlePasteEvent(event);
        expect(event.defaultPrevented).toBe(false);
        expect(state.editor?.pasteText).not.toHaveBeenCalled();
        expect(state.editor?.paste).not.toHaveBeenCalled();
    });
});
