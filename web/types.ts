/**
 * Shared TypeScript types for the ASCII Canvas editor.
 */

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

/** WASM AsciiEditor surface used by the TypeScript frontend. */
export interface AsciiEditorInterface {
    width: number;
    height: number;
    tool: string;
    zoom: number;
    pan: number[] | Float64Array;
    can_undo: boolean;
    can_redo: boolean;
    has_selection?: boolean;
    has_clipboard?: boolean;
    layerCount?: number;
    activeLayer?: number;
    setTool(toolId: string): void;
    setBorderStyle(style: string): void;
    setLineDirection(direction: string): void;
    setZoom(zoom: number): void;
    setPan(x: number, y: number): void;
    setFontMetrics(charWidth: number, lineHeight: number, fontSize: number): void;
    onPointerDown(x: number, y: number): EventResult | null;
    onPointerMove(x: number, y: number): EventResult | null;
    onPointerUp(x: number, y: number): EventResult | null;
    onKeyDown(key: string, ctrl: boolean, shift: boolean): EventResult | null;
    onKeyUp(key: string): void;
    onWheel(delta: number, x: number, y: number): EventResult | null;
    undo(): boolean;
    redo(): boolean;
    clear(): void;
    textCursorPosition(): number[] | null;
    selectAll(): void;
    exportAscii(): string;
    exportForCopy(): string;
    exportSvg(): string;
    serializeDocument(): string;
    loadDocument(json: string): boolean;
    copySelection(): boolean;
    paste(): boolean;
    layerName(index: number): string;
    layerVisible(index: number): boolean;
    setLayerVisible(index: number, visible: boolean): void;
    setActiveLayer(index: number): boolean;
    addLayer(): number;
    renameLayer(index: number, name: string): void;
    layerLocked(index: number): boolean;
    setLayerLocked(index: number, locked: boolean): void;
    deleteLayer(index: number): boolean;
    moveLayer(fromIndex: number, toIndex: number): void;
    mergeLayerDown(index: number): boolean;
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
