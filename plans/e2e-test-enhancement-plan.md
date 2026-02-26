# E2E Test Enhancement Plan

## Goal
Update e2e tests to use all buttons and functions in the ASCII Canvas application, and verify that it draws the correct output.

## Analysis

### Current Test Coverage
- Basic tool buttons (select, rectangle, line, arrow, diamond, text, freehand, eraser)
- Tool switching via click and keyboard shortcuts
- Border style selector
- Undo/redo buttons (initial state)
- Copy button
- Zoom with scroll wheel
- Cursor position display
- Status bar
- Keyboard shortcuts

### Missing Test Coverage
1. **All Tools Drawing Tests** - Need to verify each tool actually draws on canvas
2. **Undo/Redo After Drawing** - Verify these become enabled after drawing
3. **Clear Canvas Button** - Test the clear functionality
4. **All Border Styles** - Test each border style option
5. **Zoom Buttons** - Test Fit, Reset, In, Out buttons
6. **Keyboard Shortcut 'B'** - Cycle border styles
7. **Keyboard Shortcut '?'** - Show shortcuts modal
8. **Keyboard Shortcut Escape** - Cancel/deselect
9. **Pan with Space+Drag**
10. **ASCII Output Verification** - Verify export_ascii() returns correct content
11. **Selection Tool** - Move/resize shapes
12. **Text Tool** - Add text to canvas
13. **Eraser Tool** - Remove shapes

## Implementation Steps

1. Create ADR documenting the testing approach
2. Add comprehensive drawing tests for each tool
3. Add output verification tests using export_ascii()
4. Add all button interaction tests
5. Run tests and fix any issues

## Expected Outcome
All buttons and functions in the ASCII Canvas application will have comprehensive e2e test coverage with output verification.
