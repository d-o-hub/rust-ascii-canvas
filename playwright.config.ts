import { defineConfig, devices } from '@playwright/test';

// `BASE_URL` points the suite at an already-running deployment — a Netlify
// Deploy Preview during the shadow stage of ADR-044, or any external host. In
// that case there is nothing to start locally, so `webServer` is omitted
// entirely: booting vite would spawn a server the run never uses, and (with
// `strictPort`, see web/vite.config.ts) would then fail the run on a port it
// does not need.
const externalBaseUrl = process.env.BASE_URL;
// Full/CI runs serve the built dist, while focused local runs retain Vite dev.
// BASE_URL still bypasses both servers for deployment-preview/shadow checks.
const production = process.env.PRODUCTION_E2E === '1';
const localBaseUrl = production ? 'http://127.0.0.1:4173' : 'http://127.0.0.1:3003';

export default defineConfig({
  testDir: './e2e',
  fullyParallel: !process.env.CI,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 1 : 0,
  workers: process.env.CI ? 1 : '50%',
  reporter: 'list',

  // Single source of truth for "how does the suite get a server".
  //
  // This replaces two hand-rolled copies of the same boot sequence — a
  // `nohup pnpm run dev` + `curl` poll in the CI e2e job, and a second one in
  // scripts/quality-gates.sh — which had already drifted. The CI copy had a real
  // defect: its `for i in {1..60}` readiness loop `break`s when ready but does
  // not `exit 1` when it never is, so a dev server that failed to boot produced
  // a three-minute Playwright timeout that says nothing about the cause.
  // `webServer` fails at the point of failure, and tears down after itself.
  webServer: externalBaseUrl
    ? undefined
    : {
        command: production
          ? 'pnpm run preview --host 127.0.0.1 --port 4173 --strictPort'
          : 'pnpm run dev',
        cwd: 'web',
        url: localBaseUrl,
        // Locally, reuse a server the developer already has running. In CI,
        // `false` makes Playwright *throw* if something already holds the port
        // instead of silently testing a stale server — which is what a leftover
        // `pnpm run dev` used to cause.
        reuseExistingServer: !production && !process.env.CI,
        // Vite's dev server is not fast on a cold WASM-less cache; the docs
        // default is 60s and the CI job previously allowed the same.
        timeout: 60_000,
        stdout: 'pipe',
        stderr: 'pipe',
      },

  use: {
    baseURL: externalBaseUrl || localBaseUrl,
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
    actionTimeout: 30000,
  },

  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
    {
      name: 'firefox',
      use: { ...devices['Desktop Firefox'] },
    },
    {
      name: 'webkit',
      use: { ...devices['Desktop Safari'] },
    },
    {
      name: 'mobile-chrome',
      use: { ...devices['Pixel 5'] },
    },
    {
      name: 'mobile-safari',
      use: { ...devices['iPhone 12'] },
    },
    {
      name: 'tablet-safari',
      use: { ...devices['iPad Air'] },
    },
  ],

  timeout: 60000,
  expect: {
    timeout: 10000,
  },
});
