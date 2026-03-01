# Production Readiness Plan - ASCII Canvas Editor

## Executive Summary

This plan addresses production readiness gaps identified through comprehensive codebase analysis, incorporating 2026 best practices for Rust/WASM, TypeScript, and E2E testing.

**Analysis Date:** 2026-03-01
**Status:** Planning Complete - Ready for Implementation

---

## GOAP Analysis (Goal-Oriented Action Planning)

### Goal State
A production-ready ASCII Canvas Editor with:
- Zero critical code quality issues
- Comprehensive E2E test coverage
- Clean, maintainable skill documentation
- 2026 best practices compliance

### Current State Assessment

| Domain | Score | Critical Issues | Medium Issues |
|--------|-------|-----------------|---------------|
| Rust Code Quality | 6/10 | 4 | 5 |
| TypeScript Frontend | 5/10 | 4 | 6 |
| E2E Test Coverage | 5.4/10 | 3 | 4 |
| Skill Documentation | 7/10 | 3 | 3 |

---

## Action Plan (Prioritized)

### Phase 1: Critical Fixes (P0) - Must Complete Before Production

#### 1.1 Rust Critical Issues

| Issue | File | Lines | Action |
|-------|------|-------|--------|
| Duplicate `delete()` methods | `src/core/tools/text.rs` | 55-100 | Remove duplicates, keep one implementation |
| Unsafe pointer cast | `src/wasm/bindings.rs` | 185-192 | Replace with `downcast_mut()` pattern |
| Panic on `unwrap()` | `src/core/tools/text.rs` | 76, 93 | Use proper Option handling |
| Panic on char conversion | `src/core/tools/mod.rs` | 67 | Handle `None` case |

#### 1.2 TypeScript Critical Issues

| Issue | File | Lines | Action |
|-------|------|-------|--------|
| Non-null assertions | `web/main.ts` | 45-53 | Add defensive null checks |
| `any` type bypass | `web/main.ts` | 234-235 | Create proper interface |
| WASM types return `any` | `web/pkg/ascii_canvas.d.ts` | Multiple | Generate proper types |
| No unit tests | `web/` | N/A | Add Vitest tests |

#### 1.3 E2E Critical Gaps

| Gap | Priority | Action |
|-----|----------|--------|
| Pan functionality (Space+drag) | Critical | Add test suite |
| Drawing output verification | Critical | Verify ASCII output for each tool |
| Border style rendering | Critical | Verify correct characters per style |

### Phase 2: Medium Priority Fixes (P1)

#### 2.1 Rust Medium Issues

- Remove unused `select_tool` field in `bindings.rs`
- Change `debug_assert_eq!` to `assert_eq!` in `grid.rs:39`
- Remove commented-out code in `eraser.rs:55-57`
- Standardize error handling patterns

#### 2.2 TypeScript Medium Issues

- Remove console statements or use logging library
- Add proper error handling for Clipboard API
- Centralize magic numbers into config object
- Add modal accessibility (ARIA attributes, focus trap)

#### 2.3 E2E Enhancements

- Add cross-browser testing (Firefox, WebKit)
- Replace `waitForTimeout` with proper assertions
- Implement Page Object Model
- Add visual regression tests

### Phase 3: Skill Documentation Cleanup (P1)

| File | Issue | Action |
|------|-------|--------|
| `.agents/skills/vite/GENERATION.md` | Useless artifact | Delete |
| `.agents/skills/typescript-expert/SKILL.md` | Vague placeholder section | Remove section |
| `.agents/skills/ln-732-cicd-generator/diagram.html` | Broken CSS reference | Delete or fix |
| `.agents/skills/ln-732-cicd-generator/SKILL.md` | Broken path references | Update paths |

---

## 2026 Best Practices Integration

### Rust/WASM (from web search findings)

1. **Toolchain**: Use Rust 1.81+ with `wasm-pack v0.12+`
2. **Build**: `wasm-pack build --release --target bundler`
3. **Optimization**: Enable `wasm-opt` for 20-40% size reduction
4. **MIME Type**: Ensure `application/wasm` for `.wasm` files
5. **Error Handling**: Return `Result` types wrapped in `JsValue`

### Playwright E2E (from web search findings)

1. **Selectors**: Use `getByRole()`, `getByLabel()`, `getByTestId()` over CSS
2. **Auto-waiting**: Avoid `sleep()`, use `expect()` with polling
3. **Page Object Model**: Encapsulate page interactions
4. **CI Integration**: Use Docker images, sharding, retries
5. **Tracing**: Enable `trace: 'on-first-retry'` for debugging

### Clippy Standards (from web search findings)

1. **Command**: `cargo clippy --all-targets --all-features -- -D warnings`
2. **Categories to deny**: correctness, suspicious, style, complexity, perf
3. **Configuration**: Use `clippy.toml` for project-specific rules
4. **CI Integration**: Fail builds on warnings

---

## Implementation Sequence

```
Phase 1 (Critical)
├── 1.1 Fix Rust critical issues (4 items)
├── 1.2 Fix TypeScript critical issues (4 items)
└── 1.3 Add critical E2E tests (3 items)

Phase 2 (Medium)
├── 2.1 Rust medium fixes (4 items)
├── 2.2 TypeScript medium fixes (4 items)
└── 2.3 E2E enhancements (4 items)

Phase 3 (Cleanup)
├── 3.1 Remove AI sloppy content from skills
├── 3.2 Update skill documentation
└── 3.3 Verify all changes with tests
```

---

## Success Criteria

| Metric | Current | Target |
|--------|---------|--------|
| Rust critical issues | 4 | 0 |
| TypeScript critical issues | 4 | 0 |
| E2E test coverage score | 5.4/10 | 8/10 |
| Skill doc issues | 6 | 0 |
| Clippy warnings | Unknown | 0 |
| All tests passing | Yes | Yes |

---

## Related ADRs

- ADR-017: Rust Critical Fixes
- ADR-018: TypeScript Production Standards
- ADR-019: E2E Test Enhancement Strategy
- ADR-020: Skill Documentation Cleanup

---

## References

- [Rust WebAssembly Best Practices 2026](https://rustwasm.github.io/book/)
- [Playwright Best Practices](https://playwright.dev/docs/best-practices)
- [Clippy Documentation](https://doc.rust-lang.org/stable/clippy)
- [Apollo Rust Best Practices](https://github.com/apollographql/rust-best-practices)
