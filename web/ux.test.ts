import { describe, it, expect, vi, beforeEach } from 'vitest';
import { TOOL_INFO, updateToolButtons } from './main';

describe('UX Improvements', () => {
    beforeEach(() => {
        // Mock DOM elements
        document.body.innerHTML = `
            <div id="canvas-container"></div>
            <div id="status-message"></div>
            <div id="status-tool"></div>
            <button class="tool-btn" data-tool="rectangle"></button>
            <button class="tool-btn" data-tool="text"></button>
            <button class="tool-btn" data-tool="select"></button>
        `;
    });

    it('should update status message with tool instruction', () => {
        updateToolButtons('rectangle');
        const statusMessage = document.getElementById('status-message');
        expect(statusMessage?.textContent).toBe(TOOL_INFO['rectangle'].instruction);
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
});
