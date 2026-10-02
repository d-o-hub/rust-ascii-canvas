# Recommendations implementation — 2026-09-30

> 2026-10-02 status: this candidate shipped as #228–#233, except the
> truthful-sensors + skills remainder, ported onto `refactor/fail-closed-sensors`.
> The baseline and repro text below is the original dated record.

## Goal

Implement the repository audit recommendations, prioritizing recoverable editing and truthful verification before new features. Preserve the pre-existing event-handler extraction in the working tree. No commits, pushes, live ruleset changes, releases, or cloud services are part of this task.

## Baseline

HEAD: `eb3ce8f`. Existing user changes: `.loc-allowlist`, `web/events.ts`, new `web/events-file.ts` and `web/events-mobile.ts`. Audit evidence: `.omnirush/swarms/20260930-123836.md` (local, ignored). Fast gate and fresh WASM build passed, but API probes reproduced text/space deletion, resize data loss, non-atomic strokes, layer redo data loss, stale tool state, and the 33-layer save/restore mismatch.

## Work packages

1. **Rust correctness and document boundary** — regressions first; safe key classification, one history transaction per freehand/eraser gesture, preserved added-layer payload, tool-session resets, shared core v1 document schema/limits, clean export pixels. Keep per-layer history and compatible `.asc` v1 semantics.
2. **Web safety and local drafts** — viewport/document separation and explicit crop guard, focus-aware paste, visible save status, local named drafts with safe migration/switching/import and stale-writer detection, keyboard/focus-safe layer panel extraction, clean PNG integration.
3. **Verification** — fail-closed npm audit and artifact size checks, executable WASM suite, retained architecture/LOC/CI applicability fixtures, fresh generated bindings, production-shaped smoke path, locked dependency inputs. No live merge-contract or release-pin mutations.
4. **Skills and planning coherence** — repair local merge/rollback/tool QA guidance, resource links and ownership-aware skill checks; consolidate active next-work into FOLLOW_UPS rather than conflicting copies. Do not rewrite upstream-locked skills opportunistically.
5. **Integration** — focused tests, fast gates after meaningful coherent edits, full gate, independent review and self-correction. Measure render behavior before any performance optimization; avoid claiming latency improvement without measurements.

## Decisions

- **2026-09-30 approval:** the user explicitly accepted ADR-046 and ADR-047 after implementation and local verification. This approves the recorded design/control decisions, not a merge, release, or live ruleset mutation. Next delivery step: prepare narrowly scoped PRs, beginning with the security patch and editor safety.
- ADR-046 records editing/persistence boundaries and the bounded local-draft feature.
- ADR-047 records stricter verification and retained sensor fixtures; the existing server-side ruleset is unchanged.
- No collaboration/accounts, global undo timeline, format version bump, speculative rendering rewrite, or release automation expansion.
- Existing public WASM methods stay stable; clean export API is additive.
- New source files stay at or below 500 lines; remove obsolete LOC exceptions after extraction, never enlarge budgets.

## Acceptance

- Every reproduced data-loss case has a permanent regression, with failing-before/fixed-after evidence where feasible.
- One completed stroke undoes/redoes exactly even beyond the history capacity in pointer events.
- Every UI-creatable document can reload its own save; blank loads cannot resurrect old tool buffers.
- Viewport resizing does not change document dimensions/content/history.
- Draft A/B survive reload independently; failed save/switch/import/migration cannot discard the current document; stale writers do not silently overwrite known newer revisions.
- PNG excludes editor overlays; layer navigation/actions work by keyboard and retain meaningful focus.
- Missing artifacts, unexecuted tests, unavailable scanners and incomplete required audits cannot report verified success.
- New sensors state their local and CI execution locations and include executable positive/negative fixtures.
- Fast/full verification results and any remaining limitations are recorded below before handoff.

## Execution log

- Four owned work packages implemented with failing-before regression evidence, then independently integrated. Existing user event-file/mobile extraction retained.
- Rust: shared core v1 DTO/limits; scalar-key dispatch; one non-coalescing gesture transaction; preserved live added-layer payload/history; centralized cancellation; clean owned RGBA export. Public additions: `onPointerCancel()` and `exportPixelBuffer()`.
- Web: viewport-only resize and crop confirmation; native-field paste ownership; named draft shelf with prepare/save/commit replacement and failure recovery; accessible/focus-preserving layer module; desktop/mobile clean PNG and interrupted-gesture handling.
- Harness: structured fail-closed audit/artifact checks, executed Node WASM targets, content fingerprints, locked inputs, production-shaped E2E, retained architecture/LOC/CI/build/skill fixtures. New controls run locally and directly in the CI jobs documented in ADR-047 and verify.
- Skills: local merge/rollback/QA procedures corrected, shared glossary/resources repaired, provenance-aware exceptions and local TypeScript adapter added; all nine locked upstream directories and skills-lock.json unchanged.
- Integration found two root npm High advisories. Targeted pnpm-10 lockfile update changed only brace-expansion 5.0.9 -> 5.0.12; frozen install and both audits passed, no overrides/ignores.
- Integration also reproduced pnpm 10/11 implicit-install mismatch twice. Both manifests and CI now pin 10.34.5, with four new failing-before/fixed-after coherence fixtures. No release-pin or live ruleset changes.
- Removed the remaining LOC exceptions: helpers.rs 948 -> 231, ui.ts 540 -> 321; user-refactored events.ts stays below 500. The linked responsive-grid guide now explicitly supersedes its destructive window-resize advice.

