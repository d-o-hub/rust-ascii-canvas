# Release Runbook

**Scope**: cutting a GitHub Release for ASCII Canvas via `.github/workflows/release.yml` (`workflow_dispatch`).
**Preflight**: `./scripts/release.sh` — read-only; never tags, pushes, or publishes.
**Related**: harness learnings [L-004](../agents-docs/harness.md#l-004--release-guard-rails-checked-git-tags-not-github-releases-2026-08-04) and [L-007](../agents-docs/harness.md#l-007--release-dispatched-without-a-version-bump-2026-08-08-2026-09-22).

## Golden path

1. **Sync main**

   ```bash
   git checkout main && git pull --ff-only
   ```

2. **Release-prep PR** — `VERSION` is the single source of truth; edit it and propagate:

   ```bash
   # 1. Edit VERSION (e.g. 0.1.3 -> 0.1.4)
   ./scripts/propagate-version.sh    # writes Cargo.toml, package.json, web/package.json
   ./scripts/release.sh              # preflight: pins agree + target not already released
   ```

   The fast gate also runs `propagate-version.sh --check`, so pin drift fails CI.

3. **Verify + merge**

   ```bash
   npm run gate:fast          # full sensors run in CI on the PR
   # push branch → open PR → merge when green
   ```

4. **Dry run first**

   ```bash
   gh workflow run release.yml -f dry_run=true
   gh run watch
   ```

   Guard Rails → `Determine version` must pass. Dry Run / Build WASM validate the pipeline without publishing.

5. **Real run**

   ```bash
   gh workflow run release.yml
   ```

6. **Verify**

   ```bash
   gh run watch
   gh release view vX.Y.Z
   gh release list --limit 3
   ```

   The workflow pushes the changelog update to `release/vX.Y.Z-changelog`, tags it, and attaches the optimized WASM build.

## Guard rails (what blocks a bad release)

- `Determine version` fails when `VERSION` is not valid semver, equals the latest GitHub Release, or is not a valid major/minor/patch increment over it.
- Version pins are checked twice: dev-time (`gate:fast` → `scripts/propagate-version.sh --check`) and release-time (`scripts/release.sh`).
- `wasm-opt` is required in CI; the release build compiles its own optimized WASM (`wasm-pack` + `wasm-opt`), `web/pkg` is gitignored.
- Version bumps land through PRs; the Release workflow only dispatches from `main`.

## Known state (2026-09-23)

- Latest release: **v0.1.3** (2026-08-05). `VERSION` on `main` is still `0.1.3`, so a dispatch today fails by design (L-007) — a **0.1.4** release-prep PR is the next step.
- ~29 commits and four merged PRs (#176/#177, #192, #194, #195) are unreleased since v0.1.3.
- `CHANGELOG.md` on `main` stops at 0.1.1. The auto-generated 0.1.2/0.1.3 entries lived on the now-deleted `release/v0.1.3-changelog` branch and remain recoverable from the tag:

  ```bash
  git show v0.1.3:CHANGELOG.md
  ```

- Planned with the 0.1.4 bump: curated `[0.1.2]` / `[0.1.3]` / `[0.1.4]` entries so the changelog is coherent again (tracked in `plans/FOLLOW_UPS.md`).

## Failure playbook

| Symptom | Fix |
|---------|-----|
| `Determine version` → `Version X already has a GitHub Release` | Open a release-prep PR that bumps all four pins, merge it, then re-dispatch (L-007). |
| `WASM Build` fails with `schema version` mismatch | Align `wasm-bindgen` dep + CLI pins (`Cargo.toml`, `mise.toml`, CI, `netlify:build`) — L-005. |
| Guard Rails green but no release appears | Check `gh release list` — git tags alone are not GitHub Releases (L-004). |
| Changelog range re-lists old commits | The workflow derives the range from `git describe`; verify the tag sequence before dispatch (candidate improvement, see FOLLOW_UPS R-02). |
