/**
 * ASCII Canvas Editor - UI Module
 */

import { state } from './state.js';
import { BORDER_STYLES, TOOL_INFO } from './constants.js';
import { capitalize } from './utils.js';
import { logger } from './logger.js';

export function focusCanvasElement(): void {
    if (state.canvas) {
        state.canvas.focus();
        return;
    }
    const el = document.querySelector('#canvas');
    if (el instanceof HTMLCanvasElement) {
        el.focus();
    }
}

export function setTool(toolName: string): void {
    if (!state.editor) {
        logger.error('Editor not initialized');
        focusCanvasElement();
        updateToolButtons(toolName);
        return;
    }
    try {
        state.editor.setTool(toolName);
        updateToolButtons(toolName);

        if (state.statusToolEl) {
            state.statusToolEl.textContent = `Tool: ${capitalize(toolName)}`;
        }

        focusCanvasElement();
    } catch (error) {
        logger.error('Failed to set tool:', error);
    }
}

export function updateToolButtons(activeTool: string): void {
    const normalizedTool = activeTool.toLowerCase();

    const lineGroup = document.getElementById('line-direction-group');
    if (lineGroup) {
        lineGroup.style.display = normalizedTool === 'line' ? 'flex' : 'none';
    }
    const eraserGroup = document.getElementById('eraser-radius-group');
    if (eraserGroup) {
        eraserGroup.style.display = normalizedTool === 'eraser' ? 'flex' : 'none';
    }

    const buttons = document.querySelectorAll('.tool-btn');
    buttons.forEach(btn => {
        const tool = btn.getAttribute('data-tool');
        const isActive = tool?.toLowerCase() === normalizedTool;
        if (isActive) {
            btn.classList.add('active');
        } else {
            btn.classList.remove('active');
        }
        btn.setAttribute('aria-pressed', isActive.toString());
    });

    const info = TOOL_INFO[normalizedTool];
    const statusEl = document.querySelector('#status-message');
    if (statusEl instanceof HTMLElement && info) {
        statusEl.textContent = `[${info.shortcut}] ${info.instruction}`;
    }

    const container = document.querySelector('#canvas-container');
    if (container instanceof HTMLElement && info) {
        container.classList.remove('tool-text', 'tool-select', 'tool-crosshair', 'tool-eraser');
        if (info.cursor === 'text') {
            container.classList.add('tool-text');
        } else if (info.cursor === 'crosshair') {
            if (normalizedTool === 'eraser') {
                container.classList.add('tool-eraser');
            } else {
                container.classList.add('tool-crosshair');
            }
        } else if (normalizedTool === 'select') {
            container.classList.add('tool-select');
        }
    }
}

export function showToast(message: string, isError = false): void {
    if (!state.statusToast) return;
    state.statusToast.textContent = message;
    state.statusToast.classList.toggle('error', isError);
    state.statusToast.classList.remove('hidden');

    setTimeout(() => {
        if (state.statusToast) {
            state.statusToast.classList.add('hidden');
        }
    }, 2000);
}

export function setZoom(zoom: number): void {
    if (!state.editor) return;
    const clampedZoom = Math.max(0.3, Math.min(4.0, zoom));
    state.editor.setZoom(clampedZoom);
    state.editor.requestRedraw();
    if (state.requestRender) state.requestRender();
    if (state.zoomLevelEl) {
        state.zoomLevelEl.textContent = `${Math.round(clampedZoom * 100)}%`;
    }
}

export function resetZoom(): void {
    if (!state.editor || !state.canvasContainer) return;

    const containerRect = state.canvasContainer.getBoundingClientRect();
    const gridWidth = state.editor.width * state.charWidth;
    const gridHeight = state.editor.height * state.lineHeight;

    const panX = (containerRect.width - gridWidth) / 2;
    const panY = (containerRect.height - gridHeight) / 2;

    setZoom(1.0);
    state.editor.setPan(panX, panY);
    state.editor.requestRedraw();
    if (state.requestRender) state.requestRender();
    showToast('Zoom reset to 100%');
}

export function fitZoom(): void {
    if (!state.editor || !state.canvasContainer) return;

    const containerRect = state.canvasContainer.getBoundingClientRect();
    const gridWidth = state.editor.width * state.charWidth;
    const gridHeight = state.editor.height * state.lineHeight;

    const zoomX = containerRect.width / (gridWidth + 40);
    const zoomY = containerRect.height / (gridHeight + 40);
    const fitZoomLevel = Math.min(zoomX, zoomY, 1.0);

    const panX = (containerRect.width - gridWidth * fitZoomLevel) / 2;
    const panY = (containerRect.height - gridHeight * fitZoomLevel) / 2;

    setZoom(fitZoomLevel);
    state.editor.setPan(panX, panY);
    state.editor.requestRedraw();
    if (state.requestRender) state.requestRender();
    showToast('Zoom fitted to view');
}

