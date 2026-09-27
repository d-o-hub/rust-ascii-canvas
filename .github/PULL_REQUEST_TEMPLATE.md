## Summary

<!-- Briefly describe your changes -->

## Changes made

<!-- What changed and why -->

## Testing (harness)

- [ ] Fast gates: `npm run gate:fast` (fmt, clippy, tests, architecture, web lint/tsc/vitest)
- [ ] Full gates (if product behaviour): `npm run gate:full` (WASM, size, E2E)
- [ ] Or equivalent CI jobs green

## Merge contract

A PR may merge only when all of these hold. Verify with `npm run gate:pr`.

- [ ] `npm run gate:pr` reports **MERGEABLE** (not draft, no conflicts, all checks `SUCCESS`)
- [ ] Every review thread resolved (`pr-roast` / review feedback addressed, not dropped)
- [ ] No outstanding `CHANGES_REQUESTED`
- [ ] Adversarial pass done: `pr-roast` (or an equivalent attack) with no Blocker/Major

Auto-merge is armed with `gh pr merge <PR> --auto --squash`; the ruleset requires
`CI Success` + `PR Readiness (merge gate)` and review-thread resolution.

## Checklist

- [ ] I have read [CONTRIBUTING.md](../CONTRIBUTING.md) and [AGENTS.md](../AGENTS.md)
- [ ] Architecture layers respected (`core` pure; see `agents-docs/architecture.md`)
- [ ] Documentation / ADR updated if decision or harness changed
- [ ] Tests prove the fix or feature (no weakened assertions)
- [ ] If a recurring agent/CI failure was fixed: guide or sensor improved

## Screenshots

<!-- If applicable -->

## Related issues

<!-- e.g. Closes #123 -->
