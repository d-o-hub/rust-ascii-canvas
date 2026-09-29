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
  web lint/tsc/Vitest, **root lint + e2e tsc**, privacy + secret scan. Builds
  `web/pkg` if missing (it is gitignored, and CI downloads the wasm artifact
  before `tsc` — L-001).
- **full** — adds cargo audit/deny, WASM build + size budget, Playwright E2E.
- **pr** — the merge contract only; see *Merge & ship* below.

### The gate script is not a gate (learned)

`scripts/quality-gates.sh` is a **developer convenience**. Only what
`.github/workflows/ci.yml` runs can block a merge — and today **no CI job runs
`quality-gates.sh`**. A sensor added only to the script is therefore
*local-only*: it fails on your machine and merges anyway (harness **L-016**).

When you add a sensor, say **where it runs** — script, CI job, or both — and
confirm it by grepping the workflow rather than by running the script. If it
must block, wire it into the relevant `ci.yml` job.

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

`Codacy Static Code Analysis` is required by the ruleset, so it is part of
condition 2 — see the **Codacy** section and the `codacy` skill.

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
| **Codacy findings / blocked PR** | **`codacy`** (policy) + `codacy-cloud-cli`, `codacy-code-review`, `configure-codacy*` (tooling, upstream) |
| Decide + perform the merge | `merge-gate` |
| Ship safely (shadow → canary → promote) | `production-loop` |
| Rust implementation | `rust-engineer`, `rust-best-practices`, `rust-wasm` |
| TypeScript / Vite | `typescript-expert`, `vite` |
| Tool QA | `tool-validation` |
| Exploratory UX | `dogfood` |
| Maintain this doc | `agents-md` |

## Codacy

**Any Codacy warning, issue, or failing/stuck check — use the `codacy` skill.**
Read it before touching code or deciding the PR can merge.

`Codacy Static Code Analysis` is a **required** status check, so a non-`SUCCESS`
Codacy state blocks the merge and `npm run gate:pr` reports `MERGE BLOCKED`.
Codacy is a third-party GitHub App: there is no Actions run to re-dispatch. Its
findings **are** readable from the CLI — `codacy -o json pull-request <PR>` gives
file, line, pattern id and severity, so read them and fix them rather than
asking a human to fetch a dashboard. Never force the merge with `--admin` or by
dropping the required check; if a fix needs a decision above your level,
escalate with the findings quoted.

### Four rules that are not optional (harness L-017)

1. **A green PR check does not mean the backlog is clear.** Codacy's PR analysis
   is **diff-scoped** — it reports only *new* issues. #222 merged with the check
   green while 11 High findings sat repo-wide, and `plans/` had recorded "0
   actionable" on that basis. Always read the repository-level list too:
   `codacy -o json issues`, or `npm run codacy:check` (fails on Critical/High;
   warns rather than faking a pass when unauthenticated). A backlog is not a PR
   concern, so no PR will ever report it.
2. **Every rule family a required check enforces must also run locally.** Both
   `security_detect-unsafe-regex` and `security_detect-object-injection` are
   locked to Codacy's *Default coding standard* — not disableable, not
   configurable — and neither ESLint config enabled `eslint-plugin-security` at
   all, so 11 findings were invisible to `gate:fast`. Both configs now run the
   whole plugin. When adding a required check, ask what it enforces that nothing
   local enforces; if the answer is "a rule family", that is a gap to close, not
   a finding to fix one at a time.
3. **A Codacy fix is verified by a push, never locally.** Codacy re-analyses only
   the new head. Read it back with `codacy -o json pull-request <PR>`. One
   finding per commit; never bundle a Codacy fix with unrelated work. Both rules
   that fire here are non-disableable, so the **code** changes — no suppression,
   and never weaken an assertion to silence a linter.
4. **Removing or relaxing a required third-party check is a human decision.**
   Never propose it as a route to clearing a red check without asking first.
   Narrowing an *analyser* (a `biome.json`, an `exclude_paths` entry) is
   different, and legitimate: Codacy's Biome was running four `useQwik*` rules
   against a repo with no Qwik dependency. Fix, add local parity, or narrow the
   analyser — and say which you did.

**Credentials are not sensors.** `codacy login` stores an *account* token on this
machine (encrypted, `~/.codacy/credentials`); a GitHub Actions runner cannot see
it. CI needs `CODACY_API_TOKEN` or `CODACY_PROJECT_TOKEN` as a repository secret
(`gh secret list` is empty, so no Codacy step runs in CI today). Never put an
account token in CI — the CLI itself refuses to fall back to one for scoped work.
Until a secret exists, `npm run codacy:check` is an **agent procedure, not a
gate**, and is labelled as such wherever it appears.

## Reference docs

- [Harness map](agents-docs/harness.md)
- [Delivery loop runbook](agents-docs/delivery.md)
- [Architecture](agents-docs/architecture.md)
- [Best practices](agents-docs/best-practices.md)
- [Release runbook](plans/RELEASING.md)
- [Production learnings](agents-docs/learnings-archive.md)
- [Responsive grid](agents-docs/responsive-grid.md)
- ADRs: `plans/ADRs/` (see **037-harness-engineering**, **044-merge-automation-and-delivery-loop**, **045-local-sensor-parity-with-required-checks**)

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
