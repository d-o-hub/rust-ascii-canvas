import { describe, expect, it } from 'vitest';
import { DocumentHistory, MAX_HISTORY_CHARS, MAX_HISTORY_ENTRIES } from './history.js';

describe('reload-safe document history', () => {
    it('tracks new snapshots and moves across undo/redo neighbors', () => {
        const history = new DocumentHistory('A');
        expect(history.observe('B')).toBe(true);
        expect(history.observe('C')).toBe(true);
        expect(history.target('undo')).toBe('B');
        history.commit('B');
        expect(history.target('undo')).toBe('A');
        expect(history.target('redo')).toBe('C');
        history.commit('A');
        expect(history.target('redo')).toBe('B');
    });

    it('drops redo snapshots after a new branch', () => {
        const history = new DocumentHistory('A');
        history.observe('B');
        history.commit('A');
        history.observe('C');
        expect(history.target('redo')).toBeNull();
        expect(history.target('undo')).toBe('A');
    });

    it('restores a valid persisted cursor and rejects malformed history', () => {
        const persisted = new DocumentHistory('A');
        persisted.observe('B');
        const restored = new DocumentHistory('B', persisted.snapshot());
        expect(restored.canUndo).toBe(true);
        expect(restored.target('undo')).toBe('A');

        const malformed = new DocumentHistory('current', {
            version: 1,
            entries: ['old', 3],
            cursor: 1,
        });
        expect(malformed.snapshot().entries).toEqual(['current']);
    });

    it('keeps the current snapshot within entry and character limits', () => {
        const history = new DocumentHistory('0');
        for (let index = 1; index < MAX_HISTORY_ENTRIES + 20; index++) {
            history.observe(`${index}-${'x'.repeat(Math.ceil(MAX_HISTORY_CHARS / 10))}`);
        }
        const snapshot = history.snapshot();
        expect(snapshot.entries.length).toBeLessThanOrEqual(MAX_HISTORY_ENTRIES);
        expect(snapshot.entries.at(snapshot.cursor)).toContain('x');
        expect(snapshot.entries.reduce((total, entry) => total + entry.length, 0)).toBeLessThanOrEqual(MAX_HISTORY_CHARS);
    });
});
