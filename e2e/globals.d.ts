/**
 * Ambient types for the browser context the Playwright suite runs against.
 *
 * `web/main.ts` declares `Window.editor` as `AsciiEditorType | null`, but the
 * e2e suite is type-checked by its own tsconfig (`e2e/tsconfig.json`) and does
 * not compile `web/`, so it has no view of that declaration. Re-declaring the
 * surface the tests actually touch keeps `page.evaluate()` callbacks honest:
 * without it they needed `(window as any)` or a `@ts-ignore`, both of which hid
 * real type errors.
 */
export {};

declare global {
    interface Window {
        editor: {
            /**
             * Trimmed ASCII text of the composited visible layers.
             *
             * **The result has no grid origin.** Empty borders are trimmed
             * (`ExportOptions::default().trim_borders` in
             * `src/core/ascii_export.rs`), so a document whose content starts
             * at grid (10, 10) exports as if it started at (0, 0). Tests that
             * assert a cell position have to establish the origin themselves —
             * draw an anchor first — rather than index the export as if it were
             * the raw grid. Document save/load is unaffected: it goes through
             * `serializeDocument()`, which stores explicit per-cell coordinates.
             */
            exportAscii(): string;
            exportForCopy?: () => string;
            exportForCopyWithOptions?: (
                trimTrailingWhitespace: boolean,
                enforceBoundingBox: boolean,
                convertUnicodeToAscii: boolean,
            ) => string;
            clear(): void;
            serializeDocument(): string;
            loadDocument(json: string): boolean;
            exportPixelBuffer(): Uint8Array;
            readonly width: number;
            readonly height: number;
            readonly has_selection: boolean;
            readonly can_undo: boolean;
            readonly can_redo: boolean;
            selectAll(): void;
            setTool?(tool: string): void;
            getTool?(): string;
        } | null;
    }
}
