# Changelog

All notable changes to this project will be documented in this file.

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
