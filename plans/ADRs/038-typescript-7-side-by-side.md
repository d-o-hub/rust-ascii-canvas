# ADR-038: TypeScript 7 Side-by-Side Migration

## Status
Proposed

## Context

PR #145 (dependabot) bumps TypeScript from 5.8 to 7.0 in `web/`. TypeScript 7.0 is a ground-up rewrite in Go, shipping without a stable programmatic API. Tools like `typescript-eslint` that import the TypeScript compiler API directly crash with TS 7:

```
typescript-eslint does not support TS 7.0.
```

The `typescript-eslint@8.65.0` peer dependency range is `typescript: >=4.8.4 <6.1.0`, making TS 7 incompatible. There is no version of `typescript-eslint` that supports TS 7 yet — tracking issue: [typescript-eslint#10940](https://github.com/typescript-eslint/typescript-eslint/issues/10940). Support is expected in **TS 7.1** (~October 2026) when the new stable programmatic API ships.

## Decision

Use the **official side-by-side approach** recommended by Microsoft in the [TypeScript 7.0 release blog](https://devblogs.microsoft.com/typescript/announcing-typescript-7-0/#running-side-by-side-with-typescript-6.0):

```json
{
  "devDependencies": {
    "@typescript/native": "npm:typescript@^7.0.2",
    "typescript": "npm:@typescript/typescript6@^6.0.2"
  }
}
```

This gives us:
- `tsc` → TS 7 (from `@typescript/native`) for fast type-checking in CI and dev
- `tsc6` → TS 6 (from `@typescript/typescript6`) available as fallback
- `typescript-eslint` → resolves `import "typescript"` to TS 6 (satisfies peer dep `<6.1.0`)

### Changes applied

1. **`web/package.json`**:
   - Changed `"typescript": "^7.0.2"` → `"typescript": "npm:@typescript/typescript6@^6.0.2"` + `"@typescript/native": "npm:typescript@^7.0.2"`
   - `lint` script unchanged: `eslint .` (uses TS 6 via `typescript-eslint`)
   - `pnpm exec tsc --noEmit` resolves to TS 7 (from `@typescript/native`)

2. **`web/tsconfig.json`**: No changes needed — all options are TS 7-compatible.

3. **`web/eslint.config.js`**: No changes needed — `typescript-eslint` resolves to TS 6 via the npm alias.

4. **Root `package.json`**: No changes needed — `lint:web` runs `pnpm exec tsc --noEmit` in web/ which uses TS 7.

5. **`.github/workflows/ci.yml`**: No changes needed — CI runs `pnpm run lint` (ESLint) and `pnpm exec tsc --noEmit` (TS 7).

### Why not pin TS to 5.x?

- The dependabot PR intentionally upgrades to TS 7.0
- The codebase is small (~1800 LOC), modern, and has zero deprecated patterns
- The side-by-side approach lets us use TS 7 for type-checking while keeping tooling working

### Why not skip typescript-eslint entirely?

- Type-aware linting catches real bugs (unused vars, any types, etc.)
- The existing ESLint config uses `no-explicit-any: error` and `no-unused-vars: error`
- Removing it would weaken code quality gates

## Consequences

### Positive
- CI passes (ESLint + TypeScript + Vitest all green)
- TS 7 type-checking available via `@typescript/native`
- Ready for full TS 7 migration when `typescript-eslint` gains support (~Oct 2026)

### Negative
- Two TypeScript packages in devDependencies (minor overhead)
- `tsc6` vs `tsc` naming may confuse contributors (document in README)
- Minor risk of divergence between TS 6 and TS 7 type-checkers (unlikely for this codebase)

### Risks
- **Low**: The two `as unknown as` double-casts in `main.ts` should be re-validated after upgrade
- **None**: tsconfig.json is fully TS 7-compatible
- **None**: No deprecated TS features used in the codebase
