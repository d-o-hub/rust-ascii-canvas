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
