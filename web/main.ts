/**
 * ASCII Canvas Editor - Main TypeScript Entry Point
 *
 * Initializes the WASM module and sets up the editor UI.
 */

import init, { AsciiEditor } from './pkg/ascii_canvas.js';
import { logger } from './logger.js';
import type { AsciiEditorInterface } from './types.js';
import { FONT_SIZE, TOOL_INFO } from './constants.js';
import { getElement } from './utils.js';
import { tryRestoreAutoSave } from './persistence.js';
import { state } from './state.js';
import {
    computeGridDimensions,
    measureFont,
    resizeCanvas,
    requestRender,
    uploadFontAtlas,
} from './render.js';
import {
    updateToolButtons,
    updateUI,
    setTool,
    syncGridInputs,
} from './ui.js';
import { setupEventListeners } from './events.js';

// Legacy exports
export let editor: AsciiEditorInterface | null = null;
export let canvas: HTMLCanvasElement | null = null;
export let ctx: CanvasRenderingContext2D | null = null;
export let charWidth = 8.4;
export let lineHeight = 20;

// Expose editor for testing
declare global {
    interface Window {
        editor: AsciiEditorInterface | null;
        charWidth: number;
        lineHeight: number;
    }
}

if (typeof window !== 'undefined') {
    Object.defineProperty(window, 'editor', {
        get: () => state.editor,
        set: (v) => { state.editor = v; },
        configurable: true
    });
    Object.defineProperty(window, 'charWidth', {
        get: () => state.charWidth,
        set: (v) => { state.charWidth = v; },
        configurable: true
    });
    Object.defineProperty(window, 'lineHeight', {
        get: () => state.lineHeight,
        set: (v) => { state.lineHeight = v; },
        configurable: true
    });
}

/**
 * Initialize the editor
 */
async function initialize() {
    try {
        // Initialize WASM
        const wasm = await init();
        state.wasmMemory = wasm.memory;

        // Initialize DOM elements
        state.loadingOverlay = getElement('loading');
        state.canvasContainer = getElement('canvas-container');
        state.cursorIndicator = getElement('cursor-indicator');
        state.gridSizeEl = getElement('grid-size');
        state.cursorPosEl = getElement('cursor-pos');
        state.zoomLevelEl = getElement('zoom-level');
        state.fullRendersEl = getElement('full-renders');
        state.dirtyRendersEl = getElement('dirty-renders');
        state.statusToolEl = getElement('status-tool');
        state.statusMessageEl = getElement('status-message');
        state.statusToast = getElement('status-toast');
        state.undoBtn = getElement<HTMLButtonElement>('undo-btn');
        state.redoBtn = getElement<HTMLButtonElement>('redo-btn');
        state.copyBtn = getElement<HTMLButtonElement>('copy-btn');
        state.clearBtn = getElement<HTMLButtonElement>('clear-btn');
        state.helpBtn = getElement<HTMLButtonElement>('help-btn');
        state.borderStyleSelect = getElement<HTMLSelectElement>('border-style');
        state.toolButtons = document.querySelectorAll('.tool-btn');
        state.zoomFitBtn = getElement<HTMLButtonElement>('zoom-fit');
        state.zoomResetBtn = getElement<HTMLButtonElement>('zoom-reset');
        state.zoomOutBtn = getElement<HTMLButtonElement>('zoom-out');
        state.zoomInBtn = getElement<HTMLButtonElement>('zoom-in');
        state.directionBtns = document.querySelectorAll('.direction-btn');
        state.eraserRadiusSelect = getElement<HTMLSelectElement>('eraser-radius');
        state.mobileKeyboardProxy = getElement<HTMLInputElement>('mobile-keyboard-proxy');
        if (state.mobileKeyboardProxy) {
            state.mobileKeyboardProxy.value = ' '; // Initialize with space for backspace detection
        }

        // Get canvas and context
        state.canvas = getElement<HTMLCanvasElement>('canvas');
        canvas = state.canvas; // Sync with legacy export

        const ctxResult = state.canvas.getContext('2d', { alpha: false });
        if (!ctxResult) {
            throw new Error('Failed to get 2D context');
        }
        state.ctx = ctxResult;
        ctx = state.ctx; // Sync with legacy export

        // Measure font metrics MUST run before computeGridDimensions()
        measureFont();
        charWidth = state.charWidth; // Sync with legacy export
        lineHeight = state.lineHeight; // Sync with legacy export

        // Set up canvas size first to have correct container rect
        resizeCanvas();

        // If the container has collapsed height (e.g. layout pending on WebKit), wait for layout
        if (state.canvasContainer) {
            let rect = state.canvasContainer.getBoundingClientRect();
            if (rect.height < 50) {
                for (let i = 0; i < 5; i++) {
                    await new Promise(resolve => requestAnimationFrame(resolve));
                    resizeCanvas();
                    rect = state.canvasContainer.getBoundingClientRect();
                    if (rect.height >= 50) break;
                }
            }
        }

        // Create editor with responsive dimensions
        const { width, height } = computeGridDimensions();
        state.editor = new AsciiEditor(width, height) as unknown as AsciiEditorInterface;
        editor = state.editor; // Sync with legacy export

        // Update editor with font metrics again after creation
        if (state.editor) {
            state.editor.setFontMetrics(state.charWidth, state.lineHeight, FONT_SIZE);
        }

        // Restore auto-saved document if present (locks grid so resize cannot crop it).
        if (tryRestoreAutoSave(state.editor)) {
            logger.info('Restored auto-saved diagram');
            state.gridSizeLocked = true;
            state.offscreenCanvas = null;
            state.offscreenCtx = null;
        }

        // Set up event listeners
        setupEventListeners();

        // Update UI
        updateUI();
        syncGridInputs();

        // Initial render
        requestRender();

        state.isInitialized = true;

        // Hide loading overlay
        if (state.loadingOverlay) {
            state.loadingOverlay.classList.add('hidden');
        }

        // Dispatch a resize event to ensure WebKit viewport/layout is fully synchronized
        window.dispatchEvent(new Event('resize'));
        setTimeout(() => {
            window.dispatchEvent(new Event('resize'));
        }, 100);

        // Focus canvas
        if (state.canvas) {
            state.canvas.focus();
        }

        // Rasterize and upload font atlas
        document.fonts.ready.then(() => {
            uploadFontAtlas();
        });

        // Set initial active tool button
        updateToolButtons('rectangle');

        logger.info('ASCII Canvas Editor initialized');
    } catch (error) {
        logger.error('Failed to initialize:', error);
        if (state.statusMessageEl) {
            state.statusMessageEl.textContent = `Failed to initialize: ${error}`;
        }
        if (state.loadingOverlay) {
            state.loadingOverlay.classList.add('hidden');
        }
    }
}

// Initialize when DOM is ready
if (typeof document !== 'undefined') {
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', initialize);
    } else {
        // Only initialize if we're not in a test environment or if explicitly called
        if (typeof window !== 'undefined' && !((window as unknown as { process?: { env?: { VITEST?: boolean } } }).process?.env?.VITEST)) {
            initialize();
        }
    }
}

// Export for testing
export { TOOL_INFO, setTool, updateToolButtons };
