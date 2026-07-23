/**
 * ASCII Canvas Editor - Events Module
 */

import { state } from './state.js';
import {
    BORDER_STYLES,
    TOOL_INFO,
    MIN_COLS,
    MIN_ROWS,
} from './constants.js';
import { copyAsciiToClipboard, copyToClipboard as copySelectionAware } from './clipboard.js';
import { createAutoSaveScheduler, downloadDocument, openDocumentPicker } from './persistence.js';
import { exportPng } from './exportPng.js';
import { exportSvg } from './exportSvg.js';
import {
    requestRender,
    debouncedResizeCanvas,
    updateCursorIndicator,
    updateIndicator,
} from './render.js';
import {
    cycleBorderStyle,
    fitZoom,
    hideShortcutsModal,
    resetZoom,
    setTool,
    setZoom,
    showShortcutsModal,
    showToast,
    syncGridInputs,
    updateToolButtons,
    updateUI,
} from './ui.js';
import type { EventResult } from './types.js';

export const { schedule: scheduleAutoSave, flush: flushAutoSave } = createAutoSaveScheduler(() => state.editor);

state.scheduleAutoSave = scheduleAutoSave;

export function onPageHideFlushAutoSave(): void {
    if (state.editor) flushAutoSave();
}

export function onVisibilityChangeFlushAutoSave(): void {
    if (document.visibilityState === 'hidden' && state.editor) {
        flushAutoSave();
    }
}

export function handlePointerDown(e: PointerEvent): void {
    if (!state.editor || !state.canvas) return;
    if (e.pointerType === 'touch') return;
    e.preventDefault();
    state.canvas.focus();
    state.canvas.setPointerCapture(e.pointerId);

    const rect = state.canvas.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;

    const result = state.editor.onPointerDown(x, y);
    handleEventResult(result);
}

export function handlePointerMove(e: PointerEvent): void {
    if (!state.editor || !state.canvas) return;
    if (e.pointerType === 'touch') return;
    const rect = state.canvas.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;

    const pan = state.editor.pan as number[] | Float64Array;
    const gridX = Math.floor((x - pan[0]) / state.editor.zoom / state.charWidth);
    const gridY = Math.floor((y - pan[1]) / state.editor.zoom / state.lineHeight);
    if (state.cursorPosEl) {
        state.cursorPosEl.textContent = `${gridX}, ${gridY}`;
    }

    const hasTextCursor = state.editor.tool.toLowerCase() === 'text' && typeof state.editor.textCursorPosition === 'function' && state.editor.textCursorPosition() !== null;
    if (!hasTextCursor) {
        updateCursorIndicator(gridX, gridY);
    } else {
        updateIndicator();
    }

    const result = state.editor.onPointerMove(x, y);
    handleEventResult(result, { persist: false });
}

export function handlePointerUp(e: PointerEvent): void {
    if (!state.editor || !state.canvas) return;
    if (e.pointerType === 'touch') return;
    e.preventDefault();
    state.canvas.releasePointerCapture(e.pointerId);

    const rect = state.canvas.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;

    const result = state.editor.onPointerUp(x, y);
    handleEventResult(result, { persist: true });
}

export function handleTouchStart(e: TouchEvent): void {
    if (!state.editor || !state.canvas) return;
    e.preventDefault();
    state.canvas.focus();

    if (e.touches.length === 1) {
        const touch = e.touches[0];
        const rect = state.canvas.getBoundingClientRect();
        const x = touch.clientX - rect.left;
        const y = touch.clientY - rect.top;
        const result = state.editor.onPointerDown(x, y);
        handleEventResult(result);
    } else if (e.touches.length === 2) {
        state.lastTouchDistance = Math.hypot(
            e.touches[0].clientX - e.touches[1].clientX,
            e.touches[0].clientY - e.touches[1].clientY
        );
    }
}

