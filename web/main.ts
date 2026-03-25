/**
 * ASCII Canvas Editor - Main TypeScript Entry Point
 *
 * Initializes the WASM module and sets up the editor UI.
 */

import init, { AsciiEditor } from './pkg/ascii_canvas.js';

// Type definitions
interface EventResult {
    needs_redraw: boolean;
    tool: string;
    can_undo: boolean;
    can_redo: boolean;
    should_copy: boolean;
    ascii: string | null;
}

interface RenderCommand {
    type: string;
    [key: string]: unknown;
}

// WASM AsciiEditor interface
interface AsciiEditorInterface {
    width: number;
    height: number;
    tool: string;
    zoom: number;
    pan: number[] | Float64Array;
    can_undo: boolean;
    can_redo: boolean;
    setTool(toolId: string): void;
    setBorderStyle(style: string): void;
    setLineDirection(direction: string): void;
    setZoom(zoom: number): void;
    setPan(x: number, y: number): void;
    setFontMetrics(charWidth: number, lineHeight: number, fontSize: number): void;
    onPointerDown(x: number, y: number): EventResult;
    onPointerMove(x: number, y: number): EventResult;
    onPointerUp(x: number, y: number): EventResult;
    onKeyDown(key: string, ctrl: boolean, shift: boolean): EventResult;
    onKeyUp(key: string): void;
    onWheel(delta: number, x: number, y: number): EventResult;
    undo(): boolean;
    redo(): boolean;
    clear(): void;
    exportAscii(): string;
    getRenderCommands(): RenderCommand[];
    getDirtyRenderCommands(): RenderCommand[];
    getPixelBufferPtr(): number;
    getPixelBufferLen(): number;
    renderToPixelBuffer(): void;
    updateFontAtlasGlyph(chCode: number, glyphData: Uint8Array): void;
    resize(width: number, height: number): void;
    requestRedraw(): void;
    clearDirtyState(): void;
    readonly needsRedraw: boolean;
    readonly fullRenderCount: number;
    readonly dirtyRenderCount: number;
}

// Global state
let editor: AsciiEditorInterface | null = null;
// eslint-disable-next-line @typescript-eslint/no-explicit-any
let wasmMemory: any = null;
let canvas: HTMLCanvasElement | null = null;
let ctx: CanvasRenderingContext2D | null = null;
let offscreenCanvas: HTMLCanvasElement | null = null;
let offscreenCtx: CanvasRenderingContext2D | null = null;
let animationFrameId: number | null = null;
// Track initialization state
let isInitialized = false;
void isInitialized; // Suppress unused variable warning

// Expose editor for testing
declare global {
    interface Window {
        editor: AsciiEditorInterface | null;
    }
}
window.editor = null;

// Font metrics
const FONT_SIZE = 14;
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

// Grid dimensions
const MIN_COLS = 40;
const MIN_ROWS = 20;

// Rendering configuration
const USE_PIXEL_BUFFER = true;
const GLYPH_WIDTH = 8;
const GLYPH_HEIGHT = 20;

// Border styles for cycling
const BORDER_STYLES = ['single', 'double', 'heavy', 'rounded', 'ascii', 'dotted'];
let currentBorderStyleIndex = 0;

// Line direction options
const LINE_DIRECTIONS = ['auto', 'horizontal', 'vertical'];
void LINE_DIRECTIONS; // Suppress unused - reserved for future UI
let currentLineDirection = 'auto';
void currentLineDirection; // Suppress unused - state tracked via DOM

/**
 * Get DOM element with null check
 */
