# Follow-ups Backlog

**Updated**: 2026-10-04 (ISSUE-001 fixed by #250 / ADR-048; P2 dogfood pass recorded; #241–#248 merged).
**Single source of truth for active next work.**
Prior records below are dated history, not instructions to re-open shipped work.
Remote issues/releases have not been re-queried in this documentation pass.

## Active backlog

| Priority / ID | State | Next evidence or decision |
|---------------|-------|---------------------------|
| P0 / 2026-09-30 audit | **Shipped: #228–#233 + #241** | [Implementation evidence](recommendations-implementation-2026-09-30.md): the audit's editor-safety, drafts, palette, npm, R-09 and truthful-sensors pieces merged as #229/#230/#231/#228/#233/#241. Nothing in the audit row remains. |
| P0 / editing + local drafts | **Shipped: #229, #230, #231** | Keyboard/atomic-stroke/paste/session safety, clean PNG, accessible layers and bounded drafts merged with native/WASM/Vitest/browser regressions. No v1 format change, cloud accounts or collaboration; retain documented optimistic-conflict/storage/history limits. |
| P0 / R-09 LOC debt | **Shipped: #233** | The events/ui split landed on main and `.loc-allowlist` carries no oversized entries. The former working-tree remainder (file/mobile extraction, helpers/ui/events sizes) merged with #229/#230/#233. |
| P0 / truthful sensors + skills | **Shipped: #241** | Fail-closed sensors, nonzero WASM execution, production E2E, CI applicability/freshness/pnpm parity, and offline skill checks pass retained fixtures. Wiring is in both local runners and ci.yml. ADR-047 accepted; merge contract + pr-roast + Codacy all green on the final head. |
| P0 / root npm findings | **Shipped: #228** | Narrow root lockfile update `brace-expansion` 5.0.9 -> 5.0.12 fixed High GHSA-qhr7-859c-m2p7 / GHSA-6j4f-fj2g-mc7p; frozen install and both audits passed; no ignores/overrides. |
| P1 / release | **Not dispatched** | Release the next stable version only after a reviewed version-bump PR and the normal runbook. v0.1.4 is already released; never repeat its bump/changelog work. |
| P1 / release workflow validation | **Shipped: #243, commit `768cea0`** | `release.yml` passes `actionlint` cleanly: `publish-wasm` gained `guard-rails` in its `needs:` (the dry-run guard was reading an output that was never in the direct needs list), and the five ShellCheck findings (`SC2002` useless cat, `SC2086` unquoted `$GITHUB_OUTPUT` / `$GITHUB_PATH` / `$FIRST_VERSION_LINE`, `SC2034` dead `TAG_NAME`) are fixed. No release pin, toolchain SHA or version-validation logic changed. The **sensor** gap this surfaced is closed by the row below. |
| P1 / actionlint local sensor | **Shipped: #244, commit `2ab846e`** | `scripts/actionlint-check.py` closes the L-027 gap: fail-closed 0/1/2, pinned to 1.6.26 with a `--version` drift check, retained fixtures in `test-sensors.py` (9 new tests, suite now 39). Wired into `quality-gates.sh` full tier and a `Workflow lint parity (L-027)` step in the `architecture` CI job, with a checksum-verified pinned release download as the install step. `check-ci.py DIRECT['architecture']` and `ci-paths.json` updated; recorded as harness **L-027**. |
| P1 / RC workflow | **Unsupported / decision needed** | `release.yml` and `scripts/release.sh` accept only `x.y.z`. Implement/test prerelease policy separately before offering RC canary/promotion commands; use Deploy Preview shadow tests now. |
| P1 / Codacy coverage + repo intake in CI | **Credential-blocked — plan ready** | The full procedure, secret scopes and PR split are written up in [codacy-ci-integration-plan.md](codacy-ci-integration-plan.md). Blocked on `CODACY_PROJECT_TOKEN` (write, coverage) and `CODACY_API_TOKEN` (read-only, intake) as project-scoped repository secrets — `gh secret list` is empty, so neither the upload nor the intake sensor can be wired today. Do not conflate the local `codacy login` credential with a CI secret (L-016 shape). |
| P2 / dogfood + browser stability | **Done 2026-10-03 — 5 findings, 0 critical** | Production pass over `grand-kheer-862c39.netlify.app` (layers, SVG, themes, input/draft flows, mobile, tri-engine). Full report + probe scripts in `dogfood-output/` (local artefact, deliberately not committed — 1.2 MB of PNGs). **Confirmed working, not trusted:** reload persistence (`ascii-canvas-autosave`, 428 chars before == after), no horizontal overflow at 320/360/390/768 (`scrollWidth === innerWidth`), and **zero** page errors / failed requests across Chromium, Firefox and WebKit with byte-identical `exportSvg()` output (5658 chars) — so the "Firefox/WebKit flakes" premise found no flake. Three earlier claims were **withdrawn rather than carried**: the "text tool hangs the editor" was a pending native `confirm()` blocking CDP (the tool works), the "clipped sidebar labels" was the `screenshot --annotate` overlay covering its own badges, and box-drawing gaps turned out to be vertical-only. Findings became the three rows below. |
| P1 / dashed vertical box-drawing (dogfood ISSUE-001) | **Shipped: #250 (ADR-048)** | The 22 Unicode box-drawing glyphs the drawing tools emit are now synthesized as rectangles from one table (`src/render/box_drawing.rs`) that both renderers consume: the font atlas takes it at build time and in `update_glyph`, so the browser's 13px-in-20px raster can no longer re-enter, and `export_svg` emits `<rect>` scaled by cell pitch instead of `<text>`. Stem axes are the measured font axes (col 3 of 8, row 7 of 20), so nothing shifts sideways; `- \| + *` keep their glyphs because a dotted border is meant to break. Retained assertion: `e2e/box-drawing-continuity.spec.ts` (18/18 over Chromium/Firefox/WebKit/Pixel 5/iPhone 12/iPad Air), mutation-tested — removing only the atlas override gives `gaps: 13, longestGap: 5` while the horizontal control passes. **`.asc`, `exportAscii()`, clipboard and the WASM surface are unchanged; SVG output bytes for box runs are not** — shapes instead of text, so pre/post SVGs of one document are not byte-equal and box borders are no longer selectable in SVG. |
| P2 / `waitForRender` does not wait for a render | **Recorded, not queued** | Three specs (`e2e/canvas.spec.ts:10`, `e2e/responsive.spec.ts:4`, `e2e/tools-drawing.spec.ts:10`) carry hand-copied `waitForRender` helpers that only assert `canvas.width > 0` — i.e. the element has a backing store, not that a frame containing the current model has been presented. `render()` is scheduled on the next animation frame (`web/render.ts:95-99`), so a pixel-sampling assertion issued straight after an input reads the pre-release buffer. This cost #250 three engines of misleading failure before the profile began reporting `maxBrightness`: a probe that finds no ink at all is indistinguishable, in its own numbers, from a border broken once. Consolidate into `e2e/helpers.ts` as a paint-settle (two rAFs, as `web/main.ts:123` already does for font readiness) and make any canvas-sampling spec use it. |
| P1 / layer-name field collapses (dogfood ISSUE-002) | **Open — repro measured** | `.layer-name-input` is `flex: 1 1 0%` with `min-width: 0` against six non-shrinking icon buttons, so the name absorbs all layout pressure: 23px field for 74px of content on desktop (`Background` → 31% visible), and **8px** for 92px in the 390px drawer (8.7% visible). On mobile the layers list renders as unnamed icons, so with two or more layers a user cannot identify which layer they are about to move, merge or delete. Needs a width floor or a wrapping/icon-overflow layout, plus a mobile-viewport assertion that the name is legible. |
| P2 / GRID SIZE inputs stale after live resize (dogfood ISSUE-003) | **Open — repro deterministic** | The inputs sync at load only. Fresh load at 1280x720 agrees (`132x32` both); a **live** resize to 900x600 leaves INFO at `85x26` while the inputs still read `132/32`, and the adjacent **Apply** then silently forces the stale grid back onto the canvas. No content lost (ascii 461 -> 461) so the damage is a panel that misreports state next to a button that acts on the misreading. Also two low-severity items from the same pass, recorded not queued: light-theme status bar computes to 4.23:1 (under WCAG AA 4.5; dark theme is clean at >= 4.60), and destructive "Clear canvas" uses a native unthemed `confirm()` that blocks the main thread. |
| P2 / #199 do-harness | **Deferred** | Reassess only after upstream #235–#237 ship in a pinned release; start with a disposable-copy sensor audit. |
| P3 / F-30 collaboration | **Decision deferred** | Spike is complete; a prototype needs fresh scope/issue/ADR, not implicit inclusion in local drafts. |
| P3 / render performance | **Baseline measured; optimization deferred** | Production Chromium probe: one-cell 400×200 upload p95 15.6 ms / 51.2 MB submitted; 240×80 zoom still fully rerasterizes (Rust p95 18.5 ms). See the [implementation evidence](recommendations-implementation-2026-09-30.md#measured-render-baseline-not-a-performance-gate); benchmark same-machine before a separate dirty-upload/viewport change. No FPS improvement claimed. |

### Already shipped — do not re-plan

- **F-13 layer history**: #213, commit `1e639c9` + #216, commit `1d54c9b`
  (ADR-043). Not #212 — that PR closed **unmerged**; the layer code landed inside
  #213's `chore:` titled commit, which is L-011's own recorded failure.
- **R-06 loopback dev binding**: #224, commit `35726cb`.
- **R-08 npm audit inventory**: #225, commit `6628c70`; both npm lockfiles have
  a sensor. The fail-closed audit hardening ships with the sensors port.
- **L-019 Playwright-owned startup**: #226, commit `3ec1d93`.
- **L-020 shared LOC ratchet**: #227, commit `7e61d49`; CI has a dedicated `loc`
  job (not the older web-job copy described in historical notes).
- **2026-09-30 audit delivery**: #228 (npm), #229 (editor/document safety),
  #230 (bounded local drafts), #231 (palette a11y + keyboard layer controls),
  #232 (merge-gate commit scope, harness **L-021**), #233 (R-09 web split).
- **R-05 dependency pruning** and **R-07/Codacy rule-family parity**: #218–#223.
  These completions do not establish the current remote finding count.
- **L-026 Bandit parity for the Python sensors**: #242, commit `9efce87`.
  `scripts/bandit-check.py` closes the last rule-family gap Codacy enforced
  and nothing local did. Runs in the full tier and in the CI `architecture`
  job; fail-closed (missing bandit/uvx → 2). Threshold HIGH matches
  `codacy:check`; LOW/MEDIUM findings print as advisory so the 47 current
  subprocess/argv patterns the sensors structurally require do not need
  per-line suppressions.
- **Release-workflow lint fix**: #243, commit `768cea0`. `release.yml`
  actionlint-clean: `publish-wasm` gained the direct `guard-rails` needs
  edge (its dry-run `if:` was reading an output that GitHub does not
  expose transitively) and five ShellCheck findings fixed. No release pin,
  toolchain SHA, action version or version-validation logic touched.
- **L-027 actionlint parity for `.github/workflows/**`**: #244, commit
  `2ab846e`. `scripts/actionlint-check.py` mirrors `npm-audit.py` /
  `bandit-check.py`'s fail-closed contract (0/1/2), pinned to actionlint
  1.6.26 with a `--version` drift check. Every finding blocks — actionlint
  has no severity band mapping to Codacy's Critical/High, and the class it
  catches (needs-graph expression errors) shipped broken across several PRs
  before #243 caught `release.yml:310` by hand. Install step verifies the
  release tarball against `checksums.txt` via SHA-256 before extraction.
  Closes the L-017 rule-family gap for YAML/GHA.

## Historical triage and completion records

**Source**: Full recommendations bundle (issue #21 + post-merge analysis)
**Historical plan**: [full-recommendations-2026-07.md](full-recommendations-2026-07.md)
**Historical triage**: 2026-09-28 — R-07 is **done and closed in CI**: #219 added the `e2e/` lint + typecheck sensors locally, and the #220 follow-up (harness **L-016**) wired both into the blocking `web` CI job, so `e2e/` is now linted *and* gated. Cycle order now: (1) R-06 dev-server host binding, (2) #199 do-harness adoption deferred until upstream #235–#237 ship in a pinned release. F-13 layer-operation history already shipped (issue #207, ADR-043; #213 + #216). **R-03 was re-scoped after a verification swarm disproved the pin-by-override plan** (Vite pulls esbuild as an unused optional peer; the advisory is unreachable here) — see the R-03 row, R-05, and the corrected L-006. This pass also reconciled the planning docs against reality (test counts, issue states, release state, toolchain pin).

**2026-09-29 — Codacy parity landed.** The "11 open High issues" correction below is now closed: **#222** fixed every finding for real (one commit per finding, no suppressions — both patterns are locked to Codacy's *Default coding standard*), and one of them turned out to hide a **vacuous assertion** in `e2e/canvas.spec.ts` that had never evaluated its expectation. **#223** closed the two gaps that let them sit there: `eslint-plugin-security` now runs in both ESLint configs (**rule-family parity**), and `npm run codacy:check` reads the **repository-level** list, because Codacy's PR analysis is diff-scoped and a green check cannot see a backlog. A third gap: Codacy's Biome was running four `useQwik*` rules against a repo with no Qwik dependency, which blocked #222 with a High finding on a plain arrow function; `biome.json` fixes it. All recorded as harness **L-017** + **ADR-045**. `codacy:check` is an **agent procedure, not a gate** — it is not in `ci.yml`, because `gh secret list` is empty and a runner cannot see the machine-local `codacy login` credential. Repo-level backlog is now **0**. Codacy's coverage goal remains `None` and is **blocked pending a project-scoped `CODACY_PROJECT_TOKEN`**.

Use **Active backlog** above for prioritization. Keep these dated records for
provenance; their issue counts, test totals and next-step predictions are not a
fresh verification. Mirror major completions into `PROJECT_STATUS.md` only after
actual merge evidence.

---

## Completed release/security work (historical 2026-09-24–29)

| ID | Status | Issue | Notes |
|----|--------|-------|--------|
| **R-01** | ✅ done | — | 2026-09-24: **v0.1.4 released** — `VERSION` 0.1.4 propagated via `scripts/propagate-version.sh`, `scripts/release.sh` preflight OK, dry run + real run green, tag on the changelog branch, `[0.1.2]`/`[0.1.3]` curated + `[0.1.4]` generated and synced back to `main`. Runbook: [RELEASING.md](RELEASING.md) |
| **R-02** | ✅ resolved | — | 2026-09-24: `release.yml` anchors the notes range to the latest GitHub Release tag (with a `git describe` fallback). `v0.1.3` is not an ancestor of `main`, so the old behaviour re-listed 217 commits instead of the 27 unreleased ones |
| **R-03** | ✅ resolved | [security/dependabot/11](https://github.com/d-o-hub/rust-ascii-canvas/security/dependabot/11) | Dev-only esbuild advisory GHSA-g7r4-m6w7-qqqr. **Swarm finding (2026-09-25), independently verified**: `vite@8.3.0` has **no** esbuild dependency — only `peerDependencies.esbuild: ^0.27.0 \|\| ^0.28.0` with `optional: true`; nothing in `web/` **calls** esbuild; the advisory (CVSS 2.5, Windows-only) hits esbuild's *own* dev server, which this repo never runs. The earlier **pin-by-override plan is superseded**: an override alone silently *drops* esbuild instead of pinning it (verified on pnpm 10.34.5 and 11.7.0), and a direct devDependency would add a package we never call. **Closed 2026-09-28 by R-05**: esbuild is no longer installed on either pnpm major (2 surviving lockfile references are vite's `peerDependencies` *declaration*, not an installed package), so the alert auto-closes on merge |
| **R-04** | ✅ done | — | 2026-09-25: the `Publish WASM` job downloads the release build and attaches `ascii-canvas-<version>.wasm` with `--clobber` (idempotent re-runs). The artifact is **copied to a versioned name first** — `gh release upload`'s `file#label` syntax only sets a display label (verified against a scratch draft), so the download filename would otherwise remain `ascii_canvas_bg.wasm`. v0.1.4 has no asset; v0.1.5 is the first with one |
| **R-05** | ✅ done | — | **Dependency refresh** (closed alert #11), 2026-09-28: re-resolved `web/pnpm-lock.yaml` with **pnpm 10.34.5** (CI's major) and dropped `pnpm.ignoredBuiltDependencies` in the same change, as the plan required. **Prune confirmed in the installed tree, not just the lockfile**: esbuild 91 → 2 references, jsdom 7 → 2, `@esbuild/*` platform packages 78 → 0, total packages 252 → 186; nothing esbuild/jsdom-shaped under `node_modules/.pnpm` on **either** major. Surviving references are vite/vitest `peerDependencies` *declarations*. Bumps reviewed and all **patch/minor within existing ranges**: vite 8.3.0 → 8.3.1, vitest 5.0.1 → 5.0.2, rolldown 1.2.8 → 1.2.11, `@types/node` 26.5.1 → 26.6.3, `brace-expansion` 5.0.12, `ignore` 7.0.10, `magic-string` 1.4.2, `tinybench` 6.2.0, `tinyexec` 1.3.1, `ws` 8.22.0, `why-is-node-running` 3.2.2, `@oxc-project/types` 0.151.0. **Correction to the plan's prediction**: the `whatwg-mimetype` "5.0.0 → 3.0.0 downgrade" is not a downgrade of a package we use — 5.0.0 was reachable only through jsdom's `data-urls`, so it is *pruned*; 3.0.0 (the copy `@types/whatwg-mimetype` wants) is what remains. Verified: `pnpm install --frozen-lockfile` green on **pnpm 10.34.5 and 11.7.0**, ESLint + `tsc --noEmit` clean, Vitest 29/29 on 5.0.2, `vite build` OK, Playwright chromium **91/91**, `gate:full` green (incl. cargo-audit, cargo-deny, WASM 220 kB). `ERR_PNPM_IGNORED_BUILDS` no longer appears, dissolving L-006's trigger. Trap hit while doing it: a deleted lockfile is not a fresh resolve (harness **L-012**) |
| **R-06** | ✅ done | — | 2026-09-29: `server.host: true` (0.0.0.0) → **`host: '127.0.0.1'`** plus **`strictPort: true`**, with **`pnpm run dev:lan`** (`vite --host`) as the explicit opt-in, wired at the root and in `web/` and documented in CONTRIBUTING (incl. the WSL2 caveat). Driven by evidence, not taste: three 2026 Vite advisories (GHSA-v2wj-q39q-566r, GHSA-p9ff-h696-f583, CVE-2026-53571) all name *"exposes the dev server to the network"* as a precondition, and this repo (Vite 8.3.1) was holding it open. `127.0.0.1` over `localhost` because Vite binds whichever address DNS returns first (::1 on IPv6-preferring hosts) while the e2e sensors and Playwright `baseURL` use `localhost`. `strictPort` because Vite otherwise moves to the next free port — a **19-hour-old stale dev server was found holding 127.0.0.1:3004** while the sensors polled 3003. Verified by observation: `ss` shows `127.0.0.1:3003` with no `Network:` banner line and the LAN IP refused; `dev:lan` binds `*:3003` and returns 200 on the LAN IP; an occupied 3003 exits 1 without ever binding 3004; Playwright 91/91 |
| **R-07** | ✅ done (incl. L-016 follow-up #220) | — | **Codacy repo-level backlog** (2026-09-28, 41 open issues → 0 actionable). Split by cause, not by count. (1) **10 × `third-party-action-not-pinned-to-commit-sha`** — all third-party actions SHA-pinned with `# vX.Y.Z` comments; `dtolnay/rust-toolchain` additionally got an explicit `toolchain: stable` input because it infers the toolchain **from the `@ref`** and a SHA pin would otherwise install nothing (`actions/*` are GitHub-owned and out of the rule's scope). Every SHA round-tripped through `commits/<sha>` first — 3 of 7 initial lookups were **tag objects, not commits** (harness **L-015**) and would have broken CI. (2) **17 × e2e/ ESLint** — root cause was that `e2e/` was *never linted locally* (`cd web && eslint .` cannot see outside `web/`, and there was no `e2e/tsconfig.json`), so `gate:fast` was green while Codacy reported 17 issues. #219 added `eslint.config.mjs` + `e2e/tsconfig.json` and wired both into `quality-gates.sh` (`ESLint (root)`, `TypeScript (e2e)`), then fixed the code for real: `querySelector<HTMLCanvasElement>` instead of a lying `as` cast, 7 `@ts-ignore` + 5 `as any` replaced by a typed `Window.editor` and a `requireAsciiContent` helper, 2 duplicated helpers deleted (harness **L-014**). **Post-merge roast found #219's sensors were local-only** — no CI job runs `quality-gates.sh` — so #220 wired both into the blocking `web` CI job (`Install root dependencies` + `ESLint (root — covers e2e/)` + `TypeScript (e2e)` via `web/node_modules/.bin/tsc`, since `typescript` is not a root dep and clean-install `pnpm exec tsc` exits 254) and recorded harness **L-016** ("the gate script is not a gate"). (3) **11 × vendored Python** in `.agents/skills/**` — excluded via a new `.codacy.yml` `exclude_paths` (upstream-synced code; edits would be overwritten). (4) **3 × object-injection** on `TOOL_INFO[tool]` — fixed at the root by typing `TOOL_INFO` as a closed record, so a typo like `'rect'` is now a compile error rather than an `undefined` lookup. **No finding was closed by suppression.** **CORRECTION 2026-09-29**: the "41 → 0 actionable" claim above was wrong. Codacy's **PR analysis is diff-scoped** — `codacy -o json pull-request gh d-o-hub rust-ascii-canvas 220` reported `newIssues: 0` / `isUpToStandards: true` on a PR whose predecessor still carried a repo-level backlog, so a PR-scoped read can never see pre-existing issues. `codacy -o json issues` on 2026-09-29 returned **11 open High findings** (4 × `security_detect-unsafe-regex`, 7 × `security_detect-object-injection`) in `e2e/tools-drawing.spec.ts`, `web/ux.test.ts`, `e2e/pages/EditorPage.ts` and `e2e/canvas.spec.ts`. Both patterns are enabled by Codacy's **Default coding standard** and cannot be disabled or configured, so the code had to change. Second blind spot, same shape as L-013/L-014/L-016: **neither local ESLint config enables `eslint-plugin-security` at all**, so the entire `security` rule family was invisible to `gate:fast` and to the `web` CI job. Closed by the 2026-09-29 fix branch (9 commits; the repaired `canvas.spec.ts` assertion had been vacuous) — see harness **L-017** for the intake/parity rules that follow. |

---

## P0 — Ship / close the loop (complete)

| ID | Status | Issue | Notes |
|----|--------|-------|--------|
| **F-01** | ✅ | — | [PR #107](https://github.com/d-o-hub/rust-ascii-canvas/pull/107) merged; #21 closed |
| **F-02** | ✅ | [#108](https://github.com/d-o-hub/rust-ascii-canvas/issues/108) | Manual clipboard QA verified across Notepad, TextEdit, VS Code, and monospace area |
| **F-03** | ✅ | [#109](https://github.com/d-o-hub/rust-ascii-canvas/issues/109) | Closed 2026-07-16; coordinated wasm-bindgen 0.2.128 + pin-parity sensor completed 2026-09-17 (ADR-042). Web-deps PRs #188/#190 closed/merged; only R-03 remains (low, dev-only, re-scoped into R-05) |

---

## P1 — Product (complete)

| ID | Status | Issue | Notes |
|----|--------|-------|--------|
| **F-10** | ✅ | [#110](https://github.com/d-o-hub/rust-ascii-canvas/issues/110) | SVG export: `web/exportSvg.ts`, `src/wasm/render_api.rs` `export_svg`, unit + UI wiring |
| **F-11** | ✅ | [#111](https://github.com/d-o-hub/rust-ascii-canvas/issues/111) | Layer editor: rename, visible, lock, reorder (`moveLayer`), delete, merge (`mergeLayerDown`); UI in `web/ui.ts` |
| **F-12** | ✅ | — | Composite pixel render + export (#107) |
| **F-13** | ✅ | [#207](https://github.com/d-o-hub/rust-ascii-canvas/issues/207) | **Shipped** — layer ops recorded in `History` per [ADR-043](ADRs/043-layer-command-history.md) (merged in #213, hardened in #216 to address layers by stable id rather than position). Covered by `e2e/layers.spec.ts`; issue closed 2026-09-28 |
| **F-14** | ✅ | [#112](https://github.com/d-o-hub/rust-ascii-canvas/issues/112) | Enhanced text tool — cursor indicator (`web/render.ts:updateCursorIndicator`), multi-line polish (ADR-010) |
| **F-15** | ✅ | [#113](https://github.com/d-o-hub/rust-ascii-canvas/issues/113) | Preview rendering style (ADR-011; `preview_ops` in `render_api.rs`) |
| **F-16** | ✅ | [#114](https://github.com/d-o-hub/rust-ascii-canvas/issues/114) | Eraser radius 1/3/5 (`events.ts` + e2e) |
| **F-17** | ✅ | [#115](https://github.com/d-o-hub/rust-ascii-canvas/issues/115) | External paste / plain-ASCII import (e2e `clipboard.spec.ts` paste test) |

---

## P2 — Quality & engineering (complete)

| ID | Status | Issue | Notes |
|----|--------|-------|--------|
| **F-20** | ✅ | [#116](https://github.com/d-o-hub/rust-ascii-canvas/issues/116) | `web/main.ts` split → **230 LOC** (events/render/ui modules) |
| **F-21** | ✅ | [#117](https://github.com/d-o-hub/rust-ascii-canvas/issues/117) | CI E2E matrix: chromium + firefox + webkit (`.github/workflows/ci.yml:319`) |
| **F-22** | ✅ | [#118](https://github.com/d-o-hub/rust-ascii-canvas/issues/118) | WASM-generated TS types (ADR-033; `web/pkg/*.d.ts`) |
| **F-23** | ✅ | [#119](https://github.com/d-o-hub/rust-ascii-canvas/issues/119) | ADR 001–036 status audit (commit `863146e`) |
| **F-24** | ✅ | [#120](https://github.com/d-o-hub/rust-ascii-canvas/issues/120) | `wasm-opt` required in CI, local install documented (`scripts/build-wasm.sh`) |
| **F-25** | ✅ | [#121](https://github.com/d-o-hub/rust-ascii-canvas/issues/121) | `#![allow(missing_docs)]` removed from wasm modules (grep clean) |
| **F-26** | ✅ | — | `openEditor()` migration largely done in #107 |
| **F-27** | ✅ | [#122](https://github.com/d-o-hub/rust-ascii-canvas/issues/122) | Clipboard `readText` geometry assertions (`e2e/clipboard.spec.ts`, chromium) |
| **F-28** | ✅ | [#123](https://github.com/d-o-hub/rust-ascii-canvas/issues/123) | Dirty-rect pixel buffer (ADR-028; `src/render/dirty_rect.rs`) |

---

## P3 — Stretch / future (complete)

| ID | Status | Issue | Notes |
|----|--------|-------|--------|
| **F-30** | ✅ spike | [#124](https://github.com/d-o-hub/rust-ascii-canvas/issues/124) | Research spike complete (`plans/F-30-collaborative-editing-spike.md`); prototype decision = new work |
| **F-31** | ✅ | [#125](https://github.com/d-o-hub/rust-ascii-canvas/issues/125) | Light theme + switcher (`web/ui.ts` `data-theme`) |
| **F-32** | ✅ | [#126](https://github.com/d-o-hub/rust-ascii-canvas/issues/126) | Mobile UX audit; drawer Escape + focus restore in #195 |
| **F-33** | ✅ wontfix | [#127](https://github.com/d-o-hub/rust-ascii-canvas/issues/127) | App-only distribution chosen; documented in ADR-040 |

---

## Candidate-list reconciliation (2026-09-30)

The former duplicate next-work table is consolidated into **Active backlog**.
Its R-06-next and F-13-residual claims were stale: both shipped. The skill-budget
work is now an unmerged ownership-aware checker/adapter candidate, not a license
to edit locked upstream skills. Historical LOC pins describe pre-extraction debt,
not current file sizes or permission to increase `.loc-allowlist` budgets.

---

## Do not re-open

| When | Item |
|------|------|
| 2026-03-18 | Issue **#9** pixel buffer + putImageData — [PR #11](https://github.com/d-o-hub/rust-ascii-canvas/pull/11); keep closed |
| 2026-07-16 | Issue **#21** copy-paste fidelity — [PR #107](https://github.com/d-o-hub/rust-ascii-canvas/pull/107) merged |
| 2026-07-16 | Persistence, PNG, grid UI, basic layers, composite render, Codacy-clean CI |
| 2026-09-22 | Jules palette PRs **#181/#185/#191** — superseded by the rebased merges #184/#194/#195; do not reopen |

---

## Related

- [PROJECT_STATUS.md](PROJECT_STATUS.md)
- [RELEASING.md](RELEASING.md)
- [goal-state.md](goal-state.md)
- Implementation issues: #207 (F-13 layer history — **shipped** #213 + #216, closed 2026-09-28), #199 (do-harness adoption — open, deferred)
- [Harness steering log](../agents-docs/harness.md#learned-failure-modes-steering-log) — the log itself (L-001…L-027 as of 2026-10-03) is the SSOT; this row intentionally does not enumerate a subset, because every prior attempt to keep a subset in sync drifted (L-011's own rule).
- ADR-047 (fail-closed sensors + retained coherence fixtures), ADR-046 (editing safety + named local drafts), ADR-045 (local sensor parity with required checks), ADR-044 (merge automation + delivery loop), ADR-043 (layer command history), ADR-042 (wasm-bindgen pin parity), ADR-041 (clipboard export modes), ADR-036 (clipboard fidelity + product features)
- Issues: [#108](https://github.com/d-o-hub/rust-ascii-canvas/issues/108)–[#127](https://github.com/d-o-hub/rust-ascii-canvas/issues/127) (all closed)
