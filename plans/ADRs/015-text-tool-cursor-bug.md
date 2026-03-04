# ADR-015: Text Tool Cursor Position Bug

## Status
Proposed

## Context
User reports that after a few text insertions, the text no longer inserts at the cursor position. This indicates a bug in how the text tool tracks cursor position.

## Analysis

### Potential Issues Identified

1. **Backspace Logic Bug**: The backspace function checks `if x > 0` before moving cursor back, but this doesn't account for the cursor having moved forward from the starting position.

2. **Buffer vs Grid State**: The text tool maintains a buffer but commits characters individually. This could cause desync.

3. **Escape/Click Behavior**: When clicking elsewhere or pressing Escape, the buffer is committed but state might not reset properly.

4. **Cursor Position Overflow**: No bounds checking on cursor position after multiple insertions.

## Decision

### Fix 1: Improve Backspace Logic
The backspace should work relative to the buffer, not absolute X position.

### Fix 2: Proper State Reset
Ensure cursor, buffer, and start_pos are properly managed.

### Fix 3: Add E2E Tests
Create comprehensive tests for text tool with multiple insertions.

## Implementation Plan

1. Fix backspace to work correctly regardless of cursor X position
2. Ensure proper state cleanup on commit/reset
3. Add E2E tests for multi-character text input

## Test Cases to Add
- Type 5 characters at position, verify all appear
- Type, backspace, type more - verify correct behavior
- Click in new location, type - verify fresh start
