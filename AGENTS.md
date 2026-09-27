# Agent Best Practices — ASCII Canvas

Outer harness for coding agents. Full map: [agents-docs/harness.md](agents-docs/harness.md). Architecture constraints: [agents-docs/architecture.md](agents-docs/architecture.md).

## Core directives

1. **Analyze** — understand requirements and which layers you will touch.
2. **Plan** — multi-step or architectural work → `plans/` + ADR via `goap-adr-planner`.
3. **Execute** — implement; keep business logic in `src/core/`.
4. **Verify** — run **fast** sensors after changes; **full** before PR. Use the `verify` skill.
5. **Document** — update `plans/` / ADRs when decisions or learnings change.
6. **Steer the harness** — if the same failure happens twice, improve a guide or sensor, not only product code.

## Keep quality left (mandatory tiers)

Do **not** run the full E2E suite after every one-line fix. Use tiers:

| Tier | When | Command |
|------|------|---------|
| **fast** | After every meaningful edit | `npm run gate:fast` |
| **full** | Before commit/PR that touches product code | `npm run gate:full` |
| **pr** | Before merging anything | `npm run gate:pr` |
| **ruleset** | When the merge contract is questioned | `npm run gate:ruleset` |
| **focused** | While iterating on one area | relevant `cargo test …` / `cd web && pnpm test` / single Playwright file |

### What the tiers cover

- **fast** — fmt, clippy `-D warnings`, build, `cargo test`, architecture, LOC,
  web lint/tsc/Vitest, privacy + secret scan. Builds `web/pkg` if missing (it is
  gitignored, and CI downloads the wasm artifact before `tsc` — L-001).
- **full** — adds cargo audit/deny, WASM build + size budget, Playwright E2E.
- **pr** — the merge contract only; see *Merge & ship* below.

### CI vs local (learned)

