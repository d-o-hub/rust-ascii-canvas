---
name: plans-reconciliation
description: Reconcile plans/ status, GOAP state, releases, issues, pull requests, gates, and official security guidance after repository work. Use for requests to update plans, progress, follow-ups, roadmaps, project status, or planning evidence.
---

# Plans Reconciliation

Use this skill when project prose may have drifted from the repository or GitHub.
The deliverable is truthful, dated planning state—not a summary inferred from
old documents.

## Operating contract

1. **Evidence before prose.** Treat `git`, GitHub, release metadata, checks, and
   actually run commands as evidence. Treat existing plans as hypotheses until
   reconciled.
2. **GOAP is the orchestrator.** Model the current state, desired state,
   preconditions, actions, effects, blockers, and next evidence. Do not turn a
   recorded idea into active work merely because it appears in a roadmap.
3. **Use bounded swarm roles.** When parallel investigation helps, assign
   separate agents to: repository evidence, plans/stale-prose audit, and
   official-source research. Each returns facts and citations; the orchestrator
   resolves conflicts and writes the files.
4. **Preserve history.** Candidate and delivery records remain useful evidence,
   but pre-merge wording must be labeled historical after the change merges.
5. **Never claim an unrun check.** Say `not run`, `remote passed`, or
   `credential-blocked`; do not convert uncertainty into a green result.

## GOAP state model

Write the reconciliation as:

| Field | Question |
|---|---|
| Current state | What is true in `origin/main`, releases, issues, PRs, checks, and working tree? |
| Goal state | What should a new agent understand from `plans/` without querying prose history? |
| Preconditions | Which remote facts or credentials must be available before a claim is made? |
| Actions | Which rows become shipped, deferred, unsupported, blocked, or newly queued? |
| Effects | Which status files, follow-ups, links, counts, or ADR references change? |
| Blockers | Is the blocker a missing decision, credential, authority, implementation, or evidence? |
| Next evidence | What exact command, test, review, or human decision unblocks it? |

Use these state labels consistently:

- **Shipped** — merged/released and backed by a commit, PR, tag, or artifact.
- **Recorded, not queued** — known work with no implementation authorization.
- **Deferred** — an explicit future decision or prerequisite is missing.
- **Unsupported** — current policy/tooling deliberately does not provide it.
- **Credential-blocked** — the procedure is ready but required secret/access is absent.
- **Historical candidate** — evidence from an old branch or pre-merge delivery record.

## Evidence pass

Run from the repository root. Substitute the repository argument rather than
hard-coding another project.

```bash
git fetch origin main
git status --short --branch
git log --first-parent --oneline --decorate -30 origin/main
git diff --name-status <last-known-status>..origin/main
gh issue list --state open --limit 100 --json number,title,state,url
gh pr list --state open --limit 100 --json number,title,isDraft,url
gh release list --limit 20 --json tagName,isDraft,isPrerelease,publishedAt,url
gh api repos/OWNER/REPO/dependabot/alerts?state=open
```

For a release, verify the release object and asset rather than inferring from
`VERSION` or a tag alone:

```bash
gh release view vX.Y.Z --json tagName,targetCommitish,assets,url
gh run list --workflow release.yml --limit 10 --json conclusion,headSha,status,url
```

For a PR or merged change, inspect the full commit scope and check roll-up:

```bash
gh pr view <PR> --json state,isDraft,mergeable,reviewDecision,commits,statusCheckRollup,url
npm run gate:pr
```

`npm run gate:pr` is read-only and is the repository's merge-contract answer.
It must not be replaced by a green local convenience gate or by a manual
assumption. Confirm that `origin/main..HEAD` contains only intended commits;
do not carry a stale merge commit or foreign branch history into a PR.

### Evidence traps

- A release tag can target a dedicated changelog branch and therefore not be an
  ancestor of current `main`. Check `gh release view` and the run before calling
  the release missing.
- A merged PR can close an issue later than the prose was written. Count current
  GitHub state; do not preserve an old `open_issues` number.
- A delivery record can say “PR pending” after its PR merged. Keep its evidence,
  but label that sentence historical and add the merged PR/check result.
- No open issues or PRs does not mean no future work. Keep explicitly deferred,
  unsupported, credential-blocked, and recorded-but-not-queued follow-ups.
- A local script may be green without being a required CI check. Inspect the
  applicable `.github/workflows/ci.yml` job before claiming merge protection.

## Plans update map

Update only the files whose facts changed, but keep these roles distinct:

| File | Keep here |
|---|---|
| `plans/FOLLOW_UPS.md` | Single active backlog, state labels, evidence links, blockers, and next proof. |
| `plans/PROJECT_STATUS.md` | Current release/phase, recent shipped work, current verification, and explicit blockers. |
| `plans/current-plan.md` | GOAP roadmap metadata, progress table, and current issue/PR counts. |
| `plans/goal-state.md` | Machine-readable goal/current state and dated snapshot metadata. |
| `plans/README.md` | Navigation and short “next” pointer; do not duplicate the backlog. |
| `plans/RELEASING.md` | Release procedure and latest known release state only. |
| `plans/TECHNICAL_ANALYSIS.md` | Durable technical findings, tradeoffs, and research citations. |
| `plans/ADRs/` | Only decisions requiring an ADR: merge guard-rails, format/history/clipboard changes, cross-layer dependencies, or release-policy changes. |

When updating multiple status files, make the date and release/PR references
consistent. A status count should have a clear scope, for example “excluding
this PR” only while the PR is open; remove that qualifier after merge.

