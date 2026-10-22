import { beforeEach, describe, expect, it, vi } from 'vitest';
import { state } from './state.js';
import { refreshLayerList } from './ui.js';

beforeEach(() => {
    document.body.innerHTML = '<div id="layer-list"></div><button id="add-layer-btn" title="Add layer" aria-label="Add layer">+ Add Layer</button><canvas tabindex="0" id="canvas"></canvas>';
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

    it('updates add layer button disabled state and accessibility labels at layer limit', () => {
        const addBtn = document.querySelector<HTMLButtonElement>('#add-layer-btn');
        expect(addBtn?.disabled).toBe(false);
        expect(addBtn?.title).toBe('Add layer');
        expect(addBtn?.getAttribute('aria-label')).toBe('Add layer');

        if (state.editor) {
            (state.editor as { layerCount: number }).layerCount = 32;
        }
        refreshLayerList();

        expect(addBtn?.disabled).toBe(true);
        expect(addBtn?.title).toBe('Maximum limit of 32 layers reached');
        expect(addBtn?.getAttribute('aria-label')).toBe('Add layer (disabled: maximum limit of 32 layers reached)');
    });
});
