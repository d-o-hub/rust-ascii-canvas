#!/usr/bin/env node
// Shared local/CI size check, run from the repo root: absent, unreadable or
// invalid is not small/clean. The artifact path is a fixed literal so static
// scanners can prove there is no traversal input; callers never pass paths.
import fs from 'node:fs';
import process from 'node:process';
import console from 'node:console';

const { WebAssembly } = globalThis;
const ARTIFACT = './web/pkg/ascii_canvas_bg.wasm';
const limit = 1572864;
try {
  const stat = fs.statSync('./web/pkg/ascii_canvas_bg.wasm');
  if (!stat.isFile() || stat.size === 0) throw new Error('not a nonempty file');
  if (stat.size > limit) throw new Error(`WASM size ${stat.size} exceeds ${limit} bytes`);
  const bytes = fs.readFileSync('./web/pkg/ascii_canvas_bg.wasm');
  if (!WebAssembly.validate(bytes)) throw new Error('invalid WebAssembly module');
  console.log(`[PASS] Valid WASM: ${bytes.length} bytes (budget ${limit})`);
} catch (error) {
  console.error(`[FAIL] WASM artifact ${ARTIFACT}: ${error.message}`);
  console.error('FIX: run from the repo root; build fresh bindings with npm run build:wasm.');
  process.exitCode = 1;
}
