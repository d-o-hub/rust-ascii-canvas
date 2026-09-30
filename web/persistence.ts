/**
 * File persistence: localStorage auto-save and .asc download/upload.
 */

import { importDraft, markDraftDirty, saveDraft } from './drafts-ui.js';
import { logger } from './logger.js';
import type { AsciiEditor } from './types.js';
import type { ToastFn } from './clipboard.js';

/** Download current document as a `.asc` JSON file. */
export function downloadDocument(editor: AsciiEditor, showToast: ToastFn): void {
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
            if (importDraft(text, file.name)) {
                showToast(`Imported ${file.name} as a new local draft`);
                onLoaded();
            } else {
                showToast('Import failed; current draft is unchanged. See save status.', true);
            }
        };
        reader.onerror = () => {
            showToast('Failed to read file', true);
        };
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
    getEditor: () => AsciiEditor | null,
    delayMs = 1500,
): AutoSaveScheduler {
    let timer: ReturnType<typeof setTimeout> | null = null;

    function flush(): void {
        if (timer) {
            clearTimeout(timer);
            timer = null;
        }
        const editor = getEditor();
        if (editor) saveDraft();
    }

    function schedule(): void {
        markDraftDirty();
        if (timer) clearTimeout(timer);
        timer = setTimeout(() => {
            timer = null;
            const editor = getEditor();
            if (editor) saveDraft();
        }, delayMs);
    }

    return { schedule, flush };
}
