/**
 * ASCII Canvas Editor — clipboard, documents and exports.
 *
 * Extracted from `events.ts` (R-09). Everything here is about *moving data in
 * and out of the canvas* rather than about drawing:
 *
 *   - `copyToClipboard` — the OS clipboard;
 *   - `downloadDocument` / `openDocumentPicker` wiring — the `.asc` file;
 *   - the PNG and SVG export buttons;
 *   - `wireOptionalButton` and `applyHistory`, which live here because both
 *     this module and `events-mobile.ts` need them and neither should have to
 *     import `events.ts` to get them.
 *
 * Grid size and paste are NOT here: #229 already owns them in
 * `document-events.ts` (with the shrink-confirmation and text-target
 * boundaries), and `safety.test.ts` tests that module. This file calls into it
 * rather than duplicating the boundaries.
 */

import { copyToClipboard as copySelectionAware, getClipboardOptions } from './clipboard.js';
import { applyCustomGridSize } from './document-events.js';
import { exportPng } from './exportPng.js';
import { exportSvg } from './exportSvg.js';
import { downloadDocument, openDocumentPicker } from './persistence.js';
import { requestRender } from './render.js';
import { state } from './state.js';
import { showToast, syncGridInputs, updateUI } from './ui.js';
import { flushAutoSave, scheduleAutoSave } from './autosave.js';

export function wireOptionalButton(id: string, onClick: () => void): void {
    const el = document.querySelector(`#${CSS.escape(id)}`);
    if (!(el instanceof HTMLButtonElement)) return;
    el.addEventListener('mousedown', (e) => {
        e.preventDefault();
    });
    el.addEventListener('click', onClick);
}

export function applyHistory(action: 'undo' | 'redo'): void {
    if (!state.editor) return;
    const done = action === 'undo' ? state.editor.undo() : state.editor.redo();
    if (!done) {
        const pending = action === 'undo' ? state.editor.can_undo : state.editor.can_redo;
        if (pending) {
            showToast(`Cannot ${action} on a locked layer - unlock it first`, true);
        }
    }
    requestRender();
    updateUI();
    if (done) scheduleAutoSave();
    if (state.canvas) state.canvas.focus();
}

export async function copyToClipboard(): Promise<void> {
    if (!state.editor) return;
    await copySelectionAware(state.editor, showToast, getClipboardOptions());
}

/**
 * Wire the document, export and grid-size controls.
 *
 * Extracted verbatim from the middle of `setupEventListeners`; `events.ts` calls
 * this from there so the public surface of the app is unchanged.
 */
export function wireFileEvents(): void {
    wireOptionalButton('save-btn', () => {
        if (!state.editor) return;
        downloadDocument(state.editor, showToast);
        if (state.canvas) state.canvas.focus();
    });
    wireOptionalButton('draft-download', () => {
        if (state.editor) downloadDocument(state.editor, showToast);
    });
    wireOptionalButton('load-btn', importDocument);
    wireOptionalButton('png-btn', exportCurrentPng);
    wireOptionalButton('svg-btn', () => {
        if (state.editor) {
            exportSvg(state.editor, showToast);
        }
        if (state.canvas) state.canvas.focus();
    });
    function applyGridAndFocus(): void {
        applyCustomGridSize();
        if (state.canvas) state.canvas.focus();
    }

    wireOptionalButton('apply-grid-btn', applyGridAndFocus);

    const gridWidthHtmlElement = document.querySelector('#grid-width') as HTMLInputElement | null;
    const gridHeightHtmlElement = document.querySelector('#grid-height') as HTMLInputElement | null;
    function handleGridKeyDown(e: KeyboardEvent): void {
        if (e.key === 'Enter') {
            applyGridAndFocus();
        } else if (e.key === 'Escape') {
            syncGridInputs();
            if (e.target instanceof HTMLElement) {
                e.target.blur();
            }
            if (state.canvas) state.canvas.focus();
        }
    }
    if (gridWidthHtmlElement) {
        gridWidthHtmlElement.addEventListener('keydown', handleGridKeyDown as EventListener);
    }
    if (gridHeightHtmlElement) {
        gridHeightHtmlElement.addEventListener('keydown', handleGridKeyDown as EventListener);
    }
}

/** Shared desktop/mobile paths: export never redraws or mutates selection. */
export function exportCurrentPng(): void {
    if (state.editor) exportPng(state.editor, showToast);
    if (state.canvas) state.canvas.focus();
}

/**
 * Load an `.asc` document. The callback body is `setupEventListeners`' original
 * `load-btn` wiring, kept byte-for-byte: loading re-locks the grid, drops the
 * cached offscreen raster and flushes the pending autosave. Dropping those four
 * lines would be a behaviour change inside a refactor (and a stale raster in
 * the export path).
 */
export function importDocument(): void {
    if (!state.editor) return;
    openDocumentPicker(showToast, () => {
        state.gridSizeLocked = true;
        state.offscreenCanvas = null;
        state.offscreenCtx = null;
        syncGridInputs();
        requestRender();
        updateUI();
        flushAutoSave();
    });
}
