/**
 * SVG export helper.
 */

import { logger } from './logger.js';
import type { ToastFn } from './clipboard.js';
import type { AsciiEditorInterface } from './types.js';

/**
 * Export the current ASCII canvas as SVG and trigger download.
 */
export function exportSvg(
    editor: AsciiEditorInterface,
    showToast: ToastFn,
    filename = 'ascii-canvas.svg',
): void {
    try {
        const svgContent = editor.exportSvg();
        const blob = new Blob([svgContent], { type: 'image/svg+xml;charset=utf-8' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = filename;
        a.click();

        // Clean up URL object after click
        setTimeout(() => {
            URL.revokeObjectURL(url);
        }, 100);

        showToast('Exported SVG');
    } catch (err) {
        logger.error('SVG export failed:', err);
        showToast('Failed to export SVG', true);
    }
}
