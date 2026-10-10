# Wiring Codacy into CI — a credential-blocked plan

**Status**: Not started; **blocked on a project-scoped secret**. Nothing in
this file runs until that secret exists.

**Owner**: whoever holds the Codacy org admin (typically the repo owner).
**Do not confuse this with the local `codacy login`** — that stores an
*account* token in `~/.codacy/credentials` on this machine; a GitHub Actions
runner is an ephemeral VM and cannot see it. That is the L-016 shape wearing
an auth costume: a check you can run by hand is a procedure, not a sensor.

## Two independent integrations, different scopes

Codacy's cloud exposes two things this repo does not currently use in CI.
They need different secrets and different permissions, so treat them as
separate PRs; do not bundle them.

| # | Integration | Token | Direction | Blocks merge? |
|---|-------------|-------|-----------|---------------|
| 1 | **Coverage upload** | `CODACY_PROJECT_TOKEN` (project-scoped, write) | CI → Codacy | no (post-merge) |
| 2 | **Repo-intake check** | `CODACY_API_TOKEN` (project-scoped, read-only) | CI ← Codacy | **yes**, once wired as a required check |

### Why the scopes are not interchangeable

The Codacy CLI refuses to fall back from a project token to an account token
on purpose: *falling back to an account token would silently run with far
wider access than the scoped run you asked for.* Honour that guard:

- Coverage uploads are **writes**. They require a token that can mutate the
  project's coverage record. Use `CODACY_PROJECT_TOKEN` and pin the scope to
  this one repository.
- Reading the issue backlog is **read-only**. `CODACY_API_TOKEN` scoped to
  this project is enough — an account token would be strictly more privilege
  than needed and is the wrong tool.

## Integration 1 — Coverage upload

### Current state

- Rust: tests run in `Rust (Build + Tests)` in `ci.yml`, no coverage flag,
  no `lcov.info`.
- Web: `web/` runs `vitest`, `test` script has no coverage provider, no
  `@vitest/coverage-v8` in `web/package.json`, no coverage config.
- E2E: intentionally **not** covered — Playwright coverage across browsers
  produces a report that double-counts against the unit numbers and Codacy's
  merge rules do not distinguish them. If a browser-coverage figure matters
  later, use a separate `--partial` flag and a `final` command (see
  [coverage-upload.md](../.agents/skills/setup-coverage/references/coverage-upload.md)).

### Steps once the token exists

1. **Rust coverage** — add `cargo-llvm-cov` (or `cargo-tarpaulin`) to the
   `Rust (Build + Tests)` job. Prefer `cargo-llvm-cov`: it is what upstream
   recommends for `xtask`-style builds and emits an `lcov.info` directly.
   ```yaml
   - name: Rust coverage
     run: cargo llvm-cov --all-features --workspace --lcov --output-path lcov-rust.info
   ```
2. **Web coverage** — install `@vitest/coverage-v8` in `web/`, extend
   `web/vitest.config.ts` (currently absent — create it) with a `coverage`
   block that emits `lcov`, and change the test script:
   ```json
   "test": "vitest run --coverage"
   ```
   Output lands in `web/coverage/lcov.info`.
3. **Upload** — one step per report, `--partial` on each, one `final` after:
   ```yaml
   - name: Upload Rust coverage to Codacy
     uses: codacy/codacy-coverage-reporter-action@v1
     with:
       project-token: ${{ secrets.CODACY_PROJECT_TOKEN }}
       coverage-reports: lcov-rust.info
       language: Rust
       partial: true
   - name: Upload Web coverage to Codacy
     uses: codacy/codacy-coverage-reporter-action@v1
     with:
       project-token: ${{ secrets.CODACY_PROJECT_TOKEN }}
       coverage-reports: web/coverage/lcov.info
       language: TypeScript
       partial: true
   - name: Finalise Codacy coverage
     uses: codacy/codacy-coverage-reporter-action@v1
     with:
       project-token: ${{ secrets.CODACY_PROJECT_TOKEN }}
       command: final
   ```
4. **Pin the action to a SHA** — same L-015 rule as every third-party
   action here (`dtolnay/rust-toolchain`, `pnpm/action-setup`, etc.): a
   `@v1` tag is mutable. Round-trip the resolved commit through
   `commits/<sha>` (not `tags/<sha>`), then pin as
   `uses: codacy/codacy-coverage-reporter-action@<sha> # v1.x.y`.
5. **Coverage goal in Codacy** — set a per-language minimum once a baseline
   exists. Do **not** set it blind: the first upload is the baseline, not a
   pass/fail.

### What to gate on

Nothing. Coverage upload runs **after** the merge, on `main`. It reports a
percentage; it does not block. Making coverage a required check is a separate
decision with a separate threshold — see L-008's "relaxing a guard-rail is
the failure mode" caution. Do not conflate "wired" with "enforced".

## Integration 2 — Repo-intake check

### Current state

- `npm run codacy:check` wraps `codacy -o json issues` locally. It **fails
  on Critical/High** and warns rather than faking a pass when unauthenticated.
