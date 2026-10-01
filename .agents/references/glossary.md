# Codacy Shared Glossary

Local compatibility resource, added 2026-09-30. The installed Codacy skills
reference this shared path, but the installer omitted it. This is a concise
repo-maintained summary of the installed skills, **not** a verbatim upstream
file or a change to their locked content. Provenance: `github/codacy/codacy-skills`
entries in [skills-lock.json](../../skills-lock.json). Prefer CLI `--help` for
current argument shapes; the local [Codacy policy](../skills/codacy/SKILL.md)
controls merge decisions.

## Provider

| Host | CLI provider value |
|------|--------------------|
| GitHub | `gh` |
| GitLab | `gl` |
| Bitbucket | `bb` |

Repository parameters identify provider, organization and repository. The Cloud
CLI can infer them from the git remote; explicit values avoid ambiguity.

## Issue

A static-analysis result tied to a file/line, tool, pattern and analyzed commit.
A PR list is **diff-scoped**: no new issues does not prove the repository backlog
is empty. Read repository-level issues too. Do not claim a fix is verified on
Codacy until the new head has been analyzed and read back.

## Finding

A reported problem in an analysis/security result. Keep the tool, category,
severity, source location and commit attached; do not conflate a scanner failing
to run with a scanner finding no problems.

## Severity

Severity estimates impact/urgency. Preserve the API/CLI's labels (for example
Critical, High, Medium, Low) rather than assuming every analyzer has identical
numeric levels. Local repository intake fails on Critical/High. A required PR
check's verdict still governs the merge; an unauthenticated read is unverified.

## Tool and pattern

A **tool** is an analyzer; a **pattern** is a rule it applies. Identifiers are
case-sensitive. Organization/default coding standards can lock patterns so a
repository cannot disable them. Fix code or escalate; do not suppress a finding
or weaken a required check to manufacture success.

## Coverage

The fraction of executable lines/branches exercised by tests, from a generated
coverage report. Repository and diff coverage answer different questions.
No uploaded report or a goal of `None` does not mean complete coverage. CI
uploads require an authorized project-scoped credential, not a machine-local
account login.

## Analysis and capability

An analysis is evidence about a specific commit and the tools that actually ran.
Empty `toolResults`, unavailable capabilities, stale results and authentication
failures are not clean scans. Cloud-only analyzers are not proven by a local
Analysis CLI run. Record the analyzed head and any unavailable tool explicitly.
