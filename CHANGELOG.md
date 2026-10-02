# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

### Changed

- Fail-closed quality harness (ADR-047): structured npm audit evidence that
  cannot pass while a lockfile went unchecked, artifact validity required
  before size budgets, WASM binding freshness gates in local and CI checks,
  retained positive/negative sensor-CI-skill fixtures in both runners, pnpm
  pin parity across workspaces and CI, and production-dist E2E in full/CI.
  Adds the offline skill-manifest checker and the `repo-typescript` adapter.

### Fixed

- Text Space/Unicode and unknown browser keys no longer become panning/Delete.
  Tool buffers and unfinished gestures cannot leak across documents or layers.
- Freehand/eraser strokes are atomic undo steps, including long, overlapping,
  cancelled and no-op gestures. Paste during a stroke preserves exact history.
- Added-layer undo/redo preserves live content/metadata/history. Shared layer and
  dimension limits keep editable documents restorable, including history replay.
- Viewport resize no longer crops content or clears history; explicit grid shrink
  asks for confirmation. UI text fields retain native paste behavior.
- PNG export includes committed visible content without selection or preview
  overlays, and leaves editor state unchanged.
- Registered Node WASM regressions now execute in CI and local full verification,
  with nonzero execution required for each library/integration target.
- Added bounded browser-local named drafts with create, rename, switch, delete and
  import-as-new. Switching validates and saves before replacing the live editor,
  then starts with fresh history and view state.
- Added visible saved/dirty/error/conflict status and a `.asc` backup action.
  Legacy autosave migration is non-destructive; storage denial, quota, corrupt
  shelves/documents and stale-tab conflicts preserve the live work instead of
  silently replacing or overwriting it. Local drafts are not durable backups.
- Root `brace-expansion` resolution updated from 5.0.9 to 5.0.12 for two High
  advisories, without overrides or advisory ignores.

- **F-13 layer history: undo could rename the wrong layer.** The layer-history
  commands landed in #213 recorded a **positional index** and replayed it on
  undo, so any reorder or delete between record and replay made the undo target
  a different layer — e.g. rename `B`, move `B` down, then Undo renames whatever
  now sits at that index. `Layer` now carries a stable `id` and mutations resolve
  by id (`set_name_by_id`, `set_visible_by_id`, `set_locked_by_id`,
  `remove_layer_by_id`); the WASM API still takes a positional index from JS and
  translates at the boundary. ADR-043 had flagged this hazard, but every test
  exercised one mutation in isolation. 18 new tests, including two that pin the
  bug, plus 4 E2E specs.

### Notes

- **F-13 layer history (ADR-043) landed under a `chore(harness):` commit.** The
  implementation was authored in #212 but reached `main` inside #213
  (`chore(harness): make CI green a merge precondition`), because that branch
  was created from `feat/f13-layer-undo` rather than from `main` and therefore
  carried the unmerged feature commit. Attribution: **F-13 = #213 in history**,
  designed under #212. #212 is closed as already-landed. Harness L-011;
  ADR-043 updated.
- Layer-history test bodies were extracted out of `src/wasm/layer_api.rs` into
  two sibling test modules to hold the 500-line budget (467 → 306).
- The F-13 change still has **no `pr-roast` adversarial pass** — it merged
  under a `chore:` title, which is precisely the case the roast pass exists to
  catch. The positional-index bug above is a plausible instance of what that
  pass would have found.

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
