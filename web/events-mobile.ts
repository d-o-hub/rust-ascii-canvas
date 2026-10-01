/**
 * ASCII Canvas Editor — touch, mobile drawer and mobile actions.
 *
 * Extracted from `events.ts` (R-09). Three things live here because they share
 * one concern — *input that is not a mouse*:
 *
 *   1. the touch handlers (single-finger draw, two-finger pan/zoom);
 *   2. `handleMobileInput`, the hidden-input proxy that gives a phone a
 *      keyboard for the Text tool;
 *   3. the drawer and the mobile action buttons.
 *
 * `events.ts` keeps the window/canvas wiring and calls {@link wireMobileEvents}
 * from `setupEventListeners`, so the public surface of the app is unchanged.
 *
 * Depends on `eventResult.ts` and `autosave.ts` for the shared plumbing rather
 * than on `events.ts`, which is what keeps the dependency graph acyclic.
 */

import { exportSvg } from './exportSvg.js';
import { downloadDocument } from './persistence.js';
import { requestRender, updateCursorIndicator, updateIndicator } from './render.js';
import { state } from './state.js';
import {
    showShortcutsModal,
    showToast,
    toggleTheme,
    updateUI,
} from './ui.js';
import { scheduleAutoSave } from './autosave.js';

import { cancelPointerGesture, handleEventResult } from './eventResult.js';
import { applyHistory, copyToClipboard, exportCurrentPng, importDocument, wireOptionalButton } from './events-file.js';

