// @ts-check
import eslint from '@eslint/js';
import tseslint from 'typescript-eslint';
import security from 'eslint-plugin-security';

/**
 * Codacy runs `eslint-plugin-security`, and the two rules it actually fires on
 * here — `security/detect-unsafe-regex` and `security/detect-object-injection` —
 * are locked to Codacy's *Default coding standard*, so they cannot be
 * configured away and the code has to change instead. They were not enabled
 * locally, which is how 11 findings (including three in `web/ux.test.ts`)
 * stayed invisible to `gate:fast` and to this job. See harness L-017.
 *
 * The whole plugin is enabled: each rule is self-gating — it cannot fire unless
 * the API it guards is called — so the rules irrelevant to browser code cost
 * nothing, and measured baseline is 0 findings.
 */
const securityRules = Object.fromEntries(
  Object.keys(security.rules).map(name => [`security/${name}`, 'error']),
);

export default tseslint.config(
  {
    ignores: ['dist/**', 'pkg/**', 'node_modules/**'], // Ignore WASM artifacts & builds
  },
  eslint.configs.recommended,
  ...tseslint.configs.recommended,
  {
    files: ['**/*.ts'],
    plugins: { security },
    rules: {
      '@typescript-eslint/no-explicit-any': 'error',
      '@typescript-eslint/no-unused-vars': 'error',
      ...securityRules,
    },
  },
  {
    // Type-aware linting so local gates catch what Codacy flags (AGENTS.md
    // steering rule: CI-vs-local divergence must be fixed in the sensor).
    // Pre-existing violations are suppressed inline at their sites.
    files: ['**/*.ts'],
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
