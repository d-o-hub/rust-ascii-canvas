/**
 * Clipboard helpers for 1:1 ASCII paste into external editors.
 */

import { logger } from './logger.js';
import { normalizeToCRLF } from './utils.js';
import type { AsciiEditor } from './types.js';

export type ToastFn = (message: string, isError?: boolean) => void;

export interface ClipboardOptions {
    /** Convert drawing glyphs to a 7-bit ASCII representation. */
    convertUnicodeToAscii: boolean;
    /** Keep each exported line at its rectangular width, including trailing spaces. */
    enforceBoundingBox: boolean;
}

export const DEFAULT_CLIPBOARD_OPTIONS: ClipboardOptions = {
    convertUnicodeToAscii: false,
    enforceBoundingBox: true,
};

/** Read the copy controls, falling back to fidelity-preserving defaults in tests or embeds. */
export function getClipboardOptions(): ClipboardOptions {
    if (typeof document === 'undefined') {
        return { ...DEFAULT_CLIPBOARD_OPTIONS };
    }

    const format = document.querySelector<HTMLSelectElement>('#copy-format')?.value;
    const preserveWidth = document.querySelector<HTMLInputElement>('#copy-bounding-box')?.checked;
    return {
        convertUnicodeToAscii: format === 'ascii',
        enforceBoundingBox: preserveWidth ?? DEFAULT_CLIPBOARD_OPTIONS.enforceBoundingBox,
    };
}

// Mirrors core::ascii_export::ascii_fallback_char (src/core/ascii_export.rs).
// Keep both maps in sync — the Rust map is the source of truth (ADR-041).
function asciiFallbackChar(char: string): string {
    switch (char) {
        case '─': case '━': case '═': case '╌': case '╍': case '┄': case '┅': case '┈': case '┉':
        case '╴': case '╶': case '╸': case '╺':
            return '-';
        case '│': case '┃': case '║': case '┆': case '┇': case '┊': case '┋': case '╎': case '╏':
        case '╵': case '╷': case '╹': case '╻':
            return '|';
        case '┌': case '┐': case '└': case '┘': case '┏': case '┓': case '┗': case '┛':
        case '╔': case '╗': case '╚': case '╝': case '╭': case '╮': case '╰': case '╯':
        case '┼': case '╋': case '╬': case '╁': case '╂': case '╃': case '╄': case '╅': case '╆':
        case '╇': case '╈': case '╉': case '╊': case '╳': case '┬': case '┴': case '├': case '┤':
        case '╞': case '╡': case '╥': case '╨': case '╪': case '╫': case '╀':
            return '+';
        case '╱': return '/';
        case '╲': return '\\';
        case '◆': case '◇': case '●': case '•': case '·': return '*';
        case '▲': case '△': return '^';
        case '▼': case '▽': return 'v';
        case '◀': case '◁': return '<';
        case '▶': case '▷': return '>';
        default: return char.charCodeAt(0) < 128 ? char : '?';
    }
}

/** Apply the selected representation while retaining logical line boundaries. */
export function formatClipboardText(text: string, options: ClipboardOptions): string {
    const lines = text.split(/\r?\n/u).map((line) => options.convertUnicodeToAscii
        ? Array.from(line, asciiFallbackChar).join('')
        : line);

    if (options.enforceBoundingBox) {
        const width = Math.max(0, ...lines.map((line) => Array.from(line).length));
        return normalizeToCRLF(lines.map((line) => {
            const padding = width - Array.from(line).length;
            return `${line}${' '.repeat(padding)}`;
        }).join('\n'));
    }

    return normalizeToCRLF(lines.map((line) => line.replace(/[ \t]+$/u, '')).join('\n'));
}

/**
 * Copy text through the modern Clipboard API, falling back to a temporary textarea
 * for insecure, permission-restricted, or embedded browser contexts.
 */
export async function writeTextToClipboard(text: string): Promise<void> {
    let clipboardError: unknown;

    try {
        await navigator.clipboard.writeText(text);
        return;
    } catch (error) {
        clipboardError = error;
    }

    if (copyTextWithExecCommand(text)) {
        return;
    }

    if (clipboardError instanceof Error) {
        throw clipboardError;
    }
    throw new Error('Unable to copy text to the clipboard');
}

/** Attempt the legacy synchronous copy path. */
export function copyTextWithExecCommand(text: string): boolean {
    if (typeof document === 'undefined' || typeof document.execCommand !== 'function') {
        return false;
    }

    const textArea = document.createElement('textarea');
    textArea.value = text;
    textArea.setAttribute('readonly', '');
    textArea.setAttribute('aria-hidden', 'true');
    textArea.style.position = 'fixed';
    textArea.style.opacity = '0';
    textArea.style.pointerEvents = 'none';

    const parent = document.body;
    parent.appendChild(textArea);
    try {
        textArea.focus();
        textArea.select();
        return document.execCommand('copy');
    } catch {
        return false;
    } finally {
        textArea.remove();
    }
}

/** Copy ASCII to the system clipboard with CRLF line endings. */
export async function copyAsciiToClipboard(
    text: string,
    showToast: ToastFn,
    options: ClipboardOptions = getClipboardOptions(),
): Promise<void> {
    const normalized = formatClipboardText(text, options);

    try {
        await writeTextToClipboard(normalized);
        showToast(options.convertUnicodeToAscii
            ? 'Copied as pure ASCII'
            : 'Copied — paste in a monospace editor');
    } catch (err) {
        logger.error('Failed to copy:', err);
        showToast('Failed to copy', true);
    }
}

/** Selection-aware copy: fills internal clipboard and writes OS clipboard text. */
export async function copyToClipboard(
    editor: AsciiEditor,
    showToast: ToastFn,
    options: ClipboardOptions = getClipboardOptions(),
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

    await copyAsciiToClipboard(ascii, showToast, options);
}
