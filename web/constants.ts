/**
 * Shared constants for the ASCII Canvas editor.
 */

export const FONT_SIZE = 14;
export const MIN_COLS = 40;
export const MIN_ROWS = 20;
export const USE_PIXEL_BUFFER = true;
export const GLYPH_WIDTH = 8;
export const GLYPH_HEIGHT = 20;

export const TOOL_INFO: Record<string, { instruction: string; cursor: string; shortcut: string }> = {
    select: { instruction: 'Click to select, drag selection to move, or Del to erase', cursor: 'default', shortcut: 'V' },
    rectangle: { instruction: 'Drag to draw a rectangle', cursor: 'crosshair', shortcut: 'R' },
    line: { instruction: 'Drag to draw a line', cursor: 'crosshair', shortcut: 'L' },
    arrow: { instruction: 'Drag to draw an arrow', cursor: 'crosshair', shortcut: 'A' },
    diamond: { instruction: 'Drag to draw a diamond', cursor: 'crosshair', shortcut: 'D' },
    text: { instruction: 'Click to place cursor, then type', cursor: 'text', shortcut: 'T' },
    freehand: { instruction: 'Drag to draw freely', cursor: 'crosshair', shortcut: 'F' },
    eraser: { instruction: 'Drag to erase areas', cursor: 'crosshair', shortcut: 'E' },
};

export const BORDER_STYLES = ['single', 'double', 'heavy', 'rounded', 'ascii', 'dotted'];

export const LINE_DIRECTIONS = ['auto', 'horizontal', 'vertical'];

/** localStorage key for auto-saved diagrams */
export const AUTOSAVE_KEY = 'ascii-canvas-autosave';
