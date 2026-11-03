/** Bounded browser-local shelf. Metadata deliberately lives outside .asc v1.
 * One atomic storage write commits document + selection together. Replacements
 * are prepared off-screen before writing, and only committed after success.
 */
import { AUTOSAVE_KEY } from './constants.js';
import { DocumentHistory, type PersistedHistory } from './history.js';

export const DRAFTS_KEY = 'ascii-canvas-drafts-v1';
export const MAX_DRAFTS = 20;
const MAX_SHELF_CHARS = 4_000_000;

interface Draft {
    id: string;
    name: string;
    document: string;
    history: PersistedHistory;
}
interface Shelf {
    version: 1;
    revision: string;
    selected: string;
    drafts: Draft[];
}
export interface PreparedDocument {
    json: string;
    commit: () => void;
    dispose: () => void;
}
export interface SaveStatus {
    kind: 'saved' | 'dirty' | 'error' | 'conflict';
    message: string;
}
type StorageIO = Pick<Storage, 'getItem' | 'setItem'>;

function parseShelf(raw: string): Shelf {
    if (raw.length > MAX_SHELF_CHARS) throw new Error('Draft shelf exceeds the local limit');
    const value: unknown = JSON.parse(raw);
    if (!value || typeof value !== 'object' || !('version' in value) || value.version !== 1 ||
        !('revision' in value) || typeof value.revision !== 'string' || !value.revision ||
        !('selected' in value) || typeof value.selected !== 'string' ||
        !('drafts' in value) || !Array.isArray(value.drafts) ||
        value.drafts.length === 0 || value.drafts.length > MAX_DRAFTS) {
        throw new Error('Invalid local draft shelf');
    }
    const ids = new Set<string>();
    const drafts: Draft[] = value.drafts.map((draft: unknown) => {
        if (!draft || typeof draft !== 'object' || !('id' in draft) || typeof draft.id !== 'string' ||
            !draft.id || ids.has(draft.id) || !('name' in draft) || typeof draft.name !== 'string' ||
            !('document' in draft) || typeof draft.document !== 'string') {
            throw new Error('Invalid local draft');
        }
        ids.add(draft.id);
        return {
            id: draft.id,
            name: draft.name,
            document: draft.document,
            history: new DocumentHistory(draft.document, 'history' in draft ? draft.history : undefined).snapshot(),
        };
    });
    if (!ids.has(value.selected)) throw new Error('Selected draft is missing');
    return { version: 1, revision: value.revision, selected: value.selected, drafts };
}

export class DraftSession {
    private shelf: Shelf | null = null;
    private revision: string | null = null;
    private blocked = false;
    private historyDirty = false;
    status: SaveStatus = { kind: 'dirty', message: 'Not saved yet' };

    constructor(
        private readonly storage: StorageIO,
        private readonly serialize: () => string,
        private readonly prepare: (json: string) => PreparedDocument,
        private readonly changed: () => void = () => {},
    ) {}

    get selectedId(): string { return this.shelf?.selected ?? ''; }
    get selectedHistory(): PersistedHistory | undefined {
        return this.shelf?.drafts.find(draft => draft.id === this.shelf?.selected)?.history;
    }
    list(): ReadonlyArray<{ id: string; name: string }> {
        return this.shelf?.drafts.map(({ id, name }) => ({ id, name })) ?? [];
    }

    initialize(): boolean {
        try {
            const raw = this.storage.getItem(DRAFTS_KEY);
            if (raw !== null) {
                const shelf = parseShelf(raw);
                const selected = shelf.drafts.find(draft => draft.id === shelf.selected);
                if (!selected) throw new Error('Selected draft is missing');
                const prepared = this.prepare(selected.document);
                try { prepared.commit(); } finally { prepared.dispose(); }
                this.shelf = shelf;
                this.revision = shelf.revision;
                this.report('saved', 'Saved locally');
                return true;
            }
            // Never delete/overwrite the legacy key, including after success.
            const legacy = this.storage.getItem(AUTOSAVE_KEY);
            let json = this.serialize();
            if (legacy) {
                const prepared = this.prepare(legacy);
                try { json = prepared.json; prepared.commit(); } finally { prepared.dispose(); }
            }
            const id = crypto.randomUUID();
            this.shelf = { version: 1, revision: '', selected: id,
                drafts: [{ id, name: legacy ? 'Recovered diagram' : 'Untitled', document: json,
                    history: new DocumentHistory(json).snapshot() }] };
            return this.save();
        } catch (error) {
            // Unknown storage must not be overwritten by an empty editor on the
            // next timer. Reload is the explicit retry after recovering a file.
            this.blocked = true;
            return this.failed(error);
        }
    }

    markDirty(): void {
        if (this.status.kind === 'conflict' || this.status.kind === 'error') return;
        this.report('dirty', 'Unsaved changes');
    }

    save(): boolean { return this.change(() => {}); }

    /** Replace the selected draft's persisted snapshot timeline in memory. */
    setHistory(history: PersistedHistory, document = this.serialize()): void {
        const current = this.shelf?.drafts.find(draft => draft.id === this.selectedId);
        if (!current) return;
        current.document = document;
        current.history = new DocumentHistory(document, history).snapshot();
        this.historyDirty = true;
        this.report('dirty', 'Unsaved changes');
    }

    rename(name: string): boolean {
        return this.change(shelf => {
            const current = shelf.drafts.find(draft => draft.id === shelf.selected);
            if (current) current.name = this.name(name);
        });
    }

