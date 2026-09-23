# Plans Index

Living project planning for the ASCII Canvas editor (GOAP + ADRs).

**Last refreshed**: 2026-09-23

---

## Start here

| Doc | Purpose |
|-----|---------|
| [PROJECT_STATUS.md](PROJECT_STATUS.md) | What’s true right now (tests, features, next steps) |
| [FOLLOW_UPS.md](FOLLOW_UPS.md) | Prioritized backlog (F-IDs) |
| [RELEASING.md](RELEASING.md) | Release runbook (bump → PR → dry-run → dispatch) |
| [current-plan.md](current-plan.md) | Active GOAP plan + world state |
| [goal-state.md](goal-state.md) | Target state + DoD checkboxes |
| [full-recommendations-2026-07.md](full-recommendations-2026-07.md) | Completed #21 + product bundle |

## Architecture & research

| Doc | Purpose |
|-----|---------|
| [TECHNICAL_ANALYSIS.md](TECHNICAL_ANALYSIS.md) | Technical findings (includes 2026-07 addendum) |
| [gap-analysis-enhancement-roadmap.md](gap-analysis-enhancement-roadmap.md) | Historical gap analysis (status notes applied) |
| [action-log.md](action-log.md) | Chronological actions |
| [ADRs/](ADRs/) | Architectural Decision Records (001–042) |

## Key ADRs for latest work

- [042-wasm-bindgen-pin-parity.md](ADRs/042-wasm-bindgen-pin-parity.md) — coordinated 0.2.128 pins + parity sensor
- [041-clipboard-export-modes.md](ADRs/041-clipboard-export-modes.md) — export fidelity / pure-ASCII fallback
- [037-harness-engineering.md](ADRs/037-harness-engineering.md) — tiered gates + steering log

## Status quick links

- **Shipped**: all roadmap issues #110–#127 closed (F-10–F-33); code verified in the 2026-09-23 reconciliation pass
- **Next**: release v0.1.4 ([RELEASING.md](RELEASING.md)) + curated changelog backfill
