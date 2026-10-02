#!/usr/bin/env python3
"""Conservative Rust layer-path sensor; not a substitute for the Rust compiler.

Tokenizes comments/literals away, then follows absolute and root-relative paths,
including nested use trees. Every file/read/tokenization error fails closed.
"""
from pathlib import Path
import re
import sys

FORBIDDEN = {'core': {'wasm', 'render', 'ui'},
             'utils': {'wasm', 'render', 'ui', 'core'},
             'render': {'wasm', 'ui'}, 'ui': {'wasm', 'render'}}
TOKEN = re.compile(r'[A-Za-z_][A-Za-z_0-9]*|::|[^\s]')


def code_only(text):
    """Erase comments and quoted literals, preserving offsets for diagnostics."""
    result = list(text)
    i = 0
    while i < len(text):
        start = i
        if text.startswith('//', i):
            end = text.find('\n', i)
            i = len(text) if end < 0 else end
        elif text.startswith('/*', i):
            i += 2
            depth = 1
            while i < len(text) and depth:
                if text.startswith('/*', i):
                    depth += 1
                    i += 2
                elif text.startswith('*/', i):
                    depth -= 1
                    i += 2
                else:
                    i += 1
            if depth:
                raise ValueError('unterminated block comment')
        else:
            raw = re.match(r'(?:br|r)(#*)"', text[i:]) if text[i] in 'br' else None
            if raw:
                marker = '"' + raw[1]
                end = text.find(marker, i + len(raw[0]))
                if end < 0:
                    raise ValueError('unterminated raw string')
                i = end + len(marker)
            elif text[i] == '"':
                i += 1
                while i < len(text) and text[i] != '"':
                    i += 2 if text[i] == '\\' else 1
                if i >= len(text):
                    raise ValueError('unterminated string')
                i += 1
            elif text[i] == "'" and (char := re.match(r"'(?:\\.|[^'\\])'", text[i:])):
                i += len(char[0])
            else:
                i += 1
                continue
        for index in range(start, i):
            if result[index] != '\n':
                result[index] = ' '
    return ''.join(result)


def root_members(tokens, index):
    """First segments in crate::{core::X, render::{A, B}} (not nested names)."""
    if index >= len(tokens):
        raise ValueError('incomplete crate path')
    if tokens[index] != '{':
        return {tokens[index]}
    members = set()
    depth = 1
    first = True
    for word in tokens[index + 1:]:
        if word == '}':
            depth -= 1
            if depth == 0:
                return members
        elif word == '{':
            depth += 1
        elif depth == 1 and word == ',':
            first = True
        elif depth == 1 and first:
            members.add(word)
            first = False
    raise ValueError('unbalanced crate use tree')


def violations(text, layer, module_depth):
    matches = list(TOKEN.finditer(code_only(text)))
    tokens = [match[0] for match in matches]
    for index, word in enumerate(tokens):
        bad = set()
        if layer == 'core' and word in {'web_sys', 'js_sys', 'wasm_bindgen'}:
            bad.add(word)
        if word == 'crate' and tokens[index + 1:index + 2] == ['::']:
            bad |= root_members(tokens, index + 2) & FORBIDDEN[layer]
        # A path reaching the crate root through super is equally a dependency.
        if word == 'super' and (index < 2 or tokens[index - 2] != 'super'):
            cursor = index
            count = 0
            while tokens[cursor:cursor + 2] == ['super', '::']:
                count += 1
                cursor += 2
            if count >= module_depth:
                bad |= root_members(tokens, cursor) & FORBIDDEN[layer]
        if bad:
            line = text.count('\n', 0, matches[index].start()) + 1
            yield line, ', '.join(sorted(bad))


def main():
    root = Path(__file__).resolve().parent.parent
    failures = []
    checked = 0
    try:
        for layer in FORBIDDEN:
            directory = root / 'src' / layer
            if not directory.is_dir():
                raise ValueError(f'missing source directory: {directory}')
            # walk with onerror avoids Path.rglob silently suppressing scan errors.
            import os
            def scan_error(error):
                raise error
            for current, _, names in os.walk(directory, onerror=scan_error):
                for name in sorted(names):
                    if not name.endswith('.rs'):
                        continue
                    path = Path(current) / name
                    text = path.read_text(encoding='utf-8')
                    checked += 1
                    depth = len(path.relative_to(root / 'src').parts) - (name == 'mod.rs')
                    for line, dependency in violations(text, layer, depth):
                        failures.append(f'{path.relative_to(root)}:{line}: {layer} -> {dependency}')
        if checked == 0:
            raise ValueError('no Rust source files scanned')
    except (OSError, ValueError) as error:
        print(f'[UNVERIFIED] Architecture scanner: {error}', file=sys.stderr)
        return 2
    if failures:
        print('[FAIL] Layer dependency violations:\n' + '\n'.join(failures))
        print('FIX: move shared types to core/utils; see agents-docs/architecture.md')
        return 1
    print(f'[PASS] Layer paths checked in {checked} Rust files')
    return 0


if __name__ == '__main__':
    sys.exit(main())
