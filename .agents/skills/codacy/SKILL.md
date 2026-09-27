---
name: codacy
description: >
  Triage Codacy findings and status on a GitHub pull request. Use whenever a PR
  shows a Codacy check that is failing, stuck in "Action required", or reports
  warnings, issues, or "Code is up to standards" — and before deciding whether a
  Codacy-blocked PR can merge. Codacy is a third-party app and a required status
  check, so it cannot be cleared or re-run from the CLI.
---

# Codacy (third-party static analysis)

Codacy runs outside this repo as a **GitHub App**. It posts a
`Codacy Static Code Analysis` status check, comments a summary on the PR, and
can annotate individual lines. On this repo that check is **required by the
`main` ruleset**, so a Codacy state that is not `SUCCESS` blocks the merge —
`npm run gate:pr` will report `MERGE BLOCKED`.

**You cannot clear Codacy from the command line.** There is no GitHub Actions run
to re-dispatch, and the API needs a Codacy API token that the CLI does not hold.
This skill tells you what to read, how to classify a finding, and when to stop
and escalate to a human.

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
# Is it blocking, and what exactly is the state?
gh pr view <PR> --json statusCheckRollup \
  --jq '.statusCheckRollup[] | select((.name//.context)=="Codacy Static Code Analysis")
        | {state:(.conclusion//.state), url:.detailsUrl}'

# Whole-contract verdict (Codacy is one of the conditions)
npm run gate:pr <PR>

# Codacy's own findings for the repo (from its PR comment)
gh pr view <PR> --json comments \
  --jq '.comments[] | select(.author.login=="codacy-production") | .body'

## Triage a finding

Codacy reports per-tool issues plus a summary. Work in this order:

1. **Read the summary comment first.** It carries the per-tool counts and the
   overall verdict; the line annotations are the detail.
2. **Classify before fixing.** Not every finding deserves a change:
   - **Real defect** → fix it, same as any other code change, and add a test.
   - **Style / idiom** → prefer the idiomatic Rust/TS form over suppressing.
     Cite the source (see `pr-roast`'s citation rule) if you claim a tool is wrong.
   - **False positive on this codebase** → suppress it in Codacy's configuration,
     never by deleting a test or weakening an assertion.
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

Do **not** work around it:

- **Never** `gh pr merge --admin` to force past it. That is the one action this
  harness exists to prevent, and it silently deletes the gate.
- **Never** remove `Codacy Static Code Analysis` from the ruleset's required
  checks to unblock a PR. That needs an ADR, a human decision, and an updated
  `.github/ruleset-main.json` (see `goap-adr-planner`).
- **Never** mark the check green locally or disable the check for a branch.

Instead: report it. State which checks are green, that this one is
`ACTION_REQUIRED` and why, and that the maintainer needs to clear it in Codacy.
`npm run gate:pr` will keep reporting `MERGE BLOCKED`, which is the correct
behaviour — an unverified check is not a passing check.

## When Codacy is genuinely not useful

If Codacy's analysis is superseded by a local sensor that already covers the same
ground (this repo runs clippy `-D warnings`, ESLint, `cargo audit` and
`cargo deny`), say so explicitly and propose dropping it as a required check —
as an ADR, not as a drive-by edit. Do not quietly tolerate a permanently red or
permanently stuck third-party check; that is how a required check becomes
decorative.

## Official references

- [GitHub integration](https://docs.codacy.com/repositories-configure/integrations/github-integration) —
  status checks, issue annotations, issue summaries (each needs *Status checks*
  enabled), AI Reviewer, merge-queue behaviour
- [Codacy docs home](https://docs.codacy.com/) — configuration file, languages,
  local analysis, CLI
- [Codacy REST API](https://api.codacy.com/) — requires an account API token,
  which the agent CLI does not have

## Integration

- **After** `verify` (local sensors) — Codacy is the external layer
- **Before** `merge-gate` arms auto-merge, because it can block the merge
- **With** `pr-roast` for classification and the citation rule
- **Escalate to a human** on `ACTION_REQUIRED`, or when a fix would mean
  relaxing a gate

```

`detailsUrl` points at `app.codacy.com`, not a GitHub Actions run — that is the
tell that this is an app check, not a job you can re-run.
