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
            exportAscii(): string;
            serializeDocument(): string;
            loadDocument(json: string): boolean;
            setTool?(tool: string): void;
            getTool?(): string;
        } | null;
    }
}
