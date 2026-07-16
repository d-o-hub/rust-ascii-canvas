/**
 * Clipboard helpers for 1:1 ASCII paste into external editors.
 */

import { logger } from './logger.js';
import { normalizeToCRLF } from './utils.js';
import type { AsciiEditorInterface } from './types.js';

export type ToastFn = (message: string, isError?: boolean) => void;

/**
 * Escape text for embedding in a static HTML template (clipboard rich-text fallback).
 */
function escapeHtml(text: string): string {
    return text
        .replace(/&/g, '&amp;')
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;')
        .replace(/"/g, '&quot;');
}

/**
 * Copy ASCII to the system clipboard with CRLF line endings and HTML fallback.
 */
export async function copyAsciiToClipboard(text: string, showToast: ToastFn): Promise<void> {
    const normalized = normalizeToCRLF(text);

    try {
        const plain = new Blob([normalized], { type: 'text/plain' });
        // Build HTML from escaped text only (no outerHTML / DOM serialization) for Codacy XSS tools.
        const safeBody = escapeHtml(normalized);
        const htmlPayload =
            `<pre style="font-family:'JetBrains Mono','Cascadia Code','Courier New',monospace;font-size:14px;line-height:1.4;white-space:pre">${safeBody}</pre>`;

        await navigator.clipboard.write([
            new ClipboardItem({
                'text/plain': plain,
                'text/html': new Blob([htmlPayload], { type: 'text/html' }),
            }),
        ]);

        showToast('Copied — paste in a monospace editor');
    } catch (err) {
        logger.error('Failed to copy:', err);
        try {
            await navigator.clipboard.writeText(normalized);
            showToast('Copied — paste in a monospace editor');
        } catch {
            showToast('Failed to copy', true);
        }
    }
}

/**
 * Selection-aware copy: fills internal clipboard and writes OS clipboard text.
 */
export async function copyToClipboard(
    editor: AsciiEditorInterface,
    showToast: ToastFn,
): Promise<void> {
    // Populate internal SelectionClipboard for Ctrl+V paste inside the editor.
    if (typeof editor.copySelection === 'function') {
        editor.copySelection();
    }

    let ascii: string;
    if (typeof editor.exportForCopy === 'function') {
        ascii = editor.exportForCopy();
    } else {
        ascii = editor.exportAscii();
    }

    await copyAsciiToClipboard(ascii, showToast);
}
