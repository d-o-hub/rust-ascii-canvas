/** Document-size and paste boundaries; persistence stays in the existing autosave. */
import { MIN_COLS, MIN_ROWS } from './constants.js';
import { requestRender } from './render.js';
import { state } from './state.js';
import { showToast, syncGridInputs, updateUI } from './ui.js';

export function applyCustomGridSize(): void {
    if (!state.editor) return;
    const gridWidthInput = document.querySelector('#grid-width');
    const gridHeightInput = document.querySelector('#grid-height');
    if (!(gridWidthInput instanceof HTMLInputElement) || !(gridHeightInput instanceof HTMLInputElement)) {
        return;
    }
    const w = Math.max(MIN_COLS, Math.min(400, parseInt(gridWidthInput.value, 10) || MIN_COLS));
    const h = Math.max(MIN_ROWS, Math.min(200, parseInt(gridHeightInput.value, 10) || MIN_ROWS));
    if ((w < state.editor.width || h < state.editor.height) &&
        !confirm('Shrink the grid? Content outside the new size will be permanently cropped and undo history cleared.')) {
        syncGridInputs();
        return;
    }
    gridWidthInput.value = String(w);
    gridHeightInput.value = String(h);
    state.gridSizeLocked = true;
    if (state.editor.width !== w || state.editor.height !== h) {
        state.editor.resize(w, h);
        state.offscreenCanvas = null;
        state.offscreenCtx = null;
        requestRender();
        updateUI();
        syncGridInputs();
        state.scheduleAutoSave?.();
        showToast(`Grid: ${w} × ${h}`);
    } else {
        syncGridInputs();
    }
}

function isTextTarget(target: EventTarget | null): boolean {
    if (!(target instanceof Element)) return false;
    return target.closest('input, textarea, select, [contenteditable]:not([contenteditable="false"])') !== null;
}

export function handlePasteEvent(e: ClipboardEvent): void {
    if (!state.editor || isTextTarget(document.activeElement) || e.composedPath().some(isTextTarget)) return;
    e.preventDefault();
    const text = e.clipboardData?.getData('text/plain');
    const success = text ? state.editor.pasteText(text) : state.editor.paste();
    if (success) {
        requestRender();
        updateUI();
        state.scheduleAutoSave?.();
    }
}
