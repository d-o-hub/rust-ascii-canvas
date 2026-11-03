/**
 * Browser-owned document history for reload-safe undo/redo.
 *
 * The Rust history contains trait-object commands and intentionally is not part
 * of the `.asc` interchange format. This timeline stores validated document
 * snapshots instead, then asks the WASM editor to load a selected snapshot.
 */

export const HISTORY_VERSION = 1 as const;
export const MAX_HISTORY_ENTRIES = 100;
export const MAX_HISTORY_CHARS = 750_000;

export interface PersistedHistory {
    version: typeof HISTORY_VERSION;
    entries: string[];
    cursor: number;
}

function isPersistedHistory(value: unknown): value is PersistedHistory {
    if (!value || typeof value !== 'object' || !('version' in value) || value.version !== HISTORY_VERSION ||
        !('entries' in value) || !Array.isArray(value.entries) || value.entries.length === 0 ||
        value.entries.length > MAX_HISTORY_ENTRIES || value.entries.some(entry => typeof entry !== 'string') ||
        !('cursor' in value) || typeof value.cursor !== 'number' || !Number.isInteger(value.cursor) ||
        value.cursor < 0 || value.cursor >= value.entries.length) {
        return false;
    }
    return true;
}

function characterCount(entries: readonly string[]): number {
    return entries.reduce((total, entry) => total + entry.length, 0);
}

/** A bounded snapshot timeline with an explicit current cursor. */
export class DocumentHistory {
    private entries: string[];
    private cursor: number;

    constructor(current: string, persisted?: unknown) {
        if (isPersistedHistory(persisted)) {
            this.entries = [...persisted.entries];
            this.cursor = persisted.cursor;
            this.ensureCurrent(current);
        } else {
            this.entries = [current];
            this.cursor = 0;
        }
        this.trim();
    }

    /** Observe a document after a committed editor operation. */
    observe(current: string): boolean {
        if (current === this.entries[this.cursor]) return false;

        if (this.cursor > 0 && current === this.entries[this.cursor - 1]) {
            this.cursor -= 1;
            return true;
        }
        if (this.cursor + 1 < this.entries.length && current === this.entries[this.cursor + 1]) {
            this.cursor += 1;
            return true;
        }

        this.entries = this.entries.slice(0, this.cursor + 1);
        this.entries.push(current);
        this.cursor = this.entries.length - 1;
        this.trim();
        return true;
    }

    /** Return the next persisted document without moving the cursor. */
    target(action: 'undo' | 'redo'): string | null {
        const next = action === 'undo' ? this.cursor - 1 : this.cursor + 1;
        return this.entries[next] ?? null;
    }

    /** Move to a target after the caller has loaded it successfully. */
    commit(current: string): boolean {
        return this.observe(current);
    }

    snapshot(): PersistedHistory {
        return {
            version: HISTORY_VERSION,
            entries: [...this.entries],
            cursor: this.cursor,
        };
    }

    get canUndo(): boolean { return this.cursor > 0; }
    get canRedo(): boolean { return this.cursor + 1 < this.entries.length; }

    private ensureCurrent(current: string): void {
        if (this.entries[this.cursor] === current) return;
        const existing = this.entries.indexOf(current);
        if (existing >= 0) {
            this.cursor = existing;
            return;
        }
        this.entries = this.entries.slice(0, this.cursor + 1);
        this.entries.push(current);
        this.cursor = this.entries.length - 1;
    }

    private trim(): void {
        while (this.entries.length > MAX_HISTORY_ENTRIES ||
            (characterCount(this.entries) > MAX_HISTORY_CHARS && this.entries.length > 1)) {
            if (this.cursor > 0) {
                this.entries.shift();
                this.cursor -= 1;
            } else {
                this.entries.pop();
            }
        }
    }
}
