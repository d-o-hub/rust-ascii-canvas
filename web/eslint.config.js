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
      // Codacy's ESLint runs `no-confusing-void-expression` and it failed the
      // required check on #250 while `gate:fast` was green — harness L-017, third
      // instance after `security/*` (#223) and `no-inner-declarations` (#233).
      // `enabledBy: Default coding standard` (id 155121) means it cannot be
      // disabled or configured, so the code must satisfy it always and the sensor
      // has to see it first. Measured baseline when added: 0 findings.
      // Requires type information, which is why it sits in this type-aware block
      // rather than the plain-rules block above.
      '@typescript-eslint/no-confusing-void-expression': 'error',
    },
  }
);
