#!/usr/bin/env python3
"""Content provenance for generated bindings, shared locally and with CI artifacts.

--check never builds (CI download verification); --ensure rebuilds stale local
bindings. Build calls --print before compilation and pipes that digest into
--record afterwards, so an input changed during compilation cannot be stamped
as current.
"""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parent.parent
STAMP = ROOT / 'web/pkg/.build-fingerprint.json'
CONFIG = ('Cargo.toml', 'Cargo.lock', '.cargo/config.toml', 'rust-toolchain.toml',
          'mise.toml', 'package.json', 'pnpm-lock.yaml', 'scripts/build-wasm.sh',
          'scripts/wasm-freshness.py', 'scripts/check-artifact.mjs')
OUTPUTS = ('ascii_canvas.js', 'ascii_canvas.d.ts', 'ascii_canvas_bg.wasm',
           'ascii_canvas_bg.wasm.d.ts')


def input_digest():
    sources = sorted((ROOT / 'src').rglob('*.rs'))
    if not sources:
        raise ValueError('no Rust source inputs')
    digest = hashlib.sha256()
    for path in [*(ROOT / name for name in CONFIG), *sources]:
        digest.update(str(path.relative_to(ROOT)).encode())
        digest.update(b'\0')
        digest.update(path.read_bytes())
        digest.update(b'\0')
    return digest.hexdigest()


def output_digests():
    result = {}
    for name in OUTPUTS:
        data = (ROOT / 'web/pkg' / name).read_bytes()
        if not data:
            raise ValueError(f'empty output {name}')
        result[name] = hashlib.sha256(data).hexdigest()
    return result


def evidence():
    return {'inputs': input_digest(), 'outputs': output_digests()}


def check():
    if json.loads(STAMP.read_text()) != evidence():
        raise ValueError('bindings fingerprint does not match inputs/outputs')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument('--check', action='store_true')
    group.add_argument('--ensure', action='store_true')
    group.add_argument('--print', action='store_true', dest='print_digest')
    group.add_argument('--record', action='store_true')
    args = parser.parse_args()
    try:
        if args.print_digest:
            print(input_digest())
        elif args.record:
            current = evidence()
            if sys.stdin.read().strip() != current['inputs']:
                raise ValueError('build inputs changed during compilation; rebuild')
            STAMP.write_text(json.dumps(current, sort_keys=True) + '\n')
            print('[PASS] Recorded WASM input/output fingerprint')
        else:
            try:
                check()
            except (OSError, ValueError):
                if not args.ensure:
                    raise
                print('[INFO] WASM stale/missing; rebuilding before typechecks', flush=True)
                subprocess.run(['bash', 'scripts/build-wasm.sh'], cwd=ROOT, check=True)
                check()
            print('[PASS] WASM bindings match source/config/lockfile and generated outputs')
    except (OSError, ValueError, subprocess.CalledProcessError) as error:
        print(f'[UNVERIFIED] WASM freshness: {error}', file=sys.stderr)
        print('FIX: npm run build:wasm; do not typecheck stale generated APIs.', file=sys.stderr)
        return 2
    return 0


if __name__ == '__main__':
    sys.exit(main())
