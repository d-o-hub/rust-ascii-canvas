/** Browser adapter for the local draft session; no document schema knowledge. */
import { AsciiEditor } from './pkg/ascii_canvas.js';
import { DraftSession, DRAFTS_KEY, MAX_DRAFTS, type PreparedDocument } from './drafts.js';
import { DocumentHistory } from './history.js';
import { state } from './state.js';
import { BORDER_STYLES, FONT_SIZE } from './constants.js';
import { requestRender, uploadFontAtlas } from './render.js';
import { syncGridInputs, updateUI, showToast } from './ui.js';

let session: DraftSession | null = null;
let timeline: DocumentHistory | null = null;

function syncTimeline(restorePersisted = true): void {
    if (!session || !state.editor || typeof state.editor.serializeDocument !== 'function') {
        timeline = null;
        state.persistedCanUndo = false;
        state.persistedCanRedo = false;
        return;
    }
    timeline = new DocumentHistory(
        state.editor.serializeDocument(),
        restorePersisted ? session.selectedHistory : undefined,
    );
    state.persistedCanUndo = timeline.canUndo;
    state.persistedCanRedo = timeline.canRedo;
}

function prepare(json: string): PreparedDocument {
    const next = new AsciiEditor(40, 20);
    try {
        if (!next.loadDocument(json)) throw new Error('Invalid diagram file');
        next.setFontMetrics(state.charWidth, state.lineHeight, FONT_SIZE);
        next.setTheme(document.documentElement.getAttribute('data-theme') === 'light' ? 'Light' : 'Figma Dark');
        if (state.editor) {
            next.setTool(state.editor.tool.toLowerCase());
            next.setEraserSize(state.editor.eraserSize);
        }
        next.setBorderStyle(BORDER_STYLES[state.currentBorderStyleIndex]);
        next.setLineDirection(state.currentLineDirection);
        const canonical = next.serializeDocument();
        let committed = false;
        const previous = state.editor;
        return {
            json: canonical,
            commit: () => {
                state.editor = next;
                // A restored/replaced document owns its dimensions; viewport
                // changes must not silently crop it on the next layout pass.
                state.gridSizeLocked = true;
                state.pointerGestureActive = false;
                state.lastTouchDistance = null;
                state.offscreenCanvas = null;
                state.offscreenCtx = null;
                committed = true;
            },
            dispose: () => {
                if (!committed) { next.free(); return; }
                previous?.free();
                syncGridInputs();
                updateUI();
                requestRender();
                if (state.isInitialized) uploadFontAtlas();
            },
        };
    } catch (error) {
        next.free();
        throw error;
    }
}

function refresh(): void {
    if (!session) return;
    const status = document.getElementById('save-status');
    if (status) {
        status.textContent = session.status.message;
        status.dataset.state = session.status.kind;
    }
    const select = document.querySelector<HTMLSelectElement>('#draft-select');
    if (select) {
        select.replaceChildren(...session.list().map(draft => {
            const option = document.createElement('option');
            option.value = draft.id;
            option.textContent = draft.name;
            return option;
        }));
        select.value = session.selectedId;
    }
    const name = document.querySelector<HTMLInputElement>('#draft-name');
    if (name && document.activeElement !== name) {
        name.value = session.list().find(draft => draft.id === session?.selectedId)?.name ?? '';
    }
    const remove = document.querySelector<HTMLButtonElement>('#draft-delete');
    if (remove) {
        const canDelete = session.list().length > 1;
        remove.disabled = !canDelete;
        remove.title = canDelete ? 'Delete current draft' : 'Cannot delete the only remaining draft';
        remove.setAttribute('aria-label', canDelete ? 'Delete draft' : 'Cannot delete the only remaining draft');
    }
    const create = document.querySelector<HTMLButtonElement>('#draft-new');
    if (create) {
        const canCreate = session.list().length < MAX_DRAFTS;
        create.disabled = !canCreate;
        create.title = canCreate ? 'Create new draft' : `Maximum limit of ${MAX_DRAFTS} drafts reached`;
        create.setAttribute('aria-label', canCreate ? 'Create new draft' : `Create new draft (disabled: limit of ${MAX_DRAFTS} drafts reached)`);
    }
}

