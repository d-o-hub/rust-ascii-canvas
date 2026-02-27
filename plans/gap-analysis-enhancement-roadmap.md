# ASCII Canvas - Gap Analysis & Enhancement Roadmap

## Executive Summary

This document analyzes the ASCII Canvas Editor for missing features, incomplete implementations, and enhancement opportunities using GOAP (Goal-Oriented Action Planning) methodology.

---

## Current State Assessment

### Implemented Features
| Category | Feature | Status | Quality |
|----------|---------|--------|---------|
| Drawing Tools | Rectangle, Line, Arrow, Diamond, Text, Freehand, Eraser, Select | ✅ Complete | High |
| Border Styles | Single, Double, Heavy, Rounded, ASCII, Dotted | ✅ Complete | High |
| Core | Undo/Redo (100 commands), Zoom (0.3x-4x), Pan (Space+drag) | ✅ Complete | High |
| Export | ASCII to clipboard | ✅ Complete | Medium |
| UI | Toolbar, Status bar, Zoom controls, Shortcuts modal | ✅ Complete | Medium |

### Known Issues
- 52 documentation warnings (cosmetic)
- Firefox E2E tests need browser installation
- Blue-tinted preview not implemented for drag operations

---

## Gap Analysis by Category

### 1. CRITICAL: Missing Core Features

#### 1.1 Selection Copy/Paste Operations
**Gap**: Select tool can select and move regions but cannot copy/paste.

**Current State**: `src/core/tools/select.rs` supports:
- Selection creation (drag)
- Selection moving (drag inside selection)
- Selection clearing (Escape)

**Missing**:
- Copy selection to internal clipboard (Ctrl+C with selection)
- Paste from internal clipboard (Ctrl+V)
- Cut selection (Ctrl+X)
- Delete selection (Delete/Backspace)

**Impact**: Users cannot duplicate shapes or move content between areas efficiently.

#### 1.2 Text Tool Enhancements
**Gap**: Text tool is minimal - single character placement.

**Current State**: `src/core/tools/text.rs` supports:
- Click to place single characters
- Keyboard input for character

**Missing**:
- Text cursor/insertion point visualization
- Multi-character text entry mode
- Text selection within input
- Backspace to delete last character
- Enter to commit text / new line

**Impact**: Users cannot efficiently type text labels or comments.

#### 1.3 Grid Size Customization
**Gap**: Grid is hardcoded to 80x40.

**Current State**: `web/main.ts:66-67` defines `GRID_WIDTH = 80` and `GRID_HEIGHT = 40`.

**Missing**:
- UI to change grid dimensions
- Save/load custom grid sizes
- Responsive grid sizing

**Impact**: Users cannot create diagrams of different aspect ratios.

---

### 2. IMPORTANT: Incomplete Implementations

#### 2.1 Preview Operations (Blue Tint)
**Gap**: `preview_ops` stored during drag but not rendered with visual distinction.

**Current State**: `src/wasm/bindings.rs:197-198` stores preview operations, but renderer only shows final result.

**Missing**:
- Render preview operations with different color/style
- Show semi-transparent preview during drag
- Selection preview overlay

**Impact**: Users cannot see what they're drawing before committing.

#### 2.2 Selection Move Implementation
**Gap**: Selection moving state tracked but not fully implemented.

**Current State**: `src/core/tools/select.rs:83-85` sets `moving = true` but `on_pointer_move` returns empty result.

**Missing**:
- Actual content movement during drag
- Visual feedback of moved content
- Bounds checking during move

**Impact**: Select tool's move functionality doesn't work end-to-end.

#### 2.3 Eraser Tool Size
**Gap**: Eraser clears single cells only.

**Missing**:
- Adjustable eraser size (1x1, 3x3, 5x5)
- Visual indicator of eraser size

**Impact**: Users cannot efficiently erase larger areas.

---

### 3. ENHANCEMENT: Quality of Life Improvements

#### 3.1 Export Formats
**Current**: ASCII text only via clipboard.

**Missing**:
- PNG export (render canvas to image)
- SVG export (vector representation)
- Save to file (File System Access API)
- Load from file

**Priority**: Medium - Many users want to embed diagrams in documents.

#### 3.2 Touch/Mobile Support
**Current**: Mouse/keyboard only.

**Missing**:
- Touch event handlers
- Pinch-to-zoom
- Touch-friendly button sizes
- On-screen keyboard for text tool

