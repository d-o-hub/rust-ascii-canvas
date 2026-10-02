---
name: verify
description: Run tiered computational quality sensors and self-correct. Use after changes, before commits/PRs, or for verify/gate/quality-check requests. Prefer fast while iterating, full before review, and the read-only PR gate before merging.
---

# Verify

Read the [harness](../../../agents-docs/harness.md). The inventories below reflect
`scripts/quality-gates.sh` and `.github/workflows/ci.yml`, not promises that every
local check is a CI gate. CI invokes shared entrypoints directly; it does **not**
run the convenience gate script.

## Tiers and procedure

| Tier | Command | When |
|------|---------|------|
| Focused | Relevant Rust/Vitest/Playwright file | Reproduce first and iterate tightly. |
| Fast | `npm run gate:fast` | After meaningful edits. |
| Full | `npm run gate:full` | Integrated candidate before review. |
| PR | `npm run gate:pr` | Read-only merge contract, not product verification. |
| Gate fixtures | `npm run gate:pr:test` | Changing merge predicates. |
| Live ruleset | `npm run gate:ruleset` | Compare live contract to committed snapshot; requires access. |

1. Choose the smallest tier appropriate to the phase. Do not run full E2E after
   every one-line fix. With concurrent workers, run focused checks; the integrating
   parent runs fast/full on a coherent tree.
2. Read `[FAIL]`, `[UNVERIFIED]` and `FIX:` output. Fix the root cause, rerun, and
   record exact commands/results. Unavailable tooling/network is not a pass.
3. Do not disable sensors, skip tests, weaken assertions, suppress findings or
   grow LOC budgets to make green output.
4. Input/tool changes also require [tool-validation](../tool-validation/SKILL.md).
5. Before merge: full → `code-review` → clean `pr-roast` → `merge-gate`. Bots,
   dependency updates and docs PRs have no roast exemption.

## Fast inventory (local)

- Shared LOC ratchet and fail-closed architecture scan.
- CI path/applicability/aggregation coherence and retained CI, audit/artifact,
  architecture/LOC and build-freshness/discovery fixtures.
- Offline skill ownership/resources/commands/dependencies checks and fixtures:
  `python3 scripts/check-skills.py` and `python3 scripts/test-skills.py`.
- Version pins, wasm-bindgen schema pin parity, merge-gate predicate fixtures and
  offline ruleset snapshot validation (not a live ruleset read).
- rustfmt; locked clippy `-D warnings`, build and native `cargo test`.
- **Fresh** WASM bindings: `python3 scripts/wasm-freshness.py --ensure` verifies
  input/output fingerprints and rebuilds stale or missing `web/pkg` before types.
  Mere file presence is insufficient. Never hand-edit generated bindings.
- Web ESLint/types/Vitest, **root ESLint plus e2e TypeScript** using web's compiler.
- Local privacy and secret scans. These scans are not direct CI jobs.

Dependencies must already be installed from **both frozen lockfiles**. Missing
root/web dependencies fail; the gate does not fall back to a mutable install.
Use [repo-typescript](../repo-typescript/SKILL.md) for real frontend commands and
for the uninstalled specialist names in the upstream TypeScript guide.

## Full additions (local)

- `cargo audit --file Cargo.lock`, `cargo deny --locked check` and
  `bash scripts/npm-audit.sh` for **both** npm lockfiles. Missing tools, malformed
  results, registry errors or one unchecked lockfile are unverified/nonzero,
  not clean. Do not regenerate a lockfile just to audit a different graph.
- `bash scripts/test-wasm.sh`: explicit registered Node WASM suite with a
  nonzero-passed-test requirement. Native zero-test output is not WASM evidence.
- `npm run check-size`: present/readable/valid WASM before the 1.5 MiB budget.
- Production web build, then **production-preview** Chromium E2E.
- `npm run codacy:check`: local repository-level intake, **not a CI gate**. It
  warns when absent/unauthenticated rather than pretending a clean scan. The
  separate required Codacy PR check is diff-scoped; neither proves the other.

## Actual CI parity map

| CI job | Shared/local equivalent | Important distinction |
|--------|-------------------------|-----------------------|
| `fmt`, `clippy`, `rust` | Locked Rust commands; `bash scripts/test-wasm.sh` | WASM test runner needs the pinned toolchain/Node, not a browser. |
| `architecture` | `bash scripts/check-architecture.sh`; `python3 scripts/check-ci.py`; all four Python fixture scripts below + skill checker; version parity | Harness paths and skill-lock changes must activate it. |
| `loc` | `bash scripts/check-loc.sh` | Cross-cutting standalone job, not a step only in `web`. |
| `security`, `security-npm`, `deny` | Cargo audit, both npm lockfile audits, cargo-deny | Required evidence fails closed; local offline fast excludes live audits. |
| `wasm` | `npm run build:wasm` + `npm run check-size` | CI uploads provenance with the generated artifact. |
| `web` | Freshness `--check`, web lint/types/Vitest, root ESLint, e2e types | CI downloads current pkg; web compiler owns both TypeScript scopes. |
| `e2e` | Production build + `PRODUCTION_E2E=1 pnpm exec playwright test` | CI uses Chromium/Firefox/WebKit; local full uses Chromium. |
| `pr-readiness` | `npm run gate:pr:test`; live ruleset check | Readiness report is advisory; server-side thread resolution remains enforced. |

Retained offline fixture commands (root):

```bash
python3 scripts/test-ci.py
python3 scripts/test-sensors.py
python3 scripts/test-build.py
python3 scripts/test-skills.py
```

## Playwright owns startup

Focused tests use the dev server via `webServer`; full/CI set `PRODUCTION_E2E=1`
and use an optimized `web/dist` preview on loopback port 4173. Playwright starts,
awaits and tears down the server. Production and CI refuse to reuse an existing
server. Do not hand-roll `nohup`, curl readiness loops or an orphan preview.

For a local production-shaped check, from the root:

```bash
python3 scripts/wasm-freshness.py --ensure
npm run build:web
PRODUCTION_E2E=1 pnpm exec playwright test --project=chromium
```

`BASE_URL` intentionally disables **all local startup** and targets an
already-running deployment (for example a Netlify Deploy Preview). Record that
URL and build identity; don't claim the local dist was exercised in that mode.
Install the required Playwright browsers explicitly if missing.

## PR contract and steering

The mechanical PR gate checks draft/conflicts, **every** check's success, resolved
threads and no outstanding changes request. It never merges and cannot certify
a clean roast. See [merge-gate](../merge-gate/SKILL.md) for the complete procedure.

If CI is red while local is green, compare artifacts, path filters, environment
and credentials. Fix the shared sensor/fixture and its **direct CI wiring**, not
just the symptom. Repeated failures need a harness learning and a regression.
A local command succeeding does not prove CI ran or any working-tree change was
merged. No live ruleset/credential mutations are part of verification.
