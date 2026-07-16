/**
 * Small pure utilities used across the frontend.
 */

/** Normalize line endings to CRLF for cross-platform external paste (esp. Windows Notepad). */
export function normalizeToCRLF(text: string): string {
    return text.replace(/\r\n/g, '\n').replace(/\n/g, '\r\n');
}

/** Capitalize the first letter of a string. */
export function capitalize(str: string): string {
    return str.charAt(0).toUpperCase() + str.slice(1);
}

/** Debounce a function. */
export function debounce<T extends (...args: unknown[]) => unknown>(fn: T, delay: number): T {
    let timeout: ReturnType<typeof setTimeout>;
    return ((...args: Parameters<T>) => {
        clearTimeout(timeout);
        timeout = setTimeout(() => fn(...args), delay);
    }) as T;
}

/** Get a required DOM element by id. */
export function getElement<T extends HTMLElement>(id: string): T {
    const el = document.querySelector(`#${CSS.escape(id)}`);
    if (!(el instanceof HTMLElement)) {
        throw new Error(`Element #${id} not found`);
    }
    return el as T;
}

/** Get an optional DOM element by id (null when missing). */
export function getOptionalElement<T extends HTMLElement>(id: string): T | null {
    const el = document.querySelector(`#${CSS.escape(id)}`);
    return el instanceof HTMLElement ? (el as T) : null;
}