**Priority**: Medium - Growing mobile/tablet usage.

#### 3.3 Theme Customization
**Current**: Dark Figma-like theme hardcoded.

**Missing**:
- Light theme option
- Custom color picker for grid/cells
- Font selection (monospace options)

**Priority**: Low - Nice to have.

#### 3.4 Grid Features
**Missing**:
- Grid snapping toggle
- Show/hide grid lines
- Rulers/guides
- Grid background pattern options

**Priority**: Low.

---

## GOAP: Goal-Oriented Action Plan

### Goal 1: Selection Copy/Paste

**Preconditions**: Select tool with active selection
**Postconditions**: Selection content copied/pasted

| Action | Preconditions | Effects |
|--------|---------------|---------|
| Copy Selection | Active selection | Selection content in clipboard |
| Cut Selection | Active selection | Selection content in clipboard, cells cleared |
| Paste | Clipboard has content | Content inserted at cursor |
| Delete Selection | Active selection | Selection content cleared |

**Implementation Order**:
1. Add `ClipboardContent` struct to store selection data
2. Implement `copy_selection()` in AsciiEditor
3. Implement `paste_content()` with bounds checking
4. Add keyboard handlers for Ctrl+C/V/X, Delete
5. Add E2E tests

### Goal 2: Enhanced Text Tool

**Preconditions**: Text tool active
**Postconditions**: Full text editing capability

| Action | Preconditions | Effects |
|--------|---------------|---------|
| Start Text Entry | Click on grid | Text cursor appears |
| Type Character | Text cursor active | Character inserted at cursor |
| Backspace | Text cursor active, text exists | Previous character deleted |
| Enter | Text cursor active | Text committed to grid |

**Implementation Order**:
1. Add text buffer to TextTool
2. Render text cursor in preview layer
3. Handle Backspace/Delete keys
4. Handle Enter to commit
5. Add E2E tests

### Goal 3: Grid Customization

**Preconditions**: Editor initialized
**Postconditions**: Adjustable grid size

| Action | Preconditions | Effects |
|--------|---------------|---------|
| Open Grid Settings | None | Settings dialog shown |
| Set Width/Height | Dialog open | Grid resized |
| Clear and Resize | New size set | Grid cleared, new size applied |

**Implementation Order**:
1. Add `resize(width, height)` method to AsciiEditor
2. Add UI controls (menu or toolbar)
3. Confirm before clearing existing content
4. Persist size preference (localStorage)
5. Add E2E tests

---

## Prioritized Roadmap

### Phase 1: Critical Fixes (P0)
1. **Selection Copy/Paste** - Essential for productivity
2. **Text Tool Enhancement** - Critical for diagram labels
3. **Selection Move Fix** - Already partially implemented

### Phase 2: Important Features (P1)
4. **Preview Rendering** - Better UX during drawing
5. **Eraser Size Options** - Productivity improvement
6. **Grid Size Customization** - Flexibility

### Phase 3: Enhancements (P2)
7. **PNG/SVG Export** - Shareability
8. **Touch Support** - Mobile accessibility
9. **Theme Options** - User preference

### Phase 4: Nice to Have (P3)
10. **Cloud Save** - Cross-device access
11. **Collaboration** - Real-time editing
12. **Templates** - Pre-made diagram templates

---

## Success Metrics

| Feature | Test Coverage | Performance Target |
|---------|---------------|-------------------|
| Copy/Paste | 5+ E2E tests | < 50ms operation |
| Text Tool | 3+ E2E tests | < 16ms per keystroke |
| Grid Resize | 2+ E2E tests | < 100ms resize |
| PNG Export | 2+ E2E tests | < 500ms generation |

---

## Dependencies

### External Dependencies
- None required for Phase 1-2
- Phase 3 may need:
  - `html2canvas` or custom canvas capture for PNG
  - Touch event polyfills for older browsers

### Internal Dependencies
```
Selection Copy/Paste → Enhanced Text Tool (clipboard infrastructure)
Grid Resize → Clear existing content logic
PNG Export → Render optimization
```

---

## Conclusion

The ASCII Canvas Editor is a solid foundation with 8 working tools and comprehensive test coverage. The primary gaps are in **selection operations (copy/paste)** and **text editing**, which are critical for a diagram editor. The codebase is well-structured for implementing these features following the existing patterns.
