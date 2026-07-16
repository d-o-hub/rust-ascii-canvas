/**
 * ASCII Canvas Editor - Main TypeScript Entry Point
 *
 * Initializes the WASM module and sets up the editor UI.
 */

import init, { AsciiEditor } from './pkg/ascii_canvas.js';
import { logger } from './logger.js';
import type { AsciiEditorInterface, EventResult, RenderCommand } from './types.js';
import {
    FONT_SIZE,
    MIN_COLS,
    MIN_ROWS,
    USE_PIXEL_BUFFER,
    GLYPH_WIDTH,
    GLYPH_HEIGHT,
    TOOL_INFO,
    BORDER_STYLES,
} from './constants.js';
import { capitalize, debounce, getElement } from './utils.js';
import { copyAsciiToClipboard, copyToClipboard as copySelectionAware } from './clipboard.js';
import {
    createAutoSaveScheduler,
    downloadDocument,
    openDocumentPicker,
    tryRestoreAutoSave,
} from './persistence.js';
import { exportPng } from './exportPng.js';

// Global state
let editor: AsciiEditorInterface | null = null;
let wasmMemory: WebAssembly.Memory | null = null;
let canvas: HTMLCanvasElement | null = null;
let ctx: CanvasRenderingContext2D | null = null;
let offscreenCanvas: HTMLCanvasElement | null = null;
let offscreenCtx: CanvasRenderingContext2D | null = null;
let animationFrameId: number | null = null;
let isInitialized = false;
void isInitialized;
/** When true, window resize updates CSS pixels only and does not shrink the grid. */
let gridSizeLocked = false;

// Expose editor for testing
declare global {
    interface Window {
        editor: AsciiEditorInterface | null;
        charWidth: number;
        lineHeight: number;
    }
}

if (typeof window !== 'undefined') {
    window.editor = null;
}

let charWidth = 8.4;
let lineHeight = 20;

// DOM Elements - will be initialized in initialize()
let loadingOverlay: HTMLElement;
let canvasContainer: HTMLElement;
let cursorIndicator: HTMLElement;
let gridSizeEl: HTMLElement;
let cursorPosEl: HTMLElement;
let zoomLevelEl: HTMLElement;
let fullRendersEl: HTMLElement;
let dirtyRendersEl: HTMLElement;
let statusToolEl: HTMLElement;
let statusMessageEl: HTMLElement;
let statusToast: HTMLElement;
let undoBtn: HTMLButtonElement;
let redoBtn: HTMLButtonElement;
let copyBtn: HTMLButtonElement;
let clearBtn: HTMLButtonElement;
let helpBtn: HTMLButtonElement;
let borderStyleSelect: HTMLSelectElement;
let toolButtons: NodeListOf<Element>;
let zoomFitBtn: HTMLButtonElement;
let zoomResetBtn: HTMLButtonElement;
let zoomOutBtn: HTMLButtonElement;
let zoomInBtn: HTMLButtonElement;
let lineDirectionGroup: HTMLDivElement;
let directionBtns: NodeListOf<Element>;
let mobileKeyboardProxy: HTMLInputElement;


// Mobile state
let lastTouchDistance: number | null = null;

let currentBorderStyleIndex = 0;
let currentLineDirection = 'auto';
void currentLineDirection;

const { schedule: scheduleAutoSave, flush: flushAutoSave } = createAutoSaveScheduler(() => editor);

/** Module-level handlers so nested closures are not flagged as non-serializable (Biome/Qwik FP). */
function onPageHideFlushAutoSave(): void {
    if (editor) flushAutoSave();
}

function onVisibilityChangeFlushAutoSave(): void {
    if (document.visibilityState === 'hidden' && editor) {
        flushAutoSave();
    }
}

/**
 * Compute grid dimensions based on viewport
 */
function computeGridDimensions(): { width: number; height: number } {
    if (!canvasContainer) return { width: MIN_COLS, height: MIN_ROWS };
    const rect = canvasContainer.getBoundingClientRect();
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
        width:  Math.min(maxCols, Math.max(MIN_COLS, Math.floor(vw / charWidth))),
        height: Math.min(maxRows, Math.max(MIN_ROWS, Math.floor(rect.height / lineHeight))),
    };
}

/**
 * Initialize the editor
 */
