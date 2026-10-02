# ADR-046: Editing Safety and Named Local Drafts

## Status

Accepted — explicitly approved by the user on 2026-09-30. **Delivered in scoped candidates:** editor-safety in #229, named local drafts in #230, accessible layer panel in #231. The named-draft UI, storage, migration and conflict handling shipped there; the larger harness/skills overhaul ships separately with the fail-closed sensors port (PR-gated). Acceptance is not PR/remote-gate evidence.

## Context

The audit reproduced data loss at boundaries between tools, history, layers, viewport resizing, and document replacement. The editor also exposes only one local autosave slot. These boundaries must be safe before expanding multi-document workflows.

## Decision

- Preserve ADR-043's per-layer history. A completed freehand/eraser gesture is one non-coalescing transaction, retaining the original value of every affected cell; independent gestures never merge accidentally.
- Preserve the live added layer when undoing/replaying its creation, including its metadata and history. Stable layer identities remain unchanged.
- Cancel/reset transient interactions on document replacement and layer changes. No buffered text or unfinished shape may cross documents or layers.
- Viewport changes affect presentation only after initial document sizing. Intentional grid cropping requires confirmation; resizing still invalidates coordinate-based history as documented by ADR-043.
- Move the shared `.asc` v1 DTO and existing validation limits to pure core. Preserve v1 shape/defaults and existing import compatibility; do not introduce a format version or silently tighten unrelated legacy parsing policies. Apply the same layer limit when creating layers.
- Add named local drafts as a web persistence concern, outside `.asc` v1: create/name/list/switch/delete, selected draft restore, import-as-new, non-destructive migration from the legacy autosave, explicit save outcome and stale-writer conflict reporting. Successful persistence is a precondition for replacing the current document.
- Keep local storage failure recoverable: preserve the current in-memory document and offer `.asc` download; retain the original autosave during migration failures. Do not claim local browser storage is a durable backup.
- Export committed artwork through an additive clean pixel API without selection/preview overlays or destructive selection changes.

## Alternatives

- Global chronological history: rejected; out of scope and contradicts the bounded per-layer model.
- Collaboration/accounts: deferred until single-user safety and demand are established.
- Store titles/IDs inside `.asc`: unnecessary format change; shelf metadata is separate.
- Merely log persistence failures: rejected because users cannot know whether their work is recoverable.

## Consequences

Interaction/session and document limits gain shared core definitions, while browser storage remains web-owned. Draft switching requires explicit error handling and conflict tests. Gesture and layer history regressions must preserve both content and undo/redo semantics. Existing v1 files remain compatible. This does not promise IME/grapheme-wide editing, distributed synchronization, or archival storage.

## Verification

Editor-safety evidence is recorded in the [editor-safety delivery plan](../editor-safety-delivery-2026-09-30.md). Named-draft implementation and isolated-tree evidence are recorded in the [local-drafts delivery plan](../local-drafts-delivery-2026-09-30.md). This decision does not establish remote CI or merge readiness; ordinary review, Codacy and merge-gate requirements remain.