export function cycleBorderStyle(): void {
    if (!state.editor) return;
    state.currentBorderStyleIndex = (state.currentBorderStyleIndex + 1) % BORDER_STYLES.length;
    const style = BORDER_STYLES[state.currentBorderStyleIndex];
    state.editor.setBorderStyle(style);
    if (state.borderStyleSelect) {
        state.borderStyleSelect.value = style;
    }
    showToast(`Border: ${style}`);
}

export function showShortcutsModal(): void {
    const modal = document.getElementById('shortcuts-modal');
    if (modal) {
        state.lastFocusedElement = document.activeElement as HTMLElement;
        modal.classList.remove('hidden');
        const closeBtn = modal.querySelector('.modal-close') as HTMLElement;
        if (closeBtn) {
            closeBtn.focus();
        }
    }
}

export function hideShortcutsModal(): void {
    const modal = document.getElementById('shortcuts-modal');
    if (modal) {
        modal.classList.add('hidden');
        if (state.lastFocusedElement) {
            state.lastFocusedElement.focus();
            state.lastFocusedElement = null;
        }
    }
}

export function syncGridInputs(): void {
    if (!state.editor) return;
    const gridWidthInput = document.querySelector('#grid-width');
    const gridHeightInput = document.querySelector('#grid-height');
    if (!(gridWidthInput instanceof HTMLInputElement) || !(gridHeightInput instanceof HTMLInputElement)) {
        return;
    }
    gridWidthInput.value = String(state.editor.width);
    gridHeightInput.value = String(state.editor.height);
}

interface CachedLayerState {
    name: string;
    visible: boolean;
    locked: boolean;
}
let lastLayerCount = 0;
let lastActiveLayer = -1;
let lastLayersState: CachedLayerState[] = [];

