---
name: codacy
description: >
  Triage Codacy findings and status on a GitHub pull request. Use whenever a PR
  shows a Codacy check that is failing, stuck in "Action required", or reports
  warnings, issues, or "Code is up to standards" — and before deciding whether a
  Codacy-blocked PR can merge. Repo policy: what may and may not be done to a
  required third-party check. For the tooling, use codacy-cloud-cli.
---

# Codacy (third-party static analysis)

Codacy runs outside this repo as a **GitHub App**. It posts a
`Codacy Static Code Analysis` status check, comments a summary on the PR, and
can annotate individual lines. On this repo that check is **required by the
`main` ruleset**, so a Codacy state that is not `SUCCESS` blocks the merge —
`npm run gate:pr` will report `MERGE BLOCKED`.

**This skill is the policy layer.** For the tooling, use the upstream skills:
`codacy-cloud-cli` (query issues / PRs / analysis), `codacy-code-review`
(enrich reviews), `configure-codacy*` (tune config), `setup-coverage`.

## The important correction

An earlier version of this skill said the findings were unreadable without a
dashboard. **That was wrong.** The `codacy` cloud CLI (v1.11.0, already
installed here and authenticated) exposes everything:

```bash
codacy -o json pull-request gh d-o-hub rust-ascii-canvas 216
```

That returns the full `newIssues` array with `filePath`, `lineNumber`,
`patternInfo` (id, category, severity) and the message. Do not send a human to
fetch a dashboard when the API is right here. The GitHub surface genuinely does
**not** carry the detail — no inline review comments are posted — but the API
does.

> Verified 2026-09-27 on PR #216. Add `CODACY_API_TOKEN`, or run `codacy login`,
> if the CLI is not authenticated.

## When to Use

- A PR has a Codacy check that is failing, or stuck in `ACTION_REQUIRED`
- A Codacy PR comment reports issues, warnings, or "Code is up to standards"
- Deciding whether a Codacy-blocked PR may merge
- Re-triage after a push (Codacy re-analyses the new head)

## Don't Invoke When

- A non-Codacy check failed → use `verify` / `merge-gate`
- You only need the mechanical merge verdict → `npm run gate:pr` already reports
  Codacy alongside every other check

## Read the current state

```bash
# Full analysis for the PR: findings, metrics, gate reasons
codacy -o json pull-request <PR>

# Just the blocking check
gh pr view <PR> --json statusCheckRollup \
  --jq '.statusCheckRollup[] | select((.name//.context)=="Codacy Static Code Analysis")
        | {state:(.conclusion//.state), url:.detailsUrl}'

# Whole-contract verdict (Codacy is one of the conditions)
npm run gate:pr <PR>
```

`detailsUrl` points at `app.codacy.com`, not a GitHub Actions run — that is the
tell that this is an app check, not a job you can re-run.

## Triage a finding

Codacy reports per-tool issues plus a summary. Work in this order:

1. **Read the findings, not just the summary.** `codacy -o json pull-request`
   is authoritative and gives file, line, pattern id and severity. The PR
   comment is a roll-up and carries no locations.
