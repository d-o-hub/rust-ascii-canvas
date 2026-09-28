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
    ],
  },
  eslint.configs.recommended,
  ...tseslint.configs.recommended,
  {
    files: ['**/*.ts'],
    rules: {
      '@typescript-eslint/no-explicit-any': 'error',
      '@typescript-eslint/no-unused-vars': 'error',
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
    },
  }
);