    create(name: string, json: string): boolean {
        return this.change(shelf => {
            if (shelf.drafts.length >= MAX_DRAFTS) throw new Error(`Limit of ${MAX_DRAFTS} local drafts reached`);
            const id = crypto.randomUUID();
            shelf.drafts.push({ id, name: this.name(name), document: json,
                history: new DocumentHistory(json).snapshot() });
            shelf.selected = id;
        }, true);
    }

    switchTo(id: string): boolean {
        if (id === this.selectedId) return this.save();
        return this.change(shelf => {
            if (!shelf.drafts.some(draft => draft.id === id)) throw new Error('Draft is missing');
            shelf.selected = id;
        }, true);
    }

    deleteSelected(): boolean {
        return this.change(shelf => {
            if (shelf.drafts.length <= 1) throw new Error('Keep at least one local draft');
            shelf.drafts = shelf.drafts.filter(draft => draft.id !== shelf.selected);
            const next = shelf.drafts.at(0);
            if (next) shelf.selected = next.id;
        }, true);
    }

    /**
     * Apply a complete shelf revision received from another same-origin tab.
     * Clean tabs converge; dirty tabs keep their work and enter conflict state.
     */
    applyExternal(raw: string | null): boolean {
        try {
            if (raw === null) throw new Error('Remote shelf was removed');
            const shelf = parseShelf(raw);
            if (shelf.revision === this.revision) return true;
            const selected = this.shelf?.drafts.find(draft => draft.id === this.selectedId);
            const localDocumentChanged = selected !== undefined && this.serialize() !== selected.document;
            if (this.blocked || localDocumentChanged || this.status.kind === 'dirty' ||
                this.status.kind === 'error' || this.status.kind === 'conflict') {
                this.conflict();
                return false;
            }
            const remoteSelected = shelf.drafts.find(draft => draft.id === shelf.selected);
            if (!remoteSelected) throw new Error('Remote selected draft is missing');
            const prepared = this.prepare(remoteSelected.document);
            try {
                prepared.commit();
                this.shelf = shelf;
                this.revision = shelf.revision;
                this.historyDirty = false;
                this.report('saved', 'Updated from another tab');
                return true;
            } finally {
                prepared.dispose();
            }
        } catch {
            this.conflict();
            return false;
        }
    }

    /** A conservative fallback for browsers without BroadcastChannel. */
    onStorageChange(raw: string | null): void {
        try {
            if ((raw === null ? null : parseShelf(raw).revision) === this.revision) return;
        } catch {
            this.conflict();
            return;
        }
        this.applyExternal(raw);
    }

    private change(update: (shelf: Shelf) => void, replace = false): boolean {
        if (this.blocked || !this.shelf) return false;
        let prepared: PreparedDocument | undefined;
        try {
            const shelf: Shelf = { ...this.shelf, drafts: this.shelf.drafts.map(draft => ({ ...draft })) };
            const current = shelf.drafts.find(draft => draft.id === shelf.selected);
            if (!current) throw new Error('Current draft is missing');
            current.document = this.serialize(); // before any replacement or storage mutation
            update(shelf);
            if (replace) {
                const next = shelf.drafts.find(draft => draft.id === shelf.selected);
                if (!next) throw new Error('Next draft is missing');
                prepared = this.prepare(next.document); // validate without mutating live editor
                next.document = prepared.json;
            }
            // Check immediately before setItem, even when storage events have
            // not yet arrived. A token avoids revision-number reuse after reset.
            const stored = this.storage.getItem(DRAFTS_KEY);
            let storedRevision: string | null;
            try {
                storedRevision = stored === null ? null : parseShelf(stored).revision;
            } catch {
                // A changed/corrupt shelf is an external writer boundary, not
                // a reason to overwrite it with this tab's in-memory work.
                this.conflict();
                return false;
            }
            if (storedRevision !== this.revision) {
                this.conflict();
                return false;
            }
            // Blur/pagehide can flush an already-saved document. Do not create
            // a spurious revision that would unnecessarily conflict other tabs.
            if (this.revision !== null && !this.historyDirty && JSON.stringify(shelf) === JSON.stringify(this.shelf)) {
                this.report('saved', 'Saved locally');
                return true;
            }
            shelf.revision = crypto.randomUUID();
            const raw = JSON.stringify(shelf);
            if (raw.length > MAX_SHELF_CHARS) throw new Error('Local drafts exceed the 4-million-character limit');
            this.storage.setItem(DRAFTS_KEY, raw);
            this.shelf = shelf;
            this.revision = shelf.revision;
            this.historyDirty = false;
            prepared?.commit(); // commit is a prevalidated, non-throwing editor swap
            this.report('saved', 'Saved locally');
            return true;
        } catch (error) {
            return this.failed(error);
        } finally { prepared?.dispose(); }
    }

    private name(name: string): string { return name.trim().slice(0, 80) || 'Untitled'; }
    private conflict(): void {
        this.blocked = true;
        this.report('conflict', 'Save conflict: another tab changed local drafts. Download your work, then reload. Local saving is paused.');
    }
    private failed(error: unknown): false {
        const reason = error instanceof Error ? error.message : 'Browser storage unavailable';
        this.report('error', `Save failed: ${reason}. Your work stays open; download a .asc backup.`);
        return false;
    }
    private report(kind: SaveStatus['kind'], message: string): void {
        this.status = { kind, message };
        this.changed();
    }
}
