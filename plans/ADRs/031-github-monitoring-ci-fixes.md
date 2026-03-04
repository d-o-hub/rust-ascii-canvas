# ADR-031: GitHub Monitoring and CI Fix Strategy

## Status
**Proposed** - 2026-03-03

## Context

During comprehensive monitoring of the d-o-hub/rust-ascii-canvas repository using the GitHub CLI, several critical issues were identified in the CI/CD pipeline:

1. **E2E Test Failures**: 14 out of 76 tests failing (18.4% failure rate)
2. **Keyboard Shortcut Issues**: Tool buttons not receiving `active` class when using keyboard shortcuts
3. **Text Tool Problems**: Characters not being inserted in E2E test environment
4. **Select Tool Delete**: ADR-030 fix implemented but tests still failing

The repository has made significant progress on ADR compliance (Phase 1 code hygiene complete, bindings refactor complete), but E2E test reliability remains a blocker for production readiness.

## Decision

### Strategy: Phased E2E Test Stabilization

Implement a three-phase approach to fix the failing E2E tests:

#### Phase 1: Keyboard Shortcut Fixes (Priority: CRITICAL)

**Problem**: Keyboard shortcuts work in manual testing but fail in CI

**Root Cause Hypothesis**: 
- Focus not properly set on canvas before keyboard events
- Race condition between WASM state update and UI update
- Missing `needs_redraw` flag propagation

**Fix Strategy**:
```typescript
// In web/main.ts - ensure focus before keyboard handling
function handleKeyDown(event: KeyboardEvent) {
    // Ensure canvas has focus
    if (document.activeElement !== canvas) {
        canvas.focus();
    }
    
    // Add small delay for WASM state update
    const result = editor.onKeyDown(event.key, event.ctrlKey, event.shiftKey);
    
    // Force UI update
    updateToolButtons(editor.tool);
}
```

**Verification**:
- Add debug logging to trace keyboard event flow
- Create minimal reproduction test
- Verify tool button class updates synchronously

#### Phase 2: Text Tool E2E Fixes (Priority: HIGH)

**Problem**: Text characters not appearing in exported ASCII during E2E tests

**Root Cause Hypothesis**:
- Text tool not receiving focus after canvas click
- Keyboard events not being captured by text tool
- Timing issue between click and type operations

**Fix Strategy**:
```typescript
// In E2E tests - ensure proper text tool activation
async function typeText(page, text: string) {
    // Click to position cursor
    await page.click('#canvas');
    
    // Wait for text tool to be active
    await page.waitForFunction(() => {
        return (window as any).editor.tool === 'text';
    });
    
    // Small delay for tool activation
    await page.waitForTimeout(50);
    
    // Type characters
    for (const char of text) {
        await page.keyboard.press(char);
        await page.waitForTimeout(50); // Debounce
    }
}
```

**Alternative**: Skip problematic text tests temporarily with `test.skip()` until fixed

#### Phase 3: Select Tool Delete Verification (Priority: HIGH)

**Problem**: ADR-030 fix applied but E2E tests still failing

**Verification Steps**:
1. Add unit test for `delete_selection()` method
2. Verify `current_selection.is_some()` check is working
3. Check if selection is being created properly in E2E test
4. Add intermediate assertions to debug test flow

**Test Debug Code**:
```typescript
// Add to failing test
test('Select + Delete should clear selected area', async ({ page }) => {
    // ... setup ...
    
    // Debug: Check selection exists
    const hasSelection = await page.evaluate(() => {
        return (window as any).editor.has_selection();
    });
    expect(hasSelection).toBe(true);
    
    // Press Delete
    await page.keyboard.press('Delete');
    
    // Debug: Check selection was deleted
    const ascii = await page.evaluate(() => {
        return (window as any).editor.exportAscii();
    });
    console.log('ASCII after delete:', ascii);
    
    // ... assertions ...
});
```

## Consequences

### Positive
- Systematic approach to fixing E2E tests
- Clear prioritization based on user impact
- Debuggable with intermediate assertions
- Maintains ADR compliance tracking

### Negative
- Requires time investment (~8 hours estimated)
- May need to temporarily skip tests
- Could reveal deeper architectural issues

### Risks
- Phase 1 fixes may not resolve all keyboard issues
- Text tool may require WASM-side changes
- CI environment may have different behavior than local

## Implementation Plan

| Phase | Task | Est. Time | Owner |
|-------|------|-----------|-------|
| 1.1 | Add keyboard debug logging | 1h | Developer |
| 1.2 | Fix focus handling | 2h | Developer |
| 1.3 | Verify tool button updates | 1h | Developer |
| 2.1 | Add text tool E2E helpers | 2h | Developer |
| 2.2 | Skip failing text tests | 0.5h | Developer |
| 3.1 | Add select tool debug assertions | 1h | Developer |
| 3.2 | Verify ADR-030 fix | 1h | Developer |
| 4.0 | Re-enable all tests | 0.5h | Developer |

**Total Estimated Effort**: ~9 hours

## Success Criteria

- [ ] All keyboard shortcut tests passing
- [ ] Text tool tests passing OR documented skip with issue
- [ ] Select tool delete tests passing
- [ ] CI success rate > 95%
- [ ] No flaky tests (consistent pass/fail)

## References

- [ADR-003: Enhanced Keyboard UI](./003-enhanced-keyboard-ui.md)
- [ADR-010: Enhanced Text Tool](./010-enhanced-text-tool.md)
- [ADR-016: Text Tool Click Position](./016-text-tool-click-position.md)
- [ADR-024: Test Robustness Strategy](./024-test-robustness-strategy.md)
- [ADR-030: Select Delete Bug Fix](./030-select-delete-bug-fix.md)
- [GitHub Monitoring Report](../github-monitoring-report-2026-03-03.md)

## Notes

This ADR was created during active monitoring of the repository. The decisions here are based on observed CI failures and analysis of the codebase. Implementation should be tracked in the action log and current plan.
