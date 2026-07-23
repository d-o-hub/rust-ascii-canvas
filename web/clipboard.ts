/**
 * Clipboard helpers for 1:1 ASCII paste into external editors.
 */

import { logger } from './logger.js';
import { normalizeToCRLF } from './utils.js';
import type { AsciiEditorInterface } from './types.js';

export type ToastFn = (message: string, isError?: boolean) => void;

/**
 * Copy ASCII to the system clipboard with CRLF line endings.
 *
 * Uses plain text only (not text/html) so external paste targets receive the same
 * characters without Codacy/eslint-plugin-xss false positives on HTML clipboard MIME.
 */
export async function copyAsciiToClipboard(text: string, showToast: ToastFn): Promise<boolean> {
    const normalized = normalizeToCRLF(text);

    try {
        await navigator.clipboard.writeText(normalized);
        showToast('Copied — paste in a monospace editor');
        return true;
    } catch (err) {
        logger.error('Failed to copy:', err);
        showToast('Failed to copy', true);
        return false;
    }
}

/**
 * Selection-aware copy: fills internal clipboard and writes OS clipboard text.
 */
export async function copyToClipboard(
    editor: AsciiEditorInterface,
    showToast: ToastFn,
): Promise<boolean> {
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

    return await copyAsciiToClipboard(ascii, showToast);
}
