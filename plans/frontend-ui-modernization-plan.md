# Frontend UI/UX Modernization Plan

## Goal
Modernize the ASCII Canvas Editor frontend with 2026 UI/UX best practices, fixing broken dropdowns and adding missing customization options.

## Issues Identified

### 1. Border Style Dropdown Not Functional
- **Location**: `web/index.html:65-73`, `web/main.ts:206-212`
- **Problem**: Native `<select>` dropdown with limited styling and poor UX
- **Impact**: Users cannot effectively select border styles

### 2. Line Tool Missing Direction Options
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

## Success Criteria
- [ ] Border style selector works with mouse and keyboard
- [ ] Visual preview of border styles
- [ ] Line tool has direction options
- [ ] All controls match dark Figma-like theme
- [ ] WCAG 2.1 AA accessibility compliance
