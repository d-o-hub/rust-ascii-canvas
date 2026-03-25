import { describe, it, expect } from 'vitest';

// Bug 2 — clipboard text must use CRLF on all platforms
function normalizeToCRLF(text: string): string {
  return text.replace(/\r\n/g, '\n').replace(/\n/g, '\r\n');
}

describe('copyToClipboard line ending normalization', () => {
  it('converts LF-only to CRLF', () => {
    const input = 'line1\nline2\nline3';
    const result = normalizeToCRLF(input);
    expect(result).toBe('line1\r\nline2\r\nline3');
  });

  it('does not double-convert CRLF input', () => {
    const input = 'line1\r\nline2\r\nline3';
    const result = normalizeToCRLF(input);
    expect(result).toBe('line1\r\nline2\r\nline3');
  });

  it('handles mixed line endings correctly', () => {
    const input = 'a\r\nb\nc\r\nd';
    const result = normalizeToCRLF(input);
    expect(result).toBe('a\r\nb\r\nc\r\nd');
  });

  it('preserves empty string', () => {
    expect(normalizeToCRLF('')).toBe('');
  });

  it('preserves single-line content without adding CRLF', () => {
    const input = 'no newlines here';
    expect(normalizeToCRLF(input)).toBe('no newlines here');
  });

  it('ASCII box art round-trips without distortion', () => {
    // Simulates a rectangle that would be broken by LF-only in Notepad
    const box = '┌───┐\n│   │\n└───┘';
    const result = normalizeToCRLF(box);
    const lines = result.split('\r\n');
    expect(lines).toHaveLength(3);
    expect(lines[0]).toBe('┌───┐');
    expect(lines[1]).toBe('│   │');
    expect(lines[2]).toBe('└───┘');
  });
});
