# Agent Skills - Reference Documentation

Reference docs for agent skills. Operational harness: [harness.md](harness.md), [architecture.md](architecture.md). Root directives: [`../AGENTS.md`](../AGENTS.md).

## Harness-critical skills

### verify
- **Location**: `.agents/skills/verify/SKILL.md`
- **Role**: Computational feedback — run `gate:fast` / `gate:full` / `gate:pr`, self-correct
- **Use when**: After code changes, before PR, CI failures

### code-review
- **Location**: `.agents/skills/code-review/SKILL.md`
- **Role**: Inferential feedback — architecture, failure modes, harness coherence
- **Use when**: Gates green; before merging

### pr-roast
- **Location**: `.agents/skills/pr-roast/SKILL.md`
- **Role**: Inferential feedback, **adversarial** — "how do we break this?", cited against official docs
- **Use when**: Before merging; after a large agent-authored diff; when asked to "roast" or red-team a PR

### codacy
- **Location**: `.agents/skills/codacy/SKILL.md`
- **Role**: Triage Codacy findings and status on a PR. The **policy** layer: what may and may not be done to a required check, and how to classify a finding
- **Use when**: Any Codacy warning, issue, or failing/stuck `Codacy Static Code Analysis` check on a PR — and before deciding a Codacy-blocked PR may merge
- **Tooling (upstream, MIT)**: `codacy-cloud-cli`, `codacy-code-review`, `configure-codacy`, `configure-codacy-cloud`, `codacy-analysis-cli`, `setup-coverage` — requires the `codacy` CLI plus `CODACY_API_TOKEN` or `codacy login`

### merge-gate
- **Location**: `.agents/skills/merge-gate/SKILL.md`
- **Role**: The merge contract — decide, and arm auto-merge. Backed by `scripts/pr-merge-gate.sh`
- **Use when**: Before merging anything; when a PR will not merge

### production-loop
- **Location**: `.agents/skills/production-loop/SKILL.md`
- **Role**: failure → reproduce → fix → evaluate → adversarial → shadow → canary → promote/rollback
- **Use when**: A production bug arrives; shipping a risky change; rolling out or rolling back

### tool-validation
- **Location**: `.agents/skills/tool-validation/SKILL.md`
- **Role**: Behaviour harness for the 8 drawing tools
- **Use when**: Tool or canvas interaction changes

### goap-adr-planner
- **Location**: `.agents/skills/goap-adr-planner/SKILL.md`
- **Role**: Feedforward planning / ADRs
- **Use when**: Multi-step work, architecture decisions, or changing a merge guard-rail

## Available skills

### rust-engineer
- **Location**: `.agents/skills/rust-engineer/SKILL.md`
- **Purpose**: Rust specialist with expertise in async programming, ownership patterns, FFI, and WebAssembly development
- **Use when**: Building high-performance backend services, systems programming, WASM development

### rust-best-practices
- **Location**: `.agents/skills/rust-best-practices/SKILL.md`
- **Purpose**: Guide for writing idiomatic Rust code based on best practices
- **Use when**: Writing new Rust code, reviewing/refactoring, deciding ownership patterns

### agent-browser
- **Location**: `.agents/skills/agent-browser/SKILL.md`
- **Purpose**: Browser automation CLI for AI agents
- **Use when**: Navigating pages, filling forms, clicking buttons, taking screenshots, testing web apps

### dogfood
- **Location**: `.agents/skills/dogfood/SKILL.md`
- **Purpose**: Systematically explore and test a web application to find bugs and UX issues
- **Use when**: QA testing, exploratory testing, bug hunting, finding issues

### skill-creator
- **Location**: `.agents/skills/skill-creator/SKILL.md`
- **Purpose**: Guide for creating effective skills that extend agent capabilities
- **Use when**: Creating new skills or updating existing ones

### goap-adr-planner
- **Location**: `.agents/skills/goap-adr-planner/SKILL.md`
- **Purpose**: Goal-Oriented Action Planning with Architectural Decision Records
- **Use when**: Planning multi-step tasks, creating plans in plans/ folder, documenting decisions with ADRs

## Agent Configuration

### goap-adr-analyzer
- **Location**: `.opencode/agent/goap-adr-analyzer.md`
- **Purpose**: Analyze GOAP implementations and manage ADRs in plans/ folder
- **Use when**: Multi-step tasks requiring architectural decisions

## Best Practices Summary

1. Always use appropriate skill for the task
2. Document architectural decisions in plans/ folder
3. Run tests before marking tasks complete
4. Keep code under 500 LOC per file
5. Update PROJECT_STATUS.md and TECHNICAL_ANALYSIS.md with findings
6. AGENTS.md is an index: keep it ≈160 lines and push detail into `agents-docs/`
7. A guide that contradicts a sensor is a bug — reconcile before finishing

## Retired skills (2026-09-27, ADR-044)

| Skill | Why retired |
|-------|-------------|
| `my-pull-requests` | A thin `gh` wrapper; the `merge-gate` skill covers PR state properly |
| `ln-732-cicd-generator` | Generated .NET/Python CI and would **overwrite** this repo's `ci.yml` |
| `create-github-pull-request-from-specification` | Referenced tool syntax (`create_pull_request`, `${workspaceFolder}`) that does not exist in this harness |

Their `skills-lock.json` entries were removed as well — a lockfile sync would
otherwise resurrect them.

## Vendored upstream skills

| Skill | Source |
|-------|--------|
| `codacy-analysis-cli`, `codacy-cloud-cli`, `codacy-code-review`, `configure-codacy`, `configure-codacy-cloud`, `setup-coverage` | [codacy/codacy-skills](https://github.com/codacy/codacy-skills) (MIT — see `LICENSE-codacy-skills.txt`) |

Tracked in `skills-lock.json` with a content hash. **Do not edit these locally**:
a re-sync would overwrite the change, and the hash would no longer match. Repo-
specific Codacy policy lives in the local `codacy` skill instead.
