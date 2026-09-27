# Skill Template Reference

This document provides templates and guidelines for creating new agent skills.

## Skill File Structure

Each skill should have:
- `SKILL.md` - Main skill definition (required)
- `references/` - Additional reference docs (lowercase — this is what the repo uses)
- `templates/` - Reusable templates (lowercase)
- `scripts/` - Executable helpers for deterministic steps

Budget: **≤300 lines** for `SKILL.md`. Anything longer goes in `references/`.
Do not create `README.md`, `CHANGELOG.md`, or other auxiliary files inside a skill.

## SKILL.md Template

```markdown
---
name: <skill-name>
description: <brief description of what the skill does>
---

# <Skill Name>

## Purpose

<Detailed description of the skill's purpose>

## When to Use

- <Scenario 1>
- <Scenario 2>
- <Scenario 3>

## Don't Invoke When

- <Scenario 1>
- <Scenario 2>

## Core Capabilities

### <Capability 1>
- <Detail 1>
- <Detail 2>

### <Capability 2>
- <Detail 1>
- <Detail 2>

## Quick Start

<Steps to use the skill>

## Best Practices

1. <Practice 1>
2. <Practice 2>
3. <Practice 3>

## Integration Patterns

### <Related Skill>
- Handoff: <description>
- Tools: <relevant tools>
```

## Creating a New Skill

1. Create folder: `.agents/skills/<skill-name>/`
2. Add `SKILL.md` with template above
3. Add any reference docs in the `references/` subfolder
4. Add templates in the `templates/` subfolder if needed
5. Update `agents-docs/AGENTS.md` and the skills table in root `AGENTS.md`
6. If the skill is synced from an upstream source, update `skills-lock.json` —
   otherwise a later sync silently resurrects a skill you deleted

## Skill Invocation Patterns

### Direct Invocation
```
skill name:<skill-name>
```

### Via Task Tool
```json
{
  "command": "<description>",
  "description": "<short description>",
  "prompt": "<detailed prompt>",
  "subagent_type": "<skill-name>"
}
```

## Best Practices

1. Keep SKILL.md under 300 lines
2. Use clear, concise descriptions
3. Include practical examples
4. Document integration patterns with other skills
5. Specify escalation triggers when to use other skills
