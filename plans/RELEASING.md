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
   gh release view vX.Y.Z --json assets --jq '.assets[].name'   # ascii-canvas-X.Y.Z.wasm
   gh release list --limit 3
   ```

   The workflow commits the generated changelog entry to a `release/vX.Y.Z-changelog`
   branch and creates the GitHub Release **targeting that branch**, so the release
   contains its own changelog entry. Release notes are anchored to the latest GitHub
   Release tag (`R-02`): a release tag can live on a release branch and is therefore
   not an ancestor of `main`, where `git describe` would fall back to an older tag
   and re-list already-released commits.

   The `Publish WASM` job attaches the optimized build to the release as
   `ascii-canvas-X.Y.Z.wasm` (`R-04`). It copies the artifact to a versioned
   filename first: `gh release upload`'s `file#label` syntax only sets a display
   label, so the download filename would otherwise stay `ascii_canvas_bg.wasm`.
   The upload uses `--clobber`, so a re-run replaces the asset instead of
   needing a fresh release. **v0.1.4 predates this change** and therefore has no
   attached asset; v0.1.5 is the first release that ships the binary.

## Guard rails (what blocks a bad release)

- `Determine version` fails when `VERSION` is not valid semver, equals the latest GitHub Release, or is not a valid major/minor/patch increment over it.
- Version pins are checked twice: dev-time (`gate:fast` → `scripts/propagate-version.sh --check`) and release-time (`scripts/release.sh`).
- `wasm-opt` is required in CI; the release build compiles its own optimized WASM (`wasm-pack` + `wasm-opt`), `web/pkg` is gitignored.
- Version bumps land through PRs; the Release workflow only dispatches from `main`.

## Known state (2026-09-24)

- Latest release: **v0.1.4** (2026-09-24) — dry run and real run both green. The L-007 guard-rail now has a version that actually passes.
- `VERSION` on `main` is `0.1.4` and matches the published release, so the **next** release needs a fresh bump PR (0.1.5) before any dispatch.
- Notes anchoring is live: the generated `[0.1.4]` entry lists the 28 commits since `v0.1.3` (the old `git describe` fallback would have re-listed 217).
- `CHANGELOG.md` on `main` is coherent again: `[0.1.4]` (generated, synced back from the tag) -> `[0.1.3]` / `[0.1.2]` (curated) -> `[0.1.1]`.
- **R-04 resolved**: the `Publish WASM` job now attaches `ascii-canvas-<version>.wasm` to the release (`--clobber`, copy-then-upload). v0.1.4 has no asset because it shipped before the change.

## Failure playbook

| Symptom | Fix |
|---------|-----|
| `Determine version` → `Version X already has a GitHub Release` | Open a release-prep PR that bumps all four pins, merge it, then re-dispatch (L-007). |
| `WASM Build` fails with `schema version` mismatch | Align `wasm-bindgen` dep + CLI pins (`Cargo.toml`, `mise.toml`, CI, `netlify:build`) — L-005. |
| Guard Rails green but no release appears | Check `gh release list` — git tags alone are not GitHub Releases (L-004). |
| Changelog range re-lists old commits | Fixed in 0.1.4: notes anchor to the latest GitHub Release tag (R-02). On older releases, sanity-check with `git log <latest-release-tag>..HEAD`. |
