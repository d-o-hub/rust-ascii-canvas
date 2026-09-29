import { defineConfig } from 'vitest/config';

export default defineConfig({
  root: '.',
  publicDir: false,
  
  build: {
    outDir: '../dist',
    emptyOutDir: true,
    target: 'esnext',
  },

  server: {
    port: 3003,
    // Fail rather than move ports. The e2e sensors (scripts/quality-gates.sh and
    // the CI e2e job) poll http://localhost:3003 and Playwright's baseURL is
    // fixed to it, so Vite silently starting on 3004 means the suite is run
    // against whatever stale server still owns 3003 — or against nothing.
    strictPort: true,
    cors: false,
    // Loopback only, and pinned to IPv4 deliberately.
    //
    // `host: true` binds 0.0.0.0, which puts the dev server on the LAN. That is
    // not a theoretical concern: three 2026 Vite advisories
    // (GHSA-v2wj-q39q-566r fs.deny bypass, GHSA-p9ff-h696-f583 arbitrary file
    // read via the HMR WebSocket, CVE-2026-53571 Windows alternate-path bypass)
    // all name "explicitly exposes the Vite dev server to the network" as a
    // precondition. Opt in for device testing with `pnpm run dev:lan`.
    //
    // '127.0.0.1' rather than 'localhost' (Vite's default): with 'localhost' the
    // server binds whichever address DNS returns first, which is ::1 on
    // IPv6-preferring hosts. The sensors poll and Playwright connects to
    // localhost, so that is a real way to serve a port nothing can reach
    // (vitejs/vite#16522, #18469, #7075). 127.0.0.1 is unambiguous.
    host: '127.0.0.1',
  },

  optimizeDeps: {
    exclude: ['./pkg/ascii_canvas.js'],
  },

  // Ensure WASM files are served correctly
  assetsInclude: ['**/*.wasm'],
  
  // Worker configuration for potential future use
  worker: {
    format: 'es',
  },

  test: {
    environment: 'happy-dom',
    include: ['**/*.test.ts'],
  },
});
