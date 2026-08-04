# ADR-040: Codacy Security Pattern Remediation for Test Files

## Status
Accepted

## Context
PR #155 introduced 8 high-severity Codacy security alerts in `web/ux.test.ts`:
- **2x** "HTML passed in to function `Object.getPrototypeOf`" (lines 69, 110)
- **5x** "Generic Object Injection Sink" — bracket notation `state[variable]` (lines 117–119, 131, 134)
- **1x** "Non-HTML variable used to store raw HTML" (line 118)

These are triggered by Codacy's `eslint-plugin-security` rules (specifically `detect-object-injection` and `detect-html-unclosed-tag`). The underlying tool is Semgrep with ESLint security plugins layered on top.

## Decision
Fix all 8 alerts by restructuring the test code to use **direct typed property access** and **element-level spies** instead of dynamic bracket notation and prototype traversal.

### Pattern fixes

| Codacy Rule | Problem Pattern | Fix Pattern |
|---|---|---|
| `detect-object-injection` | `state[key] = value` where key is a variable | `state.canvas = value` (direct property access) |
| `detect-html-unclosed-tag` | `Object.getPrototypeOf(htmlElement)` | `vi.spyOn(element, 'method')` (spy on element directly) |
| `non-html-variable-raw-html` | `state[variableKey] = htmlElement` | `state.statusToast = element` (typed assignment) |

### Why not suppress?

Codacy's `detect-object-injection` is notoriously false-positive-prone (acknowledged by the eslint-plugin-security maintainers and Codacy's own Smart False Positive Triage system). However, the fixes here also improve code readability — direct property access is clearer than dynamic bracket notation in tests.

## Consequences
- All 8 Codacy high-severity alerts eliminated
- Test code becomes more readable with explicit property names
- `vi.spyOn(element, 'focus')` is more idiomatic Vitest than prototype spying
- No behavioral change to test assertions

## Follow-up: eliminate the remaining "Non-HTML variable" alerts

After `aa24a59`, Codacy still reported **2x** "Non-HTML variable used to store raw HTML" on
`const canvasNode = document.getElementById('canvas') as HTMLCanvasElement;` and
`const widthInput = document.getElementById('grid-width') as HTMLInputElement;`.

Root cause (verified by reproducing Codacy's analysis locally with `eslint-plugin-xss`'s
`no-mixed-html` rule at its default configuration): the `as HTMLCanvasElement` / `as
HTMLInputElement` type-cast identifiers contain `html`, so they infect the declaring
statement, and the rule then requires the **variable name** to match the `html`
naming rule. But naming the variable `canvasHtmlElement` re-triggers the sibling
`HTML passed in to function 'vi.spyOn'` alert, because html-named identifiers passed as
function arguments are treated as raw HTML.

Resolution: drop the type casts entirely. `document.querySelector('canvas')` and
`document.querySelector('input#grid-width')` infer `HTMLCanvasElement | null` /
`HTMLInputElement | null` from the tag-name literal (no cast identifier, no html-named
node), which satisfies both the type checker and the Codacy rule. The `!` non-null
assertion is used only where TS requires a non-null target (`vi.spyOn`, `dispatchEvent`).

### Learnings (for the harness)
- Codacy's `xss/no-mixed-html` rule reports on **new diff lines only**; identical
  statements are deduplicated (line 68 vs line 105 produced one finding).
- The rule can be reproduced locally: ESLint 8 + `@typescript-eslint/parser` +
  `eslint-plugin-xss` with default options (`xss/no-mixed-html: ['error']`).
- Type-cast identifiers (`HTML*Element`) in variable initializers trigger the rule;
  prefer `querySelector` tag-literal inference over `as` casts in test code.

## Follow-up 2: eliminate the "Forbidden non-null assertion" alerts

After `51f7f4f`, Codacy still reported **2x** "Forbidden non-null assertion"
(`@typescript-eslint/no-non-null-assertion`, ErrorProne/high) on the two added lines
using the `!` operator:
- `vi.spyOn(canvasNode!, 'focus')`
- `widthInput!.dispatchEvent(enterEvent)`

These were introduced by `51f7f4f` itself, which used `!` as a replacement for the
removed `as HTML*Element` casts. Codacy's rule set is stricter than the local
`web/eslint.config.js`, which does not enable `no-non-null-assertion`.

Resolution: replace `!` with guard clauses that narrow the type for TypeScript:

```ts
const canvasNode = document.querySelector('canvas');
if (!canvasNode) {
    throw new Error('canvas element must exist in test DOM');
}
const focusSpy = vi.spyOn(canvasNode, 'focus');
```

This satisfies both `tsc` narrowing and the Codacy rule without reintroducing
`as HTML*Element` casts (which would re-trigger `xss/no-mixed-html`).

### Learnings (for the harness)
- Codacy checks run `@typescript-eslint` recommended rules **on top of** the repo's
  local ESLint config; `no-non-null-assertion` is the strictest surprise candidate.
  When replacing casts with `!` assertions, prefer guard clauses instead.
- Local reproduction recipe: ESLint 8 + `@typescript-eslint/parser` +
  `@typescript-eslint/eslint-plugin` with
  `@typescript-eslint/no-non-null-assertion: ['error']`.
