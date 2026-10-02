# Plans Index

Living project planning for the ASCII Canvas editor (GOAP + ADRs).

**Last refreshed**: 2026-09-30 (working-tree documentation update; not a release)

---

## Start here

| Doc | Purpose |
|-----|---------|
| [PROJECT_STATUS.md](PROJECT_STATUS.md) | What’s true right now (tests, features, next steps) |
| [FOLLOW_UPS.md](FOLLOW_UPS.md#active-backlog) | **Single active backlog**: shipped vs candidate, queued and blocked work |
| [RELEASING.md](RELEASING.md) | Release runbook (bump → PR → dry-run → dispatch) |
| [current-plan.md](current-plan.md) | Historical GOAP snapshots + pointer to active work |
| [recommendations-implementation-2026-09-30.md](recommendations-implementation-2026-09-30.md) | Implementation evidence for the 2026-09-30 audit (shipped #228–#233) |
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

- [046-editing-safety-and-local-drafts.md](ADRs/046-editing-safety-and-local-drafts.md) — accepted; shipped via #229/#230/#231
- [047-fail-closed-verification-and-coherence.md](ADRs/047-fail-closed-verification-and-coherence.md) — accepted; #229 slice + sensors port

- [042-wasm-bindgen-pin-parity.md](ADRs/042-wasm-bindgen-pin-parity.md) — coordinated 0.2.128 pins + parity sensor
- [041-clipboard-export-modes.md](ADRs/041-clipboard-export-modes.md) — export fidelity / pure-ASCII fallback
- [037-harness-engineering.md](ADRs/037-harness-engineering.md) — tiered gates + steering log

## Status quick links

- **Shipped**: all roadmap issues #110–#127 closed (F-10–F-33); code verified in the 2026-09-23 reconciliation pass
- **Next**: [active backlog](FOLLOW_UPS.md#active-backlog). v0.1.4, F-13, R-06
  and npm-audit inventory already shipped; current audit candidates have not.
- **Release procedure**: [RELEASING.md](RELEASING.md); RC workflow is unsupported
  until a reviewed implementation changes the stable-only version policy.
