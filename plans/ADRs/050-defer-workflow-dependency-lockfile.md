# ADR-050: Retain Full-SHA Action Pins and Defer the Workflow Lockfile

## Status

Accepted — 2026-10-16. The workflow dependency lockfile pilot was evaluated and
deferred. No `.github/workflows/actions.lock` is committed.

## Context

The repository's workflows use full commit-SHA references for third-party
actions. This is enforced by the required Codacy rule family and was completed
in PR #270. The repository also has a recorded backlog item for GitHub's
workflow dependency lockfile feature, which was documented by GitHub as public
preview on 2026-10-12.

GitHub's `gh-actions-lock` extension provides a different control model: it
generates `.github/workflows/actions.lock`, rewrites workflow references to
symbolic refs, and verifies the lockfile's recorded commit, owner, and
repository identity. Its documentation says that SHA pinning in workflow files
is no longer necessary when a lockfile is adopted.

## Decision

1. Retain full commit-SHA action references in all workflows.
2. Do not add `.github/workflows/actions.lock` or a lockfile enforcement job
   while the feature remains public preview and conflicts with the repository's
   required SHA-pin policy.
3. Treat the lockfile backlog item as deferred, not as an implementation gap.
4. Revisit this decision only after a reviewed compatibility test shows that
   lockfile verification can preserve the repository's SHA-pin requirement, or
   after an explicitly approved policy change replaces that requirement. A
   future policy change must include its own ADR, Codacy impact assessment, and
   updated workflow/security verification.

## Pilot evidence

On 2026-10-16, `gh-actions-lock` v0.2.0 was installed and run against all
three workflows:

- The generated candidate covered all three workflows and 13 direct actions.
- It rewrote full SHA references such as `actions/checkout@<sha>` to symbolic
  refs such as `actions/checkout@v7.0.1`.
- The candidate passed `gh actions-lock --verify-local` and
  `gh actions-lock --verify` only in that rewritten form.
- The tool rejected the existing `dtolnay/rust-toolchain` SHA because the
  commit is not reachable from a current upstream branch or tag, even though
  the commit exists in the upstream repository. Replacing that pin would be a
  separate action-pin change, not an incidental lockfile conversion.
- The candidate was discarded; the working tree retains the existing full-SHA
  workflow policy and contains no lockfile.

## Consequences

### Benefits

- Workflow files remain directly auditable: each action reference identifies the
  exact commit that runs.
- The required Codacy SHA-pin check remains aligned with local and CI policy.
- No public-preview lockfile format, generated rewrite, or enforcement behavior
  is introduced without a compatible security review.

### Trade-offs

- The repository does not receive the lockfile's transitive-dependency coverage.
- Dependabot cannot maintain a workflow lockfile until one is adopted.
- Reconsideration is required if GitHub changes the tool or if the repository
  intentionally chooses the lockfile model over direct SHA pins.

## References

- [Workflow dependency lockfiles](https://docs.github.com/en/actions/concepts/security/workflow-dependency-lockfiles)
- [Locking action dependencies](https://docs.github.com/en/actions/how-tos/secure-your-work/lock-workflow-dependencies)
- [Enforcing lockfiles for action dependencies](https://docs.github.com/en/actions/how-tos/administer/enforce-workflow-lockfiles)
- [GitHub Actions secure use](https://docs.github.com/en/actions/reference/security/secure-use)
- [PR #270 — immutable CI action pins](https://github.com/d-o-hub/rust-ascii-canvas/pull/270)
