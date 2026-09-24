# Changelog

All notable changes to this project will be documented in this file.

## [0.1.4] - 2026-09-24

### Changes
- chore(release): prepare v0.1.4 (version bump + changelog backfill + notes anchoring) (#202) (7c0499f)
- feat(zoom): dynamic disabled state + accessible labels for zoom controls (#201) (9822696)
- chore(harness): single-source version pins (propagate script + gate/CI check) (#200) (399640e)
- docs(harness): reconcile docs with reality + release runbook (L-007) (#196) (ee47ca5)
- feat(drawer): Escape dismissal + focus restore for mobile side panel (rebased PR #191) (#195) (61337dd)
- Merge pull request #194 from d-o-hub/fix/pr-185-zoom-rebase (96055f1)
- feat(zoom): add + and - keyboard shortcuts for canvas zoom (rebased PR #185) (47d0fd2)
- Merge pull request #192 from d-o-hub/chore/wasm-bindgen-pin-parity-042 (8ba0497)
- chore(wasm): coordinated wasm-bindgen 0.2.128 upgrade + pin-parity sensor (ADR-042) (ef8e857)
- feat(clipboard): export fidelity controls and pure ASCII fallback (#176) (#177) (e5d9d58)
- build(deps-dev): bump the dependencies group across 1 directory with 6 updates (#190) (2895675)
- docs(triage): 2026-09-15 PR queue review, L-005 wasm-bindgen skew, F-03 status (#189) (42c7682)
- build(deps-dev): bump the dependencies group with 4 updates (#187) (767c0fe)
- build(deps-dev): bump the dependencies group with 2 updates (fb74305)
- build(deps-dev): bump the dependencies group in /web with 5 updates (3ff3f7d)
- palette: handle Escape key in grid dimension inputs (b8b9c7a)
- build(deps): update wasm-bindgen requirement in the dependencies group (769f42f)
- build(deps-dev): bump binaryen in the dependencies group (6f2e3ae)
- fix(a11y): add dynamic accessible labels for theme toggle buttons (fe3c8cf)
- fix(a11y): add dynamic accessible labels for theme toggle buttons (8314edf)
- fix(a11y): add dynamic accessible labels for theme toggle buttons (58ead85)
- build(deps-dev): bump the dependencies group in /web with 4 updates (#175) (caec6b8)
- build(deps-dev): bump the dependencies group with 2 updates (#174) (5bca854)
- 🎨 Palette: Dynamic Screen-Reader Labeling for State Switches (#172) (b018417)
- docs(plans): record PR queue triage results (2026-08-07) (#171) (0b44401)
- 🎨 Palette: Keyboard navigation parity and mobile ARIA disclosures (#168) (3c9c06d)
- build(deps-dev): bump the dependencies group in /web with 3 updates (#170) (716809d)
- build(deps-dev): bump the dependencies group with 4 updates (#169) (a920a5c)

## [0.1.3] - 2026-08-05

### Added
- **SVG export** next to PNG (composite, monospace layout).
- **Full layer editor**: rename, visibility, lock, reorder, delete, merge down.
- **External paste / plain-ASCII import** at the cursor.
- **Preview rendering style**: distinct tint for in-progress drawing operations.
- **Enhanced text tool**: visible caret and multi-line polish.
- **Eraser radius** selector (1 / 3 / 5).
- **Light theme** with theme switcher (`data-theme`, localStorage).
- **Mobile UX audit**: drawer, touch targets, grid and layer panels.
- **Dirty-rect pixel buffer** rendering (sparse invalidation).
- **Collaborative editing research spike** (documentation only).

### Changed
- `web/main.ts` split into `events` / `render` / `ui` modules (entry point 230 LOC).
- **CI**: E2E matrix is Chromium + Firefox + WebKit; `cargo-nextest` runs the unit tests.
- **Types**: WASM-generated TypeScript definitions (ADR-033).
- **Docs**: ADR 001-036 status audit; app-only distribution decision (ADR-040).
- **Build**: `wasm-opt` (binaryen) required in CI with a documented local install; `#![allow(missing_docs)]` removed from the WASM layer.

### Harness
- Tiered quality gates (fast / full), an architecture-fitness sensor, and a learned-failure steering log (ADR-037).
- Release workflow: `VERSION` file, guard-rails that check GitHub Releases (not only tags), changelog-branch release flow.

## [0.1.2] - 2026-04-22

### Security
- Fixed an **XSS** in the ASCII clipboard copy path (HTML-interpretable clipboard payloads).

### Fixed
- **ASCII export geometry**: preserve spatial alignment and support a dual-format clipboard (plain text + spatial layout).
- Dotted border-style rendering.
- Clippy: `std::f64` constants replaced with plain `f64`; dropped `any` casting for global `window` extensions.

### Performance
- Optimized the drawing-tool hot loops.

### Tests
- Added coverage for `math::signum` and the manhattan / chebyshev / euclidean distance helpers.

## [0.1.1] - 2026-03-20

### Added
- **Dynamic Font Atlas**: High-fidelity character rendering using frontend rasterization (JetBrains Mono).
- **Select Highlight**: Blue background highlight for active selections in the pixel buffer path.
- **Issue Automation**: GitHub Action for automatic issue closing on PR merge.
- **Tool Validation Skill**: Standardized verification checklist for all 8 drawing tools.

### Fixed
- **Text Tool**: Fixed keyboard mapping (Enter/Backspace/Delete) and coordinate drift (8x20 metric alignment).
- **Arrow Tool**: Prevented endpoint overwriting and added Unicode arrowhead support (▲▼◄►).
- **Diamond Tool**: Re-implemented with diagonal characters (╱╲) and improved small-drag handling.
- **Freehand Tool**: Synchronized drawing character with the currently selected Border Style.
- **Eraser Tool**: Added strict grid boundary checks to prevent out-of-bounds panics.
- **E2E Tests**: Expanded test suite from 63 to 152 tests, including responsive and cross-browser validation.

### Technical
- **Version bump**: 0.1.1 patch release for production tool fixes.
- **WASM Performance**: Refined pixel buffer rendering loop and font atlas mask handling.

## [0.1.0] - 2026-03-04

### Added
- **Select Tool Move**: Click and drag inside a selection to move selected objects
- **Select Tool Delete**: Delete/Backspace keys delete selected regions
- **Full Undo/Redo**: Move operations are undoable as single atomic operations

### Fixed
- **Select Tool Move Implementation**: Implemented previously stubbed-out move functionality
  - Added state tracking for move operations
  - Preview shows content at new position during drag
  - Commits as single undo operation
- **Eraser Verification**: Confirmed eraser tool works correctly
- **Grid Boundary Clarification**: Documented that grid uses zero-based indexing (columns 0-79)

### Features
- **8 Drawing Tools**: Rectangle, Line, Arrow, Diamond, Text, Freehand, Select, Eraser
- **6 Border Styles**: Single, Double, Heavy, Rounded, ASCII, Dotted
- **Full Undo/Redo**: Command pattern with 100 command history
- **Zoom & Pan**: Mouse wheel zoom, Space+drag panning
- **One-Click Copy**: Export ASCII to clipboard
- **Keyboard Shortcuts**: Full keyboard-first workflow
- **Dark Theme**: Professional Figma-inspired UI

### Technical
- **WASM Size**: 151KB (well under 1.5MB target)
- **Performance**: 60 FPS rendering with dirty-rect optimization
- **Tests**: 78 unit tests, 63 E2E tests

### Documentation
- **README.md**: Complete feature and API documentation
- **AGENTS.md**: Agent best practices and learnings
- **Technical Analysis**: Detailed implementation patterns
- **ADR Records**: Architectural decisions documented

## [0.0.0] - 2026-02-20

### Added
- Initial project setup
- Core grid and cell model
- 8 drawing tools (basic implementation)
- Canvas renderer with dirty-rect optimization
- WASM bindings for JavaScript
- Basic dark theme UI