export function refreshLayerList(): void {
    if (!state.editor || typeof state.editor.layerCount !== 'number') return;
    const count = state.editor.layerCount;
    const active = state.editor.activeLayer ?? 0;

    const currentStates: CachedLayerState[] = [];
    for (let i = 0; i < count; i++) {
        currentStates.push({
            name: state.editor.layerName(i),
            visible: state.editor.layerVisible(i),
            locked: state.editor.layerLocked(i),
        });
    }

    let hasChanged = count !== lastLayerCount || active !== lastActiveLayer || currentStates.length !== lastLayersState.length;
    if (!hasChanged) {
        for (let i = 0; i < count; i++) {
            const cur = currentStates.at(i);
            const last = lastLayersState.at(i);
            if (cur && last) {
                if (
                    cur.name !== last.name ||
                    cur.visible !== last.visible ||
                    cur.locked !== last.locked
                ) {
                    hasChanged = true;
                    break;
                }
            } else {
                hasChanged = true;
                break;
            }
        }
    }

    if (!hasChanged) {
        return;
    }

    lastLayerCount = count;
    lastActiveLayer = active;
    lastLayersState = currentStates;

    const layerList = document.querySelector('#layer-list');
    if (!(layerList instanceof HTMLElement)) return;

    while (layerList.firstChild) {
        layerList.removeChild(layerList.firstChild);
    }

    for (let i = count - 1; i >= 0; i--) {
        const item = document.createElement('div');
        item.className = 'layer-item';
        if (i === active) item.classList.add('active');

        item.addEventListener('click', (e) => {
            if (e.target instanceof HTMLButtonElement || e.target instanceof HTMLInputElement) {
                return;
            }
            if (state.editor) {
                state.editor.setActiveLayer(i);
                if (state.requestRender) state.requestRender();
                updateUI();
                if (state.scheduleAutoSave) state.scheduleAutoSave();
            }
        });

        const visible = state.editor.layerVisible(i);
        const visBtn = document.createElement('button');
        visBtn.className = 'layer-item-btn';
        if (visible) visBtn.classList.add('active');
        visBtn.type = 'button';
        visBtn.textContent = visible ? '👁' : '◌';
        visBtn.title = visible ? 'Hide layer' : 'Show layer';
        visBtn.setAttribute('aria-label', visible ? 'Hide layer' : 'Show layer');
        visBtn.addEventListener('click', () => {
            if (state.editor) {
                state.editor.setLayerVisible(i, !visible);
                if (state.requestRender) state.requestRender();
                updateUI();
                if (state.scheduleAutoSave) state.scheduleAutoSave();
            }
        });

        const locked = state.editor.layerLocked(i);
        const lockBtn = document.createElement('button');
        lockBtn.className = 'layer-item-btn';
        if (locked) lockBtn.classList.add('active');
        lockBtn.type = 'button';
        lockBtn.textContent = locked ? '🔒' : '🔓';
        lockBtn.title = locked ? 'Unlock layer' : 'Lock layer';
        lockBtn.setAttribute('aria-label', locked ? 'Unlock layer' : 'Lock layer');
        lockBtn.addEventListener('click', () => {
            if (state.editor) {
                state.editor.setLayerLocked(i, !locked);
                if (state.requestRender) state.requestRender();
                updateUI();
                if (state.scheduleAutoSave) state.scheduleAutoSave();
            }
        });

        const nameInput = document.createElement('input');
        nameInput.className = 'layer-name-input';
        nameInput.type = 'text';
        nameInput.value = state.editor.layerName(i) || `Layer ${i + 1}`;
        nameInput.title = 'Edit layer name';
        nameInput.addEventListener('change', () => {
            if (state.editor) {
                state.editor.renameLayer(i, nameInput.value);
                if (state.scheduleAutoSave) state.scheduleAutoSave();
            }
        });

        const upBtn = document.createElement('button');
        upBtn.className = 'layer-item-btn';
        upBtn.type = 'button';
        upBtn.textContent = '↑';
        upBtn.title = 'Move up';
        upBtn.setAttribute('aria-label', 'Move up');
        upBtn.disabled = i === count - 1;
        upBtn.addEventListener('click', () => {
            if (state.editor) {
                state.editor.moveLayer(i, i + 1);
                if (state.requestRender) state.requestRender();
                updateUI();
                if (state.scheduleAutoSave) state.scheduleAutoSave();
            }
        });

        const downBtn = document.createElement('button');
        downBtn.className = 'layer-item-btn';
        downBtn.type = 'button';
        downBtn.textContent = '↓';
        downBtn.title = 'Move down';
        downBtn.setAttribute('aria-label', 'Move down');
        downBtn.disabled = i === 0;
        downBtn.addEventListener('click', () => {
            if (state.editor) {
                state.editor.moveLayer(i, i - 1);
                if (state.requestRender) state.requestRender();
                updateUI();
                if (state.scheduleAutoSave) state.scheduleAutoSave();
            }
        });

        const mergeBtn = document.createElement('button');
        mergeBtn.className = 'layer-item-btn';
        mergeBtn.type = 'button';
        mergeBtn.textContent = '↴';
        mergeBtn.title = 'Merge down';
        mergeBtn.setAttribute('aria-label', 'Merge down');
        mergeBtn.disabled = i === 0;
        mergeBtn.addEventListener('click', () => {
            if (state.editor) {
                state.editor.mergeLayerDown(i);
                if (state.requestRender) state.requestRender();
                updateUI();
                if (state.scheduleAutoSave) state.scheduleAutoSave();
                showToast('Merged layer down');
            }
        });

        const delBtn = document.createElement('button');
        delBtn.className = 'layer-item-btn';
        delBtn.type = 'button';
        delBtn.textContent = '🗑';
        delBtn.title = 'Delete layer';
        delBtn.setAttribute('aria-label', 'Delete layer');
        delBtn.disabled = count <= 1;
        delBtn.addEventListener('click', () => {
            if (state.editor) {
                if (confirm('Delete this layer? This cannot be undone.')) {
                    state.editor.deleteLayer(i);
                    if (state.requestRender) state.requestRender();
                    updateUI();
                    if (state.scheduleAutoSave) state.scheduleAutoSave();
                    showToast('Layer deleted');
                }
            }
        });

        item.appendChild(visBtn);
        item.appendChild(lockBtn);
        item.appendChild(nameInput);
        item.appendChild(upBtn);
        item.appendChild(downBtn);
        item.appendChild(mergeBtn);
        item.appendChild(delBtn);

        layerList.appendChild(item);
    }
}

export function updateUI(): void {
    if (!state.editor) return;
    try {
        if (state.undoBtn) state.undoBtn.disabled = !state.editor.can_undo;
        if (state.redoBtn) state.redoBtn.disabled = !state.editor.can_redo;
        if (state.gridSizeEl) state.gridSizeEl.textContent = `${state.editor.width} × ${state.editor.height}`;
        if (state.statusToolEl) state.statusToolEl.textContent = `Tool: ${capitalize(state.editor.tool)}`;
        refreshLayerList();
    } catch (error) {
        logger.error('Failed to update UI:', error);
    }
}
