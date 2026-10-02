#!/usr/bin/env node
// Inspect the actual loaded config: no browser/server is started by fixtures.
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import process from 'node:process';
import { fileURLToPath, URL } from 'node:url';
import test from 'node:test';

const root = fileURLToPath(new URL('../', import.meta.url));
function config(extra) {
  const result = spawnSync(process.execPath, ['--input-type=module', '-e',
    "import('./playwright.config.ts').then(({default:c}) => console.log(JSON.stringify({server:c.webServer,url:c.use.baseURL})))"], {
    cwd: root,
    env: { ...process.env, CI: '', BASE_URL: '', PRODUCTION_E2E: '', ...extra },
    encoding: 'utf8',
  });
  assert.equal(result.status, 0, result.stderr);
  return JSON.parse(result.stdout);
}

test('focused local runs retain dev server', () => {
  const { server, url } = config({});
  assert.equal(server.command, 'pnpm run dev');
  assert.equal(server.reuseExistingServer, true);
  assert.equal(url, 'http://127.0.0.1:3003');
});

test('CI never reuses a stranger server', () => {
  assert.equal(config({ CI: 'true' }).server.reuseExistingServer, false);
});

test('production runs preview built dist on a strict isolated port', () => {
  const { server, url } = config({ PRODUCTION_E2E: '1' });
  assert.equal(server.command, 'pnpm run preview --host 127.0.0.1 --port 4173 --strictPort');
  assert.equal(server.reuseExistingServer, false);
  assert.equal(url, 'http://127.0.0.1:4173');
});

test('BASE_URL keeps deployment shadow runs server-free', () => {
  const { server, url } = config({ BASE_URL: 'https://example.com', PRODUCTION_E2E: '1' });
  assert.equal(server, undefined);
  assert.equal(url, 'https://example.com');
});
