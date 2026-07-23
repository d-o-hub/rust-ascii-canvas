/**
 * ASCII Canvas Editor - Render Module
 */

import { state } from './state.js';
import {
    FONT_SIZE,
    MIN_COLS,
    MIN_ROWS,
    USE_PIXEL_BUFFER,
    GLYPH_WIDTH,
    GLYPH_HEIGHT,
} from './constants.js';
import { logger } from './logger.js';
import { debounce } from './utils.js';
import type { RenderCommand } from './types.js';

export function computeGridDimensions(): { width: number; height: number } {
    if (!state.canvasContainer) return { width: MIN_COLS, height: MIN_ROWS };
    const rect = state.canvasContainer.getBoundingClientRect();
    const vw = rect.width;

    let maxCols: number, maxRows: number;
    if (vw < 600) {          // mobile
        maxCols = 60;  maxRows = 30;
    } else if (vw < 1024) { // tablet
        maxCols = 120; maxRows = 50;
    } else {                 // desktop
        maxCols = 240; maxRows = 80;
    }
    return {
        width:  Math.min(maxCols, Math.max(MIN_COLS, Math.floor(vw / state.charWidth))),
        height: Math.min(maxRows, Math.max(MIN_ROWS, Math.floor(rect.height / state.lineHeight))),
    };
}

export function measureFont(): void {
    if (!state.ctx) return;

    if (USE_PIXEL_BUFFER) {
        state.charWidth = GLYPH_WIDTH;
        state.lineHeight = GLYPH_HEIGHT;
    } else {
        state.ctx.font = `${FONT_SIZE}px 'JetBrains Mono', monospace`;
        const metrics = state.ctx.measureText('M');
        state.charWidth = metrics.width;
        state.lineHeight = FONT_SIZE * 1.4;
    }

    state.editor?.setFontMetrics(state.charWidth, state.lineHeight, FONT_SIZE);

    if (typeof window !== 'undefined') {
        window.charWidth = state.charWidth;
        window.lineHeight = state.lineHeight;
    }
}

export function resizeCanvas(): void {
    if (!state.canvas || !state.ctx || !state.canvasContainer) return;

    const dpr = window.devicePixelRatio || 1;
    const rect = state.canvasContainer.getBoundingClientRect();

    state.canvas.width = rect.width * dpr;
    state.canvas.height = rect.height * dpr;
    state.canvas.style.width = `${rect.width}px`;
    state.canvas.style.height = `${rect.height}px`;

    state.ctx.scale(dpr, dpr);

    measureFont();

    if (state.editor) {
        if (!state.gridSizeLocked) {
            const { width, height } = computeGridDimensions();
            if (state.editor.width !== width || state.editor.height !== height) {
                state.editor.resize(width, height);
                state.offscreenCanvas = null;
                state.offscreenCtx = null;
                void import('./ui.js').then(m => {
                    m.updateUI();
                });
            }
        }
        requestRender();
    }
}

// Export a debounced version of resizeCanvas
export const debouncedResizeCanvas = debounce(resizeCanvas, 100);

export function requestRender(): void {
    if (state.animationFrameId === null) {
        state.animationFrameId = requestAnimationFrame(render);
    }
}

// Bind to state
state.requestRender = requestRender;