## Swarm handoffs

Give each agent a narrow, non-overlapping prompt and require an evidence table:

1. **Repository auditor:** commits, merged PRs, releases/assets, checks, open
   issues/PRs, Dependabot, Codacy, and working-tree state.
2. **Plans auditor:** stale “pending/open/next” language, contradictory dates,
   counts, duplicate active queues, and candidate records needing historical
   labels.
3. **Official-source researcher:** retrieve only authoritative documentation,
   report the URL, retrieval date, applicable rule, and what it does *not* prove.

The orchestrator must resolve disagreements against live evidence, not average
agent opinions. Agents do not edit shared plans during the audit; one writer
applies the synthesized patch.

## Official-source research snapshot

For security or process recommendations, search official sources first and
record the retrieval date. A September 2026 “best practice” claim requires a
dated September source; otherwise describe it as current guidance retrieved on
the actual date, not as a historical September fact.

Useful authoritative sources for this repository's CI/release work:

- [GitHub secure use reference](https://docs.github.com/en/actions/reference/security/secure-use)
  — full-length commit SHA pins are the immutable-action option; verify the SHA
  belongs to the action repository, not a fork.
- [Using pre-written workflow building blocks](https://docs.github.com/actions/writing-workflows/choosing-what-your-workflow-does/using-pre-written-building-blocks-in-your-workflow)
  — full SHAs are reliable but require deliberate update work; do not claim
  pinning also provides automatic security updates.
- [GitHub Actions workflow dependency lockfiles](https://docs.github.com/en/actions/concepts/security/workflow-dependency-lockfiles)
  — check whether the repository uses a lockfile before prescribing SHA pins;
  document the repository's chosen policy rather than mixing models.
- [GITHUB_TOKEN permissions](https://docs.github.com/actions/reference/authentication-in-a-workflow)
  — grant the least permissions required by each workflow or job.
- [Available rules for rulesets](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-rulesets/available-rules-for-rulesets)
  — required checks and review rules are repository state and must be compared
  with the committed ruleset snapshot when the contract is questioned.
- [Automatic pull-request merging](https://docs.github.com/en/pull-requests/how-tos/merge-and-close-pull-requests/automatically-merging-a-pull-request)
  — auto-merge waits for required reviews and checks; it is not evidence that
  checks passed before it was armed.
- [Troubleshooting required status checks](https://docs.github.com/en/pull-requests/how-tos/merge-and-close-pull-requests/troubleshooting-required-status-checks)
  — confirm workflow events and check names when local and GitHub states differ.

For action pins, round-trip every new SHA before writing it:

```bash
gh api repos/OWNER/ACTION/commits/TAG --jq .sha
gh api repos/OWNER/ACTION/commits/FULL_SHA --jq .sha
```

The second command is load-bearing: a tag lookup can return an annotated tag
object rather than a commit. Never accept an unverified 40-hex string.

## Codacy and coverage state

Codacy pull-request analysis is diff-scoped; a green PR check does not prove the
repository-wide backlog is empty. Read both views when credentials permit:

```bash
codacy -o json pull-request <PR>
codacy -o json issues gh OWNER REPO --branch main --limit 500
```

Treat `coverage: None`, missing project-scoped tokens, or an unauthenticated
command as a real blocker/procedure limitation. Do not copy local login tokens
into CI or plans. Record the required secret name and scope, not the secret.

## Verification and handoff

After a meaningful plans or skill edit:

```bash
git diff --check
npm run gate:fast
python3 scripts/check-skills.py
python3 scripts/test-skills.py
```

Run `npm run gate:full` before a PR that touches product code or CI behavior;
for documentation-only work, the fast gate plus the skill checks are the
smallest meaningful verification. Before merge, run `npm run gate:pr`, inspect
the full commit list, and perform the repository's cited adversarial review.

Use this handoff shape:

```markdown
## Reconciliation
**As of:** YYYY-MM-DD
**Head:** `<sha>`
**Current state:** <release, issue/PR counts, working tree>
**Shipped:** <PR/commit/release + evidence>
**Active follow-ups:** <state-labeled rows only>
**Blocked:** <credential, decision, authority, or evidence>
**Checks actually run:** <commands and remote runs>
**Official sources:** <URLs + retrieval date + applicable claim>
**Next evidence:** <one concrete command or decision>
```

Stop and ask for human judgment when scope is disputed, an ADR decision is
needed, credentials/authority are missing, a required third-party check should
be relaxed, or a production rollback is proposed. Never force-push, bypass a
red required check, or mark “could not verify” as passed.

## Distilled failure learnings

- **L-011:** a correct diff can still be on the wrong branch. Start from the
  current base and inspect `git log origin/main..HEAD`; keep only intended,
  session-owned commits.
- **L-016:** a gate script is not a gate. State whether each sensor runs locally,
  in CI, or both, and inspect the workflow rather than trusting the script.
- **L-017:** required Codacy rules must have local parity, and PR analysis is not
  repository intake. Fix code or configuration; never suppress a locked rule.
- **L-001:** generated `web/pkg` freshness can differ locally and in CI. Compare
  source/output fingerprints and make the sensor fail closed.
- **L-005/L-015:** coordinated pins and tag dereferencing require verification;
  dependency/CLI pins and action SHAs must resolve to the intended commits.
- **Release drift:** a release branch may own generated changelog content while
  `main` owns the unreleased working changelog. Document both intentionally.
- **Status drift:** “open,” “pending,” and “next” are claims with expiry dates.
  Re-query remote state before changing counts and label old delivery prose.
