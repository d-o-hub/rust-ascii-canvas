# ADR-049: Adopt `do-harness` as the Sensor Runner

## Status

Accepted — Adopted on 2026-10-02. **Implementation:** Phase 1 and 2 migration complete. `do-harness.toml` is configured with `feedback`, `verification`, and `release` signal sets; `scripts/quality-gates.sh` and `scripts/pre-commit-fast.sh` delegate to `do-harness verify`.

## Context

This repository previously relied on bespoke bash runners (`scripts/quality-gates.sh` and `scripts/pre-commit-fast.sh`) to execute local computational sensors across fast and full tiers. While effective, this duplicated logic provided by `do-harness`, a compiled harness CLI that manages language packs, signal sets (`feedback`, `verification`, `release`), evidence artifacts (`.do-harness/evidence.json`), git hooks, and findings baselines/ratchets.

Aligning with `do-harness` provides a single unified verify entry point locally and in CI, per-change evidence artifacts, ratcheted advisory findings, and hook installers, while reducing bespoke script maintenance.

## Decision

1. **Adopt `do-harness` as the sensor runner via `do-harness.toml`**:
   - Define signal sets matching our tiered quality gates:
     - `feedback`: fast local feedback tier (`fmt`, `clippy`, `check`, `loc`, `architecture`, `wasm-freshness`, `web-lint`, `web-tsc`, `e2e-tsc`, `eslint-root`, `vitest`).
     - `verification`: comprehensive local and CI verification tier (`feedback` + native Rust tests, CI coherence and fixtures, skill checks and fixtures, version parity, merge gate self-test, ruleset snapshot, Playwright config fixtures, privacy/secrets, audits, bandit, actionlint, cargo-deny, codacy check, WASM tests and size check, production Chromium E2E).
     - `release`: pre-flight release signal set.
   - Prefix Cargo/Rust commands in `do-harness.toml` with `RUSTUP_TOOLCHAIN=1.94.0` to preserve toolchain stability in sandbox environments.

2. **Wrap and delegate**:
   - Re-route `scripts/quality-gates.sh` to delegate `--fast` to `do-harness verify --set feedback` and full execution to `do-harness verify --set verification`.
   - Re-route `scripts/pre-commit-fast.sh` to delegate to `do-harness verify --set feedback --changed`.
   - Extract inline privacy/secrets scan into `scripts/check-privacy-secrets.sh` for clean sensor invocation.
   - Maintain the direct execution fallback in `scripts/quality-gates.sh` if `do-harness` is unavailable on `PATH` for rollback safety.

3. **Sensor Mapping Summary**:
   | Legacy Gate | Sensor Name in `do-harness` | Command / Script |
   | ----------- | --------------------------- | ---------------- |
   | Format | `fmt` | `cargo fmt --all -- --check` |
   | Clippy | `clippy` | `cargo clippy --locked -D warnings` |
   | Rust Check | `check` | `cargo check --locked` |
   | Rust Test | `test-rust` | `cargo test --locked --all` |
   | LOC limits | `loc` | `scripts/check-loc.sh` |
   | Architecture | `architecture` | `scripts/check-architecture.sh` |
   | CI Coherence | `ci-coherence` | `scripts/check-ci.py` |
   | CI Fixtures | `ci-fixtures` | `scripts/test-ci.py` |
   | Sensor Fixtures | `sensor-fixtures` | `scripts/test-sensors.py` |
   | Build Fixtures | `build-fixtures` | `scripts/test-build.py` |
   | Skill Check | `skill-check` | `scripts/check-skills.py` |
   | Skill Fixtures | `skill-fixtures` | `scripts/test-skills.py` |
   | Version Pin | `version-pin-parity` | `scripts/propagate-version.sh --check` |
   | Merge Gate Self-Test | `pr-merge-gate-selftest` | `scripts/pr-merge-gate.sh --self-test` |
   | Ruleset Snapshot | `ruleset-snapshot` | `jq -e ... .github/ruleset-main.json` |
   | WASM Freshness | `wasm-freshness` | `scripts/wasm-freshness.py --ensure` |
   | Web Lint | `web-lint` | `cd web && pnpm run lint` |
   | Web TSC | `web-tsc` | `cd web && pnpm exec tsc --noEmit` |
   | E2E TSC | `e2e-tsc` | `web/node_modules/.bin/tsc --noEmit -p e2e/tsconfig.json` |
   | Playwright Config | `playwright-config-fixtures` | `node --test scripts/test-playwright-config.mjs` |
   | Vitest | `vitest` | `cd web && pnpm test` |
   | ESLint Root | `eslint-root` | `./node_modules/.bin/eslint .` |
   | Privacy & Secrets | `privacy-secrets` | `scripts/check-privacy-secrets.sh` |
   | Audit Cargo | `audit-cargo` | `cargo audit --file Cargo.lock` |
   | Audit NPM | `audit-npm` | `scripts/npm-audit.sh` |
   | Bandit | `bandit` | `scripts/bandit-check.py` |
   | Actionlint | `actionlint` | `scripts/actionlint-check.py` |
   | Cargo Deny | `cargo-deny` | `cargo deny --locked check` |
   | Codacy Check | `codacy-check` | `scripts/codacy-check.sh` |
   | WASM Test | `wasm-test` | `scripts/test-wasm.sh` |
   | WASM Size | `wasm-size` | `node scripts/check-artifact.mjs` |
   | E2E Chromium | `e2e-chromium` | `pnpm run build:web && PRODUCTION_E2E=1 playwright test` |

## Guard-rail Boundaries

- No sensor is weakened or skipped.
- `npm run gate:fast` and `npm run gate:full` remain as entrypoints for developers and CI/hooks.
- Rollback plan: If `do-harness` fails or causes environment issues, uninstalling or overriding `do-harness` seamlessly falls back to direct script execution in `scripts/quality-gates.sh`.

## Consequences

- Unified verification runner with declarative config in `do-harness.toml`.
- Per-change evidence artifacts recorded in `.do-harness/evidence.json`.
- Reduced duplication between script runner and CI checks.
