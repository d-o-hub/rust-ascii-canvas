/** Layer panel rendering, keyboard navigation and focus-safe row rebuilding. */
import { state } from './state.js';

interface FocusTarget { index: number; action: string }
let cachedList: HTMLElement | null = null;
let cachedEditor = state.editor;
let signature = '';
let nextFocus: FocusTarget | null = null;

export function refreshLayers(updateUI: () => void, showToast: (message: string) => void, focusCanvas: () => void): void {
    const editor = state.editor;
    const list = document.querySelector('#layer-list');
    if (!editor || typeof editor.layerCount !== 'number' || !(list instanceof HTMLElement)) return;
    const count = editor.layerCount;
    const active = editor.activeLayer;
    const layers = Array.from({ length: count }, (_, index) => ({
        name: editor.layerName(index), visible: editor.layerVisible(index), locked: editor.layerLocked(index),
    }));
    const key = JSON.stringify([active, layers]);
    if (cachedList === list && cachedEditor === editor && key === signature) {
        nextFocus = null;
        return;
    }
    const focused = document.activeElement;
    const focusedRow = focused instanceof HTMLElement ? focused.closest<HTMLElement>('[data-layer-index]') : null;
    const restore = nextFocus ?? (focusedRow && list.contains(focusedRow) ? {
        index: Number(focusedRow.dataset.layerIndex),
        action: focused instanceof HTMLElement ? focused.dataset.layerAction ?? 'select' : 'select',
    } : null);
    nextFocus = null;
    cachedList = list;
    cachedEditor = editor;
    signature = key;
    list.replaceChildren();

    function finish(index: number, action: string): void {
        nextFocus = { index, action };
        signature = ''; // even identically named layers can move or be selected
        state.requestRender?.();
        updateUI();
        state.scheduleAutoSave?.();
    }

    for (let index = count - 1; index >= 0; index--) {
        const layer = layers.at(index);
        if (!layer) continue;
        const item = document.createElement('div');
        item.className = index === active ? 'layer-item active' : 'layer-item';
        item.dataset.layerIndex = String(index);
        item.dataset.layerAction = 'select';
        item.tabIndex = index === active ? 0 : -1;
        item.setAttribute('role', 'listitem');
        item.setAttribute('aria-label', `${layer.name}, select layer`);
        if (index === active) item.setAttribute('aria-current', 'true');
        item.addEventListener('click', event => {
            if (event.target !== item) return;
            editor.setActiveLayer(index);
            item.focus();
            finish(index, 'select');
        });
        item.addEventListener('keydown', event => {
            if (event.target !== item) return;
            let target = index;
            if (event.key === 'ArrowUp') target = Math.min(count - 1, index + 1);
            else if (event.key === 'ArrowDown') target = Math.max(0, index - 1);
            else if (event.key === 'Home') target = count - 1;
            else if (event.key === 'End') target = 0;
            else if (event.key !== 'Enter' && event.key !== ' ') return;
            event.preventDefault();
            editor.setActiveLayer(target);
            finish(target, 'select');
        });
        const button = (action: string, text: string, label: string, disabled: boolean, run: () => void, pressed = false): void => {
            const control = document.createElement('button');
            control.type = 'button';
            control.className = pressed ? 'layer-item-btn active' : 'layer-item-btn';
            control.dataset.layerAction = action;
            control.textContent = text;
            control.title = label;
            control.setAttribute('aria-label', label);
            control.disabled = disabled;
            control.addEventListener('click', run);
            item.append(control);
        };
        button('visible', layer.visible ? '👁' : '◌', layer.visible ? 'Hide layer' : 'Show layer', false, () => {
            editor.setLayerVisible(index, !layer.visible);
            finish(index, 'visible');
        }, layer.visible);
        button('locked', layer.locked ? '🔒' : '🔓', layer.locked ? 'Unlock layer' : 'Lock layer', false, () => {
            editor.setLayerLocked(index, !layer.locked);
            finish(index, 'locked');
        }, layer.locked);

        const nameInput = document.createElement('input');
        nameInput.className = 'layer-name-input';
        nameInput.type = 'text';
        nameInput.dataset.layerAction = 'name';
        nameInput.value = layer.name || `Layer ${index + 1}`;
        nameInput.title = 'Edit layer name';
        nameInput.setAttribute('aria-label', `Layer ${index + 1} name`);
        nameInput.addEventListener('change', () => {
            if (nameInput.value !== editor.layerName(index)) {
                editor.renameLayer(index, nameInput.value);
                // If blur has moved focus elsewhere, don't pull it back.
                if (document.activeElement === nameInput) nextFocus = { index, action: 'name' };
                updateUI();
                state.scheduleAutoSave?.();
            }
        });
        nameInput.addEventListener('keydown', event => {
            if (event.key === 'Escape') nameInput.value = editor.layerName(index) || `Layer ${index + 1}`;
            if (event.key === 'Enter' || event.key === 'Escape') {
                nameInput.blur();
                focusCanvas();
            }
        });
        item.append(nameInput);
        button('up', '↑', index === count - 1 ? 'Cannot move the top layer further up' : 'Move up', index === count - 1, () => {
            editor.moveLayer(index, index + 1);
            finish(index + 1, 'up');
        });
        button('down', '↓', index === 0 ? 'Cannot move the bottom layer further down' : 'Move down', index === 0, () => {
            editor.moveLayer(index, index - 1);
            finish(index - 1, 'down');
        });
        button('merge', '↴', index === 0 ? 'Cannot merge the bottom layer down' : 'Merge down', index === 0, () => {
            editor.mergeLayerDown(index);
            finish(index - 1, 'merge');
            showToast('Merged layer down');
        });
        button('delete', '🗑', count <= 1 ? 'Cannot delete the only remaining layer' : 'Delete layer', count <= 1, () => {
            if (!confirm('Delete this layer? You can undo this with Ctrl+Z.')) return;
            editor.deleteLayer(index);
            finish(Math.min(index, count - 2), 'delete');
            showToast('Layer deleted');
        });
        list.append(item);
    }
    if (restore) {
        const row = list.querySelector<HTMLElement>(`[data-layer-index="${restore.index}"]`);
        const target = row?.querySelector<HTMLElement>(`[data-layer-action="${CSS.escape(restore.action)}"]`);
        // Moving/deleting can disable the old action; focus the surviving row.
        if (target && !(target instanceof HTMLButtonElement && target.disabled)) target.focus();
        else row?.focus();
    }
}
