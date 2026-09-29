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
>
> **Read G1–G4 before you conclude anything about CI.** The line above presents
> two interchangeable options, and they are not interchangeable. This was
> corrected here on 2026-09-29 after it caused a real wrong turn (harness
> L-017), and it is the single easiest mistake to make with this tool.

### Credentials: where they live, and what CI can see

| | G1 locality | G2 read vs write | G3 scope |
|---|---|---|---|
| **Local machine** | `codacy login` stores an **account** API token, encrypted, at `~/.codacy/credentials` (`salt`/`iv`/`authTag`/`encrypted`, mode 600). Nothing is in the environment. | any read works | account-wide by nature |
| **GitHub Actions** | **cannot see any of it.** A runner is an ephemeral VM; it reads repository secrets and OIDC, nothing else. | needs `CODACY_API_TOKEN` (read) or `CODACY_PROJECT_TOKEN` (read **and** coverage upload) as a repository secret | must be **project-scoped** |

- **G1 — a local login is not a CI credential.** "The CLI is authenticated" and
  "CI can reach Codacy" are unrelated facts. Concluding a sensor is wired
  because the CLI works on your machine is the L-016 error wearing a costume.
- **G2 — read and write need different privileges.** Querying issues and
  uploading coverage are different operations with different tokens. A scheduled
  read needs `CODACY_API_TOKEN`; a coverage upload needs `CODACY_PROJECT_TOKEN`.
- **G3 — never put an account token in CI.** The CLI refuses to fall back to
  one for scoped work on purpose: *"falling back to an account token would
  silently run with far wider access than the scoped run you asked for."* That
  guard is the tool telling you the scope was wrong. Honour it.
- **G4 — a check you run by hand is a procedure, not a sensor.** If intake runs
  only when an agent or a developer invokes it, it is local-only, and it must be
  labelled that way rather than described as a gate.

If the repository has no secret (`gh secret list` empty, as here on
2026-09-29), then **no CI step exists and none may be claimed**. Say "not wired"
and move on. `scripts/codacy-check.sh` in this repo is deliberately an
agent procedure for exactly this reason, and its header says so.

## A PR's green check does not mean the backlog is clear

Codacy's **pull-request analysis is diff-scoped**. It reports only *new* issues
in the diff:

```bash
codacy -o json pull-request gh d-o-hub rust-ascii-canvas 220
#   isUpToStandards: true, newIssues: 0   ← while 11 High findings sat repo-wide
```

So "the Codacy check is green" is **structurally compatible with an arbitrary
pre-existing backlog**. Always read the repository-level list as well:

```bash
codacy -o json issues          # the backlog — the only place it is visible
npm run codacy:check            # wraps it; fails on Critical/High; warns, never fakes a pass, when unauthenticated
```

This is how `plans/FOLLOW_UPS.md` came to record "41 → 0 actionable" and be
wrong. A backlog is not a PR concern, and no PR will ever report it.

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

## Narrowing Codacy's scope — a human decision, not an escape hatch

Codacy runs far more than this repo needs, and some of it is *wrong* for this
repo. Two examples from 2026-09-29, both live:

- **A framework rule set for a framework this project does not use.** Four
  `useQwik*` Biome patterns judged ordinary TypeScript and produced a High
  finding on a plain arrow function. `grep -i qwik package.json web/package.json`
  is empty.
- **A rule family no local sensor runs.** All 11 `ESLint8_security_detect-*`
  findings were invisible to `gate:fast`, because neither ESLint config enabled
  `eslint-plugin-security`.

There are three legitimate responses, and **choosing among them is a human
decision** — raise it with the findings quoted, do not pick one to make a red
check go away:

1. **Fix the code** for real findings. The default.
2. **Bring local parity** for a rule family Codacy runs (this is what
   `eslint-plugin-security` in both configs now does — see L-017).
3. **Narrow the analyser**: a `biome.json`, `.codacy.yml` `exclude_paths`, or a
   tool-level change via `configure-codacy-cloud`. Note the limit: a pattern
   locked to the **Default coding standard** cannot be disabled, and
   `codacy tool --help` exposes only `enable | disable | configuration-file` —
   no per-rule parameter. A repo config file is often the only lever, and it has
   no local oracle: verify it by pushing and reading
   `codacy -o json pull-request`.

**Never** remove `Codacy Static Code Analysis` from the ruleset's required checks
to unblock a PR. That needs an ADR, a human decision, and an updated
`.github/ruleset-main.json` — and it is not a shortcut around a finding, it is
the deletion of the gate.

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
- **Not upstream-editable** — this skill is local, but `codacy-cloud-cli`,
  `codacy-analysis-cli`, `codacy-code-review`, `configure-codacy`,
  `configure-codacy-cloud` and `setup-coverage` are in `skills-lock.json` and
  are overwritten by the next sync. Fix a gap in those upstream, or carry it in
  `AGENTS.md`, and do not edit them locally
`detailsUrl` points at `app.codacy.com`, not a GitHub Actions run — that is the
tell that this is an app check, not a job you can re-run.

