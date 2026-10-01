/**
 * ASCII Canvas Editor — window and canvas input.
 *
 * What is left here after R-09: the pointer, wheel and keyboard handlers, the
 * shared `EventResult` plumbing they feed, and `setupEventListeners`, which is
 * the app's single public surface (used only by `main.ts`).
 *
 * Three concerns moved out because they were never input handling:
 *
 *   - `events-mobile.ts` — touch, the mobile keyboard proxy, the drawer and the
 *     mobile action buttons;
 *   - `events-file.ts` — clipboard, `.asc` documents, PNG/SVG export and the
 *     grid-size button, plus `wireOptionalButton` / `applyHistory`, which two
 *     modules need; the grid-size *policy* itself stays in #229's
 *     `document-events.ts`;
 *   - `eventResult.ts` / `autosave.ts` — the shared plumbing, extracted so the
 *     three modules above do not have to import back into this one.
 *
 * This file was 850 lines; `setupEventListeners` alone was 430 of them.
 */

import { BORDER_STYLES, TOOL_INFO } from './constants.js';
import { debouncedResizeCanvas, requestRender, updateCursorIndicator, updateIndicator } from './render.js';
import { state } from './state.js';
import {
    cycleBorderStyle,
    fitZoom,
    hideShortcutsModal,
    resetZoom,
    setTool,
    setZoom,
    showShortcutsModal,
    showToast,
    toggleTheme,
    updateUI,
} from './ui.js';
import { flushAutoSave, scheduleAutoSave } from './autosave.js';
import { handlePasteEvent } from './document-events.js';
import {
    applyHistory,
    copyToClipboard,
    wireOptionalButton,
    wireFileEvents,
} from './events-file.js';
import { cancelPointerGesture, handleEventResult } from './eventResult.js';
import {
    closeDrawer,
    handleMobileInput,
    handlePointerLeave,
    handleTouchEnd,
    handleTouchMove,
    handleTouchStart,
    shouldCloseDrawerOnEscape,
    wireMobileEvents,
} from './events-mobile.js';

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
    state.pointerGestureActive = true;

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

    // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition -- defensive runtime guard: wasm impl can return null though the TS type says otherwise
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
    if (e.pointerType === 'touch' || !state.pointerGestureActive) return;
    e.preventDefault();
    state.pointerGestureActive = false;
    state.canvas.releasePointerCapture(e.pointerId);

    const rect = state.canvas.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;

    const result = state.editor.onPointerUp(x, y);
    handleEventResult(result, { persist: true });
}

export function handlePointerCancel(e: PointerEvent): void {
    // Touch has its own touchend/touchcancel lifecycle. Implicit touch capture
    // is released before touchend, so handling it here would cancel valid taps.
    if (e.pointerType !== 'touch') cancelPointerGesture();
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

        if ((key === '+' || key === '=') && !ctrl) {
            setZoom(state.editor.zoom * 1.25);
            if (state.canvas) state.canvas.focus();
        } else if ((key === '-' || key === '_') && !ctrl) {
            setZoom(state.editor.zoom * 0.8);
            if (state.canvas) state.canvas.focus();
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

export function setupEventListeners(): void {
    if (!state.canvas) return;

    window.addEventListener('resize', debouncedResizeCanvas);

    window.addEventListener('pagehide', onPageHideFlushAutoSave);
    document.addEventListener('visibilitychange', onVisibilityChangeFlushAutoSave);

    window.addEventListener('blur', () => {
        cancelPointerGesture();
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
    state.canvas.addEventListener('pointercancel', handlePointerCancel);
    state.canvas.addEventListener('lostpointercapture', handlePointerCancel);

    state.canvas.addEventListener('touchstart', handleTouchStart, { passive: false });
    state.canvas.addEventListener('touchmove', handleTouchMove, { passive: false });
    state.canvas.addEventListener('touchend', handleTouchEnd, { passive: false });
    state.canvas.addEventListener('touchcancel', cancelPointerGesture, { passive: false });

    if (state.mobileKeyboardProxy) {
        state.mobileKeyboardProxy.addEventListener('input', handleMobileInput);
    }

    state.canvas.addEventListener('contextmenu', (e) => { e.preventDefault(); });

    state.canvas.addEventListener('wheel', handleWheel, { passive: false });

    state.canvas.addEventListener('keydown', handleKeyDown);
    state.canvas.addEventListener('keyup', handleKeyUp);

    window.addEventListener('paste', handlePasteEvent);

    // Extracted wiring (R-09): mobile drawer + actions, and documents/exports/grid.
    wireMobileEvents();
    wireFileEvents();

    window.addEventListener('keydown', (e) => {
        if (e.key === 'Escape') {
            const modal = document.getElementById('shortcuts-modal');
            if (modal && !modal.classList.contains('hidden')) {
                hideShortcutsModal();
            } else if (shouldCloseDrawerOnEscape()) {
                closeDrawer(true);
            }
        } else if (e.key === 'Tab') {
            const modal = document.getElementById('shortcuts-modal');
            if (modal && !modal.classList.contains('hidden')) {
                const focusableElements = modal.querySelectorAll('button, [href], input, select, textarea, [tabindex="0"]');
                if (focusableElements.length > 0) {
                    const firstFocusableHtmlElement = focusableElements[0] as HTMLElement;
                    const lastFocusableHtmlElement = focusableElements[focusableElements.length - 1] as HTMLElement;

                    if (e.shiftKey) {
                        if (document.activeElement === firstFocusableHtmlElement) {
                            lastFocusableHtmlElement.focus();
                            e.preventDefault();
                        }
                    } else {
                        if (document.activeElement === lastFocusableHtmlElement) {
                            firstFocusableHtmlElement.focus();
                            e.preventDefault();
                        }
                    }
                }
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
                showToast(`Eraser radius: ${val} Cell${val === '1' ? '' : 's'}`);
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
                const val = state.borderStyleSelect.value;
                state.editor.setBorderStyle(val);
                state.currentBorderStyleIndex = BORDER_STYLES.indexOf(val);
                showToast(`Border: ${val}`);
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
                const capitalized = direction.charAt(0).toUpperCase() + direction.slice(1);
                showToast(`Line direction: ${capitalized}`);
            });
        });
    }

    if (state.undoBtn) {
        state.undoBtn.addEventListener('mousedown', (e) => { e.preventDefault(); });
        state.undoBtn.addEventListener('click', () => { applyHistory('undo'); });
    }

    if (state.redoBtn) {
        state.redoBtn.addEventListener('mousedown', (e) => { e.preventDefault(); });
        state.redoBtn.addEventListener('click', () => { applyHistory('redo'); });
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

    wireOptionalButton('add-layer-btn', () => {
        if (!state.editor) return;
        const before = state.editor.layerCount;
        state.editor.addLayer();
        if (state.editor.layerCount === before) {
            showToast('Layer limit reached', true);
            return;
        }
        requestRender();
        updateUI();
        scheduleAutoSave();
        showToast('Layer added');
        document.querySelector<HTMLElement>('.layer-item.active')?.focus();
    });

    if (state.helpBtn) {
        state.helpBtn.addEventListener('mousedown', (e) => { e.preventDefault(); });
        state.helpBtn.addEventListener('click', showShortcutsModal);
    }

    wireOptionalButton('theme-btn', () => {
        toggleTheme();
        if (state.canvas) state.canvas.focus();
    });

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
