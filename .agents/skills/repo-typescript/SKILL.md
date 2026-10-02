---
name: repo-typescript
description: Repository command and referral adapter for the upstream TypeScript skill. Read with typescript-expert for TypeScript, Vite, module resolution or type diagnostics in ASCII Canvas; replaces unavailable specialist referrals with installed tools and skills.
---

# Repository TypeScript Adapter

Read this alongside the locked [TypeScript guide](../typescript-expert/SKILL.md).
Do not edit that imported skill or install imaginary subagents to follow it.

## Referral map

The upstream `typescript-build-expert`, `typescript-module-expert`, and
`typescript-type-expert` skills are **not installed**. Their metadata/prose
references are preserved for provenance, not promises of available agents.

| Upstream referral | Local procedure |
|-------------------|-----------------|
| `typescript-build-expert` | Read the installed [vite skill](../vite/SKILL.md); inspect the actual Vite config and existing build scripts. |
| `typescript-module-expert` | Use `cd web && pnpm exec tsc --noEmit --traceResolution`; inspect package/module boundaries. |
| `typescript-type-expert` | Use `cd web && pnpm exec tsc --noEmit --extendedDiagnostics`; reduce the failing type case locally. |

Do the work in the current agent. Escalate a genuinely unresolved design choice,
not a request to invoke an unavailable skill. The aliases are explicitly mapped
in `.agents/skill-manifest.json` and checked offline.

## Actual repository commands

Run from the repository root unless a subshell says otherwise. Use the pinned
mise toolchain and frozen installs; never silently resolve new dependencies.

```bash
pnpm install --frozen-lockfile
(cd web && pnpm install --frozen-lockfile)
npm run build:wasm
(cd web && pnpm run lint)
(cd web && pnpm exec tsc --noEmit)
(cd web && pnpm test)
pnpm exec eslint .
web/node_modules/.bin/tsc --noEmit -p e2e/tsconfig.json
(cd web && pnpm run build)
npx playwright test --project=chromium
```

- Root `npm test` runs Rust **and Playwright**, not Vitest. Use `cd web && pnpm test`
  for frontend unit tests. There is no root `typecheck` script or root TypeScript
  dependency: use web's compiler, including for `e2e/`.
- Web `lint` currently includes ESLint and TypeScript. The explicit `tsc` command
  is useful for focused diagnostics, not a proposed second compiler version.
- Build fresh `web/pkg` before typechecking APIs changed in Rust. Never hand-edit
  generated bindings or trust presence alone to prove freshness.
- Playwright owns startup/readiness/teardown through `webServer`. `BASE_URL`
  intentionally targets an already-running deployment and disables local startup.
- Do not introduce watch processes for verification. Follow
  [verify](../verify/SKILL.md) for fast/full and production-build checks.

This adapter does not certify all generic snippets in the upstream guide against
this repository. Its declared commands and dependency targets are the local
contract; specialist performance advice still requires measurement.
