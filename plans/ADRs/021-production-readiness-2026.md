# ADR-021: Production Readiness 2026

## Status
**Accepted** - 2026-03-01

## Context

The ASCII Canvas Editor has been developed with core functionality working, but a comprehensive production readiness review identified multiple issues that must be addressed before deployment:

### Critical Issues
1. **Clippy Error**: `from_str` method should implement `std::str::FromStr` trait
2. **Rust Code Quality**: Unsafe patterns, unwrap() calls, duplicate methods
3. **TypeScript Standards**: Non-null assertions, missing types, scattered config
4. **E2E Test Gaps**: Missing pan tests, no cross-browser, flaky patterns
5. **Documentation**: Skill files with vague content or broken references

### Research Findings (2026 Best Practices)

#### Rust/WASM
- `wasm-bindgen` remains the gold standard for JS interop
- Use `serde-wasm-bindgen` for complex type serialization
- Enable `console_error_panic_hook` for debugging
- Optimize with `opt-level = "s"` and LTO

#### TypeScript
- Strict mode is now default for new projects
- Use `unknown` instead of `any` for external data
- Enable `noUncheckedIndexedAccess: true`
- Defensive programming at API boundaries

#### Playwright
- Page Object Model essential for maintainability
- Use role-based locators (`getByRole()`)
- Replace `waitForTimeout` with proper assertions
- Enable cross-browser testing with matrix

#### GitHub Actions
- Use `dtolnay/rust-toolchain` for Rust setup
- Cache Cargo directories for 3-5x speedup
- Use `fail-fast: false` in matrix strategies
- OIDC for secure cloud deployments

## Decision

Implement a phased approach to production readiness:

### Phase 1: Critical Clippy Fix
Implement `FromStr` trait for `LineDirection` instead of standalone `from_str()` method.

### Phase 2: Rust Production Fixes (ADR-017)
1. Verify `as_any_mut()` in all tools
2. Fix duplicate `delete()` in text.rs
3. Replace `unwrap()` with proper error handling

### Phase 3: TypeScript Standards (ADR-018)
1. Create centralized configuration
2. Add proper WASM type definitions
3. Add ARIA attributes for accessibility

### Phase 4: E2E Enhancement (ADR-019)
1. Enable cross-browser testing
2. Create Page Object Model
3. Add pan functionality tests

### Phase 5: Documentation Cleanup (ADR-020)
1. Commit deleted skill files
2. Verify SKILL.md fixes

### Phase 6: Git Workflow
1. Create feature branch
2. Make atomic commits
3. Create pull request

## Consequences

### Positive
- Production-ready codebase
- All CI checks passing
- Cross-browser compatibility
- Maintainable test suite
- Clean documentation

### Negative
- Time investment for refactoring
- Potential for minor regressions (mitigated by tests)
- Learning curve for new patterns

## Implementation

See `plans/current-plan.md` for detailed action sequence.

## Success Criteria

| Metric | Target |
|--------|--------|
| Clippy errors | 0 |
| Rust tests | 100% pass |
| E2E tests | 100% pass (all browsers) |
| TypeScript errors | 0 |
| GitHub Actions | All pass |

## References

- [ADR-017: Rust Critical Fixes](./017-rust-critical-fixes.md)
- [ADR-018: TypeScript Production Standards](./018-typescript-production-standards.md)
- [ADR-019: E2E Test Enhancement Strategy](./019-e2e-test-enhancement-strategy.md)
- [ADR-020: Skill Documentation Cleanup](./020-skill-documentation-cleanup.md)
- [Rust WASM Best Practices 2026](https://rustwasm.github.io/docs/wasm-bindgen)
- [Playwright Best Practices](https://playwright.dev/docs/best-practices)
- [TypeScript Strict Mode](https://www.typescriptlang.org/tsconfig/strict.html)
