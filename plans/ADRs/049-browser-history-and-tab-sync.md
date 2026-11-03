# ADR-049: Bounded browser history snapshots and conservative tab sync

## Status

Accepted — 2026-11-03

## Context

The Rust editor history is intentionally runtime-only and contains trait objects
whose command payloads are not part of the `.asc` v1 interchange format. Local
drafts already persist the current document, but a reload creates a fresh Rust
history and a second tab only reports an optimistic conflict. Issue #303 needs
reload-safe undo/redo and same-origin tab updates without changing `.asc` files or
introducing a CRDT.

## Decision

Persist a bounded timeline of serialized document snapshots alongside each local
draft. The current snapshot is always retained; older snapshots are trimmed by a
fixed entry and character budget. On reload, the web layer restores the timeline
and applies snapshots through the existing validated `loadDocument` boundary.

Use `BroadcastChannel` as the low-latency notification path and the `storage`
event as the fallback. A remote shelf revision is applied only while the local
tab is clean. If local work is dirty, the existing conflict state remains the
safe result: the remote document is not applied and the local work is not
overwritten. The sender is never notified by its own `BroadcastChannel` message,
and the revision token makes duplicate storage/broadcast delivery idempotent.

## Consequences

- Undo and redo survive reloads for the retained snapshot window.
- Two clean tabs converge on the saved document and its history without a new
  server or dependency.
- Unsaved local edits are preserved when a competing tab writes; the user must
  resolve the conflict explicitly.
- `.asc` files remain version-one compatible and deliberately do not contain
  browser-local history metadata.
- This is synchronization of complete local snapshots, not collaborative merge;
  CRDT collaboration remains a separate decision.