async function initialize() {
    try {
        // Initialize WASM
        const wasm = await init();
        wasmMemory = wasm.memory;

        // Initialize DOM elements
        loadingOverlay = getElement('loading');
        canvasContainer = getElement('canvas-container');
        cursorIndicator = getElement('cursor-indicator');
        gridSizeEl = getElement('grid-size');
        cursorPosEl = getElement('cursor-pos');
        zoomLevelEl = getElement('zoom-level');
        fullRendersEl = getElement('full-renders');
        dirtyRendersEl = getElement('dirty-renders');
        statusToolEl = getElement('status-tool');
        statusMessageEl = getElement('status-message');
        statusToast = getElement('status-toast');
        undoBtn = getElement<HTMLButtonElement>('undo-btn');
        redoBtn = getElement<HTMLButtonElement>('redo-btn');
        copyBtn = getElement<HTMLButtonElement>('copy-btn');
        clearBtn = getElement<HTMLButtonElement>('clear-btn');
        helpBtn = getElement<HTMLButtonElement>('help-btn');
        borderStyleSelect = getElement<HTMLSelectElement>('border-style');
        toolButtons = document.querySelectorAll('.tool-btn');
        zoomFitBtn = getElement<HTMLButtonElement>('zoom-fit');
        zoomResetBtn = getElement<HTMLButtonElement>('zoom-reset');
        zoomOutBtn = getElement<HTMLButtonElement>('zoom-out');
        zoomInBtn = getElement<HTMLButtonElement>('zoom-in');
        lineDirectionGroup = getElement<HTMLDivElement>('line-direction-group');
        directionBtns = document.querySelectorAll('.direction-btn');
        mobileKeyboardProxy = getElement<HTMLInputElement>('mobile-keyboard-proxy');
        mobileKeyboardProxy.value = ' '; // Initialize with space for backspace detection

        // Get canvas and context
        canvas = getElement<HTMLCanvasElement>('canvas');
        const ctxResult = canvas.getContext('2d', { alpha: false });
        if (!ctxResult) {
            throw new Error('Failed to get 2D context');
        }
        ctx = ctxResult;

        // Measure font metrics MUST run before computeGridDimensions()
        measureFont();

        // Set up canvas size first to have correct container rect
        resizeCanvas();

        // Create editor with responsive dimensions
        const { width, height } = computeGridDimensions();
        editor = new AsciiEditor(width, height) as unknown as AsciiEditorInterface;
        window.editor = editor;

        // Update editor with font metrics again after creation
        if (editor) {
            editor.setFontMetrics(charWidth, lineHeight, FONT_SIZE);
        }

        // Restore auto-saved document if present (locks grid so resize cannot crop it).
        if (tryRestoreAutoSave(editor)) {
            logger.info('Restored auto-saved diagram');
            gridSizeLocked = true;
            offscreenCanvas = null;
            offscreenCtx = null;
        }

        // Set up event listeners
        setupEventListeners();

        // Update UI
        updateUI();
        refreshLayerSelect();
        syncGridInputs();

        // Initial render
        requestRender();

        isInitialized = true;

        // Hide loading overlay
        loadingOverlay.classList.add('hidden');

        // Focus canvas
        canvas.focus();

        // Rasterize and upload font atlas
        document.fonts.ready.then(() => {
            uploadFontAtlas();
        });

        // Set initial active tool button
        updateToolButtons('rectangle');

        logger.info('ASCII Canvas Editor initialized');
    } catch (error) {
        logger.error('Failed to initialize:', error);
        if (statusMessageEl) {
            statusMessageEl.textContent = `Failed to initialize: ${error}`;
        }
        if (loadingOverlay) {
            loadingOverlay.classList.add('hidden');
        }
    }
}

/**
 * Measure font metrics for precise grid alignment
 */
function measureFont() {
    if (!ctx) return;

    if (USE_PIXEL_BUFFER) {
        charWidth = GLYPH_WIDTH;
        lineHeight = GLYPH_HEIGHT;
    } else {
        ctx.font = `${FONT_SIZE}px 'JetBrains Mono', monospace`;
        const metrics = ctx.measureText('M');
        charWidth = metrics.width;
        lineHeight = FONT_SIZE * 1.4;
    }

    // Update editor with font metrics
    if (editor) {
        editor.setFontMetrics(charWidth, lineHeight, FONT_SIZE);
    }

    // Expose for testing
    window.charWidth = charWidth;
    window.lineHeight = lineHeight;
}

/**
 * Resize canvas to match container.
 * When `gridSizeLocked` (restored doc / custom Apply / load file), only update
 * CSS/device pixels — never shrink or grow the logical grid via responsive sizing.
 */
function resizeCanvas() {
    if (!canvas || !ctx || !canvasContainer) return;
    
    const dpr = window.devicePixelRatio || 1;
    const rect = canvasContainer.getBoundingClientRect();

    canvas.width = rect.width * dpr;
    canvas.height = rect.height * dpr;
    canvas.style.width = `${rect.width}px`;
    canvas.style.height = `${rect.height}px`;

    ctx.scale(dpr, dpr);

    // Re-measure font after resize
    measureFont();

    if (editor) {
        if (!gridSizeLocked) {
            const { width, height } = computeGridDimensions();
            if (editor.width !== width || editor.height !== height) {
                editor.resize(width, height);
                offscreenCanvas = null; // force offscreen canvas recreation
                offscreenCtx = null;
                updateUI();
            }
        }
        requestRender();
    }
}

/**
 * Set up all event listeners
 */
