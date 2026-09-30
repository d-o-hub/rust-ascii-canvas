# Editor-safety delivery — 2026-09-30

## Goal and boundaries

Deliver the safety subset of [ADR-046](ADRs/046-editing-safety-and-local-drafts.md)
from an isolated branch based on `origin/main` at `7e61d49`. The source development
worktree is read-only. Initial extraction/verification performed no commits,
staging, pushes or remote mutations; the parent delivery phase follows below.
Acceptance is not merge/release evidence.

The existing pure-core → render/UI → WASM → web architecture remains intact.

## Scope and decisions

- Core/WASM: distinguish printable Unicode scalars from browser key names; keep
  literal Space in active Text; atomic non-coalescing freehand/eraser strokes;
  cancel provisional work at interaction/document/layer boundaries; preserve
  exact undo/redo and added-layer payload/history. No-op strokes neither consume
  undo nor discard redo. Accepted external/internal paste cancels provisional
  cells before recording originals; empty/rejected paste leaves the stroke alone.
- Extract the existing `.asc` v1 DTO and validation into
  [`src/core/document.rs`](../src/core/document.rs), retaining import compatibility.
  Apply shared dimension/layer limits to creation, resize and history replay.
  Failed loads retain the existing document/session/history.
- Export committed visible pixels through an additive WASM API and wire both
  existing PNG buttons to it; never export selection/preview overlays or viewport
  clipping. Keep export immutable.
- Browser: window/layout resize is presentation only; manual shrinking requires
  explicit crop confirmation; editable fields retain native paste; cancel on
  pointercancel, lost capture, touchcancel or blur without cancelling completed
  text taps. Layer-limit refusal is visible instead of claiming success.
- Retain base `web/events.ts` event-result processing and base `main.ts`,
  `persistence.ts`, `ui.ts`, HTML and styling. A narrow
  [`web/document-events.ts`](../web/document-events.ts) extraction owns paste/crop
  boundaries and uses the existing state autosave callback, avoiding a cycle.
  Do not import the foreign `eb3ce8f` event-result extraction or draft-dependent
  source modules. Keep `web/events.ts=850` and `web/ui.ts=540` LOC budgets unchanged;
  remove only the stale `src/wasm/helpers.rs` entry after real extraction.
- Excluded: named drafts and their UI/status/storage/migration/conflicts, layer
  panel accessibility rewrite, full harness/skills overhaul, release changes and
  remote/ruleset mutations. ADR-046's draft decision is accepted but deferred.

## Shared prerequisite

