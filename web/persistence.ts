/**
 * File persistence: localStorage auto-save and .asc download/upload.
 */

import { AUTOSAVE_KEY } from './constants.js';
import { logger } from './logger.js';
import type { AsciiEditorInterface } from './types.js';
import type { ToastFn } from './clipboard.js';

/** Save current document JSON to localStorage. */
export function autoSave(editor: AsciiEditorInterface): void {
    try {
        const json = editor.serializeDocument();
        localStorage.setItem(AUTOSAVE_KEY, json);
    } catch (err) {
        logger.warn('Auto-save failed:', err);
    }
}

/** Load autosaved document if present. Returns true if restored. */
export function tryRestoreAutoSave(editor: AsciiEditorInterface): boolean {
    try {
        const json = localStorage.getItem(AUTOSAVE_KEY);
        if (!json) return false;
        return editor.loadDocument(json);
    } catch (err) {
        logger.warn('Auto-restore failed:', err);
        return false;
    }
}

/** Download current document as a `.asc` JSON file. */
export function downloadDocument(editor: AsciiEditorInterface, showToast: ToastFn): void {
    try {
        const json = editor.serializeDocument();
        const blob = new Blob([json], { type: 'application/json' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = `diagram-${new Date().toISOString().slice(0, 10)}.asc`;
        a.click();
        URL.revokeObjectURL(url);
        showToast('Saved diagram (.asc)');
    } catch (err) {
        logger.error('Download failed:', err);
        showToast('Failed to save file', true);
    }
}

/** Open a file picker and load a `.asc` / JSON document. */
export function openDocumentPicker(
    editor: AsciiEditorInterface,
    showToast: ToastFn,
    onLoaded: () => void,
): void {
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = '.asc,.json,application/json';
    input.addEventListener('change', () => {
        const file = input.files?.[0];
        if (!file) return;
        const reader = new FileReader();
        reader.onload = () => {
            const text = typeof reader.result === 'string' ? reader.result : '';
            if (editor.loadDocument(text)) {
                showToast(`Loaded ${file.name}`);
                onLoaded();
            } else {
                showToast('Invalid diagram file', true);
            }
        };
        reader.onerror = () => showToast('Failed to read file', true);
        reader.readAsText(file);
    });
    input.click();
}

export interface AutoSaveScheduler {
    /** Debounced schedule (for content mutations). */
    schedule: () => void;
    /** Immediate flush (pagehide / visibilitychange / explicit saves). */
    flush: () => void;
}

/** Debounced auto-save helper with immediate flush for unload paths. */
export function createAutoSaveScheduler(
    getEditor: () => AsciiEditorInterface | null,
    delayMs = 1500,
): AutoSaveScheduler {
    let timer: ReturnType<typeof setTimeout> | null = null;

    const flush = (): void => {
        if (timer) {
            clearTimeout(timer);
            timer = null;
        }
        const editor = getEditor();
        if (editor) autoSave(editor);
    };

    const schedule = (): void => {
        if (timer) clearTimeout(timer);
        timer = setTimeout(() => {
            timer = null;
            const editor = getEditor();
            if (editor) autoSave(editor);
        }, delayMs);
    };

    return { schedule, flush };
}
