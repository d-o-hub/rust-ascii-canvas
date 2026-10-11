/** Browser adapter for the local draft session; no document schema knowledge. */
import { AsciiEditor } from './pkg/ascii_canvas.js';
import { DraftSession, DRAFTS_KEY, MAX_DRAFTS, type PreparedDocument } from './drafts.js';
import { state } from './state.js';
import { BORDER_STYLES, FONT_SIZE } from './constants.js';
import { requestRender, uploadFontAtlas } from './render.js';
import { syncGridInputs, updateUI, showToast } from './ui.js';

let session: DraftSession | null = null;

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
    session = new DraftSession({
        getItem: key => localStorage.getItem(key),
        setItem: (key, value) => { localStorage.setItem(key, value); },
    }, () => {
        if (!state.editor) throw new Error('Editor unavailable');
        return state.editor.serializeDocument();
    }, prepare, refresh);
    session.initialize();
    refresh();

    window.addEventListener('storage', event => {
        if (event.key === DRAFTS_KEY || event.key === null) session?.onStorageChange(event.newValue);
    });
    document.getElementById('draft-select')?.addEventListener('change', event => {
        if (event.target instanceof HTMLSelectElement) session?.switchTo(event.target.value);
        refresh(); // restore selection after a failed switch
    });
    document.getElementById('draft-name')?.addEventListener('change', event => {
        if (event.target instanceof HTMLInputElement) {
            session?.rename(event.target.value);
            event.target.value = session?.list().find(draft => draft.id === session?.selectedId)?.name ?? '';
        }
    });
    document.getElementById('draft-new')?.addEventListener('click', () => {
        if (!state.editor) return;
        const name = prompt('Name the new local draft', 'Untitled');
        if (name === null) return;
        const blank = new AsciiEditor(state.editor.width, state.editor.height);
        try {
            session?.create(name, blank.serializeDocument());
            showToast('Created new draft');
        } finally {
            blank.free();
        }
    });
    document.getElementById('draft-delete')?.addEventListener('click', () => {
        if (confirm('Delete this local draft? Download a backup first. This cannot be undone.')) {
            session?.deleteSelected();
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
export function importDraft(json: string, name: string): boolean {
    return session?.create(name.replace(/\.(asc|json)$/i, ''), json) ?? false;
}
