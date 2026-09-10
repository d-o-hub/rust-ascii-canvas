import { describe, it, expect, vi } from 'vitest';
import { normalizeToCRLF } from './utils.js';
import { copyAsciiToClipboard, copyToClipboard } from './clipboard.js';
import type { AsciiEditor } from './types.js';

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

describe('copyAsciiToClipboard and copyToClipboard', () => {
    it('returns true on successful clipboard write', async () => {
        const writeTextMock = vi.fn().mockResolvedValue(undefined);
        vi.stubGlobal('navigator', {
            clipboard: {
                writeText: writeTextMock,
            },
        });

        const showToast = vi.fn();
        const success = await copyAsciiToClipboard('hello\nworld', showToast);

        expect(success).toBe(true);
        expect(writeTextMock).toHaveBeenCalledWith('hello\r\nworld');
        expect(showToast).toHaveBeenCalledWith('Copied — paste in a monospace editor');
    });

    it('returns false on failed clipboard write and displays error toast', async () => {
        const writeTextMock = vi.fn().mockRejectedValue(new Error('Permission denied'));
        vi.stubGlobal('navigator', {
            clipboard: {
                writeText: writeTextMock,
            },
        });

        const showToast = vi.fn();
        const success = await copyAsciiToClipboard('hello\nworld', showToast);

        expect(success).toBe(false);
        expect(writeTextMock).toHaveBeenCalledWith('hello\r\nworld');
        expect(showToast).toHaveBeenCalledWith('Failed to copy', true);
    });

    it('copyToClipboard delegates to copyAsciiToClipboard and handles selection', async () => {
        const writeTextMock = vi.fn().mockResolvedValue(undefined);
        vi.stubGlobal('navigator', {
            clipboard: {
                writeText: writeTextMock,
            },
        });

        const mockEditor = {
            copySelection: vi.fn(),
            exportForCopy: vi.fn().mockReturnValue('selected ascii'),
            exportAscii: vi.fn().mockReturnValue('all ascii'),
        } as unknown as AsciiEditor;

        const showToast = vi.fn();
        const success = await copyToClipboard(mockEditor, showToast);

        expect(success).toBe(true);
        expect(mockEditor.copySelection).toHaveBeenCalled();
        expect(mockEditor.exportForCopy).toHaveBeenCalled();
        expect(writeTextMock).toHaveBeenCalledWith('selected ascii');
        expect(showToast).toHaveBeenCalledWith('Copied — paste in a monospace editor');
    });
});