function setupEventListeners() {
    if (!canvas) return;
    
    // Window resize
    window.addEventListener('resize', debounce(resizeCanvas, 100));

    // Flush pending auto-save so a short-lived tab close does not drop edits.
    window.addEventListener('pagehide', onPageHideFlushAutoSave);
    document.addEventListener('visibilitychange', onVisibilityChangeFlushAutoSave);

    // Reset panning state when window loses focus
    window.addEventListener('blur', () => {
        if (editor) {
            editor.onKeyUp(' ');
        }
        canvasContainer.classList.remove('panning');
    });

    // Canvas pointer events
    canvas.addEventListener('pointerdown', handlePointerDown);
    canvas.addEventListener('pointermove', handlePointerMove);
    canvas.addEventListener('pointerup', handlePointerUp);
    canvas.addEventListener('pointerleave', handlePointerLeave);

    // Touch events for mobile
    canvas.addEventListener('touchstart', handleTouchStart, { passive: false });
    canvas.addEventListener('touchmove', handleTouchMove, { passive: false });
    canvas.addEventListener('touchend', handleTouchEnd, { passive: false });

    // Mobile keyboard proxy events
    mobileKeyboardProxy.addEventListener('input', handleMobileInput);

    // Prevent context menu
    canvas.addEventListener('contextmenu', (e) => e.preventDefault());

    // Wheel for zoom
    canvas.addEventListener('wheel', handleWheel, { passive: false });

    // Keyboard events
    canvas.addEventListener('keydown', handleKeyDown);
    canvas.addEventListener('keyup', handleKeyUp);

    // Global keyboard listener for modal
    window.addEventListener('keydown', (e) => {
        if (e.key === 'Escape') {
            const modal = document.getElementById('shortcuts-modal');
            if (modal && !modal.classList.contains('hidden')) {
                hideShortcutsModal();
            }
        }
    });

    // Keyboard shortcuts modal
    const shortcutsModal = document.getElementById('shortcuts-modal');
    if (shortcutsModal) {
        const closeBtn = shortcutsModal.querySelector('.modal-close');
        if (closeBtn) {
            closeBtn.addEventListener('click', hideShortcutsModal);
        }
        shortcutsModal.addEventListener('click', (e) => {
            if (e.target === shortcutsModal) {
                hideShortcutsModal();
            }
        });
    }

    // Tool buttons
    toolButtons.forEach(btn => {
        btn.addEventListener('mousedown', (e) => e.preventDefault());
        btn.addEventListener('click', () => {
            const tool = btn.getAttribute('data-tool');
            if (tool) {
                setTool(tool);
            }
        });
    });

    // Border style - use click instead of mousedown to allow dropdown to open
    borderStyleSelect.addEventListener('click', () => {
        // No action needed here, just allow click through
    });
    borderStyleSelect.addEventListener('change', () => {
        if (editor) {
            editor.setBorderStyle(borderStyleSelect.value);
            currentBorderStyleIndex = BORDER_STYLES.indexOf(borderStyleSelect.value);
        }
    });

    // Line direction buttons
    directionBtns.forEach(btn => {
        btn.addEventListener('mousedown', (e) => e.preventDefault());
        btn.addEventListener('click', () => {
            const direction = btn.getAttribute('data-direction');
            if (direction === null) return;

            currentLineDirection = direction;
            
            // Update active state
            directionBtns.forEach(b => {
                b.classList.remove('active');
                b.setAttribute('aria-pressed', 'false');
            });
            btn.classList.add('active');
            btn.setAttribute('aria-pressed', 'true');
            
            // Call WASM to set line direction
            if (editor) {
                editor.setLineDirection(direction);
            }
        });
    });

    // Action buttons
    undoBtn.addEventListener('mousedown', (e) => e.preventDefault());
    undoBtn.addEventListener('click', () => {
        if (!editor) return;
        editor.undo();
        requestRender();
        updateUI();
        if (canvas) canvas.focus();
    });

    redoBtn.addEventListener('mousedown', (e) => e.preventDefault());
    redoBtn.addEventListener('click', () => {
        if (!editor) return;
        editor.redo();
        requestRender();
        updateUI();
        if (canvas) canvas.focus();
    });

    copyBtn.addEventListener('mousedown', (e) => e.preventDefault());
    copyBtn.addEventListener('click', () => {
        void copyToClipboard();
        if (canvas) canvas.focus();
    });

    clearBtn.addEventListener('mousedown', (e) => e.preventDefault());
    clearBtn.addEventListener('click', () => {
        if (!editor) return;
        if (confirm('Clear the canvas? This cannot be undone.')) {
            editor.clear();
            requestRender();
            updateUI();
            scheduleAutoSave();
            showToast('Canvas cleared');
            if (canvas) canvas.focus();
        }
    });

    // Optional feature controls: resolve and wire without storing Element refs in nullable module vars
    // (avoids Codacy ESLint xss/no-mixed-html false positives on DOM lookups).
    wireOptionalButton('save-btn', () => {
        if (!editor) return;
        downloadDocument(editor, showToast);
        if (canvas) canvas.focus();
    });
    wireOptionalButton('load-btn', () => {
        if (!editor) return;
        openDocumentPicker(editor, showToast, () => {
            gridSizeLocked = true;
            offscreenCanvas = null;
            offscreenCtx = null;
            refreshLayerSelect();
            syncGridInputs();
            requestRender();
            updateUI();
            flushAutoSave();
        });
    });
    wireOptionalButton('png-btn', () => {
        if (editor) {
            editor.requestRedraw();
            requestRender();
            requestAnimationFrame(() => {
                exportPng(offscreenCanvas, canvas, showToast);
            });
        }
        if (canvas) canvas.focus();
    });
    wireOptionalButton('apply-grid-btn', () => {
        applyCustomGridSize();
        if (canvas) canvas.focus();
    });
    wireOptionalButton('add-layer-btn', () => {
        if (!editor) return;
        editor.addLayer();
        refreshLayerSelect();
        requestRender();
        updateUI();
        scheduleAutoSave();
        showToast('Layer added');
        if (canvas) canvas.focus();
    });

    const layerSelectEl = document.querySelector('#layer-select');
    if (layerSelectEl instanceof HTMLSelectElement) {
        layerSelectEl.addEventListener('change', () => {
            if (!editor) return;
            const idx = parseInt(layerSelectEl.value, 10);
            if (!Number.isNaN(idx)) {
                editor.setActiveLayer(idx);
                requestRender();
                updateUI();
                scheduleAutoSave();
            }
            if (canvas) canvas.focus();
        });
    }

    helpBtn.addEventListener('mousedown', (e) => e.preventDefault());
    helpBtn.addEventListener('click', showShortcutsModal);

    // Zoom buttons
    zoomFitBtn.addEventListener('mousedown', (e) => e.preventDefault());
    zoomFitBtn.addEventListener('click', () => {
        if (!editor) return;
        fitZoom();
        if (canvas) canvas.focus();
    });

    zoomResetBtn.addEventListener('mousedown', (e) => e.preventDefault());
    zoomResetBtn.addEventListener('click', () => {
        if (!editor) return;
        resetZoom();
        if (canvas) canvas.focus();
    });

    zoomOutBtn.addEventListener('mousedown', (e) => e.preventDefault());
    zoomOutBtn.addEventListener('click', () => {
        if (!editor) return;
        setZoom(editor.zoom * 0.8);
        if (canvas) canvas.focus();
    });

    zoomInBtn.addEventListener('mousedown', (e) => e.preventDefault());
    zoomInBtn.addEventListener('click', () => {
        if (!editor) return;
        setZoom(editor.zoom * 1.25);
        if (canvas) canvas.focus();
    });
}

