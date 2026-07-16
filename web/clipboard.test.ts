import { describe, it, expect } from 'vitest';
import { normalizeToCRLF } from './utils.js';

describe('normalizeToCRLF', () => {
    it('converts LF-only to CRLF', () => {
        expect(normalizeToCRLF('line1\nline2\nline3')).toBe('line1\r\nline2\r\nline3');
    });

    it('does not double-convert CRLF input', () => {
        expect(normalizeToCRLF('line1\r\nline2\r\nline3')).toBe('line1\r\nline2\r\nline3');
    });

    it('handles mixed line endings correctly', () => {
        expect(normalizeToCRLF('a\r\nb\nc\r\nd')).toBe('a\r\nb\r\nc\r\nd');
    });

    it('preserves empty string', () => {
        expect(normalizeToCRLF('')).toBe('');
    });

    it('preserves single-line content without adding CRLF', () => {
        expect(normalizeToCRLF('no newlines here')).toBe('no newlines here');
    });

    it('ASCII box art round-trips without distortion', () => {
        const box = '┌───┐\n│   │\n└───┘';
        const result = normalizeToCRLF(box);
        const lines = result.split('\r\n');
        expect(lines).toHaveLength(3);
        expect(lines[0]).toBe('┌───┐');
        expect(lines[1]).toBe('│   │');
        expect(lines[2]).toBe('└───┘');
    });
});
