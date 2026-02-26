# ADR-003: Enhanced Keyboard Shortcuts and UI Controls

## Status

**Accepted** - 2026-02-26

## Context

The user requested several additional features to improve the ASCII Canvas Editor:

1. **B key** - Cycle through 6 rectangle border styles
2. **? key** - Show keyboard shortcuts modal  
3. **Zoom buttons** - Fit/Reset/+/- buttons in the side panel
4. **Blue-tinted preview** - While drawing shapes (deferred - preview_ops exist but not rendered)

## Decision

Implemented features 1-3. Feature 4 deferred as it requires significant renderer changes.

### Feature 1: B Key Border Style Cycling

Added keyboard shortcut to cycle through border styles:
- Single → Double → Heavy → Rounded → ASCII → Dotted → Single

Implementation:
- Added `BORDER_STYLES` array in `web/main.ts`
- Added `cycleBorderStyle()` function
- Added 'b' to keyboard shortcut prevention list
- Syncs with border style dropdown

### Feature 2: Keyboard Shortcuts Modal

Added modal dialog showing all keyboard shortcuts:
- Press `?` or `Shift+/` to show modal
- Press `Escape` or click outside to close
- Shows all tool shortcuts and actions

Implementation:
- Added modal HTML in `web/index.html`
- Added modal CSS in `web/style.css`
- Added `showShortcutsModal()` and `hideShortcutsModal()` functions

### Feature 3: Zoom Control Buttons

Added buttons in side panel:
- **Fit** - Fit canvas to container
- **1:1** - Reset to 100% zoom
- **-** - Zoom out 20%
- **+** - Zoom in 25%

Implementation:
- Added button HTML in side panel
- Added CSS for zoom controls
- Added `setZoom()` and `fitZoom()` functions
- Zoom range: 0.3x to 4x

## Consequences

### What This Adds

1. **B key**: Quick border style switching without using dropdown
2. **? modal**: Discoverable keyboard shortcuts for new users
3. **Zoom buttons**: Mouse-based zoom controls alongside scroll wheel
4. **Focus fix**: Keyboard works after button clicks (from ADR-002)

### Files Modified

- `web/main.ts` - Added B key handler, modal functions, zoom functions
- `web/index.html` - Added shortcuts modal, zoom buttons
- `web/style.css` - Added modal styles, zoom button styles

### Test Results

- All 44 Rust unit tests: PASS
- All 12 E2E tests: PASS
