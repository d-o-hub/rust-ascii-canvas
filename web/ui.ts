/**
 * ASCII Canvas Editor - UI Module
 */

import { state } from './state.js';
import { BORDER_STYLES, TOOL_INFO, type ToolId } from './constants.js';
import { capitalize } from './utils.js';
import { logger } from './logger.js';
import { refreshLayers } from './layers.js';

const VALID_TOOLS = new Set(Object.keys(TOOL_INFO));

// Zoom bounds enforced in setZoom; also drive the disabled state of the
// zoom-in / zoom-out controls.
const ZOOM_MIN = 0.3;
const ZOOM_MAX = 4.0;

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

    if (VALID_TOOLS.has(normalizedTool)) {
        const info = TOOL_INFO[normalizedTool as ToolId];
        const statusEl = document.querySelector('#status-message');
        if (statusEl instanceof HTMLElement) {
            statusEl.textContent = `[${info.shortcut}] ${info.instruction}`;
        }

        const container = document.querySelector('#canvas-container');
        if (container instanceof HTMLElement) {
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

/**
 * Reflect the zoom boundary state on the zoom controls: at the lower/upper
 * bound the matching control is disabled and both its tooltip and its
 * accessible name explain why. Reads the cached state refs (wired in main.ts)
 * instead of re-querying the DOM, so no HTML-named local ever sits next to a
 * cast (the ADR-040 `xss/no-mixed-html` pattern).
 */
export function updateZoomButtonsState(zoom: number): void {
    const atMax = zoom >= ZOOM_MAX;
    const atMin = zoom <= ZOOM_MIN;

    const zoomInBtn = state.zoomInBtn;
    if (zoomInBtn) {
        zoomInBtn.disabled = atMax;
        zoomInBtn.title = atMax ? 'Maximum zoom level reached (400%)' : 'Zoom In (+ or =)';
        zoomInBtn.setAttribute('aria-label', atMax ? 'Zoom in (Maximum zoom 400% reached)' : 'Zoom in');
    }

    const zoomOutBtn = state.zoomOutBtn;
    if (zoomOutBtn) {
        zoomOutBtn.disabled = atMin;
        zoomOutBtn.title = atMin ? 'Minimum zoom level reached (30%)' : 'Zoom Out (- or _)';
        zoomOutBtn.setAttribute('aria-label', atMin ? 'Zoom out (Minimum zoom 30% reached)' : 'Zoom out');
    }
}

export function setZoom(zoom: number): void {
    if (!state.editor) return;
    const clampedZoom = Math.max(ZOOM_MIN, Math.min(ZOOM_MAX, zoom));
    state.editor.setZoom(clampedZoom);
    state.editor.requestRedraw();
    if (state.requestRender) state.requestRender();
    if (state.zoomLevelEl) {
        state.zoomLevelEl.textContent = `${Math.round(clampedZoom * 100)}%`;
    }
    updateZoomButtonsState(clampedZoom);
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
        state.lastFocusedHtmlElement = document.activeElement as HTMLElement;
        modal.classList.remove('hidden');
        const closeButtonHtmlElement = modal.querySelector('.modal-close') as HTMLElement | null;
        if (closeButtonHtmlElement !== null) {
            closeButtonHtmlElement.focus();
        }
    }
}

export function hideShortcutsModal(): void {
    const modal = document.getElementById('shortcuts-modal');
    if (modal) {
        modal.classList.add('hidden');
        if (state.lastFocusedHtmlElement) {
            state.lastFocusedHtmlElement.focus();
            state.lastFocusedHtmlElement = null;
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

export function refreshLayerList(): void {
    refreshLayers(updateUI, showToast, focusCanvasElement);
}

export function updateUI(): void {
    if (!state.editor) return;
    try {
        const canUndo = state.editor.can_undo;
        const canRedo = state.editor.can_redo;
        // Name the step being undone, so a layer change reads as "Undo Hide
        // layer" rather than a mystery (ADR-043 decision 4).
        const undoLabel = state.editor.undoLabel;
        const redoLabel = state.editor.redoLabel;
        const undoTitle = canUndo && undoLabel ? `Undo ${undoLabel} (Ctrl+Z)` : canUndo ? 'Undo (Ctrl+Z)' : 'Nothing to undo';
        const undoAria = canUndo && undoLabel ? `Undo ${undoLabel}` : canUndo ? 'Undo' : 'Nothing to undo';
        const redoTitle = canRedo && redoLabel ? `Redo ${redoLabel} (Ctrl+Shift+Z or Ctrl+Y)` : canRedo ? 'Redo (Ctrl+Shift+Z or Ctrl+Y)' : 'Nothing to redo';
        const redoAria = canRedo && redoLabel ? `Redo ${redoLabel}` : canRedo ? 'Redo' : 'Nothing to redo';

        if (state.undoBtn) {
            state.undoBtn.disabled = !canUndo;
            state.undoBtn.title = undoTitle;
            state.undoBtn.setAttribute('aria-label', undoAria);
        }
        if (state.redoBtn) {
            state.redoBtn.disabled = !canRedo;
            state.redoBtn.title = redoTitle;
            state.redoBtn.setAttribute('aria-label', redoAria);
        }
        if (state.mobileUndoBtn) {
            state.mobileUndoBtn.disabled = !canUndo;
            const title = undoAria;
            state.mobileUndoBtn.title = title;
            state.mobileUndoBtn.setAttribute('aria-label', title);
        }
        if (state.mobileRedoBtn) {
            state.mobileRedoBtn.disabled = !canRedo;
            const title = redoAria;
            state.mobileRedoBtn.title = title;
            state.mobileRedoBtn.setAttribute('aria-label', title);
        }

        if (state.gridSizeEl) state.gridSizeEl.textContent = `${state.editor.width} × ${state.editor.height}`;
        if (state.statusToolEl) state.statusToolEl.textContent = `Tool: ${capitalize(state.editor.tool)}`;
        updateZoomButtonsState(state.editor.zoom);
        refreshLayerList();
    } catch (error) {
        logger.error('Failed to update UI:', error);
    }
}

export function toggleTheme(): void {
    const currentTheme = localStorage.getItem('ascii-canvas-theme') || 'dark';
    const newTheme = currentTheme === 'dark' ? 'light' : 'dark';
    localStorage.setItem('ascii-canvas-theme', newTheme);

    const mobileThemeIcon = document.getElementById('mobile-theme-icon');
    const mobileThemeBtn = document.getElementById('mobile-theme-btn');

    const nextThemeLabel = newTheme === 'light' ? 'Switch to dark theme' : 'Switch to light theme';

    if (state.themeBtn) {
        state.themeBtn.title = nextThemeLabel;
        state.themeBtn.setAttribute('aria-label', nextThemeLabel);
    }
    if (mobileThemeBtn) {
        mobileThemeBtn.title = nextThemeLabel;
        mobileThemeBtn.setAttribute('aria-label', nextThemeLabel);
    }

    if (newTheme === 'light') {
        document.documentElement.setAttribute('data-theme', 'light');
        if (state.themeIcon) state.themeIcon.textContent = '☀️';
        if (mobileThemeIcon) mobileThemeIcon.textContent = '☀️';
        if (state.editor && typeof state.editor.setTheme === 'function') state.editor.setTheme('Light');
    } else {
        document.documentElement.removeAttribute('data-theme');
        if (state.themeIcon) state.themeIcon.textContent = '🌙';
        if (mobileThemeIcon) mobileThemeIcon.textContent = '🌙';
        if (state.editor && typeof state.editor.setTheme === 'function') state.editor.setTheme('Figma Dark');
    }

    if (state.editor && typeof state.editor.requestRedraw === 'function') {
        state.editor.requestRedraw();
    }
    if (state.requestRender) {
        state.requestRender();
    }
    showToast(`Theme: ${newTheme === 'light' ? 'Light' : 'Dark'}`);
}
