# Skill Contract and Template

Updated 2026-09-30. Local skill inventory and exceptions live in
[.agents/skill-manifest.json](../.agents/skill-manifest.json). The importer lock is
[skills-lock.json](../skills-lock.json); it is not the ownership policy by itself.

## Layout and template

Every installed skill has `.agents/skills/<name>/SKILL.md`. Local main files have
at most **300 lines**. Put detail in lowercase `references/`, repeatable commands
in `scripts/`, reusable output in `templates/`, and assets in `assets/`.

```markdown
---
name: <skill-name>
description: <what this does, when it is useful, and concrete trigger phrases>
---

# <Skill Name>

## Procedure
1. Read prerequisites and choose the smallest safe scope.
2. Run the real repository command from its declared working directory.
3. Verify positive and negative outcomes; record what was actually run.

## Failure and escalation
Stop for <specific ambiguous/destructive decision>; never bypass a red sensor.

## References
[Procedure details](references/<resource>.md)
```

The frontmatter checker handles the repository's small subset: top-level keys
with scalar, folded or literal text and indented metadata continuations. It is
not a general YAML parser. `name` must match the directory; `description` must be
nonempty text. The manifest lists the **exact metadata field set** per skill;
missing/extra fields fail. Imported Codacy `license`/`metadata` and TypeScript
`bundle`/display fields are explicitly preserved, not silently normalized to a
local two-field schema. Do not put executable expressions in frontmatter.

## Ownership and provenance

Each `skills` entry declares:

- `owner`: `local` or `upstream` (only locked entries may be upstream).
- `metadata`: exact allowed top-level frontmatter fields.
- `requires`: installed skill names or explicit adapter aliases.
- For upstream entries: an exact `lock` snapshot (`source`, `sourceType`,
  `computedHash`) and SHA-256 of the installed `SKILL.md`.

Installer hashes are **opaque**: some include resources, so they are not assumed
to be hashes of `SKILL.md`. The separate SHA-256 detects accidental local edits
to the imported main file; resource existence checks still cover imported
references. It is a coherence sensor, not cryptographic upstream attestation.
All lock entries must be installed and manifest-owned; an orphan lock entry,
unmanifested skill or ownership mismatch fails. Preserve license and provenance
when re-importing. Do not silently patch locked files or regenerate their lock
hashes to bless local changes.

Current intentional compatibility repairs:

- `.agents/references/glossary.md` is a **locally authored shared resource**, not
  a claimed upstream copy. It restores the omitted Codacy glossary path without
  touching six locked skills.
- `repo-typescript` maps the three uninstalled TypeScript specialist referrals
  to installed tools/skills and real repository commands. Read it alongside
  `typescript-expert`; never ask for a nonexistent specialist agent.
- Five exact `rust-wasm` prose paths are skill-root-relative in the locked
  upstream references. Manifest resource exceptions name the specific document,
  literal target, provenance, reason and **existing replacement file**. They do
  not exclude the skill or its resources wholesale.
- Seven upstream main files exceed the local length rule. Their exceptions pin
  the **exact existing line count**; growth or shrinkage requires review, and an
  import reduced to 300 lines makes its exception stale. Local skills get no
  length exception. `skill-creator` was shortened instead.

## Checker contract

From the repository root, with Python 3.9+ and **no third-party packages/network**:

```bash
python3 scripts/check-skills.py
python3 scripts/test-skills.py
```

Both commands run in local fast/full gates and directly in CI's applicable
architecture job. Check the current `.github/workflows/ci.yml` when changing
wiring: the workflow does not call `quality-gates.sh`.

The checker inspects Markdown files under `.agents/`:

- Real relative Markdown links (including reference-style links) resolve against
  the containing document. Fragments are ignored for existence checks.
- Explicit bundled paths in prose are checked too. Locally owned procedure
  paths may resolve to an existing repository-root script; Markdown links never
  get that fallback.
- Fenced examples, remote URLs and explicit placeholders (`<name>`, `{PATH}`,
  glob/variable paths, generic template `link`) are not real resource claims.
  Use placeholders in report templates, not fake concrete screenshot filenames.
- `commands` declares skill, literal command text, working directory, kind
  (`package` or `file`) and target. The literal must still occur in that skill;
  the package script/file must exist. Commands are **never executed** by this
  checker. Declare all load-bearing local commands when adding a procedure.
- `adapters` names unavailable aliases, installed target skill and reason. The
  adapter skill must document the alias. Missing targets and stale aliases fail.
- Exceptions are upstream-only, source-matched and exact. Missing reasons,
  invalid replacement paths, duplicate/stale exceptions all fail. No wildcard
  exception can exempt an entire owned guide.

Limits: this is not an arbitrary shell/YAML/Markdown interpreter, external-link
crawler, command-behavior test, credential check or semantic review. It does not
infer every skill name hidden in prose; explicit dependency declarations are the
contract. Retain tests for missing files/commands/skills and for stale exceptions.

## Maintenance workflow

1. Use `skill-creator` to create or edit a **locally owned** guide.
2. Add real resources and declare ownership/metadata/dependencies/commands.
3. For a locked import, prefer a missing shared resource or local adapter. If
   upstream assumptions cannot apply here, add only a source-specific documented
   exception. Fix or remove it on re-import; do not bypass stale detection.
4. Run both offline commands, then the appropriate verification tier.
5. Update the root index only if needed; keep detailed guidance in the skill.
   Do not edit a nonexistent `agents-docs/AGENTS.md`.
6. Preserve historical decisions and distinguish working-tree candidates from
   changes actually merged/released.
