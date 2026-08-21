# ADR-041: Configurable Clipboard Export Fidelity

## Status
Accepted

## Context

Issue #176 reports that copying box-drawing diagrams into external editors can lose
right borders when trailing spaces are removed, and that Unicode box glyphs can
render inconsistently in Windows Notepad and other non-monospace environments.
Clipboard writes also need to work when the asynchronous Clipboard API is blocked
by permissions, an insecure context, or an iframe policy.

## Decision

Keep the core exporter as the source of truth and add additive export options for:

- preserving rectangular line widths by default,
- optionally trimming trailing whitespace when rectangular preservation is disabled,
- converting supported drawing glyphs to a 7-bit ASCII fallback.

Expose an additive WASM export method for callers that need explicit Rust-side
options. The web copy path mirrors those options for event-generated exports and
presents them as a Copy Format selector plus a rectangular-width checkbox. Normalize
all resulting text to CRLF, attempt `navigator.clipboard.writeText`, and fall back
to a temporary textarea with `document.execCommand('copy')` when the modern API
cannot be used.

## Consequences

- Existing Unicode copy behavior remains the default and existing export APIs stay
  source-compatible for callers using `Default` or the existing functions.
- Pure ASCII mode intentionally replaces unsupported non-ASCII glyphs with `?` so
  the requested mode is guaranteed to remain 7-bit.
- Rectangular exports may contain trailing spaces; this is deliberate because those
  spaces preserve columns in external editors.
- The legacy fallback is synchronous internally and is used only after the modern
  API is unavailable or rejects the write.