export function handleTouchMove(e: TouchEvent): void {
    if (!state.editor || !state.canvas) return;
    e.preventDefault();

    if (e.touches.length === 1) {
        const touch = e.touches[0];
        const rect = state.canvas.getBoundingClientRect();
        const x = touch.clientX - rect.left;
        const y = touch.clientY - rect.top;

        const pan = state.editor.pan as number[] | Float64Array;
        const gridX = Math.floor((x - pan[0]) / state.editor.zoom / state.charWidth);
        const gridY = Math.floor((y - pan[1]) / state.editor.zoom / state.lineHeight);
        const hasTextCursor = state.editor.tool.toLowerCase() === 'text' && typeof state.editor.textCursorPosition === 'function' && state.editor.textCursorPosition() !== null;
        if (!hasTextCursor) {
            updateCursorIndicator(gridX, gridY);
        } else {
            updateIndicator();
        }

        const result = state.editor.onPointerMove(x, y);
        handleEventResult(result, { persist: false });
    } else if (e.touches.length === 2) {
        const currentDistance = Math.hypot(
            e.touches[0].clientX - e.touches[1].clientX,
            e.touches[0].clientY - e.touches[1].clientY
        );

        if (state.lastTouchDistance !== null) {
            const delta = state.lastTouchDistance - currentDistance;
            const rect = state.canvas.getBoundingClientRect();
            const centerX = (e.touches[0].clientX + e.touches[1].clientX) / 2 - rect.left;
            const centerY = (e.touches[0].clientY + e.touches[1].clientY) / 2 - rect.top;

            const result = state.editor.onWheel(delta * 2, centerX, centerY);
            handleEventResult(result, { persist: false });
        }
        state.lastTouchDistance = currentDistance;
    }
}

export function handleTouchEnd(e: TouchEvent): void {
    if (!state.editor || !state.canvas) return;
    e.preventDefault();

    if (e.touches.length === 0 && e.changedTouches.length === 1) {
        const touch = e.changedTouches[0];
        const rect = state.canvas.getBoundingClientRect();
        const x = touch.clientX - rect.left;
        const y = touch.clientY - rect.top;
        const result = state.editor.onPointerUp(x, y);
        handleEventResult(result);
    }
    state.lastTouchDistance = null;
}

export function handleMobileInput(e: Event): void {
    if (!state.editor) return;
    if (e.target instanceof HTMLInputElement) {
        const value = e.target.value;

        if (value.length === 0) {
            const result = state.editor.onKeyDown('Backspace', false, false);
            handleEventResult(result);
            e.target.value = ' ';
        } else if (value.length > 1) {
            const newChars = value.substring(1);
            for (const char of newChars) {
                const result = state.editor.onKeyDown(char, false, false);
                handleEventResult(result);
            }
            e.target.value = ' ';
        }
    }
}

export function handlePointerLeave(): void {
    const hasTextCursor = state.editor && state.editor.tool.toLowerCase() === 'text' && typeof state.editor.textCursorPosition === 'function' && state.editor.textCursorPosition() !== null;
    if (!hasTextCursor && state.cursorIndicator) {
        state.cursorIndicator.classList.add('hidden');
    }
}

export function handleWheel(e: WheelEvent): void {
    if (!state.editor || !state.canvas) return;
    e.preventDefault();

    const rect = state.canvas.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;

    const result = state.editor.onWheel(e.deltaY, x, y);
    handleEventResult(result, { persist: false });
}

export function handlePasteEvent(e: ClipboardEvent): void {
    if (!state.editor) return;

    e.preventDefault();

    const text = e.clipboardData?.getData('text/plain');
    if (text) {
        const success = state.editor.pasteText(text);
        if (success) {
            requestRender();
            updateUI();
            scheduleAutoSave();
        }
    } else {
        const success = state.editor.paste();
        if (success) {
            requestRender();
            updateUI();
            scheduleAutoSave();
        }
    }
}

