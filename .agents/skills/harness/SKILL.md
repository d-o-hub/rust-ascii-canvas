---
name: harness
description: Use the do-harness CLI to map feedforward guides and feedback sensors, select the smallest verification scope, run the self-correction protocol when a sensor fires, and keep latest-main freshness before coding. Trigger for harness setup, sensor failures, CI failure, workflow performance, or reducing agent context.
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

Distilled harness heuristics live in [references/heuristics.md](references/heuristics.md);
treat them as patterns to adapt, not rules to copy verbatim.

## First-step freshness (latest-main first)

Before planning, inspecting, editing, or delegating subagents in this checkout,
refresh `origin/main` with the CLI:

```bash
bash scripts/do-harness.sh -q sync --format json
```

- **Default (fetch):** updates `refs/remotes/origin/main` without touching the
  working tree or index; a clean `main` fast-forwards only when strictly behind.
- **`--switch`:** checks out `main` first when the worktree is clean, then
  fast-forwards.
- **`--read-only`:** queries `origin/main` via `git ls-remote` without fetching.
- Exit codes: `0` refreshed or current; `1` dirty or diverged checkout refused;
  `2` git/remote/config failure. Fail closed: never present stale local state as
  "latest".

When the installed binary predates the `sync` command, the bundled bootstrap
fallback performs the fetch/query half:

```bash
bash .agents/skills/harness/scripts/latest-main.sh [--root DIR] [--read-only]
```

## Feedforward guides

| Guide | Path |
|-------|------|
| Agent contract | `AGENTS.md` |
| Sensor contract | `do-harness.toml` |
| Harness map | `agents-docs/harness.md` |
| Steering log | `agents-docs/harness.md` (learned failure modes) |
| Harness skill | `.agents/skills/harness` (this skill) |

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

The same commands are wrapped as `npm run harness:explain`, `npm run
harness:feedback`, and `npm run harness:status`.

If the change is already committed, `--changed` sees no working-tree paths.
Use the upstream committed-range selector when available; until then, run the
appropriate focused command or `npm run gate:fast` rather than treating an empty
selection as proof. Path selection narrows work; it does not prove semantic
independence.

Before an expensive run, pre-flight the exact plan with nothing executed:

```bash
bash scripts/do-harness.sh -q verify --set feedback --changed --simulate --format json
```

## Completion loop

Before handoff, run `npm run gate:full` for product changes. For harness-only
changes, run `bash scripts/check-do-harness.sh`, the affected offline fixtures,
and `npm run gate:fast`. Run `npm run gate:pr` only for an existing pull request.

Treat these states as distinct:

- `PASS`: the selected sensor ran and passed.
- `FAIL`: a sensor ran and found a problem; follow its `FIX:` hint.
- `UNVERIFIED`: the CLI, tool, artifact, or evidence was unavailable.
- `SKIP`: a sensor did not provide strong evidence; strict verification rejects it.

A sensor suite that runs zero checks passes vacuously; that is not evidence.

## Output contract

Prefer `--format json` and `-q` for agent calls. Report only:

1. command and exit code,
2. failed/unverified sensor names,
3. changed files,
4. the next fix command.

Do not paste full passing logs into the conversation. Preserve evidence files
under ignored `.do-harness/` when a later status check needs them.

## Self-correction protocol

When a computational sensor fires:

1. Read the full error message — it includes a fix hint.
2. Classify the error: fmt / check / lint / test / schema.
3. Apply the minimal fix — do not refactor unrelated code.
4. Re-run the specific sensor directly.
5. Only proceed when the sensor is green, then re-run the broader tier.

Run the failed sensor directly, fix its root cause, and rerun it before the
broader tier. If the same sensor failure recurs three times, inspect recorded
strikes and improve the guide or sensor instead of repeating the same prompt.

## Fail-fast policy

If the same subtask fails a sensor 3 consecutive times: halt, record the error
signature, and surface a diagnostic to the developer.

## Steering loop

When any sensor fires repeatedly, update the feedforward guide that should have
prevented it — `AGENTS.md`, `agents-docs/`, or a skill — instead of only fixing
product code, and record the pattern in the steering log
(`agents-docs/harness.md`). The loop closes: sensors fire -> guides update ->
sensors fire less.

Related skills: `verify`, `skill-creator`, `code-review`, and `merge-gate`.
