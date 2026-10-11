// @ts-check
//
// Root ESLint config: covers the repository OUTSIDE `web/`.
// Why this exists: `web/eslint.config.js` only ever saw `web/`, because the lint
// entrypoint is `cd web && eslint .`. That left `e2e/` — 8 spec files plus the
// page object — completely unlinted locally, so every issue Codacy reported there
// (17 at the time of writing) was invisible to `gate:fast` and surfaced only in
// the required Codacy check. AGENTS.md's rule is that a CI-vs-local divergence
// must be fixed in the sensor, so `e2e/` now gets local treatment too.
//
// Scope: this config deliberately does NOT lint `web/` — that directory has its
// own config and tsconfig and is linted by `pnpm run lint` inside `web/`. Running
// both would double-report the same files.
import eslint from '@eslint/js';
import tseslint from 'typescript-eslint';
import security from 'eslint-plugin-security';

/**
 * Codacy runs `eslint-plugin-security` and its two rules that actually fire in
 * this repository — `security/detect-unsafe-regex` and
 * `security/detect-object-injection` — are enforced there by the *Default
 * coding standard*, which cannot be disabled or configured. Until these rules
 * ran locally, all 11 of those findings were invisible to `gate:fast` and to the
 * `web` CI job: the sensor existed, it was green, and the rule family had never
 * been enabled. That is harness L-017.
 *
 * The whole plugin is enabled, not just those two. Each rule is self-gating —
 * `detect-child-process`, `detect-non-literal-fs-filename` and friends cannot
 * fire unless the API they guard is used, and this is browser code plus a Node
 * test harness — so the extra rules cost nothing and cover the day that code
 * does. Measured baseline: 0 findings across `web/` and `e2e/` at the time of
 * writing, after the 11 were fixed for real (no suppressions).
 */
const securityRules = Object.fromEntries(
  Object.keys(security.rules).map(name => [`security/${name}`, 'error']),
);

export default tseslint.config(
  {
    // `web/` is linted by its own config; ignore it (and build output) here so
    // the two setups never overlap.
    ignores: [
      'web/**',
      'node_modules/**',
      'target/**',
      'dist/**',
      'test-results/**',
      'playwright-report/**',
      '.agents/**',
      // `dogfood/` output is QA evidence — throwaway probe scripts that mix Node
      // and in-page browser globals, so `no-undef` reports 100+ errors on code
      // that is not part of the product and is never shipped.
      'dogfood-output/**',
    ],
  },
  eslint.configs.recommended,
  ...tseslint.configs.recommended,
  {
    files: ['**/*.ts'],
    plugins: { security },
    rules: {
      '@typescript-eslint/no-explicit-any': 'error',
      '@typescript-eslint/no-unused-vars': 'error',
      // Codacy's ESLint runs `no-inner-declarations` (an `eslint:recommended`
      // rule in ESLint 8) and flagged a block-level function declaration in
      // web/layers.ts. Local ESLint 10's recommended set no longer includes it,
      // so the finding was invisible here — the same divergence class as L-017.
      // Codacy's ESLint 8 default is `blockScopedFunctions: 'disallow'`; ESLint 10's
      // recommended set resolves to `'allow'`, which is why simply enabling the rule
      // would look like parity while never firing on the pattern Codacy flags.
      'no-inner-declarations': ['error', 'functions', { blockScopedFunctions: 'disallow' }],
      ...securityRules,
    },
  },
  {
    // Type-aware linting for `e2e/`, so the local gate sees the same class of
    // problems Codacy's ESLint does (notably `no-unnecessary-condition`).
    // Uses a dedicated tsconfig because `e2e/` sits outside `web/tsconfig.json`.
    files: ['e2e/**/*.ts'],
    languageOptions: {
      parserOptions: {
        projectService: true,
        tsconfigRootDir: import.meta.dirname,
      },
    },
    rules: {
      '@typescript-eslint/no-non-null-assertion': 'error',
      '@typescript-eslint/no-unnecessary-condition': 'error',
      // Codacy's ESLint runs `no-confusing-void-expression` and it put a *required
      // check* into `fail` on #250 for `requestAnimationFrame(() => requestAnimationFrame(() => resolve()))`
      // while `gate:fast` stayed green — harness L-017 for the third time, after
      // `security/*` (#223) and `no-inner-declarations` (#233). The pattern is
      // `enabledBy: Default coding standard` (id 155121), the same lock as both
      // `security/detect-*` rules, so there is no config to change and the code has
      // to satisfy it permanently — which makes a local sensor the only way to see
      // it before the push does. Measured baseline when it was added: 0 findings.
      //
      // It lives in this block on purpose. The rule requires type information;
      // running it on a file with no `parserOptions.project` makes ESLint abort, not
      // pass — so `playwright.config.ts`, the only root `.ts` outside `e2e/`, is
      // outside its reach until a root tsconfig exists. Covered set: `e2e/**/*.ts`.
      '@typescript-eslint/no-confusing-void-expression': 'error',
    },
  }
);