export function initializeDrafts(): void {
    const channel = typeof BroadcastChannel === 'function'
        ? new BroadcastChannel('ascii-canvas-drafts-sync-v1')
        : null;
    session = new DraftSession({
        getItem: key => localStorage.getItem(key),
        setItem: (key, value) => {
            localStorage.setItem(key, value);
            if (key === DRAFTS_KEY) channel?.postMessage({ type: 'shelf', raw: value });
        },
    }, () => {
        if (!state.editor) throw new Error('Editor unavailable');
        return state.editor.serializeDocument();
    }, prepare, refresh);
    session.initialize();
    syncTimeline();
    refresh();

    window.addEventListener('storage', event => {
        if (event.key === DRAFTS_KEY || event.key === null) {
            session?.onStorageChange(event.newValue);
            syncTimeline();
            updateUI();
            refresh();
        }
    });
    channel?.addEventListener('message', event => {
        const message: unknown = event.data;
        if (!message || typeof message !== 'object' || !('type' in message) || message.type !== 'shelf' ||
            !('raw' in message) || typeof message.raw !== 'string') return;
        session?.applyExternal(message.raw);
        syncTimeline();
        updateUI();
        refresh();
    });
    document.getElementById('draft-select')?.addEventListener('change', event => {
        if (event.target instanceof HTMLSelectElement) session?.switchTo(event.target.value);
        syncTimeline(false);
        updateUI();
        refresh(); // restore selection after a failed switch
    });
    const draftNameInput = document.querySelector<HTMLInputElement>('#draft-name');
    if (draftNameInput) {
        draftNameInput.addEventListener('change', () => {
            const activeDraft = session?.list().find(d => d.id === session?.selectedId);
            const oldName = activeDraft?.name ?? '';
            const newName = draftNameInput.value.trim();
            if (newName && newName !== oldName) {
                if (session?.rename(newName)) {
                    showToast('Draft renamed');
                }
            }
            draftNameInput.value = session?.list().find(d => d.id === session?.selectedId)?.name ?? '';
        });
        draftNameInput.addEventListener('keydown', (event: KeyboardEvent) => {
            if (event.key === 'Escape') {
                const activeDraft = session?.list().find(d => d.id === session?.selectedId);
                draftNameInput.value = activeDraft?.name ?? '';
                draftNameInput.blur();
                if (state.canvas) state.canvas.focus();
            } else if (event.key === 'Enter') {
                draftNameInput.blur();
                if (state.canvas) state.canvas.focus();
            }
        });
    }
    document.getElementById('draft-new')?.addEventListener('click', () => {
        if (!state.editor) return;
        const name = prompt('Name the new local draft', 'Untitled');
        if (name === null) return;
        const blank = new AsciiEditor(state.editor.width, state.editor.height);
        try {
            session?.create(name, blank.serializeDocument());
            syncTimeline(false);
            updateUI();
            showToast('Created new draft');
        } finally {
            blank.free();
        }
    });
    document.getElementById('draft-delete')?.addEventListener('click', () => {
        if (confirm('Delete this local draft? Download a backup first. This cannot be undone.')) {
            session?.deleteSelected();
            syncTimeline(false);
            updateUI();
            showToast('Draft deleted');
        }
    });
    document.getElementById('draft-save')?.addEventListener('click', () => {
        if (session?.save()) {
            showToast('Draft saved locally');
        }
    });
}

export function saveDraft(): boolean { return session?.save() ?? false; }
export function markDraftDirty(): void { session?.markDirty(); }

/** Discard the active timeline after an operation that explicitly clears history. */
export function resetDocumentHistory(): void {
    if (!session || !state.editor || typeof state.editor.serializeDocument !== 'function') return;
    const document = state.editor.serializeDocument();
    timeline = new DocumentHistory(document);
    state.persistedCanUndo = false;
    state.persistedCanRedo = false;
    session.setHistory(timeline.snapshot(), document);
}

/** Record the post-operation document in the selected draft's snapshot timeline. */
export function observeDocumentHistory(): void {
    if (!session || !state.editor || typeof state.editor.serializeDocument !== 'function') return;
    const document = state.editor.serializeDocument();
    if (!timeline) syncTimeline();
    if (timeline?.observe(document)) session.setHistory(timeline.snapshot(), document);
    if (timeline) {
        state.persistedCanUndo = timeline.canUndo;
        state.persistedCanRedo = timeline.canRedo;
    }
}

/** Apply a reload-restored undo/redo step when native Rust history is empty. */
export function applyPersistedHistory(action: 'undo' | 'redo'): boolean {
    if (!session || !state.editor || !timeline || typeof state.editor.loadDocument !== 'function' ||
        typeof state.editor.serializeDocument !== 'function') return false;
    const target = timeline.target(action);
    if (!target || !state.editor.loadDocument(target)) return false;
    const document = state.editor.serializeDocument();
    timeline.commit(document);
    session.setHistory(timeline.snapshot(), document);
    state.persistedCanUndo = timeline.canUndo;
    state.persistedCanRedo = timeline.canRedo;
    state.offscreenCanvas = null;
    state.offscreenCtx = null;
    syncGridInputs();
    return true;
}

export function importDraft(json: string, name: string): boolean {
    const imported = session?.create(name.replace(/\.(asc|json)$/i, ''), json) ?? false;
    if (imported) syncTimeline(false);
    return imported;
}
