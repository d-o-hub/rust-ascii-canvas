---
name: pr-roast
description: >
  Adversarially review a pull request against official upstream documentation
  and this repo's harness rules. Use when asked to "roast the PR", "review
  this PR", "red team this diff", "what did we miss", or as the adversarial
  stage before merge. Attacks the change for correctness, API misuse, test
  adequacy, and harness violations — citing the source for every finding.
---

# PR Roast (Adversarial Review)

The **adversarial evaluation** stage of the delivery loop
([agents-docs/delivery.md](../../../agents-docs/delivery.md)). `code-review`
asks "is this sound?"; the roast asks **"how do we break this?"** and must
assume the author — including a previous agent — was wrong.

## When to Use

- Before enabling auto-merge on a PR
- After a large agent-authored diff
- Explicitly asked to "roast" / "red team" / "review this PR"

## Don't Invoke When

- Gates are red — run `verify`; there is no point reviewing broken code
- The diff is docs-only with no ADR or harness claim to check

## Prerequisites

```bash
npm run gate:fast              # minimum
npm run gate:pr                # the mechanical merge contract
```

Gates being green is a **precondition, not a defense**. Every defect the roast
exists to find is, by construction, one the sensors missed.

## Method

For each changed file, run the six passes below. A finding needs **both** a
concrete failure and a citation. Opinions without a failure mode are noise.

### 1. Correctness — can this actually break?

- What input makes this wrong? Trace the error path, not the happy path.
- Off-by-one, empty/degenerate input, integer overflow, `usize` underflow.
- Rust: `unwrap` / `expect` / indexing / `as` casts on a reachable path.
- TS: `undefined` vs `null`, `noUncheckedIndexedAccess`, async races,
  stale closures in event handlers.
- State: does undo/redo restore *every* mutated field, or only the obvious one?

### 2. API misuse — official docs, not memory

Cite the upstream source. Do not assert from recollection.

| Area | Source |
|---|---|
| Rust API/idiom | [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/) (F-naming, C-flexibility, S-type safety, R-errors) |
| Lint rules | [Clippy lint groups](https://rust-lang.github.io/rust-clippy/master/index.html#lint-groups) · [deny-by-default lints](https://doc.rust-lang.org/rustc/lints/listing/deny-by-default.html) |
| `wasm-bindgen` | [wasm-bindgen guide](https://rustwasm.github.io/docs/wasm-bindgen/) — JS↔Rust ownership, `&str` lifetimes |
| TypeScript | [TS Handbook](https://www.typescriptlang.org/docs/handbook/intro.html) · [`strict`](https://www.typescriptlang.org/tsconfig/#strict) |
| DOM / a11y | [MDN](https://developer.mozilla.org/) · [WAI-ARIA APG](https://www.w3.org/WAI/ARIA/apg/) |
| Playwright | [Best practices](https://playwright.dev/docs/best-practices) · [Locators](https://playwright.dev/docs/locators) |
| GitHub Actions | [Workflow syntax](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax) |
| Rulesets / merge | [About rulesets](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-rulesets/about-rulesets) · [Auto-merge](https://docs.github.com/en/pull-requests/collaborating-with-pull-requests/incorporating-changes-from-a-pull-request/automatically-merging-a-pull-request) |


### 3. Test adequacy — would these tests catch it?

- For each test: name the bug it catches. If you cannot, the test is decoration.
- Mutation test the claim: revert the fix, confirm a test goes red.
- Missing: negative case, boundary, regression test for the *original* bug.
- `#[ignore]`, `.skip`, `.todo`, `#[allow]` — all red flags; justify or delete.
- E2E must assert **user-visible** behaviour via resilient locators, never
  `waitForTimeout` (already a hard rule here).

### 4. Architecture fitness

- `src/core/**` must stay pure — no `wasm`, `render`, `ui`, `wasm_bindgen`,
  `web_sys`, `js_sys`.
- Business logic belongs in `core`, not in `wasm/helpers.rs` or `web/*.ts`.
- New file >500 LOC, or growth of an allowlisted file (`.loc-allowlist`).

### 5. Blast radius — what else does this touch?

- Public WASM surface / `.asc` format / clipboard semantics / history-model
  changes are breaking; they need an ADR.
- Version pins: `VERSION` is the SSOT; `wasm-bindgen` dep **and** CLI must move together (L-005).
- Callers of a changed signature; silent behaviour drift behind an unchanged type.

### 6. Harness honesty

- Do the claims match reality? Verify doc/status claims against `git log`, not prose.
- Are PR-template checkboxes truthful?
- If this fixes a recurring failure, was a **guide or sensor** hardened too?
  If not, the same failure recurs — that is the real finding.

## Severity rubric

| Severity | Meaning |
|---|---|
| **Blocker** | Data loss, corruption, wrong result, or a guard-rail that can be bypassed. Do not merge. |
| **Major** | Real defect on a plausible path, or a missing test for a fixed bug. |
| **Minor** | Correct but fragile; will cause a future incident. |
| **Nit** | Style. Do not block on nits. |

## Output format

```markdown
## Roast — PR #<n>

**Verdict:** Ship it | Fix first | Blocked
**Gates:** green? · **Merge contract:** satisfied?

### Blockers
- **<file:line>** — <the failure, concretely>
  Cite: <official doc URL> · Repro: <the input or step that triggers it>

### Major / Minor / Nits
- ...

### What I could not break
- <attacks that failed — evidence the change is sound>

### Harness follow-ups
- <sensor/guide to add, or "none">
```

Including **"what I could not break"** is mandatory. A roast that only lists
faults gives no signal about coverage.

## Rules for the reviewer

- **Never soften a blocker to be agreeable.** False alarms are cheaper than defects.
- **Every finding cites a source or a concrete repro.** No vibes.
- **Do not fix it yourself here** — report, then let the author respond.
- Disputes are settled by evidence, not seniority. If a blocker is wrong, say so.

## Integration

- **After** `verify` (gates) and `code-review` (soundness)
- **Before** `merge-gate` arms auto-merge
- **Related** `verify`, `code-review`, `merge-gate`, `production-loop`, `dogfood`
