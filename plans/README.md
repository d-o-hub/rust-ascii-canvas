# Plans Index

Living project planning for the ASCII Canvas editor (GOAP + ADRs).

**Last refreshed**: 2026-10-15 against `origin/main` `0c2f0f3` (v0.1.5, #261, #265, #270, #271, #272, and #273 reconciliation)

---

## Start here

| Doc | Purpose |
|-----|---------|
| [PROJECT_STATUS.md](PROJECT_STATUS.md) | What’s true right now (tests, features, next steps) |
| [FOLLOW_UPS.md](FOLLOW_UPS.md#active-backlog) | **Single active backlog**: shipped vs candidate, queued and blocked work |
| [RELEASING.md](RELEASING.md) | Release runbook (bump → PR → dry-run → dispatch) |
| [current-plan.md](current-plan.md) | Historical GOAP snapshots + pointer to active work |
| [recommendations-implementation-2026-09-30.md](recommendations-implementation-2026-09-30.md) | Historical implementation evidence for the 2026-09-30 audit: #228–#233, with sensors/skills completion in #241 |
| [editor-safety-delivery-2026-09-30.md](editor-safety-delivery-2026-09-30.md) | #229 delivery record (isolated-tree verification) |
| [local-drafts-delivery-2026-09-30.md](local-drafts-delivery-2026-09-30.md) | #230 drafts delivery record |
| [goal-state.md](goal-state.md) | Target state + DoD checkboxes |
| [full-recommendations-2026-07.md](full-recommendations-2026-07.md) | Completed #21 + product bundle |

## Architecture & research

| Doc | Purpose |
|-----|---------|
| [TECHNICAL_ANALYSIS.md](TECHNICAL_ANALYSIS.md) | Technical findings (includes 2026-07 addendum) |
| [gap-analysis-enhancement-roadmap.md](gap-analysis-enhancement-roadmap.md) | Historical gap analysis (status notes applied) |
| [action-log.md](action-log.md) | Chronological actions |
| [ADRs/](ADRs/) | Architectural Decision Records; preserve status and implementation-scope notes |

## Key ADRs for latest work

- [046-editing-safety-and-local-drafts.md](ADRs/046-editing-safety-and-local-drafts.md) — accepted; shipped via #229/#230/#231, with verification completion in #241
- [047-fail-closed-verification-and-coherence.md](ADRs/047-fail-closed-verification-and-coherence.md) — accepted; editor-safety slice in #229 and remaining sensors/skills implementation in #241
- [048-box-drawing-glyphs-as-geometry.md](ADRs/048-box-drawing-glyphs-as-geometry.md) — accepted; shipped via #250 (SVG box runs become `<rect>`)

- [042-wasm-bindgen-pin-parity.md](ADRs/042-wasm-bindgen-pin-parity.md) — coordinated 0.2.128 pins + parity sensor
- [041-clipboard-export-modes.md](ADRs/041-clipboard-export-modes.md) — export fidelity / pure-ASCII fallback
- [037-harness-engineering.md](ADRs/037-harness-engineering.md) — tiered gates + steering log

## Status quick links

- **Shipped**: all roadmap issues #110–#127 closed (F-10–F-33); code verified in the 2026-09-23 reconciliation pass
- **Next**: [active backlog](FOLLOW_UPS.md#active-backlog). Do not treat v0.1.5,
  F-13, R-06, R-08, #199, or the 2026-09-30 audit delivery as pending; remaining
  work is limited to explicitly deferred, recorded, or credential-blocked rows.
- **Release procedure**: [RELEASING.md](RELEASING.md); RC workflow is unsupported
  until a reviewed implementation changes the stable-only version policy.
