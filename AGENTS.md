# Agent Best Practices - ASCII Canvas Project

This document outlines the core principles and workflows for agents working on this project. Detailed technical references and historical learnings are maintained in the `agents-docs/` directory.

## Core Directives

1. **Analyze** - Understand requirements before acting.
2. **Plan** - Create steps in `plans/` using GOAP and ADR before execution.
3. **Execute** - Run commands and tests autonomously.
4. **Document** - Update `plans/` with findings and results.
5. **Always Verify** - Every change must be followed by a verification tool call.

## Standard Workflow Checklist

- [ ] Run `cargo clippy --all-targets --all-features -- -D warnings`
- [ ] Run `cargo test` (unit + integration + doc tests)
- [ ] Build WASM with `npm run build:wasm`
- [ ] Run TypeScript check: `cd web && npx tsc --noEmit`
- [ ] Run E2E tests: `npx playwright test`
- [ ] Validate all 8 tools using the **Tool Verification Checklist** below.
- [ ] Update ADRs for any new architectural decisions in `plans/ADRs/`
- [ ] Document technical findings in `plans/TECHNICAL_ANALYSIS.md`
- [ ] Call `pre_commit_instructions` before submitting.

## Reference Documentation

- [Core Best Practices](agents-docs/best-practices.md) - Task execution, documentation standards, and file organization.
- [Production Readiness Learnings](agents-docs/learnings-archive.md) - Collected technical learnings on Rust, GitHub Actions, Playwright, and TypeScript.
- [Responsive Grid Guide](agents-docs/responsive-grid.md) - Mandatory patterns for handling the responsive WASM grid.

## Proactive Testing

For any code change, attempt to find and run relevant tests. Practice test-driven development when practical. If you encounter failures, diagnose the root cause before attempting environment changes.

## Tool Verification Checklist

| Tool | Core Requirement |
|------|------------------|
| **Select** | Visible blue highlight; move & delete work. |
| **Rect** | Correct border style corners/lines. |
| **Line** | Continuous lines in all directions. |
| **Arrow** | Visible arrowhead (▲▼◄►) at end. |
| **Diamond** | Uses diagonal characters (╱╲). |
| **Text** | Supports Enter/Backspace/Delete keys. |
| **Free** | Character syncs with Border Style. |
| **Erase** | Radius-based clearing; no out-of-bounds. |

---

## Learnings from Phase 1 Implementation (2026-03-03)

### Rust Code Hygiene

1. **Finding unused dependencies**: Use `cargo remove <crate>` or manually edit Cargo.toml, then verify with `cargo check`
2. **Dead code detection**: Removing crate-level `#![allow(dead_code)]` exposes hidden unused code - must address each piece individually
3. **Duplicate type patterns**: When both a public and private version of a type exist, check which is actually used before removing
4. **Rc<RefCell> patterns**: Unused `Rc<RefCell<T>>` fields that are never read should be removed entirely

### ADR Management

1. **Numbering conflicts**: When duplicate ADR numbers exist, rename to sequential suffix (005a, 005b)
2. **ADR numbering format**: Use leading zeros (001-099) for consistent sorting

### Configuration Cleanup

1. **Redundant configs**: Root-level configs that duplicate web/ versions should be removed
2. **Empty config files**: Files with no meaningful content (empty plugins array) should be removed

### Select Tool Move Implementation (2026-03-04)

**Problem**: Select tool had move infrastructure (flags, offset tracking) but move logic was stubbed out.

**Solution**: Implemented move functionality at editor level (not tool level) because:
1. Tool doesn't have access to grid content
2. Move = copy → clear original → paste at new location
3. Editor already has clipboard infrastructure

**Key Pattern**:
- Use boolean flags (`is_moving_selection`) instead of complex trait downcasting
- Reuse existing clipboard + preview ops infrastructure
- Commit entire move as single undo operation

### Release Workflow (2026-03-04)

**GitHub Release Best Practices 2026**:
- Use semantic versioning: `vMAJOR.MINOR.PATCH` (e.g., `v0.1.0`)
- Release title: Just version number `v0.1.0` (not "v0.1.0 - Production Ready")
- Use GitHub Actions for automated releases with guard-rails

**Release Workflow**:
```yaml
# .github/workflows/release.yml
on:
  workflow_dispatch:
    inputs:
      version:
        description: 'Version (patch/minor/major or exact like 0.1.0'
        default: 'patch'
```

**Guard-rails Required**:
1. `cargo test --lib` - Unit tests
2. `cargo clippy -- -D warnings` - No warnings allowed
3. `cargo fmt --check` - Code formatting
4. `wasm-pack build --release` - WASM build

**Local Release**:
```bash
# Dry run first
./scripts/release.sh patch true

# Actual release
./scripts/release.sh patch
```

### Release Build: wasm-opt Feature Flags (2026-03-19)

**Problem**: Release build failed at wasm-opt step with `wasm-validator error: all used features should be allowed` on `i32.extend16_s`, `i32.extend8_s`, `i32.trunc_sat_f32_u`.

**Root Cause**: Rustc 1.94.0 (2026-03-02) generates WASM using sign extension and non-trapping float-to-int instructions. Ubuntu's `binaryen` package is too old and doesn't support the required flags.

**Fix**: Download binaryen from GitHub releases and use correct flag names:
```yaml
- run: |
    wget -q https://github.com/WebAssembly/binaryen/releases/download/version_123/binaryen-version_123-x86_64-linux.tar.gz
    tar xzf binaryen-version_123-x86_64-linux.tar.gz
    echo "$(pwd)/binaryen-version_123/bin" >> $GITHUB_PATH
- run: wasm-opt --enable-simd --enable-bulk-memory --enable-sign-ext --enable-nontrapping-float-to-int -Oz -o web/pkg/ascii_canvas_bg.wasm web/pkg/ascii_canvas_bg.wasm
```

**Key Insights**:
1. Flag is `--enable-sign-ext` (NOT `--enable-sign-extension`)
2. Ubuntu's `apt-get install binaryen` is too old — download from GitHub releases
3. As Rust/WASM evolves, `wasm-opt` feature flags must match generated instructions

---

*Last Updated: 2026-03-20*
*Part of Production Readiness 2026 Initiative*