## Verification and remaining work

- Final `gate:fast`: PASS (`/tmp/recommendations-fast-final.log`). Final `gate:full`: PASS after dependency/pnpm and independent-review corrections (`/tmp/recommendations-full-final.log`).
- Rust: 168 library + 49 core integration + 31 layer integration + 2 utils + 2 doctests = 252 passing; native target's 0 WASM tests is expected and separately verified by the explicit WASM runner.
- Node WASM: 11 binding regressions + 11 registered integration tests = 22 passing, nonzero execution required independently for each target.
- Vitest: 59 passing across 8 files. Production Chromium: 105 passing. Additional production Firefox/WebKit: final **28/28 passing** across draft/document/gesture suites, including the paste/stroke regression (`/tmp/recommendations-cross-browser-final.log`).
- Retained harness checks: 20 sensor + 17 CI + 13 build + 4 Playwright-config + 28 skill fixtures. CI checker covers 33 path cases. Changed `ci.yml` actionlint and diff checks pass.
- Final optimized WASM: 228,742 bytes, validated and within 1.5 MiB budget. Both npm audits clean at High; cargo audit and deny pass (deny emits existing advisory-free license/duplicate warnings).
- Codacy repository intake: 0 High/Critical, one existing informational ShellCheck finding at the last analyzed remote head. The working tree was not pushed; this is not cloud validation of the new diff.
- Independent frontend review: no substantiated new blockers; actual-WASM in-memory quota/replacement probes passed. Independent Rust review: one blocker (accepted paste during unfinished stroke could lose paste/resurrect cancelled cells) fixed by a shared accepted-paste transaction boundary. Failing-before/fixed-after matrix covers internal/external paste × freehand/eraser × cancel/completion, rejected paste, exact history and clean snapshots; browser regression exercises actual paste routing. The reviewer also smoke-tested 20,000 structural transitions. All final gates above were repeated after the fix.
- No-op stroke test exposed blank erases consuming undo/discarding redo. Net-effect detection now avoids those entries. The independent-gesture fixture was strengthened by seeding content under both eraser gestures, retaining its two-entry/exact-restoration assertions.
- Final cross-browser correction was test-only: Firefox's synthetic ClipboardEvent did not retain data assigned to its constructor argument. An isolated three-engine probe confirmed this; the fixture now populates/asserts event.clipboardData itself. Full and cross-browser suites were rerun after correction, with no assertion weakened or browser skipped.

### Measured render baseline (not a performance gate)

Headless Chromium 153.0.8010.12, production preview, this local machine; 5 warmups +
40 samples, actual UI key dispatch and instrumentation around `renderToPixelBuffer`
and `putImageData`. Times are CPU-side API timings, not end-to-end FPS or GPU
completion, and are not portable device budgets. No render optimization claimed.

| Workload | Rust p95 ms | Pixel upload p50 / p95 ms | Submitted bytes/frame |
|----------|-------------|---------------------------|-----------------------|
| 240×80, 1 layer, one-cell text | 0.1 | 3.3 / 6.3 | 12,288,000 |
| 400×200, 1 layer, one-cell text | 0.1 | 11.3 / 15.6 | 51,200,000 |
| 240×80, 32 layers, one-cell text | 0.1 | 3.2 / 3.9 | 12,288,000 |
| 240×80, 1 layer, alternating zoom | 18.5 | 2.3 / 4.7 | 12,288,000 |

Single-cell runs produced 1 full + 44 dirty renders; zoom produced 45 full
renders. This justifies a separate measured dirty-upload/viewport-invalidation
change, not speculative optimization bundled into the safety fixes. Raw local
evidence: `/tmp/recommendations-render-baseline.json`. Desktop/mobile screenshots
were inspected; the drawer settles inside the viewport with scrollable content.

### Deliberate limitations and remaining work

- Drafts: max 20 / 4 million serialized shelf characters, potentially lower browser
  quota, fresh history/view on switching. Shelf-wide optimistic revision conflicts
  pause storage writes; users can continue editing/download, then reload. This is
  not cross-process CAS, collaboration, or a durable backup. Legacy autosave is
  retained as recovery bytes. Corrupt/denied initial storage fails closed.
- Scalar-key text handling is not an IME/grapheme-layout implementation.
- CI/ruleset/PR readiness are not remotely verified; no commit, push, merge or
  release was performed. ADR-046/047 were accepted by the user on 2026-09-30;
  PR review and remote merge gates remain outstanding.
- Unchanged `release.yml` has pre-existing actionlint findings, including a
  publish condition referencing `needs.guard-rails` without that dependency.
  Release wiring/pins and RC support need a separate reviewed change; the edited
  CI workflow passes actionlint. Do not describe repository-wide workflow lint as
  green on the basis of the CI-only check.