export function handleKeyDown(e: KeyboardEvent): void {
    if (!state.editor) return;
    const key = e.key;
    const ctrl = e.ctrlKey || e.metaKey;
    const shift = e.shiftKey;

    if (['r', 'l', 'a', 'd', 't', 'f', 'v', 'e', 'b', ' ', 'escape', '?'].includes(key.toLowerCase()) && !ctrl) {
        e.preventDefault();
    }
    if (ctrl && ['z', 'y', 'c', 'a', 'x'].includes(key.toLowerCase())) {
        e.preventDefault();
    }

    if (ctrl && key.toLowerCase() === 'v') {
        return;
    }

    const result = state.editor.onKeyDown(key, ctrl, shift);

    if (key === ' ' && !ctrl && !shift && state.canvasContainer) {
        state.canvasContainer.classList.add('panning');
    }

    handleEventResult(result);

    const isTextTool = state.editor.tool.toLowerCase() === 'text';

    if (!isTextTool) {
        if (key.toLowerCase() === 'b' && !ctrl && !shift) {
            cycleBorderStyle();
        }

        if ((key === '0' || key === ')') && !ctrl) {
            if (shift || key === ')') {
                fitZoom();
            } else {
                resetZoom();
            }
        }

        if (key === '?' || (key === '/' && shift)) {
            showShortcutsModal();
        }

        const lowerKey = key.toLowerCase();
        for (const [toolName, info] of Object.entries(TOOL_INFO)) {
            if (info.shortcut.toLowerCase() === lowerKey && !ctrl && !shift) {
                setTool(toolName);
                break;
            }
        }
    } else if (key === '?' || (key === '/' && shift)) {
        showShortcutsModal();
    }
}

export function handleKeyUp(e: KeyboardEvent): void {
    if (!state.editor) return;
    state.editor.onKeyUp(e.key);

    if (e.key === ' ' && state.canvasContainer) {
        state.canvasContainer.classList.remove('panning');
    }
}

export function handleEventResult(result: EventResult | null, options: { persist?: boolean } = {}): void {
    if (!result) {
        updateIndicator();
        return;
    }
    const persist = options.persist !== false;

    if (result.needs_redraw) {
        requestRender();
    }

    if (result.tool) {
        updateToolButtons(result.tool);
    }

    if (state.editor && state.editor.tool.toLowerCase() === 'text') {
        const touchUi = typeof window.matchMedia === 'function'
            && window.matchMedia('(pointer: coarse)').matches;
        if (touchUi && state.mobileKeyboardProxy && document.activeElement !== state.mobileKeyboardProxy) {
            state.mobileKeyboardProxy.focus();
        }
    } else if (state.mobileKeyboardProxy && document.activeElement === state.mobileKeyboardProxy) {
        state.mobileKeyboardProxy.blur();
        if (state.canvas) state.canvas.focus();
    }

    if (result.should_copy && result.ascii) {
        void copyAsciiToClipboard(result.ascii, showToast);
    }

    updateUI();
    if (persist) {
        scheduleAutoSave();
    }
    updateIndicator();
}

export async function copyToClipboard(): Promise<void> {
    if (!state.editor) return;
    await copySelectionAware(state.editor, showToast);
}

export function applyCustomGridSize(): void {
    if (!state.editor) return;
    const gridWidthInput = document.querySelector('#grid-width');
    const gridHeightInput = document.querySelector('#grid-height');
    if (!(gridWidthInput instanceof HTMLInputElement) || !(gridHeightInput instanceof HTMLInputElement)) {
        return;
    }
    const w = Math.max(MIN_COLS, Math.min(400, parseInt(gridWidthInput.value, 10) || MIN_COLS));
    const h = Math.max(MIN_ROWS, Math.min(200, parseInt(gridHeightInput.value, 10) || MIN_ROWS));
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
        scheduleAutoSave();
        showToast(`Grid: ${w} × ${h}`);
    } else {
        syncGridInputs();
    }
}

export function wireOptionalButton(id: string, onClick: () => void): void {
    const el = document.querySelector(`#${CSS.escape(id)}`);
    if (!(el instanceof HTMLButtonElement)) return;
    el.addEventListener('mousedown', (e) => {
        e.preventDefault();
    });
    el.addEventListener('click', onClick);
}

