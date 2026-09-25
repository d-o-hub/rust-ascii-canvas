# Follow-ups Backlog

**Updated**: 2026-09-23
**Source**: Full recommendations bundle (issue #21 + post-merge analysis)
**Primary plan**: [full-recommendations-2026-07.md](full-recommendations-2026-07.md)
**Latest triage**: 2026-09-25 — **next cycle planned by a read-only agent swarm + critic**: v0.1.4 shipped and R-04 done. Cycle order: (1) F-13 layer-operation history under [ADR-043](ADRs/043-layer-command-history.md) and issue #207, (2) R-05 dependency refresh, (3) #199 do-harness adoption deferred until upstream #235–#237 ship in a pinned release. **R-03 was re-scoped after a verification swarm disproved the pin-by-override plan** (Vite pulls esbuild as an unused optional peer; the advisory is unreachable here) — see the R-03 row, R-05, and the corrected L-006. This pass also reconciled the planning docs against reality (test counts, issue states, release state, toolchain pin).

Use this list for prioritization. Mark items done in-place and mirror major completions into `PROJECT_STATUS.md`.  
**GitHub issues** track open work (numbers below).

---

## Next — release + new work (2026-09-23)

| ID | Status | Issue | Notes |
|----|--------|-------|--------|
| **R-01** | ✅ done | — | 2026-09-24: **v0.1.4 released** — `VERSION` 0.1.4 propagated via `scripts/propagate-version.sh`, `scripts/release.sh` preflight OK, dry run + real run green, tag on the changelog branch, `[0.1.2]`/`[0.1.3]` curated + `[0.1.4]` generated and synced back to `main`. Runbook: [RELEASING.md](RELEASING.md) |
| **R-02** | ✅ resolved | — | 2026-09-24: `release.yml` anchors the notes range to the latest GitHub Release tag (with a `git describe` fallback). `v0.1.3` is not an ancestor of `main`, so the old behaviour re-listed 217 commits instead of the 27 unreleased ones |
| **R-03** | re-scoped | [security/dependabot/11](https://github.com/d-o-hub/rust-ascii-canvas/security/dependabot/11) | Dev-only esbuild advisory GHSA-g7r4-m6w7-qqqr. **Swarm finding (2026-09-25), independently verified**: `vite@8.3.0` has **no** esbuild dependency — only `peerDependencies.esbuild: ^0.27.0 \|\| ^0.28.0` with `optional: true` (`web/pnpm-lock.yaml:1187-1212`); nothing in `web/` references esbuild; the advisory (CVSS 2.5, Windows-only) hits esbuild's *own* dev server, which this repo never runs. The earlier **pin-by-override plan is superseded**: an override alone silently *drops* esbuild instead of pinning it (verified on pnpm 10.34.5 and 11.7.0), and a direct devDependency would add a package we never call. Removal is the right end state but is not a surgical lock edit — a fresh resolve prunes 27 esbuild + 43 jsdom entries (both unused optional peers) and bumps 28 unrelated packages including a `whatwg-mimetype` downgrade, so it belongs in a reviewed dependency refresh (**R-05**). Alert stays open until the prune lands |
| **R-04** | ✅ done | — | 2026-09-25: the `Publish WASM` job downloads the release build and attaches `ascii-canvas-<version>.wasm` with `--clobber` (idempotent re-runs). The artifact is **copied to a versioned name first** — `gh release upload`'s `file#label` syntax only sets a display label (verified against a scratch draft), so the download filename would otherwise remain `ascii_canvas_bg.wasm`. v0.1.4 has no asset; v0.1.5 is the first with one |
| **R-05** | open | — | **Dependency refresh** (closes alert #11): re-resolve `web/pnpm-lock.yaml` to prune the unused optional-peer entries (27 esbuild + 43 jsdom) and review the 28 resulting bumps (vite 8.3.1, vitest 5.0.2, rolldown 1.2.11, `whatwg-mimetype` 5.0.0 → 3.0.0). Must verify `pnpm install --frozen-lockfile` on **both** pnpm 10 (CI) and 11 (local), full web lint/tsc/test/build, the E2E matrix, and that `ERR_PNPM_IGNORED_BUILDS` disappears (dissolves L-006's trigger). Removing `pnpm.ignoredBuiltDependencies` is only safe in the same PR that drops esbuild |
| **R-06** | open | — | `web/vite.config.ts:14` sets `server.host: true`, binding the dev server to `0.0.0.0` (LAN-visible). Decide the default (loopback) and document the opt-in for device testing; independent of R-03 |

---

## P0 — Ship / close the loop (complete)

| ID | Status | Issue | Notes |
|----|--------|-------|--------|
| **F-01** | ✅ | — | [PR #107](https://github.com/d-o-hub/rust-ascii-canvas/pull/107) merged; #21 closed |
| **F-02** | ✅ | [#108](https://github.com/d-o-hub/rust-ascii-canvas/issues/108) | Manual clipboard QA verified across Notepad, TextEdit, VS Code, and monospace area |
| **F-03** | ✅ | [#109](https://github.com/d-o-hub/rust-ascii-canvas/issues/109) | Closed 2026-07-16; coordinated wasm-bindgen 0.2.128 + pin-parity sensor completed 2026-09-17 (ADR-042). Web-deps PRs #188/#190 closed/merged; only R-03 (low, dev-only) remains |

---

## P1 — Product (complete)

| ID | Status | Issue | Notes |
|----|--------|-------|--------|
| **F-10** | ✅ | [#110](https://github.com/d-o-hub/rust-ascii-canvas/issues/110) | SVG export: `web/exportSvg.ts`, `src/wasm/render_api.rs` `export_svg`, unit + UI wiring |
| **F-11** | ✅ | [#111](https://github.com/d-o-hub/rust-ascii-canvas/issues/111) | Layer editor: rename, visible, lock, reorder (`moveLayer`), delete, merge (`mergeLayerDown`); UI in `web/ui.ts` |
| **F-12** | ✅ | — | Composite pixel render + export (#107) |
| **F-13** | ❌ | [#111](https://github.com/d-o-hub/rust-ascii-canvas/issues/111) | **Not implemented** — layer ops bypass undo history (verified 2026-09-23: `History::undo(&mut Grid)`, layer calls have no commands). Issue was closed with F-11; needs a fresh issue |
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
| `web/events.ts` 853 LOC (> 500 guideline) + LOC sensor gap | Sensor scans only `src/**/*.rs`; `web/` growth is unchecked | Either extract modules or extend the LOC sensor (ADR), then keep gate honest |
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
- [Harness steering log](../agents-docs/harness.md#learned-failure-modes-steering-log) (L-004/L-005/L-006/L-007)
- [ADR-042](ADRs/042-wasm-bindgen-pin-parity.md), [ADR-041](ADRs/041-clipboard-export-modes.md), [ADR-036](ADRs/036-clipboard-fidelity-and-product-features.md)
- Issues: [#108](https://github.com/d-o-hub/rust-ascii-canvas/issues/108)–[#127](https://github.com/d-o-hub/rust-ascii-canvas/issues/127) (all closed)
