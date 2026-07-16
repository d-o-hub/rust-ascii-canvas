/**
 * PNG export from the rendered canvas / offscreen buffer.
 */

import { logger } from './logger.js';
import type { ToastFn } from './clipboard.js';

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

/**
 * Export the pixel-buffer offscreen canvas if available, otherwise the main canvas.
 */
export function exportPng(
    offscreen: HTMLCanvasElement | null,
    mainCanvas: HTMLCanvasElement | null,
    showToast: ToastFn,
): void {
    const source = offscreen ?? mainCanvas;
    if (!source) {
        showToast('Nothing to export', true);
        return;
    }
    exportCanvasAsPng(source, showToast);
}
