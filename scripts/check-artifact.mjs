#!/usr/bin/env node
// Shared local/CI size check: absent, unreadable or invalid is not small/clean.
import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import console from 'node:console';
import { fileURLToPath } from 'node:url';

const { WebAssembly } = globalThis;
const here = path.dirname(fileURLToPath(import.meta.url));
const artifact = process.argv[2] || path.join(here, '../web/pkg/ascii_canvas_bg.wasm');
const limit = 1572864;
try {
  const stat = fs.statSync(artifact);
  if (!stat.isFile() || stat.size === 0) throw new Error('not a nonempty file');
  if (stat.size > limit) throw new Error(`WASM size ${stat.size} exceeds ${limit} bytes`);
  const bytes = fs.readFileSync(artifact);
  if (!WebAssembly.validate(bytes)) throw new Error('invalid WebAssembly module');
  console.log(`[PASS] Valid WASM: ${bytes.length} bytes (budget ${limit})`);
} catch (error) {
  console.error(`[FAIL] WASM artifact ${artifact}: ${error.message}`);
  console.error('FIX: build fresh bindings with npm run build:wasm and retry.');
  process.exitCode = 1;
}