/**
 * Handle pointer down event
 */
function handlePointerDown(e: PointerEvent) {
    if (!editor || !canvas) return;
    if (e.pointerType === 'touch') return; // Handled by touch events
    e.preventDefault();
    canvas.focus();
    canvas.setPointerCapture(e.pointerId);

    const rect = canvas.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;

    const result = editor.onPointerDown(x, y);
    handleEventResult(result);
}

/**
 * Handle pointer move event
 */
function handlePointerMove(e: PointerEvent) {
    if (!editor || !canvas) return;
    if (e.pointerType === 'touch') return; // Handled by touch events
    const rect = canvas.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;

    // Update cursor position display
    const pan = editor.pan as number[] | Float64Array;
    const gridX = Math.floor((x - pan[0]) / editor.zoom / charWidth);
    const gridY = Math.floor((y - pan[1]) / editor.zoom / lineHeight);
    cursorPosEl.textContent = `${gridX}, ${gridY}`;

    // Update cursor indicator
    updateCursorIndicator(gridX, gridY);

    const result = editor.onPointerMove(x, y);
    // Do not auto-save on pointermove (pan/hover/preview) — avoids thrashing and empty saves.
    handleEventResult(result, { persist: false });
}

/**
 * Handle pointer up event
 */
function handlePointerUp(e: PointerEvent) {
    if (!editor || !canvas) return;
    if (e.pointerType === 'touch') return; // Handled by touch events
    e.preventDefault();
    canvas.releasePointerCapture(e.pointerId);

    const rect = canvas.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;

    const result = editor.onPointerUp(x, y);
    handleEventResult(result, { persist: true });
}

/**
 * Handle touch start event
 */
function handleTouchStart(e: TouchEvent) {
    if (!editor || !canvas) return;
    e.preventDefault();
    canvas.focus();

    if (e.touches.length === 1) {
        const touch = e.touches[0];
        const rect = canvas.getBoundingClientRect();
        const x = touch.clientX - rect.left;
        const y = touch.clientY - rect.top;
        const result = editor.onPointerDown(x, y);
        handleEventResult(result);
    } else if (e.touches.length === 2) {
        lastTouchDistance = Math.hypot(
            e.touches[0].clientX - e.touches[1].clientX,
            e.touches[0].clientY - e.touches[1].clientY
        );
    }
}

/**
 * Handle touch move event
 */
function handleTouchMove(e: TouchEvent) {
    if (!editor || !canvas) return;
    e.preventDefault();

    if (e.touches.length === 1) {
        const touch = e.touches[0];
        const rect = canvas.getBoundingClientRect();
        const x = touch.clientX - rect.left;
        const y = touch.clientY - rect.top;

        // Update cursor indicator
        const pan = editor.pan as number[] | Float64Array;
        const gridX = Math.floor((x - pan[0]) / editor.zoom / charWidth);
        const gridY = Math.floor((y - pan[1]) / editor.zoom / lineHeight);
        updateCursorIndicator(gridX, gridY);

        const result = editor.onPointerMove(x, y);
        handleEventResult(result, { persist: false });
    } else if (e.touches.length === 2) {
        const currentDistance = Math.hypot(
            e.touches[0].clientX - e.touches[1].clientX,
            e.touches[0].clientY - e.touches[1].clientY
        );

        if (lastTouchDistance !== null) {
            const delta = lastTouchDistance - currentDistance;
            const rect = canvas.getBoundingClientRect();
            const centerX = (e.touches[0].clientX + e.touches[1].clientX) / 2 - rect.left;
            const centerY = (e.touches[0].clientY + e.touches[1].clientY) / 2 - rect.top;

            // Map distance change to onWheel for zoom
            const result = editor.onWheel(delta * 2, centerX, centerY);
            handleEventResult(result, { persist: false });
        }
        lastTouchDistance = currentDistance;
    }
}

