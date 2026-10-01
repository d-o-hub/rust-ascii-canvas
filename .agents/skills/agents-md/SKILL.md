---
name: agents-md
description: Agent documentation specialist - creates and maintains AGENTS.md files, documentation structure, and best practices guides
---

# Agents MD

## Purpose

Specializes in creating and maintaining agent documentation, best practices guides, and documentation structure for multi-agent systems.

## When to Use

- Creating new AGENTS.md files from scratch
- Updating existing agent documentation
- Organizing agent skills and workflows
- Creating reference documentation for agent teams
- Documenting task execution patterns and learnings

## Don't Invoke When

- Writing code (use rust-engineer or other coding skills)
- Testing applications (use agent-browser or dogfood)
- Database work (no database-specialist skill is installed; scope and select appropriate tooling first)

## Core Capabilities

### Documentation Creation
- Creating AGENTS.md with best practices
- Organizing skill-based agent documentation
- Setting up agents-docs/ folder structure
- Writing SKILL.md templates for new skills

### Documentation Maintenance
- Updating existing documentation with new learnings
- Keeping PROJECT_STATUS.md current
- Maintaining TECHNICAL_ANALYSIS.md with findings
- Organizing ADRs in plans/ folder

### Best Practices Implementation
- Following length budgets for AGENTS.md and per-skill files
- Creating clear agent invocation patterns
- Documenting escalation triggers
- Setting up integration patterns between skills

## Quick Start

### Create New AGENTS.md
1. Define available agents and their purposes
2. Document best practices for task execution
3. Include file organization structure
4. Add testing workflow guidelines

### Update Documentation After Tasks
1. Run tests and capture results
2. Update PROJECT_STATUS.md with test outcomes
3. Add technical findings to TECHNICAL_ANALYSIS.md
4. Create new ADRs for significant decisions

## Documentation Template

```markdown
# Agent Best Practices - <Project Name>

## Agent System Overview

<Description of agent system>

## Available Agents

### <Agent Name>
- **Purpose**: <What it does>
- **Location**: <File path>
- **Trigger**: <When to use>

## Best Practices

### Task Execution
1. Analyze - Understand requirements
2. Plan - Create steps
3. Execute - Run commands
4. Document - Update plans/

## File Organization
```

## Best Practices

1. **AGENTS.md is an index, not a manual.** Budget ≈160 lines. If a section
   outgrows that, the detail belongs in `agents-docs/` and AGENTS.md keeps the
   link. Root `AGENTS.md` is the only file loaded by default — anything critical
   that lives only in `agents-docs/` will be missed, so the merge contract and
   the architecture rules stay here even though they push the file past the
   original 120-line guideline. Do not duplicate a skill's content here: point at
   the skill instead.
2. **Local per-skill budget: ≤300 lines** for `SKILL.md`. Push detail into
   `references/` and `templates/` (lowercase). Ownership, metadata and exact
   upstream exceptions are declared in `.agents/skill-manifest.json`; locked
   imports are not silently rewritten to meet a local budget. See
   [the schema and maintenance procedure](../../../agents-docs/SKILL_TEMPLATE.md).
   Run `python3 scripts/check-skills.py` and `python3 scripts/test-skills.py`;
   both are offline stdlib checks, not the upstream PyYAML packaging helper.
3. Always document in plans/ folder for architectural decisions
4. Update documentation after each successful task
5. Use consistent formatting across all docs
6. Link skills to their SKILL.md files
7. **Coherence check before finishing:** a guide that contradicts a sensor is a
   bug. If you change a threshold in `AGENTS.md`, confirm `quality-gates.sh` and
   CI agree — see `agents-docs/harness.md` §Coherence rules.
