import { describe, expect, it, vi } from 'vitest';
import { DraftSession, DRAFTS_KEY } from './drafts.js';
import { AUTOSAVE_KEY } from './constants.js';

function fixture(storage = new Map<string, string>()) {
    let document = 'blank';
    const io = {
        getItem: vi.fn((key: string) => storage.get(key) ?? null),
        setItem: vi.fn((key: string, value: string) => { storage.set(key, value); }),
    };
    const prepare = vi.fn((json: string) => {
        if (json === 'invalid') throw new Error('Invalid document');
        return { json, commit: () => { document = json; }, dispose: vi.fn() };
    });
    const session = new DraftSession(io, () => document, prepare);
    return { session, storage, io, prepare, edit: (value: string) => { document = value; }, content: () => document };
}

describe('local drafts: atomic replacement and fault recovery', () => {
    it('isolates A and B across switches and reload, restoring the selected draft', () => {
        const f = fixture();
        expect(f.session.initialize()).toBe(true);
        expect(f.session.rename('A')).toBe(true);
        const a = f.session.selectedId;
        f.edit('work A');
        expect(f.session.create('B', 'blank B')).toBe(true);
        const b = f.session.selectedId;
        f.edit('work B');
        expect(f.session.switchTo(a)).toBe(true);
        expect(f.content()).toBe('work A');
        const restored = fixture(f.storage);
        expect(restored.session.initialize()).toBe(true);
        expect(restored.content()).toBe('work A');
        expect(restored.session.switchTo(b)).toBe(true);
        expect(restored.content()).toBe('work B');
    });

    it.each(['save', 'switch', 'create', 'rename', 'delete'] as const)('a failed %s cannot replace current work or corrupt stored data', (operation) => {
        const f = fixture();
        f.session.initialize();
        const a = f.session.selectedId;
        f.session.create('B', 'blank B');
        f.edit('unsaved work');
        const stored = f.storage.get(DRAFTS_KEY);
        const selected = f.session.selectedId;
        f.io.setItem.mockImplementation(() => { throw new Error('QuotaExceededError'); });
        const operations = new Map([
            ['save', () => f.session.save()], ['switch', () => f.session.switchTo(a)],
            ['create', () => f.session.create('C', 'imported C')], ['rename', () => f.session.rename('Renamed')],
            ['delete', () => f.session.deleteSelected()],
        ]);
        expect(operations.get(operation)?.()).toBe(false);
        expect(f.content()).toBe('unsaved work');
        expect(f.session.selectedId).toBe(selected);
        expect(f.storage.get(DRAFTS_KEY)).toBe(stored);
        expect(f.session.status.kind).toBe('error');
    });

    it('invalid imports and failed serialization leave current content and shelf untouched', () => {
        const f = fixture();
        f.session.initialize();
        f.edit('working');
        const stored = f.storage.get(DRAFTS_KEY);
        expect(f.session.create('Bad', 'invalid')).toBe(false);
        expect(f.content()).toBe('working');
        expect(f.storage.get(DRAFTS_KEY)).toBe(stored);
        const session = new DraftSession(f.io, () => { throw new Error('serialization'); }, f.prepare);
        session.initialize();
        expect(session.create('Other', 'other')).toBe(false);
        expect(f.content()).toBe('blank'); // initialization restores the previously saved document
        expect(f.storage.get(DRAFTS_KEY)).toBe(stored);
    });

    it('failed legacy migration retains original bytes and restored in-memory work, and can retry', () => {
        const f = fixture(new Map([[AUTOSAVE_KEY, 'legacy work']]));
        f.io.setItem.mockImplementationOnce(() => { throw new Error('quota'); });
        expect(f.session.initialize()).toBe(false);
        expect(f.content()).toBe('legacy work');
        expect(f.storage.get(AUTOSAVE_KEY)).toBe('legacy work');
        expect(f.storage.has(DRAFTS_KEY)).toBe(false);
        expect(f.session.save()).toBe(true);
        expect(f.storage.get(AUTOSAVE_KEY)).toBe('legacy work');
    });

    it('a stale tab cannot overwrite a newer revision even without a storage event', () => {
        const a = fixture();
        a.session.initialize();
        const b = fixture(a.storage);
        b.session.initialize();
        a.edit('newer work');
        expect(a.session.save()).toBe(true);
        const newer = a.storage.get(DRAFTS_KEY);
        b.edit('stale work');
        expect(b.session.save()).toBe(false);
        expect(b.session.status.kind).toBe('conflict');
        expect(b.content()).toBe('stale work');
        expect(b.storage.get(DRAFTS_KEY)).toBe(newer);
        expect(b.session.create('Other', 'other')).toBe(false);
    });

    it('storage events immediately latch a conflict without replacing current work', () => {
        const f = fixture();
        f.session.initialize();
        f.edit('local edits');
        f.session.onStorageChange(null);
        expect(f.session.status.kind).toBe('conflict');
        expect(f.content()).toBe('local edits');
        expect(f.session.save()).toBe(false);
    });

    it('unchanged unload flushes do not generate conflicting revisions', () => {
        const f = fixture();
        f.session.initialize();
        const writes = f.io.setItem.mock.calls.length;
        expect(f.session.save()).toBe(true);
        expect(f.io.setItem).toHaveBeenCalledTimes(writes);
    });

    it('denied storage reads never replace work, including on later save attempts', () => {
        const f = fixture();
        f.edit('in memory');
        f.io.getItem.mockImplementation(() => { throw new Error('SecurityError'); });
        expect(f.session.initialize()).toBe(false);
        expect(f.session.save()).toBe(false);
        expect(f.content()).toBe('in memory');
        expect(f.io.setItem).not.toHaveBeenCalled();
        expect(f.session.status.message).toContain('download a .asc backup');
    });

    it('bounds the shelf without losing current work and validates every replacement first', () => {
        const f = fixture();
        f.session.initialize();
        for (let index = 1; index < 20; index++) expect(f.session.create(`Draft ${index}`, 'blank')).toBe(true);
        f.edit('unsaved last draft');
        const stored = f.storage.get(DRAFTS_KEY);
        expect(f.session.create('Overflow', 'new')).toBe(false);
        expect(f.session.list()).toHaveLength(20);
        expect(f.content()).toBe('unsaved last draft');
        expect(f.storage.get(DRAFTS_KEY)).toBe(stored);
        expect(f.session.save()).toBe(true);
    });

    it('rejects an oversized shelf before storage mutation', () => {
        const f = fixture();
        f.session.initialize();
        f.edit('X'.repeat(4_000_000));
        const stored = f.storage.get(DRAFTS_KEY);
        expect(f.session.save()).toBe(false);
        expect(f.content()).toHaveLength(4_000_000);
        expect(f.storage.get(DRAFTS_KEY)).toBe(stored);
    });

    it('unreadable or corrupt storage is never replaced by an empty document', () => {
        const f = fixture(new Map([[DRAFTS_KEY, '{bad json']]));
        expect(f.session.initialize()).toBe(false);
        expect(f.session.save()).toBe(false);
        expect(f.storage.get(DRAFTS_KEY)).toBe('{bad json');
        expect(f.content()).toBe('blank');
    });

    it('deletes only the selected draft and refuses deletion of the final draft', () => {
        const f = fixture();
        f.session.initialize();
        const a = f.session.selectedId;
        f.edit('A');
        f.session.create('B', 'B');
        expect(f.session.deleteSelected()).toBe(true);
        expect(f.session.selectedId).toBe(a);
        expect(f.content()).toBe('A');
        expect(f.session.list()).toHaveLength(1);
        expect(f.session.deleteSelected()).toBe(false);
    });
});
