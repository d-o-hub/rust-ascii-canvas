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
    pan: number[];
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
    export_ascii(): string;
    getRenderCommands(): RenderCommand[];
    requestRedraw(): void;
}

// Global state
let editor: AsciiEditorInterface | null = null;
let canvas: HTMLCanvasElement | null = null;
let ctx: CanvasRenderingContext2D | null = null;
let animationFrameId: number | null = null;
let isInitialized = false;

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

// Grid dimensions
const GRID_WIDTH = 80;
const GRID_HEIGHT = 40;

// Border styles for cycling
const BORDER_STYLES = ['single', 'double', 'heavy', 'rounded', 'ascii', 'dotted'];
let currentBorderStyleIndex = 0;

// Line direction options
const LINE_DIRECTIONS = ['auto', 'horizontal', 'vertical'];
let currentLineDirection = 'auto';

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
 * Initialize the editor
 */
async function initialize() {
    try {
        // Initialize WASM
        await init();

        // Initialize DOM elements
        loadingOverlay = getElement('loading');
        canvasContainer = getElement('canvas-container');
        cursorIndicator = getElement('cursor-indicator');
        gridSizeEl = getElement('grid-size');
        cursorPosEl = getElement('cursor-pos');
        zoomLevelEl = getElement('zoom-level');
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

        // Get canvas and context
        canvas = getElement<HTMLCanvasElement>('canvas');
        const ctxResult = canvas.getContext('2d', { alpha: false });
        if (!ctxResult) {
            throw new Error('Failed to get 2D context');
        }
        ctx = ctxResult;

        // Set up canvas size
        resizeCanvas();

        // Create editor
        editor = new AsciiEditor(GRID_WIDTH, GRID_HEIGHT);
        window.editor = editor;

        // Measure font metrics
        measureFont();

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
    if (!ctx || !editor) return;
    ctx.font = `${FONT_SIZE}px 'JetBrains Mono', monospace`;
    const metrics = ctx.measureText('M');
    charWidth = metrics.width;
    lineHeight = FONT_SIZE * 1.4;

    // Update editor with font metrics
    editor.setFontMetrics(charWidth, lineHeight, FONT_SIZE);
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
    if (editor) {
        measureFont();
        requestRender();
    }
}

/**
 * Set up all event listeners
 */
function setupEventListeners() {
    // Window resize
    window.addEventListener('resize', debounce(resizeCanvas, 100));

    // Canvas pointer events
    canvas.addEventListener('pointerdown', handlePointerDown);
    canvas.addEventListener('pointermove', handlePointerMove);
    canvas.addEventListener('pointerup', handlePointerUp);
    canvas.addEventListener('pointerleave', handlePointerLeave);

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
    const rect = canvas.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;

    // Update cursor position display
    const gridX = Math.floor((x - editor.pan[0]) / editor.zoom / charWidth);
    const gridY = Math.floor((y - editor.pan[1]) / editor.zoom / lineHeight);
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
    e.preventDefault();
    canvas.releasePointerCapture(e.pointerId);

    const rect = canvas.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;

    const result = editor.onPointerUp(x, y);
    handleEventResult(result);
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

    if (result.should_copy && result.ascii) {
        navigator.clipboard.writeText(result.ascii).then(() => {
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

    // Get render commands
    const commands = editor.getRenderCommands();

    // Execute render commands
    for (const cmd of commands) {
        executeRenderCommand(cmd as RenderCommand);
    }

    // Update zoom display
    zoomLevelEl.textContent = `${Math.round(editor.zoom * 100)}%`;
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
    const ascii = editor.export_ascii();
    try {
        await navigator.clipboard.writeText(ascii);
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
export { editor, canvas, ctx };
