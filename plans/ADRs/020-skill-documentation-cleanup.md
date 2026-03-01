# ADR-020: Skill Documentation Cleanup

## Status
**Proposed** - 2026-03-01

## Context

Analysis of `.agents/skills/` directory identified "AI sloppy" content that should be removed:
- Useless generation artifacts
- Vague placeholder sections
- Broken file references
- Outdated path references

## Decision

Clean up skill documentation by removing or fixing identified issues:

### Files to Delete

| File | Reason |
|------|--------|
| `.agents/skills/vite/GENERATION.md` | Useless build artifact metadata |
| `.agents/skills/ln-732-cicd-generator/diagram.html` | Broken CSS reference, orphaned file |

### Files to Edit

#### 1. typescript-expert/SKILL.md

Remove vague placeholder section (lines ~339-340):
```markdown
## When to Use
This skill is applicable to execute the workflow or actions described in the overview.
```

Remove unused metadata from frontmatter:
- `category: framework`
- `displayName: TypeScript`
- `color: blue`
- `risk: unknown`
- `source: community`
- `bundle: [typescript-type-expert, typescript-build-expert]`

#### 2. ln-732-cicd-generator/SKILL.md

Remove or update the "Paths:" note at the top:
```markdown
> **Paths:** File paths (`shared/`, `references/`, `../ln-*`) are relative to skills repo root.
```

This reference to "skills repo root" doesn't exist in this project context.

#### 3. skill-creator/SKILL.md

Either:
- Add `LICENSE.txt` file to the skill directory, OR
- Remove `license: Complete terms in LICENSE.txt` from frontmatter

### Files to Evaluate

| File | Consideration |
|------|---------------|
| `.agents/skills/ln-732-cicd-generator/` | Entire skill may not be needed - designed for .NET/Python stacks, this is a Rust/WASM project |

## Consequences

### Positive
- Cleaner, more maintainable documentation
- No confusing placeholder content
- All file references work correctly

### Negative
- Minor: Need to verify no other files reference deleted content

## Implementation

1. Delete `.agents/skills/vite/GENERATION.md`
2. Delete `.agents/skills/ln-732-cicd-generator/diagram.html`
3. Edit `.agents/skills/typescript-expert/SKILL.md`:
   - Remove "When to Use" section
   - Remove unused frontmatter fields
4. Edit `.agents/skills/ln-732-cicd-generator/SKILL.md`:
   - Remove "Paths:" note
5. Evaluate if `ln-732-cicd-generator` skill is needed for this project

## Cleanup Summary

| Action | Count |
|--------|-------|
| Files to delete | 2 |
| Files to edit | 3 |
| Skills to evaluate | 1 |