export function setupEventListeners(): void {
    if (!state.canvas) return;

    window.addEventListener('resize', debouncedResizeCanvas);

    window.addEventListener('pagehide', onPageHideFlushAutoSave);
    document.addEventListener('visibilitychange', onVisibilityChangeFlushAutoSave);

    window.addEventListener('blur', () => {
        if (state.editor) {
            state.editor.onKeyUp(' ');
        }
        if (state.canvasContainer) {
            state.canvasContainer.classList.remove('panning');
        }
    });

    state.canvas.addEventListener('pointerdown', handlePointerDown);
    state.canvas.addEventListener('pointermove', handlePointerMove);
    state.canvas.addEventListener('pointerup', handlePointerUp);
    state.canvas.addEventListener('pointerleave', handlePointerLeave);

    state.canvas.addEventListener('touchstart', handleTouchStart, { passive: false });
    state.canvas.addEventListener('touchmove', handleTouchMove, { passive: false });
    state.canvas.addEventListener('touchend', handleTouchEnd, { passive: false });

    if (state.mobileKeyboardProxy) {
        state.mobileKeyboardProxy.addEventListener('input', handleMobileInput);
    }

    state.canvas.addEventListener('contextmenu', (e) => { e.preventDefault(); });

    state.canvas.addEventListener('wheel', handleWheel, { passive: false });

    state.canvas.addEventListener('keydown', handleKeyDown);
    state.canvas.addEventListener('keyup', handleKeyUp);

    window.addEventListener('paste', handlePasteEvent);

    window.addEventListener('keydown', (e) => {
        if (e.key === 'Escape') {
            const modal = document.getElementById('shortcuts-modal');
            if (modal && !modal.classList.contains('hidden')) {
                hideShortcutsModal();
            }
        }
    });

    const shortcutsModal = document.getElementById('shortcuts-modal');
    if (shortcutsModal) {
        const closeBtn = shortcutsModal.querySelector('.modal-close');
        if (closeBtn) {
            closeBtn.addEventListener('click', hideShortcutsModal);
        }
        shortcutsModal.addEventListener('click', (e) => {
            if (e.target === shortcutsModal) {
                hideShortcutsModal();
            }
        });
    }

    if (state.toolButtons) {
        state.toolButtons.forEach(btn => {
            btn.addEventListener('mousedown', (e) => { e.preventDefault(); });
            btn.addEventListener('click', () => {
                const tool = btn.getAttribute('data-tool');
                if (tool) {
                    setTool(tool);
                }
            });
        });
    }

    if (state.eraserRadiusSelect) {
        state.eraserRadiusSelect.addEventListener('change', () => {
            if (state.editor && state.eraserRadiusSelect) {
                const val = state.eraserRadiusSelect.value;
                let size = 1;
                if (val === '1') size = 1;
                else if (val === '3') size = 2;
                else if (val === '5') size = 3;
                state.editor.setEraserSize(size);
            }
        });
    }

    if (state.editor && state.eraserRadiusSelect) {
        const val = state.eraserRadiusSelect.value;
        let size = 1;
        if (val === '1') size = 1;
        else if (val === '3') size = 2;
        else if (val === '5') size = 3;
        state.editor.setEraserSize(size);
    }

    if (state.borderStyleSelect) {
        state.borderStyleSelect.addEventListener('click', () => {
        });
        state.borderStyleSelect.addEventListener('change', () => {
            if (state.editor && state.borderStyleSelect) {
                state.editor.setBorderStyle(state.borderStyleSelect.value);
                state.currentBorderStyleIndex = BORDER_STYLES.indexOf(state.borderStyleSelect.value);
            }
        });
    }

    if (state.directionBtns) {
        state.directionBtns.forEach(btn => {
            btn.addEventListener('mousedown', (e) => { e.preventDefault(); });
            btn.addEventListener('click', () => {
                const direction = btn.getAttribute('data-direction');
                if (direction === null) return;

                state.currentLineDirection = direction;

                if (state.directionBtns) {
                    state.directionBtns.forEach(b => {
                        b.classList.remove('active');
                        b.setAttribute('aria-pressed', 'false');
                    });
                }
                btn.classList.add('active');
                btn.setAttribute('aria-pressed', 'true');

                if (state.editor) {
                    state.editor.setLineDirection(direction);
                }
            });
        });
    }

    if (state.undoBtn) {
        state.undoBtn.addEventListener('mousedown', (e) => { e.preventDefault(); });
        state.undoBtn.addEventListener('click', () => {
            if (!state.editor) return;
            state.editor.undo();
            requestRender();
            updateUI();
            if (state.canvas) state.canvas.focus();
        });
    }

    if (state.redoBtn) {
        state.redoBtn.addEventListener('mousedown', (e) => { e.preventDefault(); });
        state.redoBtn.addEventListener('click', () => {
            if (!state.editor) return;
            state.editor.redo();
            requestRender();
            updateUI();
            if (state.canvas) state.canvas.focus();
        });
    }

    if (state.copyBtn) {
        state.copyBtn.addEventListener('mousedown', (e) => { e.preventDefault(); });
        state.copyBtn.addEventListener('click', () => {
            void copyToClipboard();
            if (state.canvas) state.canvas.focus();
        });
    }

    if (state.clearBtn) {
        state.clearBtn.addEventListener('mousedown', (e) => { e.preventDefault(); });
        state.clearBtn.addEventListener('click', () => {
            if (!state.editor) return;
            if (confirm('Clear the canvas? This cannot be undone.')) {
                state.editor.clear();
                requestRender();
                updateUI();
                scheduleAutoSave();
                showToast('Canvas cleared');
                if (state.canvas) state.canvas.focus();
            }
        });
    }

    wireOptionalButton('save-btn', () => {
        if (!state.editor) return;
        downloadDocument(state.editor, showToast);
        if (state.canvas) state.canvas.focus();
    });
    wireOptionalButton('load-btn', () => {
        if (!state.editor) return;
        openDocumentPicker(state.editor, showToast, () => {
            state.gridSizeLocked = true;
            state.offscreenCanvas = null;
            state.offscreenCtx = null;
            syncGridInputs();
            requestRender();
            updateUI();
            flushAutoSave();
        });
    });
    wireOptionalButton('png-btn', () => {
        if (state.editor) {
            state.editor.requestRedraw();
            requestRender();
            requestAnimationFrame(() => {
                exportPng(state.offscreenCanvas, state.canvas, showToast);
            });
        }
        if (state.canvas) state.canvas.focus();
    });
    wireOptionalButton('svg-btn', () => {
        if (state.editor) {
            exportSvg(state.editor, showToast);
        }
        if (state.canvas) state.canvas.focus();
    });
    wireOptionalButton('apply-grid-btn', () => {
        applyCustomGridSize();
        if (state.canvas) state.canvas.focus();
    });
    wireOptionalButton('add-layer-btn', () => {
        if (!state.editor) return;
        state.editor.addLayer();
        requestRender();
        updateUI();
        scheduleAutoSave();
        showToast('Layer added');
        if (state.canvas) state.canvas.focus();
    });

    if (state.helpBtn) {
        state.helpBtn.addEventListener('mousedown', (e) => { e.preventDefault(); });
        state.helpBtn.addEventListener('click', showShortcutsModal);
    }

    if (state.zoomFitBtn) {
        state.zoomFitBtn.addEventListener('mousedown', (e) => { e.preventDefault(); });
        state.zoomFitBtn.addEventListener('click', () => {
            if (!state.editor) return;
            fitZoom();
            if (state.canvas) state.canvas.focus();
        });
    }

    if (state.zoomResetBtn) {
        state.zoomResetBtn.addEventListener('mousedown', (e) => { e.preventDefault(); });
        state.zoomResetBtn.addEventListener('click', () => {
            if (!state.editor) return;
            resetZoom();
            if (state.canvas) state.canvas.focus();
        });
    }

    if (state.zoomOutBtn) {
        state.zoomOutBtn.addEventListener('mousedown', (e) => { e.preventDefault(); });
        state.zoomOutBtn.addEventListener('click', () => {
            if (!state.editor) return;
            setZoom(state.editor.zoom * 0.8);
            if (state.canvas) state.canvas.focus();
        });
    }

    if (state.zoomInBtn) {
        state.zoomInBtn.addEventListener('mousedown', (e) => { e.preventDefault(); });
        state.zoomInBtn.addEventListener('click', () => {
            if (!state.editor) return;
            setZoom(state.editor.zoom * 1.25);
            if (state.canvas) state.canvas.focus();
        });
    }
}