export function render(): void {
    if (!state.editor || !state.canvas || !state.ctx) return;
    state.animationFrameId = null;

    if (state.wasmMemory) {
        const bufferWidth = state.editor.width * 8;
        const bufferHeight = state.editor.height * 20;

        if (!state.offscreenCanvas) {
            state.offscreenCanvas = document.createElement('canvas');
            state.offscreenCanvas.width = bufferWidth;
            state.offscreenCanvas.height = bufferHeight;
            state.offscreenCtx = state.offscreenCanvas.getContext('2d', { alpha: false });
        }

        if (state.editor.needsRedraw) {
            state.editor.renderToPixelBuffer();

            const ptr = state.editor.getPixelBufferPtr();
            const len = state.editor.getPixelBufferLen();
            const data = new Uint8ClampedArray(state.wasmMemory.buffer, ptr, len);
            const imageData = new ImageData(data, bufferWidth, bufferHeight);

            state.offscreenCtx?.putImageData(imageData, 0, 0);
            state.editor.clearDirtyState();
        }

        const dpr = window.devicePixelRatio || 1;
        state.ctx.save();
        state.ctx.setTransform(1, 0, 0, 1, 0, 0);
        state.ctx.scale(dpr, dpr);

        state.ctx.fillStyle = '#1e1e1e';
        state.ctx.fillRect(0, 0, state.canvas.width / dpr, state.canvas.height / dpr);

        state.ctx.imageSmoothingEnabled = false;

        const pan = state.editor.pan as number[] | Float64Array;
        state.ctx.drawImage(
            state.offscreenCanvas,
            pan[0],
            pan[1],
            bufferWidth * state.editor.zoom,
            bufferHeight * state.editor.zoom
        );
        state.ctx.restore();
    } else {
        const commands = state.editor.getDirtyRenderCommands();
        for (const cmd of commands) {
            executeRenderCommand(cmd as RenderCommand);
        }
    }

    updateIndicator();

    if (state.zoomLevelEl) {
        state.zoomLevelEl.textContent = `${Math.round(state.editor.zoom * 100)}%`;
    }
    if (state.fullRendersEl) {
        state.fullRendersEl.textContent = state.editor.fullRenderCount.toString();
    }
    if (state.dirtyRendersEl) {
        state.dirtyRendersEl.textContent = state.editor.dirtyRenderCount.toString();
    }

    if (state.editor.needsRedraw) {
        requestRender();
    }
}

export function executeRenderCommand(cmd: RenderCommand): void {
    if (!state.ctx || !state.canvas) return;

    switch (cmd.type || Object.keys(cmd)[0]) {
        case 'Clear':
            state.ctx.fillStyle = cmd.color as string || '#1e1e1e';
            state.ctx.fillRect(0, 0, state.canvas.width, state.canvas.height);
            break;

        case 'SetFont':
            state.ctx.font = `${FONT_SIZE * (cmd.scale as number || 1)}px 'JetBrains Mono', monospace`;
            state.ctx.textBaseline = 'top';
            break;

        case 'DrawChar':
            state.ctx.fillStyle = '#d4d4d4';
            state.ctx.fillText(cmd.char as string, cmd.x as number, cmd.y as number);
            break;

        case 'DrawPreviewChar':
            state.ctx.fillStyle = 'rgba(86, 156, 214, 0.7)';
            state.ctx.fillText(cmd.char as string, cmd.x as number, cmd.y as number);
            break;

        case 'DrawRect':
            state.ctx.fillStyle = cmd.color as string || '#264f78';
            state.ctx.fillRect(cmd.x as number, cmd.y as number, cmd.width as number, cmd.height as number);
            break;

        case 'DrawGrid': {
            state.ctx.strokeStyle = cmd.color as string || '#333333';
            state.ctx.lineWidth = 0.5;
            state.ctx.beginPath();

            const panX = cmd.pan_x as number || 0;
            const panY = cmd.pan_y as number || 0;
            const cellW = cmd.cell_width as number || state.charWidth;
            const cellH = cmd.cell_height as number || state.lineHeight;

            for (let x = panX % cellW; x < state.canvas.width; x += cellW) {
                state.ctx.moveTo(x, 0);
                state.ctx.lineTo(x, state.canvas.height);
            }

            for (let y = panY % cellH; y < state.canvas.height; y += cellH) {
                state.ctx.moveTo(0, y);
                state.ctx.lineTo(state.canvas.width, y);
            }

            state.ctx.stroke();
            break;
        }
    }
}

