# Named local drafts — isolated delivery

## Goal and baseline

Deliver the accepted named-local-draft portion of [ADR-046](ADRs/046-editing-safety-and-local-drafts.md)
from `main` at `cdd4c505bff1ced808a76dce3efee2d045291211`. The prerequisite
security PR [#228](https://github.com/d-o-hub/rust-ascii-canvas/pull/228) and
editor-safety PR [#229](https://github.com/d-o-hub/rust-ascii-canvas/pull/229) are
merged (`3156725`, `cdd4c50`). Their startup-layout, atomic-history and PNG-test
follow-ups must be preserved, not replaced by older source copies.

The larger development workspace remains read-only. Extract in a separate
worktree and branch from the merged base, not from its unrelated feature branch.
This document records a delivery candidate, not a release or merge assertion.

## Scope

- Browser-local named drafts: create, rename, list, switch, delete and import-as-new.
- Selected draft restore; migrate the legacy autosave without removing or
  overwriting its original value.
- Bound the shelf to 20 documents and 4 million serialized characters. Names,
  IDs and shelf revisions are metadata outside the unchanged `.asc` v1 schema.
- Save the current document before replacement. Prepare/validate a replacement
  off-screen; do not discard the live editor/history when persistence is refused.
- Explicit dirty/saved/error/conflict status and an available `.asc` backup action.
- Conservative optimistic shelf-wide stale-writer detection. Stop mutations after
  a conflict; preserve local editing/download. This is not compare-and-swap,
  multi-writer transactional storage, cloud synchronization or collaboration.
- Successful switching uses a new editor with fresh undo/view state, retaining
  chosen tool preferences where supported. Keep browser paste/keyboard ownership.
- Only a narrowly necessary file-event extraction may accompany the wiring.
  Existing LOC budgets may not increase.

Excluded: accessible layer-panel rewrite, full event/mobile decomposition, the
foreign event-result refactor, broad harness/skills hardening, CI policy changes,
dependency/version/release changes, and rendering optimization. ADR-047 remains
only partially delivered by #229; this feature must not imply otherwise.

## Actions and ownership

1. Confirm baseline/scope, accepted ADR, frozen dependency/tool versions and
   fresh worktree-local WASM — complete.
2. Extract persistence/session/UI and focused unit tests; preserve merged safety
   behavior — complete.
3. Independently review session/storage failure boundaries, author browser and
   adapter regressions, retain failing-before evidence — complete.
4. Integrate, run fast/full plus production-shaped cross-browser draft tests,
   inspect actual CI wiring and review exact staged paths — parent review in
   progress.
5. Open a narrow PR, read current-head Codacy and repository backlog, address
   every actionable review comment. Merge only after all required checks and
   the read-only merge gate pass; no bypasses or live ruleset changes — pending.

The implementation deliberately keeps the legacy autosave key untouched during
migration. It uses a single shelf write for document + selection metadata,
validates replacements in a fresh WASM editor before commit, and latches a
conservative shelf-wide conflict after an external revision. It does not claim
collaboration, durable backup, CAS semantics or a new `.asc` schema.

## Verification contract

Use pnpm **10.34.5** for both frozen installs. Build this worktree's own WASM,
not bindings copied from the larger implementation. The base convenience full
gate uses development E2E unless `BASE_URL` is set; an explicit production-shaped
run must name the actual build/server configuration rather than claiming the
pending harness overhaul shipped.

New Vitest tests run through the existing CI `web` job; new Playwright cases run
through the existing Chromium/Firefox/WebKit `e2e` matrix. Inspect
`.github/workflows/ci.yml` to confirm this: CI does not run `quality-gates.sh`.
No new required check is proposed.

Required evidence:

- Named draft lifecycle and selected reload, with distinct document content.
- Successful switch/save, empty/no-op saves, dirty reporting and fresh undo/view.
- Legacy migration success and failure retain original bytes.
- Quota/security exceptions, invalid `.asc`/shelf, count/size limits preserve live
  state and do not overwrite previously valid storage.
- Other-tab mutation/conflict and unloaded storage-event races do not silently
  replace the known newer shelf; limitation for simultaneous writers documented.
- `.asc` downloads still work while saving is blocked; import is additive.
- Mobile controls and native input remain usable; tool/document/initial-layout
  regressions from #229 still pass.
- Fast/full, exact production build, diff scope, Codacy and adversarial review.

## Results

Isolated worktree evidence so far:

- pnpm 10.34.5 frozen root/web installs and fresh WASM build succeeded; output
  is 217,411 bytes.
- Focused drafts unit suite: **16 passed**; full Vitest: **57 passed across 7
  files**; web lint/typecheck/build passed.
- Draft lifecycle E2E: **36 passed** across Chromium, Firefox and WebKit, no
  retries. A Firefox reload race in the first test run was reproduced once and
  hardened with an explicit post-reload editor readiness assertion; 10 repeated
  Firefox launches then passed.
- Existing initial-layout/document/gesture/input safety coverage: **36 passed**
  across Chromium, Firefox and WebKit.
- `npm run gate:fast` and `npm run gate:full` passed. Full gate included native
  Rust, both nonzero Node WASM targets, audits/deny, size budget and the local
  **115-test Chromium E2E inventory** (10 files). The repository Codacy procedure reports the existing
  non-High `shellcheck_SC2034` warning only.
- Explicit local Codacy procedure on the committed changed TypeScript/E2E scope:
  ESLint8 scanned 8 files and Opengrep scanned 9 files, both with 0 findings.
  ESLint's type-aware parserServices rules were unavailable, so this is not a
  full local/cloud parity claim; no analyser rules were relaxed.

Remote CI, current-head Codacy and the read-only merge gate remain pending.
Evidence logs live outside the tree under `/tmp/ascii-drafts-*.log`; do not
substitute source-workspace counts for this isolated branch's results.
