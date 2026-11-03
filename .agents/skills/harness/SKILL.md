---
name: harness
description: Use the do-harness CLI to select the smallest verification scope, produce concise machine-readable output, and inspect fresh evidence. Trigger for harness setup, sensor failures, workflow performance, or reducing agent context.
---

# Harness workflow

The repository's sensor contract is `do-harness.toml`; the pinned CLI version is
`.do-harness-version`. Use the repository resolver so setup failures are explicit:

```bash
bash scripts/check-do-harness.sh
```

Do not substitute `npx` in the normal edit loop. It adds package-manager notices,
network resolution, and avoidable output. In a devcontainer, the pinned binary is
installed by `scripts/install-do-harness.sh`.

## Progressive disclosure

Load only metadata before selecting a long guide:

```bash
bash scripts/do-harness.sh skills suggest \
  --query "<task>" --limit 5 --format json
```

Read the one or two returned skills that match the task. Do not load every
`SKILL.md` into context.

## Edit loop

Use the smallest useful command and retain JSON for agent-facing output:

```bash
bash scripts/do-harness.sh -q explain --set feedback --changed --format json
bash scripts/do-harness.sh -q verify --set feedback --changed --fail-fast --format json
bash scripts/do-harness.sh -q status --set feedback --format json
```

If the change is already committed, `--changed` sees no working-tree paths.
Use the upstream committed-range selector when available; until then, run the
appropriate focused command or `npm run gate:fast` rather than treating an empty
selection as proof. Path selection narrows work; it does not prove semantic
independence.

## Completion loop

Before handoff, run `npm run gate:full` for product changes. For harness-only
changes, run `bash scripts/check-do-harness.sh`, the affected offline fixtures,
and `npm run gate:fast`. Run `npm run gate:pr` only for an existing pull request.

Treat these states as distinct:

- `PASS`: the selected sensor ran and passed.
- `FAIL`: a sensor ran and found a problem; follow its `FIX:` hint.
- `UNVERIFIED`: the CLI, tool, artifact, or evidence was unavailable.
- `SKIP`: a sensor did not provide strong evidence; strict verification rejects it.

Never use `verify --dry-run` as a safety preview with v0.4.1: upstream tracks a
bug where that mode still spawns sensor processes. Use `explain` for planning.

## Output contract

Prefer `--format json` and `-q` for agent calls. Report only:

1. command and exit code,
2. failed/unverified sensor names,
3. changed files,
4. the next fix command.

Do not paste full passing logs into the conversation. Preserve evidence files
under ignored `.do-harness/` when a later status check needs them.

## Failure handling

Run the failed sensor directly, fix its root cause, and rerun it before the
broader tier. If the same sensor failure recurs three times, inspect recorded
strikes and improve the guide or sensor instead of repeating the same prompt.

Related skills: `verify`, `skill-creator`, `code-review`, and `merge-gate`.
