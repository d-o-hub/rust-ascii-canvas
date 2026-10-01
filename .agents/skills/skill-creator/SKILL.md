---
name: skill-creator
description: Guide for creating or updating effective agent skills with focused instructions, reusable resources, and verifiable repository integration.
license: Complete terms in LICENSE.txt
---

# Skill Creator

A skill is a focused procedure, not an encyclopedia. Keep local `SKILL.md` files
at **300 lines or fewer**; put conditional detail in `references/`, executable
helpers in `scripts/`, and reusable output in `templates/` or `assets/`.

## Start with ownership and concrete uses

1. Read [the repository skill contract](../../../agents-docs/SKILL_TEMPLATE.md)
   and `.agents/skill-manifest.json` before editing.
2. If `skills-lock.json` owns the skill, do not patch imported content locally.
   Repair omitted shared resources, add a named local adapter, or document a
   narrow provenance-aware exception. An upstream re-import is a separate,
   reviewed change that updates lock metadata honestly.
3. For locally owned skills, identify what requests trigger the skill, what
   inputs it needs, what it produces, and when it must stop for human judgment.
4. Reuse an existing skill instead of adding a near-duplicate.

## Keep context focused

- Assume the agent already knows general programming. Include repository-specific
  commands, unusual failure modes and decision points, not generic tutorials.
- Put invocation conditions in the frontmatter `description`; keep `name` equal
  to the directory name. Use plain or folded text, not executable metadata.
- Give exact commands for fragile operations; permit judgment where several
  safe approaches are valid.
- Link every supporting file from the main skill or a directly linked reference.
  Resolve Markdown links relative to the containing file, not the repository.
- Mark hypothetical paths with explicit placeholders or put examples in fenced
  blocks. Real prose resource paths are checked for existence.
- Do not add unrelated README, changelog, installation guide or license changes.
  Preserve an existing license when adapting locally maintained content.

## Create or update

For a new local skill, the existing scaffolder is available:

```bash
python3 .agents/skills/skill-creator/scripts/init_skill.py <skill-name> --path .agents/skills
```

Then:

1. Replace the scaffold's placeholders; remove unused example directories.
2. Write the shortest complete procedure: prerequisites, actions, expected
   evidence, failure handling, escalation and integration with installed skills.
3. Prefer tested scripts for deterministic repeated work rather than asking
   agents to reconstruct the same shell/Python logic each time.
4. Use [workflow patterns](references/workflows.md) for sequential/conditional
   procedures and [output patterns](references/output-patterns.md) for templates.
5. Declare ownership, metadata fields, required skills and load-bearing commands
   in `.agents/skill-manifest.json`. Missing specialist skills need an explicit
   local adapter, not a request to invoke a nonexistent subagent.
6. Run new helpers with positive **and negative** fixtures. Do not execute
   destructive or network-mutating examples merely to validate their spelling.

## Verify repository integration

From the repository root:

```bash
python3 scripts/check-skills.py
python3 scripts/test-skills.py
```

These use only Python's standard library and run offline. They validate the
manifest, installed lock entries, metadata, real resource paths, declared
commands and skills, length budgets and stale exceptions. They do not execute
commands, authenticate services, prove remote URLs reachable, or validate every
hypothetical shell snippet.

Run `npm run gate:fast` before handoff once concurrent work is integrated. A new
sensor must run directly in the applicable CI job as well as locally; adding it
only to the convenience gate script does not block a merge.

## Packaging (optional distribution step)

For distribution outside this repository, the existing packaging helper remains:

```bash
python3 .agents/skills/skill-creator/scripts/package_skill.py <path/to/skill-folder>
```

It uses [quick_validate.py](scripts/quick_validate.py), whose PyYAML dependency
and upstream metadata rules are **not** the repository's offline checker.
Packaging is not required for ordinary edits to installed local skills. Do not
install PyYAML merely to run the two repository sensor commands above.

## Iterate from evidence

Use the skill on a real task; note failures, missing resources, stale commands
and excess context. Fix the guide or sensor for repeated failure classes, run
fixtures again, and record the result without claiming unrun CI or merged work.
