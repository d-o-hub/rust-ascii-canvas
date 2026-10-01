/**
 * ASCII Canvas Editor — shared event-result processing.
 *
 * Every handler in `events.ts`, `events-mobile.ts` and `events-file.ts` ends by
 * handing its `EventResult` to {@link handleEventResult}: redraw, tool button
 * state, focus hand-off between the canvas and the mobile keyboard proxy, OS
 * clipboard write, UI refresh and autosave.
 *
 * It lives in its own module rather than in `events.ts` because all three of
 * those modules need it. Leaving it in `events.ts` would mean they import back
 * into a module that imports them — a cycle whose only purpose is to share one
 * function.
 */

import { copyAsciiToClipboard, getClipboardOptions } from './clipboard.js';
import { requestRender, updateIndicator } from './render.js';
import { state } from './state.js';
import type { EventResult } from './types.js';
import { showToast, updateToolButtons, updateUI } from './ui.js';
import { scheduleAutoSave } from './autosave.js';

/** Cancel only an unfinished pointer gesture; normal capture release must not
 * dismiss a text cursor established by the just-completed click. */
export function cancelPointerGesture(): void {
    if (!state.pointerGestureActive) return;
    state.pointerGestureActive = false;
    state.lastTouchDistance = null;
    state.editor?.onPointerCancel();
    requestRender();
    updateUI();
    updateIndicator();
    scheduleAutoSave();
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
        void copyAsciiToClipboard(result.ascii, showToast, getClipboardOptions());
    }

    updateUI();
    if (persist) {
        scheduleAutoSave();
    }
    updateIndicator();
}
