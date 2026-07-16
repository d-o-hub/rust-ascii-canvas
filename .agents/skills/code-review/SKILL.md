---
name: code-review
description: >
  Inferential code review sensor for this repo. Use before asking a human to review,
  after verify gates are green, or when asked to "review", "review changes", or
  "pre-review". Checks harness compliance, architecture layers, test adequacy, and
  agent failure modes — not a substitute for cargo/clippy/CI.
---

# Code Review (Inferential Feedback)

Semantic review **after** computational sensors (`verify` skill / `npm run gate:*`) are green. Complements CI; does not replace it.

## When to Use

- Before opening a PR or requesting human review
- After a large agent-authored diff
- User asks for review / pre-review / sanity check of changes

## Don't Invoke When

- Gates are still red — run `verify` first
- You need automated rustc/clippy (use computational sensors)

## Prerequisites

```bash
npm run gate:fast   # minimum
# Prefer gate:full if product behaviour changed
```

## Review checklist

### 1. Intent and scope

- [ ] Diff matches the requested goal (no drive-by features)
- [ ] No over-engineering or speculative abstractions
- [ ] Spec / issue / ADR referenced when behaviour changes

### 2. Architecture fitness

- [ ] `core` stays pure (no wasm/web-sys)
- [ ] Logic not dumped into `wasm/helpers.rs` or `web/main.ts` when it belongs in core/modules
- [ ] No new file >500 LOC; allowlisted files not grown without extraction plan
- [ ] Public WASM / document format changes are intentional and tested

### 3. Maintainability

- [ ] Names and module split match existing patterns
- [ ] Error paths handled; no silent `unwrap` on fallible user paths without justification
- [ ] Comments explain **why**, not narrate **what**
- [ ] No secrets, personal emails, or debug leftovers

### 4. Behaviour / tests

- [ ] Tests fail for the bug they claim to catch (not only happy path)
- [ ] Tool/UI changes covered by unit, Vitest, or E2E as appropriate
- [ ] Clipboard, persistence, layers: fidelity edge cases considered
- [ ] No flaky `waitForTimeout` introduced in E2E

### 5. Harness coherence

- [ ] If a recurring agent mistake was fixed, a guide or sensor was updated
- [ ] CI/scripts/docs still agree (`AGENTS.md` tiers vs `quality-gates.sh`)
- [ ] PR description will mention harness changes if any

## Known agent failure modes (watch for)

| Mode | What to look for |
|------|------------------|
| Misdiagnosis | Fix unrelated to root cause; tests green but bug remains |
| Brute-force | Huge unrelated rewrites; `#[allow]` spam; deleted tests |
| Over-engineering | Extra traits/layers for a local fix |
| Incomplete verification | Only ran one unit test; skipped web/architecture |
| Layer bleed | Browser types in `core`; business rules only in TS |

## Output format

```markdown
## Review summary
**Verdict:** Approve | Approve with nits | Request changes
**Gates:** fast/full assumed green? yes/no

## Findings
### Blocking
- ...

### Nits
- ...

## Harness follow-ups
- (sensors/guides to add, or none)
```

## Integration

- **After** `verify`
- **Before** human PR / `create-github-pull-request-from-specification`
- **Escalation:** architecture disputes → ADR via `goap-adr-planner`