If CI is red and local is green, **do not** only re-run CI. Diff assumptions (artifacts, path filters, env). Fix the **sensor** so local fails the same way next time — see [harness learned failures](agents-docs/harness.md#learned-failure-modes-steering-log) (e.g. L-001 `web/pkg`).

Self-correct on red sensors before asking a human to review. Sensor output includes fix hints — follow them.

`gate:pr` is the **only** thing that answers "may this merge?" — it is read-only
and never merges.

## Architecture constraints (non-negotiable)

```
core (pure) ← render, ui ← wasm ← web/
```

- `src/core/**` must **not** import `wasm`, `render`, or `ui`.
- Prefer extracting modules over growing files past **500 LOC**.
- Details and allowed edges: [agents-docs/architecture.md](agents-docs/architecture.md).
- Computational check: `./scripts/check-architecture.sh`.

## Rust testing

1. **Unit tests**: same file, `#[cfg(test)] mod tests { … }` (can touch private items).
2. **Integration tests**: `tests/` for public API only.
3. On failure: fix root cause; do not weaken assertions or skip CI to “make it green”.

## Web / WASM

- Build WASM: `npm run build:wasm` (wasm-bindgen **0.2.128**, see `mise.toml`).
- Web lint/test: `cd web && pnpm lint && pnpm exec tsc --noEmit && pnpm test`.
- E2E: Playwright from repo root; prefer `--project=chromium` locally.

## Tool behaviour checklist (behaviour harness)

All 8 drawing tools (Select / Rect / Line / Arrow / Diamond / Text / Free /
Erase) have per-tool acceptance criteria — use the `tool-validation` skill for
the full procedure whenever tools or canvas interaction change.

## Merge & ship (less human in the loop)

A PR may be merged when **all** of these hold — nothing else:

| # | Condition | Enforced locally | Enforced by GitHub |
|---|-----------|------------------|--------------------|
| 1 | Not a draft, no conflicts | `npm run gate:pr` | — |
| 2 | **Every** CI check `SUCCESS` | `npm run gate:pr` | ruleset: `CI Success`, `PR Readiness (merge gate)` |
| 3 | **Every** review thread resolved | `npm run gate:pr` | ruleset: `required_review_thread_resolution` |
| 4 | No outstanding `CHANGES_REQUESTED` | `npm run gate:pr` | — |
| 5 | Adversarial pass clean | `pr-roast` | — |

```bash
npm run gate:fast          # while iterating
npm run gate:full          # before the PR is mergeable
npm run gate:pr            # the merge contract (read-only, never merges)
gh pr merge <PR> --auto --squash   # CI performs the merge when it goes green
```

**Do not** use `--admin` to force past a red check, and do not treat
"could not verify" as "verified" — `pr-merge-gate.sh` errors rather than
assumes. Squash is required: the repo enforces linear history.

Conditions 1–4 live in the **`main` ruleset**, which is repository state, not a
file: `git revert` does not undo it and no code review sees it. So it is
snapshotted in `.github/ruleset-main.json` and drift-checked by
`npm run gate:ruleset` (a required check). Changing it needs an ADR **and** an
updated snapshot in the same change.

### The delivery loop

```
production → failure → reproduce → candidate fix → evaluate
           → adversarial → shadow → canary → promote / rollback
```

Two stages carry most of the value: **reproduce** (an automated failing test —
no repro, no fix) and **shadow** (full E2E against a production-shaped build;
`playwright.config.ts` already honours `BASE_URL`, so a Netlify Deploy Preview
works with no config change). Canary here is an opt-in RC tag, **not** a
percentage rollout — this is a static app with no traffic splitting.

Runbook and per-stage exit criteria: [agents-docs/delivery.md](agents-docs/delivery.md)
· skill `production-loop`.

**Human judgment stays** for scope/spec, ADRs, disputed roast blockers, and
production rollback. **Human chore is gone** for gates, self-correction, the
roast, comment tracking, shadow E2E, and clicking merge.

## Skills (by harness role)

| Need | Skill |
|------|--------|
| Plan / ADR | `goap-adr-planner` |
| Run sensors / self-correct | `verify` |
| Semantic review | `code-review` |
| Adversarial review ("roast") | `pr-roast` |
| Decide + perform the merge | `merge-gate` |
| Ship safely (shadow → canary → promote) | `production-loop` |
| Rust implementation | `rust-engineer`, `rust-best-practices`, `rust-wasm` |
| TypeScript / Vite | `typescript-expert`, `vite` |
| Tool QA | `tool-validation` |
| Exploratory UX | `dogfood` |
| Maintain this doc | `agents-md` |

## Reference docs

- [Harness map](agents-docs/harness.md)
- [Delivery loop runbook](agents-docs/delivery.md)
- [Architecture](agents-docs/architecture.md)
- [Best practices](agents-docs/best-practices.md)
- [Release runbook](plans/RELEASING.md)
- [Production learnings](agents-docs/learnings-archive.md)
- [Responsive grid](agents-docs/responsive-grid.md)
- ADRs: `plans/ADRs/` (see **037-harness-engineering**, **044-merge-automation-and-delivery-loop**)

## Bot-generated PRs

- Automated PRs (e.g., from Jules, Dependabot, or other bots) are held to the
  **same** merge contract: the gates decide, not the bot.
- A human is **not** required to click merge (ADR-044). A human is required when
  a `pr-roast` **Blocker** is disputed, or when the diff touches a merge
  guard-rail, a release pin, or the document format.
- ADR-only PRs must have status changes verified against actual commit history — not just PROJECT_STATUS.md.
- ADR `## Status` changes must preserve cross-references and implementation scope notes.
- `plans/` changes trigger harness CI checks (fmt, architecture, lint).
- Learn observed bot failures in `agents-docs/harness.md` (steering log).

## PR / handoff

- **Branch from the base branch, never from another feature branch.** Before
  opening a PR, `git log origin/main..HEAD` must list only commits you authored
  this session. If it lists more, the branch is wrong — rebase it. A per-commit
  "no product code staged" check cannot catch a foreign commit that was never
  staged; that is exactly how the F-13 feature merged under a `chore:` title
  (harness **L-011**). `npm run gate:pr` now prints the commit list.
- Fast gates green on every push-worthy change; full gates green before review.
- PR template checkboxes must reflect reality.
- Call out harness changes (new sensors, allowlist, CI) explicitly in the PR body.
- Never bypass a red check with `--admin`; fix the cause or ask a human.
