# Frontend UI/UX Modernization Plan

## Status: Historical proposal; current state verified 2026-10-15

- Border-style selection is implemented as a functioning native `<select>`; the
  proposed custom dropdown and visual previews were not adopted.
- Auto, Horizontal, and Vertical line-direction controls are present in the Line
  tool.
- Accessibility attributes and copy-format status feedback were further improved
  in #264.

## Goal (historical)
Modernize the ASCII Canvas Editor frontend with 2026 UI/UX best practices, fixing broken dropdowns and adding missing customization options.

## Issues Identified

### 1. Border Style Dropdown Not Functional (historical finding)
- **Location**: `web/index.html:65-73`, `web/main.ts:206-212`
- **Problem**: Native `<select>` dropdown with limited styling and poor UX
- **Impact**: Users cannot effectively select border styles

### 2. Line Tool Missing Direction Options (historical finding)
- **Problem**: No horizontal/vertical/ diagonal customization for Line tool
- **Impact**: Limited drawing capabilities

### 3. Toolbar UX Issues
- Native select element doesn't match Figma-like dark theme
- No visual feedback for selected options
- Poor keyboard navigation

## Research: 2026 UI/UX Trends

Based on Interaction Design Foundation research (Feb 2026):

### Key Principles Applied
1. **Recognition over Recall** - Visual selectors with previews
2. **Consistency and Standards** - Match Figma/Adobe patterns
3. **Aesthetic Minimalist Design** - Clean, focused toolbars
4. **User Control and Freedom** - Easy switching/canceling

### Color Palette Best Practices (2026)
- 60-30-10 rule for visual balance
- High contrast for accessibility (4.5:1 ratio minimum)
- Avoid pure white/black to reduce eye strain
- Use accent colors for interactive elements

## Action Plan

### Phase 1: Border Style Selector Redesign
- Replace native `<select>` with custom dropdown
- Add visual preview of each border style
- Add keyboard navigation (B to cycle)
- Show current selection in toolbar

### Phase 2: Line Tool Enhancement
- Add line direction options (horizontal, vertical, diagonal)
- Create visual toggle buttons for direction
- Update Rust backend if needed

### Phase 3: Toolbar Modernization
- Implement consistent button styling
- Add hover states and tooltips
- Improve keyboard accessibility
- Add visual feedback for all selections

## Historical success criteria
- [x] Border style selector works with mouse and keyboard
- [ ] Custom visual preview of border styles — not adopted; native select retained
- [x] Line tool has Auto/Horizontal/Vertical direction options
- [x] Toolbar controls retain the project's dark/light UI treatment
- [ ] A blanket WCAG 2.1 AA claim — not established by this plan