/**
 * Handle touch end event
 */
function handleTouchEnd(e: TouchEvent) {
    if (!editor || !canvas) return;
    e.preventDefault();

    if (e.touches.length === 0 && e.changedTouches.length === 1) {
        const touch = e.changedTouches[0];
        const rect = canvas.getBoundingClientRect();
        const x = touch.clientX - rect.left;
        const y = touch.clientY - rect.top;
        const result = editor.onPointerUp(x, y);
        handleEventResult(result);
    }
    lastTouchDistance = null;
}

/**
 * Handle mobile input proxy
 */
function handleMobileInput(e: Event) {
    if (!editor) return;
    const input = e.target as HTMLInputElement;
    const value = input.value;

    if (value.length === 0) {
        // Backspace pressed (consumed the initial space or all content)
        const result = editor.onKeyDown('Backspace', false, false);
        handleEventResult(result);
        input.value = ' ';
    } else if (value.length > 1) {
        // Character(s) added
        const newChars = value.substring(1);
        for (const char of newChars) {
            const result = editor.onKeyDown(char, false, false);
            handleEventResult(result);
        }
        input.value = ' ';
    }
}

/**
 * Handle pointer leave event
 */
function handlePointerLeave() {
    cursorIndicator.classList.add('hidden');
}

/**
 * Handle wheel event for zoom
 */
function handleWheel(e: WheelEvent) {
    if (!editor || !canvas) return;
    e.preventDefault();

    const rect = canvas.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;

    const result = editor.onWheel(e.deltaY, x, y);
    handleEventResult(result, { persist: false });
}

/**
 * Handle key down event
 */
function handleKeyDown(e: KeyboardEvent) {
    if (!editor) return;
    const key = e.key;
    const ctrl = e.ctrlKey || e.metaKey;
    const shift = e.shiftKey;

    // Prevent default for our shortcuts
    if (['r', 'l', 'a', 'd', 't', 'f', 'v', 'e', 'b', ' ', 'escape', '?'].includes(key.toLowerCase()) && !ctrl) {
        e.preventDefault();
    }
    if (ctrl && ['z', 'y', 'c', 'a', 'x', 'v'].includes(key.toLowerCase())) {
        e.preventDefault();
    }

    const result = editor.onKeyDown(key, ctrl, shift);

    if (key === ' ' && !ctrl && !shift) {
        canvasContainer.classList.add('panning');
    }

    handleEventResult(result);

    // While the Text tool is selected, letter keys must type characters — not switch tools
    // (e.g. typing "HELLO" must not activate Eraser on 'E').
    // Switch tools via the toolbar or press Escape first, then a shortcut.
    const isTextTool = editor.tool.toLowerCase() === 'text';

    if (!isTextTool) {
        // Handle B key - cycle border styles
        if (key.toLowerCase() === 'b' && !ctrl && !shift) {
            cycleBorderStyle();
        }

        // Handle 0 key - reset zoom or fit to view
        if ((key === '0' || key === ')') && !ctrl) {
            if (shift || key === ')') {
                fitZoom();
            } else {
                resetZoom();
            }
        }

        // Handle ? key - show keyboard shortcuts modal
        if (key === '?' || (key === '/' && shift)) {
            showShortcutsModal();
        }

        // Handle tool shortcuts (desktop + keyboard UI)
        const lowerKey = key.toLowerCase();
        for (const [toolName, info] of Object.entries(TOOL_INFO)) {
            if (info.shortcut.toLowerCase() === lowerKey && !ctrl && !shift) {
                setTool(toolName);
                break;
            }
        }
    } else if (key === '?' || (key === '/' && shift)) {
        showShortcutsModal();
    }
}

/**
 * Handle key up event
 */
function handleKeyUp(e: KeyboardEvent) {
    if (!editor) return;
    editor.onKeyUp(e.key);

    if (e.key === ' ') {
        canvasContainer.classList.remove('panning');
    }
}

/**
 * Handle event result from WASM.
 * @param persist When true (default), schedule debounced auto-save. Pass false for
 *   navigation-only paths (pointermove, wheel, pan) so we do not serialize on every hover.
 */
function handleEventResult(result: EventResult | null, options: { persist?: boolean } = {}) {
    if (!result) return;
    const persist = options.persist !== false;

    if (result.needs_redraw) {
        requestRender();
    }

    if (result.tool) {
        updateToolButtons(result.tool);
    }

    // Focus the mobile keyboard proxy only on coarse pointers (touch).
    // Auto-focusing it on desktop steals focus from the canvas so tool shortcuts
    // (e.g. E for eraser after T for text) never reach the editor.
    if (editor && editor.tool.toLowerCase() === 'text') {
        const touchUi = typeof window.matchMedia === 'function'
            && window.matchMedia('(pointer: coarse)').matches;
        if (touchUi && document.activeElement !== mobileKeyboardProxy) {
            mobileKeyboardProxy.focus();
        }
    } else if (document.activeElement === mobileKeyboardProxy) {
        mobileKeyboardProxy.blur();
        if (canvas) canvas.focus();
    }

    if (result.should_copy && result.ascii) {
        void copyAsciiToClipboard(result.ascii, showToast);
    }

    updateUI();
    if (persist) {
        scheduleAutoSave();
    }
}

