# ADR-047: Fail-Closed Verification and Retained Coherence Fixtures

## Status

Accepted — explicitly approved by the user on 2026-09-30. The editor-safety WASM test slice shipped in #229; the remaining fail-closed audit/artifact sensors, WASM binding freshness, pnpm pin parity, offline skill-manifest checks, production-dist E2E, and CI wiring shipped in #241 (`642d1d4`). No live ruleset change was authorized or performed.

## Context

The audit found npm audits and missing WASM artifacts could exit successfully without verification; dedicated WASM tests were undiscovered; architecture checks suppressed scanner failures; CI path/coherence checks and skill resources had untested gaps. Repeated manual mutation experiments are not durable regression tests.

## Decision

- Required audit results distinguish clean, findings, and unverified. A required CI audit cannot pass unless both committed npm lockfiles were checked successfully. Never classify arbitrary advisory prose as a network error.
- Artifact size checking requires a present, readable, valid WASM artifact before comparing its size budget.
- Register and execute the dedicated WASM behavior suite using a runner consistent with its actual API requirements; assert nonzero discovery. Browser E2E remains the DOM/browser integration sensor.
- Retain positive/negative fixtures for audit, artifact, architecture, LOC and CI applicability/coherence. Validate jobs, aggregation and path coverage, not just a hand-maintained subset.
- Ensure generated WASM bindings are fresh before typechecking, use locked build/install inputs, and test a production-shaped web build in full/CI verification using existing BASE_URL support.
- Add an offline ownership-aware skill resource/manifest checker. Explicit upstream exceptions are allowed and documented; no blanket exclusion of owned guidance.
- Run shared checks both locally and directly in the appropriate existing CI jobs. The workflow still does not invoke quality-gates.sh; local-only wiring is insufficient.

## Guard-rail boundaries

The server-side required-check set, thread/review requirements, auto-merge behavior, release pins and release-version policy are unchanged. These changes make existing verification truthful and add local/CI regression evidence; they never remove or relax required checks. A merge of this harness change still needs the normal gate and human review for guard-rail changes.

## Alternatives

- Warn and exit success when a required audit cannot run: rejected; GitHub sees the exit status, not the warning's intent.
- Add more prose after each sensor failure: insufficient without retained fixtures.
- Install a new orchestration framework: unnecessary for these bounded checks.

## Consequences

Offline full verification may now fail as unverified instead of appearing green; fast offline checks remain useful. CI failures should identify the missing evidence and next corrective action. New sensor fixtures add small deterministic costs. Production-shaped smoke adds a web build, while focused tests retain the fast dev-server path.

## Verification

See the [editor-safety delivery plan](../editor-safety-delivery-2026-09-30.md) for this slice's narrow test wiring and isolated-tree evidence. The full harness, skill/coherence fixtures, fail-closed audit/artifact checks and production-build gate rewrite are not shipped here. No claim that repository state, third-party checks or remote CI have been verified by local tests.

2026-10-02 update: those pieces ship with the fail-closed sensors + skills change; see [harness L-022–L-025](../../agents-docs/harness.md).