export function handleTouchStart(e: TouchEvent): void {
    if (!state.editor || !state.canvas) return;
    e.preventDefault();
    state.canvas.focus();

    if (e.touches.length === 1) {
        state.pointerGestureActive = true;
        const touch = e.touches[0];
        const rect = state.canvas.getBoundingClientRect();
        const x = touch.clientX - rect.left;
        const y = touch.clientY - rect.top;
        const result = state.editor.onPointerDown(x, y);
        handleEventResult(result);
    } else if (e.touches.length === 2) {
        cancelPointerGesture();
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
    // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition -- defensive runtime guard: wasm impl can return null though the TS type says otherwise
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

    if (e.touches.length === 0 && e.changedTouches.length === 1 && state.pointerGestureActive) {
        state.pointerGestureActive = false;
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
    // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition -- defensive runtime guard: wasm impl can return null though the TS type says otherwise
    const hasTextCursor = state.editor && state.editor.tool.toLowerCase() === 'text' && typeof state.editor.textCursorPosition === 'function' && state.editor.textCursorPosition() !== null;
    if (!hasTextCursor && state.cursorIndicator) {
        state.cursorIndicator.classList.add('hidden');
    }
}

/** DOM handles for the mobile drawer, looked up when the drawer is wired. */
interface DrawerElements {
    sidePanel: HTMLElement | null;
    drawerOverlay: HTMLElement | null;
    mobileMenuBtn: HTMLElement | null;
    closeDrawerBtn: HTMLElement | null;
}

let drawer: DrawerElements | null = null;

/**
 * Close the drawer, optionally returning focus to the menu button that opened it.
 *
 * Module-level rather than a closure inside `wireMobileEvents` so the global
 * Escape handler in `events.ts` can reach it: Escape has to close the drawer
 * unless the Text tool owns it (ADR-040), and the Text tool lives in
 * `events.ts`. Keeping the drawer *knowledge* in this module means the Escape
 * handler does not need to know how the drawer is built.
 */
export function closeDrawer(restoreFocus = false): void {
    if (!drawer) return;
    const { sidePanel, drawerOverlay, mobileMenuBtn } = drawer;
    const isOpen = sidePanel?.classList.contains('open');
    if (sidePanel) sidePanel.classList.remove('open');
    if (drawerOverlay) drawerOverlay.classList.remove('open');
    if (mobileMenuBtn) mobileMenuBtn.setAttribute('aria-expanded', 'false');
    if (restoreFocus && isOpen && mobileMenuBtn) {
        mobileMenuBtn.focus();
    }
}

/**
 * Escape closes the drawer unless the Text tool owns Escape (the canvas
 * commits/dismisses the text cursor). Narrowed before use (ADR-040).
 */
export function shouldCloseDrawerOnEscape(): boolean {
    if (!drawer?.sidePanel) return false;
    if (!drawer.sidePanel.classList.contains('open')) return false;
    return state.editor?.tool.toLowerCase() !== 'text';
}

/**
 * Wire the mobile drawer and the mobile action buttons.
 *
 * Extracted verbatim from the tail of `setupEventListeners`; the only change is
 * that `closeDrawer` and `shouldCloseDrawerOnEscape` are now closures over
 * `drawer`, created here instead of mid-function.
 */
export function wireMobileEvents(): void {
    // Mobile Drawer Setup
    drawer = {
        sidePanel: document.getElementById('side-panel'),
        drawerOverlay: document.getElementById('drawer-overlay'),
        mobileMenuBtn: document.getElementById('mobile-menu-btn'),
        closeDrawerBtn: document.getElementById('close-drawer-btn'),
    };
    const { sidePanel, drawerOverlay, mobileMenuBtn, closeDrawerBtn } = drawer;

    if (mobileMenuBtn && sidePanel && drawerOverlay) {
        mobileMenuBtn.setAttribute('aria-expanded', 'false');
        mobileMenuBtn.setAttribute('aria-controls', 'side-panel');

        mobileMenuBtn.addEventListener('mousedown', (e) => { e.preventDefault(); });
        mobileMenuBtn.addEventListener('click', () => {
            const isOpen = sidePanel.classList.contains('open');
            sidePanel.classList.toggle('open');
            drawerOverlay.classList.toggle('open');
            mobileMenuBtn.setAttribute('aria-expanded', (!isOpen).toString());
        });
    }

    if (closeDrawerBtn) {
        closeDrawerBtn.addEventListener('mousedown', (e) => { e.preventDefault(); });
        closeDrawerBtn.addEventListener('click', () => { closeDrawer(); });
    }

    if (drawerOverlay) {
        drawerOverlay.addEventListener('click', () => { closeDrawer(); });
    }

    // Mobile Actions Wiring
    wireOptionalButton('mobile-undo-btn', () => {
        applyHistory('undo');
        closeDrawer();
    });

    wireOptionalButton('mobile-redo-btn', () => {
        applyHistory('redo');
        closeDrawer();
    });

    wireOptionalButton('mobile-copy-btn', () => {
        void copyToClipboard();
        closeDrawer();
        if (state.canvas) state.canvas.focus();
    });

    wireOptionalButton('mobile-clear-btn', () => {
        if (!state.editor) return;
        if (confirm('Clear the canvas? This cannot be undone.')) {
            state.editor.clear();
            requestRender();
            updateUI();
            scheduleAutoSave();
            showToast('Canvas cleared');
            closeDrawer();
            if (state.canvas) state.canvas.focus();
        }
    });

    wireOptionalButton('mobile-save-btn', () => {
        if (!state.editor) return;
        downloadDocument(state.editor, showToast);
        closeDrawer();
        if (state.canvas) state.canvas.focus();
    });

    wireOptionalButton('mobile-load-btn', () => {
        importDocument();
        closeDrawer();
    });

    wireOptionalButton('mobile-png-btn', () => {
        exportCurrentPng();
        closeDrawer();
    });

    wireOptionalButton('mobile-svg-btn', () => {
        if (state.editor) {
            exportSvg(state.editor, showToast);
        }
        closeDrawer();
        if (state.canvas) state.canvas.focus();
    });

    wireOptionalButton('mobile-theme-btn', () => {
        toggleTheme();
        closeDrawer();
        if (state.canvas) state.canvas.focus();
    });

    wireOptionalButton('mobile-help-btn', () => {
        showShortcutsModal();
        closeDrawer();
    });
}
