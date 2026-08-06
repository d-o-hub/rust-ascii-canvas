import { describe, it, expect, vi, beforeEach } from 'vitest';
import { TOOL_INFO, updateToolButtons, setTool } from './main';
import { state } from './state';
import { setupEventListeners } from './events';
import { refreshLayerList } from './ui';

describe('UX Improvements', () => {
    beforeEach(() => {
        // Mock DOM elements
        document.body.innerHTML = `
            <div id="canvas-container"></div>
            <div id="status-message"></div>
            <div id="status-tool"></div>
            <div id="line-direction-group" style="display: none;"></div>
            <div id="eraser-radius-group" style="display: none;">
                <select id="eraser-radius">
                    <option value="1">1 Cell</option>
                    <option value="3">3 Cells</option>
                    <option value="5">5 Cells</option>
                </select>
            </div>
            <button class="tool-btn" data-tool="rectangle"></button>
            <button class="tool-btn" data-tool="text"></button>
            <button class="tool-btn" data-tool="select"></button>
            <button class="tool-btn" data-tool="eraser"></button>
            <canvas id="canvas"></canvas>
            <input type="number" id="grid-width" value="100">
            <input type="number" id="grid-height" value="50">
            <button id="apply-grid-btn"></button>
            <div id="status-toast"></div>
        `;
    });

    it('should update status message with tool instruction and shortcut', () => {
        updateToolButtons('rectangle');
        const statusMessage = document.getElementById('status-message');
        const info = TOOL_INFO['rectangle'];
        expect(statusMessage?.textContent).toBe(`[${info.shortcut}] ${info.instruction}`);
    });

    it('should apply correct cursor class to canvas container', () => {
        const container = document.getElementById('canvas-container');

        updateToolButtons('text');
        expect(container?.classList.contains('tool-text')).toBe(true);
        expect(container?.classList.contains('tool-crosshair')).toBe(false);

        updateToolButtons('rectangle');
        expect(container?.classList.contains('tool-crosshair')).toBe(true);
        expect(container?.classList.contains('tool-text')).toBe(false);

        updateToolButtons('select');
        expect(container?.classList.contains('tool-select')).toBe(true);
        expect(container?.classList.contains('tool-crosshair')).toBe(false);

        updateToolButtons('eraser');
        expect(container?.classList.contains('tool-eraser')).toBe(true);
        expect(container?.classList.contains('tool-crosshair')).toBe(false);
    });

    it('should ensure panning class can be applied', () => {
        const container = document.getElementById('canvas-container');
        container?.classList.add('panning');
        expect(container?.classList.contains('panning')).toBe(true);
        // The CSS handles the !important override
    });

    it('should focus canvas when setting tool', () => {
        const canvasNode = document.querySelector('canvas');
        if (!canvasNode) {
            throw new Error('canvas element must exist in test DOM');
        }
        const focusSpy = vi.spyOn(canvasNode, 'focus');

        setTool('rectangle');
        expect(focusSpy).toHaveBeenCalled();
    });

    it('should sync tool button active states', () => {
        updateToolButtons('text');
        const textBtn = document.querySelector('[data-tool="text"]');
        const rectBtn = document.querySelector('[data-tool="rectangle"]');

        expect(textBtn?.classList.contains('active')).toBe(true);
        expect(textBtn?.getAttribute('aria-pressed')).toBe('true');
        expect(rectBtn?.classList.contains('active')).toBe(false);
        expect(rectBtn?.getAttribute('aria-pressed')).toBe('false');
    });

    it('should have instructions for all defined tools', () => {
        const tools = ['select', 'rectangle', 'line', 'arrow', 'diamond', 'text', 'freehand', 'eraser'];
        tools.forEach(tool => {
            expect(TOOL_INFO[tool]).toBeDefined();
            expect(TOOL_INFO[tool].instruction).toBeTruthy();
            expect(TOOL_INFO[tool].cursor).toBeTruthy();
        });
    });

    it('should toggle visibility of eraser radius group on tool change', () => {
        setTool('eraser');
        const eraserGroup = document.getElementById('eraser-radius-group');
        expect(eraserGroup?.style.display).toBe('flex');

        setTool('rectangle');
        expect(eraserGroup?.style.display).toBe('none');
    });

    it('should apply grid size and focus canvas when Enter is pressed in grid inputs', () => {
        const canvasNode = document.querySelector('canvas');
        const widthInput = document.querySelector('input#grid-width');
        if (!canvasNode || !widthInput) {
            throw new Error('canvas and grid-width elements must exist in test DOM');
        }

        const focusSpy = vi.spyOn(canvasNode, 'focus');

        state.canvas = canvasNode;
        state.statusToast = document.getElementById('status-toast');
        state.editor = {
            width: 80,
            height: 40,
            resize: vi.fn(),
            tool: 'rectangle',
        } as unknown as typeof state.editor;

        setupEventListeners();

        const enterEvent = new KeyboardEvent('keydown', { key: 'Enter', bubbles: true });
        widthInput.dispatchEvent(enterEvent);

        if (!state.editor) {
            throw new Error('state.editor must be defined');
        }
        expect(state.editor.resize).toHaveBeenCalledWith(100, 50);
        expect(focusSpy).toHaveBeenCalled();
    });

    it('should blur layer-name-input and focus canvas on Enter or Escape', () => {
        const canvasNode = document.querySelector('canvas');
        if (!canvasNode) throw new Error('canvas must exist');
        const focusSpy = vi.spyOn(canvasNode, 'focus');
        state.canvas = canvasNode;

        state.editor = {
            layerCount: 1,
            activeLayer: 0,
            layerName: vi.fn().mockReturnValue('Layer 1'),
            layerVisible: vi.fn().mockReturnValue(true),
            layerLocked: vi.fn().mockReturnValue(false),
            renameLayer: vi.fn(),
        } as unknown as typeof state.editor;

        document.body.innerHTML += '<div id="layer-list"></div>';

        refreshLayerList();

        const input = document.querySelector('.layer-name-input') as HTMLInputElement;
        expect(input).toBeTruthy();

        const blurSpy = vi.spyOn(input, 'blur');

        const enterEvent = new KeyboardEvent('keydown', { key: 'Enter', bubbles: true });
        input.dispatchEvent(enterEvent);

        expect(blurSpy).toHaveBeenCalled();
        expect(focusSpy).toHaveBeenCalled();

        blurSpy.mockClear();
        focusSpy.mockClear();

        const escapeEvent = new KeyboardEvent('keydown', { key: 'Escape', bubbles: true });
        input.dispatchEvent(escapeEvent);

        expect(blurSpy).toHaveBeenCalled();
        expect(focusSpy).toHaveBeenCalled();
    });

    it('should update mobile menu button accessibility attributes when clicked', () => {
        document.body.innerHTML += `
            <div id="side-panel"></div>
            <div id="drawer-overlay"></div>
            <button id="mobile-menu-btn"></button>
            <button id="close-drawer-btn"></button>
        `;

        const sidePanel = document.getElementById('side-panel');
        const menuBtn = document.getElementById('mobile-menu-btn');
        const closeBtn = document.getElementById('close-drawer-btn');

        expect(menuBtn).toBeTruthy();

        setupEventListeners();

        expect(menuBtn?.getAttribute('aria-expanded')).toBe('false');
        expect(menuBtn?.getAttribute('aria-controls')).toBe('side-panel');

        menuBtn?.dispatchEvent(new MouseEvent('click'));
        expect(sidePanel?.classList.contains('open')).toBe(true);
        expect(menuBtn?.getAttribute('aria-expanded')).toBe('true');

        closeBtn?.dispatchEvent(new MouseEvent('click'));
        expect(sidePanel?.classList.contains('open')).toBe(false);
        expect(menuBtn?.getAttribute('aria-expanded')).toBe('false');
    });
});