export function updateCursorIndicator(gridX: number, gridY: number): void {
    if (!state.editor || !state.cursorIndicator) return;
    const zoom = state.editor.zoom;
    const pan = state.editor.pan;

    let w = state.charWidth * zoom;
    let h = state.lineHeight * zoom;
    let xOffset = 0;
    let yOffset = 0;

    if (state.editor.tool.toLowerCase() === 'eraser') {
        const size = state.editor.eraserSize || 1;
        const radius = size - 1;
        xOffset = -radius * state.charWidth * zoom;
        yOffset = -radius * state.lineHeight * zoom;
        w = (2 * size - 1) * state.charWidth * zoom;
        h = (2 * size - 1) * state.lineHeight * zoom;
    }

    const screenX = gridX * state.charWidth * zoom + pan[0] + xOffset;
    const screenY = gridY * state.lineHeight * zoom + pan[1] + yOffset;

    state.cursorIndicator.style.left = `${screenX}px`;
    state.cursorIndicator.style.top = `${screenY}px`;
    state.cursorIndicator.style.width = `${w}px`;
    state.cursorIndicator.style.height = `${h}px`;
    state.cursorIndicator.classList.remove('hidden');
}

export function updateIndicator(): void {
    if (!state.editor || !state.cursorIndicator) return;

    let textPos: number[] | null = null;
    if (state.editor.tool.toLowerCase() === 'text' && typeof state.editor.textCursorPosition === 'function') {
        textPos = state.editor.textCursorPosition();
    }

    if (textPos) {
        const [gridX, gridY] = textPos;
        const zoom = state.editor.zoom;
        const pan = state.editor.pan as number[] | Float64Array;

        const screenX = gridX * state.charWidth * zoom + pan[0];
        const screenY = gridY * state.lineHeight * zoom + pan[1];

        state.cursorIndicator.style.left = `${screenX}px`;
        state.cursorIndicator.style.top = `${screenY}px`;
        state.cursorIndicator.style.width = `${state.charWidth * zoom}px`;
        state.cursorIndicator.style.height = `${state.lineHeight * zoom}px`;
        state.cursorIndicator.classList.add('caret');
        state.cursorIndicator.classList.remove('hidden');
    } else {
        state.cursorIndicator.classList.remove('caret');
    }
}

export function uploadFontAtlas(): void {
    if (!state.editor) return;

    const canvas = document.createElement('canvas');
    canvas.width = GLYPH_WIDTH;
    canvas.height = GLYPH_HEIGHT;
    const ctx = canvas.getContext('2d', { alpha: true });
    if (!ctx) return;

    const RASTER_FONT_SIZE = 13;
    ctx.font = `${RASTER_FONT_SIZE}px 'JetBrains Mono', monospace`;
    ctx.textBaseline = 'top';
    ctx.fillStyle = 'white';

    const charsToRasterize: string[] = [];
    for (let i = 32; i < 127; i++) {
        charsToRasterize.push(String.fromCharCode(i));
    }
    charsToRasterize.push('┌', '┐', '└', '┘', '─', '│', '╔', '╗', '╚', '╝', '═', '║', '┏', '┓', '┗', '┛', '━', '┃', '╭', '╮', '╰', '╯', '+', '-', '|', '*', '·', '•', '●', '▲', '▼', '◄', '►', '╱', '╲', '◆');

    for (const char of charsToRasterize) {
        ctx.clearRect(0, 0, GLYPH_WIDTH, GLYPH_HEIGHT);
        ctx.fillText(char, 0, 2);

        const imageData = ctx.getImageData(0, 0, GLYPH_WIDTH, GLYPH_HEIGHT);
        const alphaData = new Uint8Array(GLYPH_WIDTH * GLYPH_HEIGHT);

        let hasContent = false;
        for (let i = 0; i < imageData.data.length; i += 4) {
            const alpha = imageData.data[i + 3];
            alphaData[i / 4] = alpha;
            if (alpha > 0) hasContent = true;
        }

        if (!hasContent && char !== ' ') {
            logger.warn(`Glyph for '${char}' (${char.charCodeAt(0)}) rasterized empty. Font might not be loaded.`);
        }

        state.editor.updateFontAtlasGlyph(char.charCodeAt(0), alphaData);
    }

    logger.debug(`Rasterized and uploaded ${charsToRasterize.length} glyphs to WASM atlas`);
    state.editor.requestRedraw();
}