function getElement<T extends HTMLElement>(id: string): T {
    const el = document.getElementById(id);
    if (!el) {
        throw new Error(`Required element not found: ${id}`);
    }
    return el as T;
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

        // Set up canvas size first to have correct container rect
        resizeCanvas();

        // Measure font metrics MUST run before computeGridDimensions()
        measureFont();

        // Create editor with responsive dimensions
        const { width, height } = computeGridDimensions();
        editor = new AsciiEditor(width, height);
        window.editor = editor;

        // Update editor with font metrics again after creation
        if (editor) {
            editor.setFontMetrics(charWidth, lineHeight, FONT_SIZE);
        }

        // Set up event listeners
        setupEventListeners();

        // Update UI
        updateUI();

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

        console.log('ASCII Canvas Editor initialized');
    } catch (error) {
        console.error('Failed to initialize:', error);
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
    (window as any).charWidth = charWidth;
    (window as any).lineHeight = lineHeight;
}

/**
 * Resize canvas to match container
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
        const { width, height } = computeGridDimensions();
        if (editor.width !== width || editor.height !== height) {
            editor.resize(width, height);
            offscreenCanvas = null; // force offscreen canvas recreation
            offscreenCtx = null;
            updateUI();
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
            const direction = btn.getAttribute('data-direction') as string;
            currentLineDirection = direction;
            
            // Update active state
            directionBtns.forEach(b => b.classList.remove('active'));
            btn.classList.add('active');
            
            // Call WASM to set line direction (if supported)
            if (editor && (editor as any).setLineDirection) {
                (editor as any).setLineDirection(direction);
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
    });

    redoBtn.addEventListener('mousedown', (e) => e.preventDefault());
    redoBtn.addEventListener('click', () => {
        if (!editor) return;
        editor.redo();
        requestRender();
        updateUI();
    });

    copyBtn.addEventListener('mousedown', (e) => e.preventDefault());
    copyBtn.addEventListener('click', copyToClipboard);

    clearBtn.addEventListener('mousedown', (e) => e.preventDefault());
    clearBtn.addEventListener('click', () => {
        if (!editor) return;
        if (confirm('Clear the canvas? This cannot be undone.')) {
            editor.clear();
            requestRender();
            updateUI();
            showToast('Canvas cleared');
        }
    });

    // Zoom buttons
    zoomFitBtn.addEventListener('mousedown', (e) => e.preventDefault());
    zoomFitBtn.addEventListener('click', () => {
        if (!editor) return;
        fitZoom();
    });

    zoomResetBtn.addEventListener('mousedown', (e) => e.preventDefault());
    zoomResetBtn.addEventListener('click', () => {
        if (!editor) return;
        setZoom(1.0);
    });

    zoomOutBtn.addEventListener('mousedown', (e) => e.preventDefault());
    zoomOutBtn.addEventListener('click', () => {
        if (!editor) return;
        setZoom(editor.zoom * 0.8);
    });

    zoomInBtn.addEventListener('mousedown', (e) => e.preventDefault());
    zoomInBtn.addEventListener('click', () => {
        if (!editor) return;
        setZoom(editor.zoom * 1.25);
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
    handleEventResult(result);
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
    handleEventResult(result);
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
        handleEventResult(result);
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
            handleEventResult(result);
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
    handleEventResult(result);
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
    if (ctrl && ['z', 'y', 'c'].includes(key.toLowerCase())) {
        e.preventDefault();
    }

    const result = editor.onKeyDown(key, ctrl, shift);
    handleEventResult(result);

    // Handle B key - cycle border styles
    if (key.toLowerCase() === 'b' && !ctrl && !shift) {
        cycleBorderStyle();
    }

    // Handle ? key - show keyboard shortcuts modal
    if (key === '?' || (key === '/' && shift)) {
        showShortcutsModal();
    }

    // Update active tool button
    if (result && result.tool) {
        updateToolButtons(result.tool);
    }
}

/**
 * Handle key up event
 */
function handleKeyUp(e: KeyboardEvent) {
    if (!editor) return;
    editor.onKeyUp(e.key);
}

/**
 * Handle event result from WASM
 */
