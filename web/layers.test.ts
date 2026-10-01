import { beforeEach, describe, expect, it, vi } from 'vitest';
import { state } from './state.js';
import { refreshLayerList } from './ui.js';

beforeEach(() => {
    document.body.innerHTML = '<div id="layer-list"></div><canvas tabindex="0" id="canvas"></canvas>';
    let active = 0;
    let visible = true;
    state.editor = {
        layerCount: 2, get activeLayer() { return active; },
        layerName: (index: number) => `Layer ${index + 1}`,
        layerVisible: () => visible, layerLocked: () => false,
        setLayerVisible: (_index: number, value: boolean) => { visible = value; },
        setActiveLayer: (index: number) => { active = index; },
    } as unknown as typeof state.editor;
    state.requestRender = vi.fn();
    state.scheduleAutoSave = vi.fn();
    refreshLayerList();
});

describe('accessible layer navigation', () => {
    it('selects layers with Enter and arrows and retains keyboard focus after rebuild', () => {
        const bottom = document.querySelectorAll<HTMLElement>('.layer-item')[1];
        expect(bottom.tabIndex).toBe(0);
        bottom.focus();
        bottom.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowUp', bubbles: true }));
        expect(state.editor?.activeLayer).toBe(1);
        expect(document.activeElement).toBe(document.querySelector('.layer-item'));
        document.activeElement?.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowDown', bubbles: true }));
        expect(state.editor?.activeLayer).toBe(0);
        document.activeElement?.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true }));
        expect(state.editor?.activeLayer).toBe(0);
    });

    it('retains the corresponding action button focus after toggling visibility', () => {
        const button = document.querySelector<HTMLButtonElement>('button[aria-label="Hide layer"]');
        button?.focus();
        button?.click();
        expect(document.activeElement?.getAttribute('aria-label')).toBe('Show layer');
    });
});
