# Follow-ups Backlog

**Updated**: 2026-09-23
**Source**: Full recommendations bundle (issue #21 + post-merge analysis)
**Primary plan**: [full-recommendations-2026-07.md](full-recommendations-2026-07.md)
**Latest triage**: 2026-09-25 — **next cycle planned by a read-only agent swarm + critic**: v0.1.4 shipped and R-04 done. Cycle order: (1) F-13 layer-operation history under [ADR-043](ADRs/043-layer-command-history.md) and issue #207, (2) R-05 dependency refresh, (3) R-06 dev-server host binding, (4) #199 do-harness adoption deferred until upstream #235–#237 ship in a pinned release. **R-03 was re-scoped after a verification swarm disproved the pin-by-override plan** (Vite pulls esbuild as an unused optional peer; the advisory is unreachable here) — see the R-03 row, R-05, and the corrected L-006. This pass also reconciled the planning docs against reality (test counts, issue states, release state, toolchain pin).

Use this list for prioritization. Mark items done in-place and mirror major completions into `PROJECT_STATUS.md`.  
**GitHub issues** track open work (numbers below).

---

## Next — release + new work (2026-09-23)

| ID | Status | Issue | Notes |
|----|--------|-------|--------|
| **R-01** | ✅ done | — | 2026-09-24: **v0.1.4 released** — `VERSION` 0.1.4 propagated via `scripts/propagate-version.sh`, `scripts/release.sh` preflight OK, dry run + real run green, tag on the changelog branch, `[0.1.2]`/`[0.1.3]` curated + `[0.1.4]` generated and synced back to `main`. Runbook: [RELEASING.md](RELEASING.md) |
| **R-02** | ✅ resolved | — | 2026-09-24: `release.yml` anchors the notes range to the latest GitHub Release tag (with a `git describe` fallback). `v0.1.3` is not an ancestor of `main`, so the old behaviour re-listed 217 commits instead of the 27 unreleased ones |
| **R-03** | ✅ resolved | [security/dependabot/11](https://github.com/d-o-hub/rust-ascii-canvas/security/dependabot/11) | Dev-only esbuild advisory GHSA-g7r4-m6w7-qqqr. **Swarm finding (2026-09-25), independently verified**: `vite@8.3.0` has **no** esbuild dependency — only `peerDependencies.esbuild: ^0.27.0 \|\| ^0.28.0` with `optional: true`; nothing in `web/` **calls** esbuild; the advisory (CVSS 2.5, Windows-only) hits esbuild's *own* dev server, which this repo never runs. The earlier **pin-by-override plan is superseded**: an override alone silently *drops* esbuild instead of pinning it (verified on pnpm 10.34.5 and 11.7.0), and a direct devDependency would add a package we never call. **Closed 2026-09-28 by R-05**: esbuild is no longer installed on either pnpm major (2 surviving lockfile references are vite's `peerDependencies` *declaration*, not an installed package), so the alert auto-closes on merge |
| **R-04** | ✅ done | — | 2026-09-25: the `Publish WASM` job downloads the release build and attaches `ascii-canvas-<version>.wasm` with `--clobber` (idempotent re-runs). The artifact is **copied to a versioned name first** — `gh release upload`'s `file#label` syntax only sets a display label (verified against a scratch draft), so the download filename would otherwise remain `ascii_canvas_bg.wasm`. v0.1.4 has no asset; v0.1.5 is the first with one |
| **R-05** | ✅ done | — | **Dependency refresh** (closed alert #11), 2026-09-28: re-resolved `web/pnpm-lock.yaml` with **pnpm 10.34.5** (CI's major) and dropped `pnpm.ignoredBuiltDependencies` in the same change, as the plan required. **Prune confirmed in the installed tree, not just the lockfile**: esbuild 91 → 2 references, jsdom 7 → 2, `@esbuild/*` platform packages 78 → 0, total packages 252 → 186; nothing esbuild/jsdom-shaped under `node_modules/.pnpm` on **either** major. Surviving references are vite/vitest `peerDependencies` *declarations*. Bumps reviewed and all **patch/minor within existing ranges**: vite 8.3.0 → 8.3.1, vitest 5.0.1 → 5.0.2, rolldown 1.2.8 → 1.2.11, `@types/node` 26.5.1 → 26.6.3, `brace-expansion` 5.0.12, `ignore` 7.0.10, `magic-string` 1.4.2, `tinybench` 6.2.0, `tinyexec` 1.3.1, `ws` 8.22.0, `why-is-node-running` 3.2.2, `@oxc-project/types` 0.151.0. **Correction to the plan's prediction**: the `whatwg-mimetype` "5.0.0 → 3.0.0 downgrade" is not a downgrade of a package we use — 5.0.0 was reachable only through jsdom's `data-urls`, so it is *pruned*; 3.0.0 (the copy `@types/whatwg-mimetype` wants) is what remains. Verified: `pnpm install --frozen-lockfile` green on **pnpm 10.34.5 and 11.7.0**, ESLint + `tsc --noEmit` clean, Vitest 29/29 on 5.0.2, `vite build` OK, Playwright chromium **91/91**, `gate:full` green (incl. cargo-audit, cargo-deny, WASM 220 kB). `ERR_PNPM_IGNORED_BUILDS` no longer appears, dissolving L-006's trigger. Trap hit while doing it: a deleted lockfile is not a fresh resolve (harness **L-012**) |
| **R-06** | open | — | `web/vite.config.ts:16` sets `server.host: true`, binding the dev server to `0.0.0.0` (LAN-visible). Decide the default (loopback) and document the opt-in for device testing; independent of R-03 |
| **R-07** | ✅ done | — | **Codacy repo-level backlog** (2026-09-28, 41 open issues → 0 actionable). Split by cause, not by count. (1) **10 × `third-party-action-not-pinned-to-commit-sha`** — all third-party actions SHA-pinned with `# vX.Y.Z` comments; `dtolnay/rust-toolchain` additionally got an explicit `toolchain: stable` input because it infers the toolchain **from the `@ref`** and a SHA pin would otherwise install nothing (`actions/*` are GitHub-owned and out of the rule's scope). Every SHA round-tripped through `commits/<sha>` first — 3 of 7 initial lookups were **tag objects, not commits** (harness **L-015**) and would have broken CI. (2) **17 × e2e/ ESLint** — root cause was that `e2e/` was *never linted locally* (`cd web && eslint .` cannot see outside `web/`, and there was no `e2e/tsconfig.json`), so `gate:fast` was green while Codacy reported 17 issues. Added `eslint.config.mjs` + `e2e/tsconfig.json` and wired both into `quality-gates.sh` (`ESLint (root)`, `TypeScript (e2e)`), then fixed the code for real: `querySelector<HTMLCanvasElement>` instead of a lying `as` cast, 7 `@ts-ignore` + 5 `as any` replaced by a typed `Window.editor` and a `requireAsciiContent` helper, 2 duplicated helpers deleted (harness **L-014**). (3) **11 × vendored Python** in `.agents/skills/**` — excluded via a new `.codacy.yml` `exclude_paths` (upstream-synced code; edits would be overwritten). (4) **3 × object-injection** on `TOOL_INFO[tool]` — fixed at the root by typing `TOOL_INFO` as a closed record, so a typo like `'rect'` is now a compile error rather than an `undefined` lookup. **No finding was closed by suppression.** |

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
| **F-13** | ✅ | [#207](https://github.com/d-o-hub/rust-ascii-canvas/issues/207) | **Shipped** — layer ops recorded in `History` per [ADR-043](ADRs/043-layer-command-history.md) (merged in #212, hardened in #216 to address layers by stable id rather than position). Covered by `e2e/layers.spec.ts`; issue closed 2026-09-28 |
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

## New work candidates (need issues)

Open implementation issues: **#207** (F-13, next) and **#199** (do-harness adoption, deferred). Candidates for the cycle:

| Candidate | Why | First step |
|-----------|-----|------------|
| Layer-operation undo/history (**F-13**) | Layer ops bypass history (verified 2026-09-23); issue #111 closed without it | **Next**: core `LayerStack` + `LayerCommand` per [ADR-043](ADRs/043-layer-command-history.md), issue #207 |
| `web/events.ts` 850 LOC + `web/ui.ts` 540 LOC (> 500 guideline) + LOC sensor gap | Sensor scans only `src/**/*.rs`; `web/` growth is unchecked | Either extract modules or extend the LOC sensor (ADR), then keep gate honest |
| **Skill length budget** — `skill-creator` (357), `typescript-expert` (432), `rust-wasm` (416) exceed the 300-line `SKILL.md` budget in `agents-md` | `agents-md` sets ≤300; these predate it | **Do NOT locally edit the two upstream-synced ones** — `typescript-expert` and `rust-wasm` are in `skills-lock.json`, so a sync silently overwrites the change. Either exempt upstream skills from the budget or split them at next sync. `skill-creator` is local and can be split any time (low value). |
| Dogfood pass over "closed" features (layers, SVG fidelity, light theme) | Fastest way to catch regressions behind closed-issue claims | `dogfood` skill run; file findings |
| F-30 prototype decision (collaborative editing) | Spike complete, no product decision | Open issue + ADR |
| Render performance follow-ups (ADR-028 residual) | Dirty-rect shipped; measure and set budgets | Open perf issue with metric |
| WebKit / Firefox flake watch | Multi-browser CI matrix is new | Track flake rate across releases |
| do-harness adoption (**#199**) | XL / high risk; sensor mapping and migration hazards already captured | Defer until upstream #235–#237 ship in a pinned release; then phase 0 audit in a disposable copy |

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
- Implementation issues: #198 (version single-source — closed 2026-09-24), #207 (F-13 layer history — open, next), #199 (do-harness adoption — open, deferred)
- [Harness steering log](../agents-docs/harness.md#learned-failure-modes-steering-log) (L-004/L-005/L-006/L-007, **L-008/L-009/L-010**)
- ADR-044 (merge automation + delivery loop), ADR-042 (wasm-bindgen pin parity), ADR-041 (clipboard export modes), ADR-036 (clipboard fidelity + product features)
- Issues: [#108](https://github.com/d-o-hub/rust-ascii-canvas/issues/108)–[#127](https://github.com/d-o-hub/rust-ascii-canvas/issues/127) (all closed)