/**
 * Request a render frame
 */
function requestRender() {
    if (animationFrameId === null) {
        animationFrameId = requestAnimationFrame(render);
    }
}

/**
 * Render the canvas
 */
function render() {
    if (!editor || !canvas || !ctx) return;
    animationFrameId = null;

    if (USE_PIXEL_BUFFER && wasmMemory) {
        // Pixel buffer rendering path (Issue #9)
        const bufferWidth = editor.width * 8;
        const bufferHeight = editor.height * 20;

        if (!offscreenCanvas) {
            offscreenCanvas = document.createElement('canvas');
            offscreenCanvas.width = bufferWidth;
            offscreenCanvas.height = bufferHeight;
            offscreenCtx = offscreenCanvas.getContext('2d', { alpha: false });
        }

        if (editor.needsRedraw) {
            editor.renderToPixelBuffer();

            const ptr = editor.getPixelBufferPtr();
            const len = editor.getPixelBufferLen();
            const data = new Uint8ClampedArray(wasmMemory.buffer, ptr, len);
            const imageData = new ImageData(data, bufferWidth, bufferHeight);

            offscreenCtx?.putImageData(imageData, 0, 0);
            editor.clearDirtyState();
        }

        // Scale pixel buffer for zoom/pan
        const dpr = window.devicePixelRatio || 1;
        ctx.save();
        ctx.setTransform(1, 0, 0, 1, 0, 0);
        ctx.scale(dpr, dpr);

        // Fill background first to clear outside scaled area
        ctx.fillStyle = '#1e1e1e';
        ctx.fillRect(0, 0, canvas.width / dpr, canvas.height / dpr);

        // For pixel-art look, disable smoothing
        ctx.imageSmoothingEnabled = false;

        const pan = editor.pan as number[] | Float64Array;
        ctx.drawImage(
            offscreenCanvas,
            pan[0],
            pan[1],
            bufferWidth * editor.zoom,
            bufferHeight * editor.zoom
        );
        ctx.restore();
    } else {
        // Traditional command-based rendering path
        const commands = editor.getDirtyRenderCommands();
        for (const cmd of commands) {
            executeRenderCommand(cmd as RenderCommand);
        }
    }

    // Update zoom display
    zoomLevelEl.textContent = `${Math.round(editor.zoom * 100)}%`;
    fullRendersEl.textContent = editor.fullRenderCount.toString();
    dirtyRendersEl.textContent = editor.dirtyRenderCount.toString();

    // If still needs redraw (e.g. for animations), request another frame
    if (editor.needsRedraw) {
        requestRender();
    }
}

/**
 * Execute a single render command
 */
function executeRenderCommand(cmd: RenderCommand) {
    if (!ctx || !canvas) return;
    
    switch (cmd.type || Object.keys(cmd)[0]) {
        case 'Clear':
            ctx.fillStyle = cmd.color as string || '#1e1e1e';
            ctx.fillRect(0, 0, canvas.width, canvas.height);
            break;

        case 'SetFont':
            ctx.font = `${FONT_SIZE * (cmd.scale as number || 1)}px 'JetBrains Mono', monospace`;
            ctx.textBaseline = 'top';
            break;

        case 'DrawChar':
            ctx.fillStyle = '#d4d4d4';
            ctx.fillText(cmd.char as string, cmd.x as number, cmd.y as number);
            break;

        case 'DrawRect':
            ctx.fillStyle = cmd.color as string || '#264f78';
            ctx.fillRect(cmd.x as number, cmd.y as number, cmd.width as number, cmd.height as number);
            break;

        case 'DrawGrid': {
            ctx.strokeStyle = cmd.color as string || '#333333';
            ctx.lineWidth = 0.5;
            ctx.beginPath();
            
            const panX = cmd.pan_x as number || 0;
            const panY = cmd.pan_y as number || 0;
            const cellW = cmd.cell_width as number || charWidth;
            const cellH = cmd.cell_height as number || lineHeight;
            
            // Vertical lines
            for (let x = panX % cellW; x < canvas.width; x += cellW) {
                ctx.moveTo(x, 0);
                ctx.lineTo(x, canvas.height);
            }
            
            // Horizontal lines
            for (let y = panY % cellH; y < canvas.height; y += cellH) {
                ctx.moveTo(0, y);
                ctx.lineTo(canvas.width, y);
            }
            
            ctx.stroke();
            break;
        }
    }
}

/**
 * Update cursor indicator position
 */
function updateCursorIndicator(gridX: number, gridY: number) {
    if (!editor || !cursorIndicator) return;
    const zoom = editor.zoom;
    const pan = editor.pan;

    const screenX = gridX * charWidth * zoom + pan[0];
    const screenY = gridY * lineHeight * zoom + pan[1];

    cursorIndicator.style.left = `${screenX}px`;
    cursorIndicator.style.top = `${screenY}px`;
    cursorIndicator.style.width = `${charWidth * zoom}px`;
    cursorIndicator.style.height = `${lineHeight * zoom}px`;
    cursorIndicator.classList.remove('hidden');
}

/**
 * Set the current tool
 */
