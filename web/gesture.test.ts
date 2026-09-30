import { beforeEach, describe, expect, it, vi } from 'vitest';
import { setupEventListeners } from './events.js';
import { state } from './state.js';

beforeEach(() => {
    document.body.innerHTML = '<canvas tabindex="0"></canvas>';
    state.canvas = document.querySelector('canvas');
    state.animationFrameId = 1;
    state.editor = {
        onPointerDown: vi.fn(() => null), onPointerUp: vi.fn(() => null), onPointerCancel: vi.fn(),
        onKeyUp: vi.fn(), tool: 'rectangle', width: 80, height: 40,
    } as unknown as typeof state.editor;
    if (!state.canvas) throw new Error('Canvas missing');
    state.canvas.setPointerCapture = vi.fn();
    state.canvas.releasePointerCapture = vi.fn();
    setupEventListeners();
});

describe('interrupted browser gestures', () => {
    it.each(['pointercancel', 'lostpointercapture'])('rolls back %s and ignores its late pointerup', event => {
        state.canvas?.dispatchEvent(new PointerEvent('pointerdown', { pointerId: 1 }));
        state.canvas?.dispatchEvent(new PointerEvent(event, { pointerId: 1 }));
        expect(state.editor?.onPointerCancel).toHaveBeenCalledOnce();
        state.canvas?.dispatchEvent(new PointerEvent('pointerup', { pointerId: 1 }));
        expect(state.editor?.onPointerUp).not.toHaveBeenCalled();
    });

    it('does not cancel the text cursor on normal capture release after pointerup', () => {
        state.canvas?.dispatchEvent(new PointerEvent('pointerdown', { pointerId: 1 }));
        state.canvas?.dispatchEvent(new PointerEvent('pointerup', { pointerId: 1 }));
        state.canvas?.dispatchEvent(new PointerEvent('lostpointercapture', { pointerId: 1 }));
        expect(state.editor?.onPointerUp).toHaveBeenCalledOnce();
        expect(state.editor?.onPointerCancel).not.toHaveBeenCalled();
    });

    it('keeps touch gestures until touchend rather than implicit capture release', () => {
        const touch = { clientX: 10, clientY: 10 };
        const start = new Event('touchstart');
        Object.defineProperty(start, 'touches', { value: [touch] });
        state.canvas?.dispatchEvent(start);
        state.canvas?.dispatchEvent(new PointerEvent('lostpointercapture', { pointerType: 'touch' }));
        expect(state.editor?.onPointerCancel).not.toHaveBeenCalled();
        const end = new Event('touchend');
        Object.defineProperties(end, { touches: { value: [] }, changedTouches: { value: [touch] } });
        state.canvas?.dispatchEvent(end);
        expect(state.editor?.onPointerUp).toHaveBeenCalledOnce();
    });

    it('rolls back touchcancel', () => {
        const start = new Event('touchstart');
        Object.defineProperty(start, 'touches', { value: [{ clientX: 10, clientY: 10 }] });
        state.canvas?.dispatchEvent(start);
        state.canvas?.dispatchEvent(new Event('touchcancel'));
        expect(state.editor?.onPointerCancel).toHaveBeenCalledOnce();
    });

    it('rolls back an active gesture on window blur', () => {
        state.canvas?.dispatchEvent(new PointerEvent('pointerdown', { pointerId: 1 }));
        window.dispatchEvent(new Event('blur'));
        expect(state.editor?.onPointerCancel).toHaveBeenCalledOnce();
    });
});