function handleEventResult(result: EventResult) {
    if (result.needs_redraw) {
        requestRender();
    }

    // Handle mobile keyboard proxy
    if (editor && editor.tool === 'text') {
        if (document.activeElement !== mobileKeyboardProxy) {
            mobileKeyboardProxy.focus();
        }
    } else {
        if (document.activeElement === mobileKeyboardProxy) {
            mobileKeyboardProxy.blur();
        }
    }

    if (result.should_copy && result.ascii) {
        const normalized = result.ascii.replace(/\r\n/g, '\n').replace(/\n/g, '\r\n');
        navigator.clipboard.writeText(normalized).then(() => {
            showToast('Copied to clipboard!');
        }).catch(() => {
            showToast('Failed to copy', true);
        });
    }

    updateUI();
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

        case 'DrawGrid':
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
function setTool(toolName: string) {
    if (!editor) {
        console.error('Editor not initialized');
        return;
    }
    try {
        editor.setTool(toolName);
        updateToolButtons(toolName);
        
        // Show/hide line direction options
        if (toolName.toLowerCase() === 'line') {
            lineDirectionGroup.style.display = 'flex';
        } else {
            lineDirectionGroup.style.display = 'none';
        }
        
        statusToolEl.textContent = `Tool: ${capitalize(toolName)}`;
    } catch (error) {
        console.error('Failed to set tool:', error);
    }
}

/**
 * Update tool button states
 */
function updateToolButtons(activeTool: string) {
    toolButtons.forEach(btn => {
        const tool = btn.getAttribute('data-tool');
        if (tool?.toLowerCase() === activeTool.toLowerCase()) {
            btn.classList.add('active');
        } else {
            btn.classList.remove('active');
        }
    });
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
        console.error('Failed to update UI:', error);
    }
}

/**
 * Copy ASCII to clipboard
 */
async function copyToClipboard() {
    if (!editor) return;
    const ascii = editor.exportAscii();
    const normalized = ascii.replace(/\r\n/g, '\n').replace(/\n/g, '\r\n');
    try {
        await navigator.clipboard.writeText(normalized);
        showToast('Copied to clipboard!');
    } catch {
        showToast('Failed to copy', true);
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
 * Fit zoom to show entire canvas
 */
function fitZoom() {
    if (!editor || !canvasContainer) return;
    
    const containerRect = canvasContainer.getBoundingClientRect();
    const gridWidth = editor.width * charWidth;
    const gridHeight = editor.height * lineHeight;
    
    const zoomX = containerRect.width / (gridWidth + 40);
    const zoomY = containerRect.height / (gridHeight + 40);
    const fitZoom = Math.min(zoomX, zoomY, 1.0);
    
    setZoom(fitZoom);
    editor.setPan(0, 0);
    editor.requestRedraw();
    requestRender();
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

/**
 * Show keyboard shortcuts modal
 */
function showShortcutsModal() {
    const modal = document.getElementById('shortcuts-modal');
    if (modal) {
        modal.classList.remove('hidden');
    }
}

/**
 * Hide keyboard shortcuts modal
 */
function hideShortcutsModal() {
    const modal = document.getElementById('shortcuts-modal');
    if (modal) {
        modal.classList.add('hidden');
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
            console.warn(`Glyph for '${char}' (${char.charCodeAt(0)}) rasterized empty. Font might not be loaded.`);
        }

        editor.updateFontAtlasGlyph(char.charCodeAt(0), alphaData);
    }

    console.log(`Rasterized and uploaded ${charsToRasterize.length} glyphs to WASM atlas`);
    editor.requestRedraw();
}

/**
 * Capitalize first letter
 */
function capitalize(str: string): string {
    return str.charAt(0).toUpperCase() + str.slice(1);
}

/**
 * Debounce utility
 */
function debounce<T extends (...args: unknown[]) => unknown>(fn: T, delay: number): T {
    let timeout: ReturnType<typeof setTimeout>;
    return ((...args: Parameters<T>) => {
        clearTimeout(timeout);
        timeout = setTimeout(() => fn(...args), delay);
    }) as T;
}

// Initialize when DOM is ready
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', initialize);
} else {
    initialize();
}

// Export for testing
export { editor, canvas, ctx, charWidth, lineHeight };