- It is not in `ci.yml`, because `gh secret list` is empty. So today's
  "backlog is clean" claim has no sensor behind it — exactly the L-017 gap
  that Bandit parity (#242) and actionlint parity (#244) closed for their
  rule families.
- Codacy's PR-level analysis (`isUpToStandards`) is **diff-scoped**; it is
  structurally compatible with an arbitrary pre-existing backlog. A green
  PR check ≠ a clear repo. That was the "41 → 0 actionable" bug the harness
  recorded.

### Steps once the token exists

1. **Sensor**: `scripts/codacy-issues-check.py`, fail-closed 0/1/2 contract
   (matches `npm-audit.py`, `bandit-check.py`, `actionlint-check.py`):
   - **0** — 0 Critical/High findings in `codacy -o json issues`.
   - **1** — ≥ 1 Critical/High finding. Print `file:line — patternId` per
     finding (bounded to top 20) so a developer can act on it.
   - **2** — unverified. Missing token, non-2xx response, prose payload,
     malformed `issues[]`, or `codacy` CLI present but `--version` drift.
   Retain fixtures in `test-sensors.py` (clean, HIGH, CRITICAL, sub-threshold
   advisories, missing CLI, malformed JSON, prose payload, non-dict entry,
   findings vs exit-status mismatch, stale CLI version).
2. **CI wiring** — a new `codacy-intake` job (or add to `architecture`, but
   it is not really architecture):
   ```yaml
   - name: Codacy repo-intake (L-028)
     env:
       CODACY_API_TOKEN: ${{ secrets.CODACY_API_TOKEN }}
     run: python3 scripts/codacy-issues-check.py
   ```
   Install the CLI pinned: `npm install -g @codacy/codacy-cloud-cli@<pinned>`
   with a SHA-verified tarball, same supply-chain discipline as actionlint
   in #244.
3. **Ruleset — do not touch this in the same PR.** Adding a new required
   check is a **merge contract change**, which needs (a) an ADR (ADR-048?)
   and (b) an updated `.github/ruleset-main.json` in the same change. Wire
   the job as a *non-blocking* check first, confirm it produces signal on
   real PRs for a week, then propose making it required via the ADR route.
   Never drop `Codacy Static Code Analysis` (the existing app check) to make
   room for a second one.

### Why two Codacy surfaces and not one

The GitHub-App check `Codacy Static Code Analysis` is diff-scoped and is
already required. The `codacy issues` backlog read is repo-scoped and closes
the L-017 gap that a diff-scoped check **structurally cannot close** — the
same lesson as Bandit parity: a green Codacy PR check still means nothing
for regressions in a rule family nothing local invokes, and now nothing
continuous invokes either.

## Version drift

Same L-005 shape as `wasm-bindgen` dep vs CLI. The pinned cloud-CLI version
must appear in one place; anything else is drift. Prefer:

- A single `PIN = 'X.Y.Z'` constant at the top of the script.
- Verify the installed `codacy --version` matches before trusting it. A
  stale CLI on a developer's PATH is a **different sensor** than CI's.

## ADR needed

`ADR-048: Codacy coverage and repo-intake in CI`. Covers:

- **Why the two surfaces are separate** — write vs read; blocking vs
  report-only.
- **Why the coverage upload is not a gate** — no threshold at baseline.
- **Why the intake check starts as a non-blocking check** — L-017 rule:
  bring the parity, then require it after it has proven it produces signal
  rather than noise. Do not require an untested sensor.
- **What would change the ruleset snapshot, and when** — the exact PR that
  promotes `codacy-intake` to required; the ruleset snapshot diff that ships
  with it.

## What NOT to do

- **Never** put an account-scoped token in CI. Scope to this project only.
- **Never** mark the intake check as required without an ADR and a ruleset
  snapshot in the same PR (L-008: relaxing a guard-rail is the failure
  mode; adding one without evidence is the same failure in reverse).
- **Never** use the coverage upload as a merge gate without a measured
  baseline. A first-run threshold is a coin flip.
- **Never** bundle a Codacy config change (`biome.json`, `.codacy.yaml`,
  tool enablement) with the credential wiring. Different decisions,
  different reviews.
- **Never** `--admin` past a red check.

## Handoff checklist for the human holding Codacy admin

Copy/paste when you are ready to unblock:

1. Codacy dashboard → project settings → **Project API token**. Generate one
   scoped to `d-o-hub/rust-ascii-canvas` only.
2. `gh secret set CODACY_PROJECT_TOKEN` and paste.
3. Same dashboard → **Account API token** with project scope →
   `gh secret set CODACY_API_TOKEN`.
4. Reply to the P1 row in `plans/FOLLOW_UPS.md`; the two PRs below are ready
   to be opened once these exist:
   - PR-A: coverage sensors + partial uploads + SHA-pinned action.
   - PR-B: `scripts/codacy-issues-check.py` + fixtures + non-blocking CI job.
5. After a week of green PR-B runs, propose PR-C: ADR-048 + ruleset snapshot
   update to promote the check to required.

## Related

- AGENTS.md § *Codacy* — the four non-optional rules (L-017).
- `.agents/skills/codacy/SKILL.md` § *Credentials: where they live* — G1–G4.
- `.agents/skills/setup-coverage/references/coverage-upload.md` — CLI and
  action invocation, partial-vs-final semantics.
- Harness L-005 (pin drift), L-015 (SHA vs tag object), L-016 (script is not
  a gate), L-017 (rule-family parity), L-026 (Bandit parity), L-027
  (actionlint parity).
