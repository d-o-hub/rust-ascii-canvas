/**
 * ASCII Canvas Editor - Shared Application State
 */

import type { AsciiEditorInterface } from './types.js';

export interface AppState {
    editor: AsciiEditorInterface | null;
    wasmMemory: WebAssembly.Memory | null;
    canvas: HTMLCanvasElement | null;
    ctx: CanvasRenderingContext2D | null;
    offscreenCanvas: HTMLCanvasElement | null;
    offscreenCtx: CanvasRenderingContext2D | null;
    animationFrameId: number | null;
    isInitialized: boolean;
    gridSizeLocked: boolean;
    charWidth: number;
    lineHeight: number;
    currentBorderStyleIndex: number;
    currentLineDirection: string;
    lastTouchDistance: number | null;
    lastFocusedElement: HTMLElement | null;

    // Callbacks to avoid circular dependency
    requestRender: (() => void) | null;
    scheduleAutoSave: (() => void) | null;

    // DOM Elements
    loadingOverlay: HTMLElement | null;
    canvasContainer: HTMLElement | null;
    cursorIndicator: HTMLElement | null;
    gridSizeEl: HTMLElement | null;
    cursorPosEl: HTMLElement | null;
    zoomLevelEl: HTMLElement | null;
    fullRendersEl: HTMLElement | null;
    dirtyRendersEl: HTMLElement | null;
    statusToolEl: HTMLElement | null;
    statusMessageEl: HTMLElement | null;
    statusToast: HTMLElement | null;
    undoBtn: HTMLButtonElement | null;
    redoBtn: HTMLButtonElement | null;
    copyBtn: HTMLButtonElement | null;
    clearBtn: HTMLButtonElement | null;
    helpBtn: HTMLButtonElement | null;
    borderStyleSelect: HTMLSelectElement | null;
    toolButtons: NodeListOf<Element> | null;
    zoomFitBtn: HTMLButtonElement | null;
    zoomResetBtn: HTMLButtonElement | null;
    zoomOutBtn: HTMLButtonElement | null;
    zoomInBtn: HTMLButtonElement | null;
    directionBtns: NodeListOf<Element> | null;
    eraserRadiusSelect: HTMLSelectElement | null;
    mobileKeyboardProxy: HTMLInputElement | null;
}

export const state: AppState = {
    editor: null,
    wasmMemory: null,
    canvas: null,
    ctx: null,
    offscreenCanvas: null,
    offscreenCtx: null,
    animationFrameId: null,
    isInitialized: false,
    gridSizeLocked: false,
    charWidth: 8.4,
    lineHeight: 20,
    currentBorderStyleIndex: 0,
    currentLineDirection: 'auto',
    lastTouchDistance: null,
    lastFocusedElement: null,

    // Callbacks
    requestRender: null,
    scheduleAutoSave: null,

    // DOM Elements
    loadingOverlay: null,
    canvasContainer: null,
    cursorIndicator: null,
    gridSizeEl: null,
    cursorPosEl: null,
    zoomLevelEl: null,
    fullRendersEl: null,
    dirtyRendersEl: null,
    statusToolEl: null,
    statusMessageEl: null,
    statusToast: null,
    undoBtn: null,
    redoBtn: null,
    copyBtn: null,
    clearBtn: null,
    helpBtn: null,
    borderStyleSelect: null,
    toolButtons: null,
    zoomFitBtn: null,
    zoomResetBtn: null,
    zoomOutBtn: null,
    zoomInBtn: null,
    directionBtns: null,
    eraserRadiusSelect: null,
    mobileKeyboardProxy: null,
};
