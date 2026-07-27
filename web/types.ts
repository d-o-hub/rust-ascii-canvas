/**
 * Shared TypeScript types for the ASCII Canvas editor.
 */

import { AsciiEditor } from './pkg/ascii_canvas.js';

export type { AsciiEditor };

export interface EventResult {
    needs_redraw: boolean;
    tool: string;
    can_undo: boolean;
    can_redo: boolean;
    should_copy: boolean;
    ascii: string | null;
}

export interface RenderCommand {
    type: string;
    [key: string]: unknown;
}
