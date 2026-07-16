/**
 * Clipboard helpers for 1:1 ASCII paste into external editors.
 */

import { logger } from './logger.js';
import { normalizeToCRLF } from './utils.js';
import type { AsciiEditorInterface } from './types.js';

export type ToastFn = (message: string, isError?: boolean) => void;

/**
 * Copy ASCII to the system clipboard with CRLF line endings and HTML fallback.
 */
export async function copyAsciiToClipboard(text: string, showToast: ToastFn): Promise<void> {
    const normalized = normalizeToCRLF(text);

    try {
        const plain = new Blob([normalized], { type: 'text/plain' });

        const pre = document.createElement('pre');
        pre.style.fontFamily = "'JetBrains Mono','Cascadia Code','Courier New',monospace";
        pre.style.fontSize = '14px';
        pre.style.lineHeight = '1.4';
        pre.style.whiteSpace = 'pre';
        pre.textContent = text; // HTML preserves visual lines; text nodes use original content
        const html = pre.outerHTML;
        const rich = new Blob([html], { type: 'text/html' });

        await navigator.clipboard.write([
            new ClipboardItem({
                'text/plain': plain,
                'text/html': rich,
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