2. **Classify before fixing.** Not every finding deserves a change:
   - **Real defect** → fix it, same as any other code change, and add a test.
   - **Style / idiom** → prefer the idiomatic Rust/TS form over suppressing.
     Cite the source (see `pr-roast`'s citation rule) if you claim a tool is wrong.
   - **False positive on this codebase** → fix the *code* where that is honestly
     simpler, or suppress it in Codacy's configuration. Never delete a test or
     weaken an assertion to silence a linter.
     Note: `.codacy.yaml` supports `exclude_paths` and per-engine settings, but
     **not** disabling an individual pattern. A pattern locked to Codacy's
     "Default coding standard" cannot be disabled at all — the CLI answers
     *"Pattern enforced by Default coding standard, can't be modified."* In that
     case a whole-path exclusion is too blunt, so simplify the code instead.
3. **Re-push and let Codacy re-analyse.** Each push resets the check; a stale
   result never carries over.
4. **One finding per commit.** Do not bundle a Codacy fix with unrelated work.

## The states, and what they mean

| State | Meaning | What to do |
|---|---|---|
| `SUCCESS` | Analysed, gates passed | Nothing. |
| `FAILURE` | Analysed, a gate failed | Triage the comment; fix or escalate. |
| `PENDING` / `""` | Analysis in flight | Wait; re-check. |
| `ACTION_REQUIRED` | Not evaluated; waiting on a maintainer | **Escalate.** See below. |

> **Verification boundary.** The `SUCCESS` / `FAILURE` / `PENDING` behaviour is
> documented. Codacy's public docs do **not** document what puts a check into
> `ACTION_REQUIRED`; that is an observation from this repo on 2026-09-27, where
> the check sat in that state with no GitHub Actions run attached. Treat the
> explanation below as observed behaviour, not a documented contract.

### When Codacy sits in `ACTION_REQUIRED`

Observed: the check never reports, and `detailsUrl` is a Codacy dashboard URL.
This means **Codacy has not evaluated the commit**, which is not the same as
passing. It usually needs a maintainer in the Codacy dashboard — to authorise or
re-authorise the GitHub App, or to run the analysis on demand.

Do **not** work around a *failing* gate:

- **Never** `gh pr merge --admin` to force past it. That is the one action this
  harness exists to prevent, and it silently deletes the gate.
- **Never** remove `Codacy Static Code Analysis` from the ruleset's required
  checks to unblock a PR. That needs an ADR, a human decision, and an updated
  `.github/ruleset-main.json` (see `goap-adr-planner`).
- **Never** mark the check green locally or disable the check for a branch.

Instead: read the findings with the CLI, fix them, and re-push. If a finding
genuinely cannot be fixed without a decision above your level, say so and
escalate — with the specific findings quoted, not "check the dashboard".

`npm run gate:pr` will keep reporting `MERGE BLOCKED` while the check is not
`SUCCESS`, which is the correct behaviour — an unverified check is not a passing
check.

## When Codacy is genuinely not useful

If Codacy's analysis is superseded by a local sensor that already covers the same
ground (this repo runs clippy `-D warnings`, ESLint, `cargo audit` and
`cargo deny`), say so explicitly and propose dropping it as a required check —
as an ADR, not as a drive-by edit. Use `configure-codacy-cloud` for the tooling.
Do not quietly tolerate a permanently red or permanently stuck third-party check;
that is how a required check becomes decorative.

Note the asymmetry to keep in mind: **clippy and ESLint are local and fast, so a
local fix is visible in `gate:fast` immediately. Codacy findings are not** — only
a push re-analyses them. That makes a Codacy fix the one kind of change this
harness cannot self-verify, which is a reason to read the finding carefully
rather than pattern-match it.

## Official references

- [GitHub integration](https://docs.codacy.com/repositories-configure/integrations/github-integration) —
  status checks, issue annotations, issue summaries (each needs *Status checks*
  enabled), AI Reviewer, merge-queue behaviour
- [Codacy configuration file](https://docs.codacy.com/repositories-configure/codacy-configuration-file/) —
  `exclude_paths`, per-engine settings, language config
- [Codacy cloud CLI](https://github.com/codacy/codacy-cloud-cli) — the tool that
  made these findings readable
- [Codacy REST API](https://api.codacy.com/) — the CLI wraps this; an account
  API token is needed if you are not logged in

## Integration

- **After** `verify` (local sensors) — Codacy is the external layer
- **Before** `merge-gate` arms auto-merge, because it can block the merge
- **With** `pr-roast` for classification and the citation rule
- **Tooling skills**: `codacy-cloud-cli`, `codacy-code-review`,
  `configure-codacy`, `configure-codacy-cloud`, `setup-coverage`
- **Escalate to a human** when a fix would mean relaxing a gate or a check
`detailsUrl` points at `app.codacy.com`, not a GitHub Actions run — that is the
tell that this is an app check, not a job you can re-run.

