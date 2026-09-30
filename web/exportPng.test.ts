import { afterEach, describe, expect, it, vi } from 'vitest';
import { exportPng } from './exportPng.js';
import type { AsciiEditor } from './types.js';

describe('clean PNG export', () => {
    afterEach(() => {
        vi.restoreAllMocks();
        vi.unstubAllGlobals();
    });

    it('uses committed RGBA pixels at document dimensions, never the editor viewport', () => {
        const pixels = new Uint8Array(16 * 20 * 4).fill(77);
        const editor = { width: 2, height: 1, exportPixelBuffer: vi.fn(() => pixels),
            deselect: vi.fn(), requestRedraw: vi.fn() };
        const canvas = document.createElement('canvas');
        const link = document.createElement('a');
        vi.spyOn(document, 'createElement').mockReturnValueOnce(canvas).mockReturnValueOnce(link);
        const putImageData = vi.fn();
        vi.spyOn(canvas, 'getContext').mockReturnValue({ putImageData } as unknown as CanvasRenderingContext2D);
        const encode = vi.spyOn(canvas, 'toDataURL').mockReturnValue('data:image/png;base64,AA==');
        const click = vi.spyOn(link, 'click').mockImplementation(() => {});
        vi.stubGlobal('ImageData', class {
            constructor(public data: Uint8ClampedArray, public width: number, public height: number) {}
        });
        exportPng(editor as unknown as AsciiEditor, vi.fn());
        expect(editor.exportPixelBuffer).toHaveBeenCalledOnce();
        expect(putImageData).toHaveBeenCalledWith(expect.objectContaining({ width: 16, height: 20, data: new Uint8ClampedArray(pixels) }), 0, 0);
        expect(encode).toHaveBeenCalledWith('image/png');
        expect(click).toHaveBeenCalledOnce();
        expect(editor.deselect).not.toHaveBeenCalled();
        expect(editor.requestRedraw).not.toHaveBeenCalled();
        expect(canvas.width).toBe(16);
        expect(canvas.height).toBe(20);
        expect(link.href).toBe('data:image/png;base64,AA==');
        expect(link.download).toBe('ascii-canvas.png');
    });
});