function focusCanvasElement(): void {
    if (canvas) {
        canvas.focus();
        return;
    }
    const el = document.querySelector('#canvas');
    if (el instanceof HTMLCanvasElement) {
        el.focus();
    }
}

/** Wire click (+ mousedown preventDefault) for an optional button by id. */
function wireOptionalButton(id: string, onClick: () => void): void {
    const el = document.querySelector(`#${CSS.escape(id)}`);
    if (!(el instanceof HTMLButtonElement)) return;
    el.addEventListener('mousedown', (e) => {
        e.preventDefault();
    });
    el.addEventListener('click', onClick);
}

function setTool(toolName: string) {
    if (!editor) {
        logger.error('Editor not initialized');
        // Still focus canvas when present (unit tests / early UI)
        focusCanvasElement();
        return;
    }
    try {
        editor.setTool(toolName);
        updateToolButtons(toolName);

        // Module refs are initialized before tools are interactive.
        lineDirectionGroup.style.display =
            toolName.toLowerCase() === 'line' ? 'flex' : 'none';
        statusToolEl.textContent = `Tool: ${capitalize(toolName)}`;

        // Ensure canvas keeps focus for keyboard shortcuts
        focusCanvasElement();
    } catch (error) {
        logger.error('Failed to set tool:', error);
    }
}

/**
 * Update tool button states.
 * Uses live DOM queries so unit tests work before module refs are initialized.
 */
