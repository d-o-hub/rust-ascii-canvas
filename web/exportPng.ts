/**
 * PNG export from clean, committed document pixels (never editor overlays).
 */

import { logger } from './logger.js';
import type { ToastFn } from './clipboard.js';
import type { AsciiEditor } from './types.js';

/**
 * Export the given canvas (or offscreen buffer drawn into a temp canvas) as PNG download.
 */
export function exportCanvasAsPng(
    source: HTMLCanvasElement,
    showToast: ToastFn,
    filename = 'ascii-canvas.png',
): void {
    try {
        const url = source.toDataURL('image/png');
        const a = document.createElement('a');
        a.href = url;
        a.download = filename;
        a.click();
        showToast('Exported PNG');
    } catch (err) {
        logger.error('PNG export failed:', err);
        showToast('Failed to export PNG', true);
    }
}

/** The WASM export is immutable: selection, preview, zoom and history survive. */
export function exportPng(editor: AsciiEditor, showToast: ToastFn): void {
    try {
        const width = editor.width * 8;
        const height = editor.height * 20;
        const pixels = editor.exportPixelBuffer();
        if (pixels.length !== width * height * 4) throw new Error('Invalid export pixel dimensions');
        const source = document.createElement('canvas');
        source.width = width;
        source.height = height;
        const context = source.getContext('2d');
        if (!context) throw new Error('Canvas context unavailable');
        context.putImageData(new ImageData(new Uint8ClampedArray(pixels), width, height), 0, 0);
        exportCanvasAsPng(source, showToast);
    } catch (error) {
        logger.error('Clean PNG export failed:', error);
        showToast('Failed to export PNG', true);
    }
}
