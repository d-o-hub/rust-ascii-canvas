import { describe, it, expect, vi, afterEach } from 'vitest';
import {
    copyAsciiToClipboard,
    formatClipboardText,
    writeTextToClipboard,
} from './clipboard.js';
import { normalizeToCRLF } from './utils.js';

const preserveWidth: Parameters<typeof formatClipboardText>[1] = {
    convertUnicodeToAscii: false,
    enforceBoundingBox: true,
};

afterEach(() => {
    vi.unstubAllGlobals();
});

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

    it('normalizes lone carriage returns to CRLF too', () => {
        expect(normalizeToCRLF('a\rb\n\r\nc')).toBe('a\r\nb\r\n\r\nc');
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

describe('clipboard formatting and fallback', () => {
    it('keeps spaced diagram rows rectangular', () => {
        const text = ' [foo] ---> [bar]  \n       |           |';
        const result = formatClipboardText(text, preserveWidth);
        const lines = result.split('\r\n');

        expect(lines).toHaveLength(2);
        expect(lines[0]).toHaveLength(lines[1].length);
        expect(lines[0].endsWith('  ')).toBe(true);
    });

    it('converts box drawing to pure ASCII without changing columns', () => {
        const result = formatClipboardText('┌──┐\n│  │\n└──┘', {
            convertUnicodeToAscii: true,
            enforceBoundingBox: true,
        });

        expect(result).toBe('+--+\r\n|  |\r\n+--+');
        expect([...result].every((char) => char === '\r' || char === '\n' || char.charCodeAt(0) < 128)).toBe(true);
    });

    it('trims trailing spaces only when rectangular preservation is disabled', () => {
        const result = formatClipboardText('A  \n B ', {
            convertUnicodeToAscii: false,
            enforceBoundingBox: false,
        });

        expect(result).toBe('A\r\n B');
    });

    it('uses execCommand when the Clipboard API is rejected', async () => {
        const writeText = vi.fn().mockRejectedValue(new Error('permission denied'));
        vi.stubGlobal('navigator', { clipboard: { writeText } });
        const execCommand = vi.fn().mockReturnValue(true);
        Object.defineProperty(document, 'execCommand', {
            configurable: true,
            value: execCommand,
        });

        await writeTextToClipboard('copy me');

        expect(writeText).toHaveBeenCalledWith('copy me');
        expect(execCommand).toHaveBeenCalledWith('copy');
    });

    it('reports copy success after the fallback path', async () => {
        vi.stubGlobal('navigator', { clipboard: undefined });
        const execCommand = vi.fn().mockReturnValue(true);
        Object.defineProperty(document, 'execCommand', {
            configurable: true,
            value: execCommand,
        });
        const toast = vi.fn();

        await copyAsciiToClipboard('┌─┐', toast, {
            convertUnicodeToAscii: true,
            enforceBoundingBox: true,
        });

        expect(execCommand).toHaveBeenCalledWith('copy');
        expect(toast).toHaveBeenCalledWith('Copied as pure ASCII');
    });
});
