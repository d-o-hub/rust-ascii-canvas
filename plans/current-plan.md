# Current Plan: Production Readiness 2026

## Status: IN PROGRESS

**Started**: 2026-03-01
**Branch**: feat/production-readiness-2026 (to be created)

## Current State

### Critical Blocker
- **Clippy Error**: `from_str` method in `line.rs` should implement `std::str::FromStr` trait

### Pre-existing Issues (from ADRs 017-020)
1. **Rust (ADR-017)**: 4 critical issues
2. **TypeScript (ADR-018)**: 4 critical, 6 medium issues
3. **E2E Tests (ADR-019)**: 3 critical gaps
4. **Documentation (ADR-020)**: 3 cleanup items

### Uncommitted Changes
- Modified tool files (already have `as_any_mut()` implemented)
- Modified bindings.rs (already using safe downcast)
- Modified main.ts (partial TypeScript improvements)
- Deleted skill documentation files

## Action Sequence

### Phase 1: Critical Clippy Fix ⏳ IN PROGRESS

**Action 1.1**: Implement `FromStr` trait for `LineDirection`
- **Precondition**: Read current `line.rs` implementation
- **Effect**: Clippy error resolved
- **Verification**: `cargo clippy -- -D warnings` passes

```rust
// Implement FromStr trait instead of from_str() method
impl std::str::FromStr for LineDirection {
    type Err = std::convert::Infallible;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "horizontal" => Ok(LineDirection::Horizontal),
            "vertical" => Ok(LineDirection::Vertical),
            _ => Ok(LineDirection::Auto),
        }
    }
}
```

### Phase 2: Rust Production Fixes (ADR-017)

**Action 2.1**: Verify `as_any_mut()` is implemented in all tools
- **Precondition**: Phase 1 complete
- **Effect**: Safe downcasting available
- **Verification**: Check all tool files

**Action 2.2**: Fix duplicate `delete()` in text.rs (if still present)
- **Precondition**: Phase 1 complete
- **Effect**: Single correct implementation
- **Verification**: `cargo test` passes

**Action 2.3**: Replace remaining `unwrap()` calls with proper handling
- **Precondition**: Phase 1 complete
- **Effect**: No panic potential
- **Verification**: `grep -r "unwrap()" src/` shows only test code

### Phase 3: TypeScript Production Standards (ADR-018)

**Action 3.1**: Create centralized configuration
- **Precondition**: Phase 2 complete
- **Effect**: No magic numbers scattered
- **File**: `web/src/config.ts`

**Action 3.2**: Add proper WASM type definitions
- **Precondition**: Phase 2 complete
- **Effect**: Full type safety
- **File**: `web/src/types/wasm.ts`

**Action 3.3**: Add ARIA attributes to modal
- **Precondition**: Phase 2 complete
- **Effect**: Accessibility compliance
- **File**: `web/index.html`

### Phase 4: E2E Test Enhancement (ADR-019)

**Action 4.1**: Update playwright.config.ts for cross-browser
- **Precondition**: Phase 3 complete
- **Effect**: Firefox, WebKit testing enabled
- **Verification**: `npx playwright test` runs all browsers

**Action 4.2**: Create Page Object Model
- **Precondition**: Phase 3 complete
- **Effect**: Maintainable test code
- **File**: `e2e/pages/EditorPage.ts`

**Action 4.3**: Add pan functionality tests
- **Precondition**: Phase 4.2 complete
- **Effect**: Pan feature tested
- **File**: `e2e/canvas.spec.ts`

### Phase 5: Documentation Cleanup (ADR-020)

**Action 5.1**: Verify deleted files are committed
- **Precondition**: Phase 4 complete
- **Files**: 
  - `.agents/skills/vite/GENERATION.md` (deleted)
  - `.agents/skills/ln-732-cicd-generator/diagram.html` (deleted)

**Action 5.2**: Verify SKILL.md fixes
- **Precondition**: Phase 4 complete
- **Files**:
  - `.agents/skills/typescript-expert/SKILL.md`
  - `.agents/skills/ln-732-cicd-generator/SKILL.md`

### Phase 6: Git Workflow

**Action 6.1**: Create feature branch
- **Precondition**: All phases complete
- **Command**: `git checkout -b feat/production-readiness-2026`

**Action 6.2**: Stage and commit changes atomically
- **Precondition**: Action 6.1 complete
- **Commits**:
  1. `fix(rust): implement FromStr trait for LineDirection`
  2. `fix(rust): add as_any_mut to all tools for safe downcasting`
  3. `refactor(typescript): add centralized configuration`
  4. `feat(typescript): add proper WASM type definitions`
  5. `fix(a11y): add ARIA attributes to shortcuts modal`
  6. `test(e2e): enable cross-browser testing`
  7. `test(e2e): add Page Object Model`
  8. `test(e2e): add pan functionality tests`
  9. `docs: cleanup skill documentation`
  10. `docs: add ADR-021 production readiness plan`

**Action 6.3**: Push and create PR
- **Precondition**: Action 6.2 complete
- **Command**: `git push -u origin feat/production-readiness-2026`

## Blockers

None currently.

## Notes

- The `as_any_mut()` pattern is already implemented in tool files (verified in git status)
- The safe downcast pattern is already in `bindings.rs`
- Some TypeScript improvements already in `main.ts`
- Skill documentation files already deleted

## Progress Tracking

| Phase | Status | Completion |
|-------|--------|------------|
| Phase 1 | ⏳ In Progress | 0% |
| Phase 2 | ⏸️ Pending | 0% |
| Phase 3 | ⏸️ Pending | 0% |
| Phase 4 | ⏸️ Pending | 0% |
| Phase 5 | ⏸️ Pending | 0% |
| Phase 6 | ⏸️ Pending | 0% |