Root [`pnpm-lock.yaml`](../pnpm-lock.yaml) is copied byte-for-byte from the source's
patched lockfile (brace-expansion 5.0.12). It is an explicit **SHARED prerequisite**
with security PR [#228](https://github.com/d-o-hub/rust-ascii-canvas/pull/228),
commit `fc94995`, carried separately from the editor-safety commit. No dependency
manifests or web lockfile resolutions change. Merge #228 first, then rebase this
branch onto updated main to remove the already-landed prerequisite from its PR
diff and rerun remote checks. Neither slice claims independent dependency-fix
ownership. Both frozen installs use pnpm 10.34.5.

## Minimum executable verification wiring

The narrow test-discovery portion of accepted
[ADR-047](ADRs/047-fail-closed-verification-and-coherence.md) is included:

- [`Cargo.toml`](../Cargo.toml) registers `tests/wasm/mod.rs` as `wasm_tests`.
- Library binding regressions use `wasm_bindgen_test`; public binding tests no
  longer request a browser for APIs that need only Node. Existing incorrect dirty
  redraw expectations use the actual consuming API (`getDirtyRenderCommands`).
- [`scripts/test-wasm.sh`](../scripts/test-wasm.sh) runs **each** of `--lib` and
  `--test wasm_tests` in Node and fails on zero passing tests. The existing
  `.cargo/config.toml` runner and pinned mise toolchain are sufficient.
- `npm run test:wasm`, local `gate:full`, and the existing CI `rust` job invoke the
  same script. CI does not run `quality-gates.sh`; direct workflow wiring was
  inspected. No required-check set or aggregator is altered.

This does not claim ADR-047's audit/artifact/freshness/skill/coherence or production
server overhaul shipped. Base full E2E still uses the Playwright-owned dev server
on port **3003**. Port 3005 belongs to the independent security worker.

## Actions and evidence

1. Inspect base/source boundaries and preserve excluded work — complete.
2. Extract implementation and focused core/WASM/Vitest/Playwright regressions — complete.
3. Install frozen root/web dependencies, build fresh local `web/pkg`, run focused
   checks, then `gate:fast` and `gate:full` on this isolated tree — complete.
4. Run focused Firefox/WebKit safety integration, review the final diff, record
   exact results and stop any owned servers — complete. Parent review remains.

Evidence logs are stored outside the tree at
`/tmp/ascii-canvas-delivery-20260930/logs/safety-*.log`; the results below describe
this branch's execution, never the source superset's gates. Commands use
`PATH=/home/do/.npm/_npx/381139ee5d646d31/node_modules/.bin:$PATH` (pnpm 10.34.5).

### Initial evidence

- `pnpm install --frozen-lockfile` at root and in `web/`: exit 0, pnpm 10.34.5
  (`safety-install.log`).
- Before adapting base browser events,
  `cd web && pnpm exec vitest run gesture.test.ts`: **4 failed / 2 passed**, exit 1;
  base did not invoke cancellation on pointercancel/lost capture/touchcancel/blur
  (`safety-gesture-before.log`). This is a real failing-before regression, not
  inferred from source tests.

### Completed isolated verification

| Command / check | Result | Log suffix (`safety-…log`) |
|---|---|---|
| `cargo test --lib` | 168 passed | `rust-focused.` |
| `npm run build:wasm` | Fresh local bindings, wasm-opt optimized | `rust-focused.` |
| `bash scripts/test-wasm.sh` | 11 library + 11 registered integration tests passed in Node | `rust-focused.`, `full.` |
| `cd web && pnpm exec vitest run safety.test.ts gesture.test.ts exportPng.test.ts` | 12 passed | `web-focused.` |
| `cargo test --all --locked` | 250 native tests + 2 doctests passed; native `wasm_tests` has zero by design, not counted as binding evidence | `unit-counts.` |
| `cd web && pnpm test` | 41 tests in 6 files passed | `unit-counts.` |
| `pnpm exec playwright test --project=chromium e2e/document-safety.spec.ts e2e/gesture-safety.spec.ts e2e/input-safety.spec.ts` | 10 passed | `chromium-focused.` |
| Same three files with `--project=firefox --project=webkit` | 20 passed | `cross-browser.` |
| `npm run gate:fast` | Exit 0: Rust, architecture/LOC, web/root lint, both TS scopes, Vitest, privacy/secrets | `fast.` |
| `npm run gate:full` | Exit 0: above + cargo audit/deny, both npm lockfiles, nonzero Node WASM, optimized size, full Chromium E2E | `full.` |
| `pnpm exec playwright test --project=chromium --list` | 101 tests in 8 files; full gate executes this Chromium inventory | `e2e-inventory.` |
| Zero-test/success/nonzero-exit runner fixtures | 3/3 expected exit statuses; synthetic evidence about the script, not WASM behavior | `wasm-runner-fixtures.` |
| Scope/history/index/link checks | No foreign commits or staged files; excluded paths unchanged; local doc links valid | `scope-review.` |

The optimized WASM artifact is **217,411 bytes** (1.5 MiB budget). The full gate's
actual npm audit checked **both** lockfiles with no High findings. The gate retains
base sensor implementations; a green run here is not a claim that the deferred
fail-closed harness overhaul exists.

Additional production-shaped evidence: `npm run build:web` built the base
configuration's root `dist/`, followed by all **30** focused safety tests across
Chromium/Firefox/WebKit against that production build. A one-shot, ignored
`target/safety-preview.config.ts` extends the base Playwright config, uses strict
`vite preview` on port 3003, and lets Playwright own startup/teardown. Command:

```bash
npm run build:web
pnpm exec playwright test -c target/safety-preview.config.ts \
  --project=chromium --project=firefox --project=webkit \
  e2e/document-safety.spec.ts e2e/gesture-safety.spec.ts e2e/input-safety.spec.ts
```

Log: `safety-production-focused.log`. This is additional local evidence, **not** a
new production CI/full-gate promise. No manual background server remains.

### Review boundaries and remaining work

- Scope review confirms all extracted `src/` files are byte-identical to source;
  the source itself was never edited. Base `main.ts`, persistence, UI, HTML,
  styling, web dependencies and Playwright config are unchanged. No
  `eventResult.ts` or draft modules were imported.
- `helpers.rs` is 231 lines; `events.ts` shrank 850 → 819; `ui.ts` stays 540.
  Only the helpers allowlist entry was removed. No budget was raised or repinned.
- The shared root lockfile must be split into the parent's identical prerequisite
  commit before independently delivering safety/security. ADR-047 is recorded
  here with **test-wiring-only partial implementation**; if another delivery also
  adds this ADR, reconcile status text without claiming its other decisions ship
  in this branch.
- Codacy repository intake reported **one existing Warning**:
  `shellcheck_SC2034`, `scripts/quality-gates.sh:21` (unused pre-existing
  `LOC_ALLOWLIST_FILE`). Zero High/Critical were reported. This remote backlog
  read is not analysis of the unpushed candidate; no PR analysis/coverage or
  merge readiness is claimed. The unrelated shellcheck cleanup was not bundled.
- Firefox's dev-server log contained `Auto-save failed: {}` warnings near test
  teardown; all 20 Firefox/WebKit safety assertions passed. Existing single-slot
  persistence was intentionally retained, and these tests do not certify storage
  durability or the deferred draft failure UX. Production focused tests also pass.
- The preparation phase performed no staging, commit, push, PR or remote mutation;
  parent delivery follows separately. No product/test blocker was observed by
  the isolated verification above.

## Parent delivery review

- Verified the exact Rust source is byte-identical to the previously independently
  reviewed implementation, including the paste/stroke and no-op-history fixes.
  Reviewed the smaller base-event adaptation, crop/paste module, immutable PNG
  bridge and minimal nonzero WASM runner/CI wiring; no new Blocker/Major found.
- Corrected the linked responsive-grid guide in this same slice: its old rule
  explicitly required the destructive window-resize behavior these tests reject.
  This is a behavior-specific guide correction, not the deferred skills overhaul.
- Commits/push and current-head CI/Codacy results are tracked in the PR body;
  local evidence above is not a claim of remote success or merge readiness.
- Deferred harness observation: base `pr-merge-gate.sh --json` prints commit-scope
  prose before its JSON document. Its exit status/verdict are usable, but stdout
  is not pure JSON. Fix stdout/stderr ownership in the separate harness delivery,
  not by silently changing merge predicates in this safety PR.