function updateToolButtons(activeTool: string) {
    const normalizedTool = activeTool.toLowerCase();

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

    // TOOL_INFO is a full Record for known tools; callers pass known tool ids.
    const info = TOOL_INFO[normalizedTool];
    const statusEl = document.querySelector('#status-message');
    if (statusEl instanceof HTMLElement) {
        statusEl.textContent = `[${info.shortcut}] ${info.instruction}`;
    }

    const container = document.querySelector('#canvas-container');
    if (container instanceof HTMLElement) {
        // Clear previous tool classes
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

/**
 * Update UI state
 */
function updateUI() {
    if (!editor) return;
    try {
        undoBtn.disabled = !editor.can_undo;
        redoBtn.disabled = !editor.can_redo;
        gridSizeEl.textContent = `${editor.width} × ${editor.height}`;
        statusToolEl.textContent = `Tool: ${capitalize(editor.tool)}`;
    } catch (error) {
        logger.error('Failed to update UI:', error);
    }
}

/**
 * Selection-aware copy to OS clipboard (CRLF + internal clipboard).
 */
async function copyToClipboard() {
    if (!editor) return;
    await copySelectionAware(editor, showToast);
}

/**
 * Apply custom grid dimensions from side-panel inputs.
 */
function applyCustomGridSize() {
    if (!editor) return;
    const gridWidthInput = document.querySelector('#grid-width');
    const gridHeightInput = document.querySelector('#grid-height');
    if (!(gridWidthInput instanceof HTMLInputElement) || !(gridHeightInput instanceof HTMLInputElement)) {
        return;
    }
    const w = Math.max(MIN_COLS, Math.min(400, parseInt(gridWidthInput.value, 10) || MIN_COLS));
    const h = Math.max(MIN_ROWS, Math.min(200, parseInt(gridHeightInput.value, 10) || MIN_ROWS));
    gridWidthInput.value = String(w);
    gridHeightInput.value = String(h);
    gridSizeLocked = true;
    if (editor.width !== w || editor.height !== h) {
        editor.resize(w, h);
        offscreenCanvas = null;
        offscreenCtx = null;
        requestRender();
        updateUI();
        syncGridInputs();
        scheduleAutoSave();
        showToast(`Grid: ${w} × ${h}`);
    } else {
        syncGridInputs();
    }
}

function syncGridInputs() {
    if (!editor) return;
    const gridWidthInput = document.querySelector('#grid-width');
    const gridHeightInput = document.querySelector('#grid-height');
    if (!(gridWidthInput instanceof HTMLInputElement) || !(gridHeightInput instanceof HTMLInputElement)) {
        return;
    }
    gridWidthInput.value = String(editor.width);
    gridHeightInput.value = String(editor.height);
}

function refreshLayerSelect() {
    if (!editor || typeof editor.layerCount !== 'number') return;
    const layerSelect = document.querySelector('#layer-select');
    if (!(layerSelect instanceof HTMLSelectElement)) return;
    const count = editor.layerCount;
    const active = editor.activeLayer ?? 0;
    while (layerSelect.firstChild) {
        layerSelect.removeChild(layerSelect.firstChild);
    }
    for (let i = 0; i < count; i++) {
        const opt = document.createElement('option');
        opt.value = String(i);
        opt.textContent = editor.layerName(i) || `Layer ${i + 1}`;
        if (i === active) opt.selected = true;
        layerSelect.appendChild(opt);
    }
}

/**
 * Show a toast message
 */
function showToast(message: string, isError = false) {
    statusToast.textContent = message;
    statusToast.classList.toggle('error', isError);
    statusToast.classList.remove('hidden');

    setTimeout(() => {
        statusToast.classList.add('hidden');
    }, 2000);
}

/**
 * Set zoom level with bounds (0.3x - 4x)
 */
function setZoom(zoom: number) {
    if (!editor) return;
    const clampedZoom = Math.max(0.3, Math.min(4.0, zoom));
    editor.setZoom(clampedZoom);
    editor.requestRedraw();
    requestRender();
    zoomLevelEl.textContent = `${Math.round(clampedZoom * 100)}%`;
}

/**
 * Reset zoom level to 100% and center the grid
 */
function resetZoom() {
    if (!editor || !canvasContainer) return;

    const containerRect = canvasContainer.getBoundingClientRect();
    const gridWidth = editor.width * charWidth;
    const gridHeight = editor.height * lineHeight;

    const panX = (containerRect.width - gridWidth) / 2;
    const panY = (containerRect.height - gridHeight) / 2;

    setZoom(1.0);
    editor.setPan(panX, panY);
    editor.requestRedraw();
    requestRender();
    showToast('Zoom reset to 100%');
}

/**
 * Fit zoom to show entire canvas
 */
function fitZoom() {
    if (!editor || !canvasContainer) return;

    const containerRect = canvasContainer.getBoundingClientRect();
    const gridWidth = editor.width * charWidth;
    const gridHeight = editor.height * lineHeight;

    const zoomX = containerRect.width / (gridWidth + 40);
    const zoomY = containerRect.height / (gridHeight + 40);
    const fitZoomLevel = Math.min(zoomX, zoomY, 1.0);

    const panX = (containerRect.width - gridWidth * fitZoomLevel) / 2;
    const panY = (containerRect.height - gridHeight * fitZoomLevel) / 2;

    setZoom(fitZoomLevel);
    editor.setPan(panX, panY);
    editor.requestRedraw();
    requestRender();
    showToast('Zoom fitted to view');
}

/**
 * Cycle to the next border style
 */
function cycleBorderStyle() {
    if (!editor) return;
    currentBorderStyleIndex = (currentBorderStyleIndex + 1) % BORDER_STYLES.length;
    const style = BORDER_STYLES[currentBorderStyleIndex];
    editor.setBorderStyle(style);
    borderStyleSelect.value = style;
    showToast(`Border: ${style}`);
}

// Modal accessibility state
let lastFocusedElement: HTMLElement | null = null;

/**
 * Show keyboard shortcuts modal
 */
function showShortcutsModal() {
    const modal = document.getElementById('shortcuts-modal');
    if (modal) {
        lastFocusedElement = document.activeElement as HTMLElement;
        modal.classList.remove('hidden');
        const closeBtn = modal.querySelector('.modal-close') as HTMLElement;
        if (closeBtn) {
            closeBtn.focus();
        }
    }
}

/**
 * Hide keyboard shortcuts modal
 */
function hideShortcutsModal() {
    const modal = document.getElementById('shortcuts-modal');
    if (modal) {
        modal.classList.add('hidden');
        if (lastFocusedElement) {
            lastFocusedElement.focus();
            lastFocusedElement = null;
        }
    }
}

/**
 * Rasterize all required glyphs and upload to WASM font atlas
 */
function uploadFontAtlas() {
    if (!editor) return;

    const canvas = document.createElement('canvas');
    canvas.width = GLYPH_WIDTH;
    canvas.height = GLYPH_HEIGHT;
    const ctx = canvas.getContext('2d', { alpha: true });
    if (!ctx) return;

    // Use a slightly smaller font size for rasterization to prevent clipping
    const RASTER_FONT_SIZE = 13;
    ctx.font = `${RASTER_FONT_SIZE}px 'JetBrains Mono', monospace`;
    ctx.textBaseline = 'top';
    ctx.fillStyle = 'white';

    const charsToRasterize = [];
    // Basic ASCII
    for (let i = 32; i < 127; i++) {
        charsToRasterize.push(String.fromCharCode(i));
    }
    // Custom symbols
    charsToRasterize.push('┌', '┐', '└', '┘', '─', '│', '╔', '╗', '╚', '╝', '═', '║', '┏', '┓', '┗', '┛', '━', '┃', '╭', '╮', '╰', '╯', '+', '-', '|', '*', '·', '•', '●', '▲', '▼', '◄', '►', '╱', '╲', '◆');

    for (const char of charsToRasterize) {
        ctx.clearRect(0, 0, GLYPH_WIDTH, GLYPH_HEIGHT);

        // Vertically center the character in the 20px high cell
        // Monospace fonts usually have some built-in padding
        ctx.fillText(char, 0, 2);

        const imageData = ctx.getImageData(0, 0, GLYPH_WIDTH, GLYPH_HEIGHT);
        const alphaData = new Uint8Array(GLYPH_WIDTH * GLYPH_HEIGHT);

        let hasContent = false;
        for (let i = 0; i < imageData.data.length; i += 4) {
            // Use alpha channel as mask
            const alpha = imageData.data[i + 3];
            alphaData[i / 4] = alpha;
            if (alpha > 0) hasContent = true;
        }

        if (!hasContent && char !== ' ') {
            logger.warn(`Glyph for '${char}' (${char.charCodeAt(0)}) rasterized empty. Font might not be loaded.`);
        }

        editor.updateFontAtlasGlyph(char.charCodeAt(0), alphaData);
    }

    logger.debug(`Rasterized and uploaded ${charsToRasterize.length} glyphs to WASM atlas`);
    editor.requestRedraw();
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
export { editor, canvas, ctx, charWidth, lineHeight, TOOL_INFO, setTool, updateToolButtons };
