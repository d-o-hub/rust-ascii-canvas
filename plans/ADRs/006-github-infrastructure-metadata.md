# ADR 006: GitHub Infrastructure and Package Metadata

## Status
Accepted

## Date
2026-02-26

## Context
The project lacked standard GitHub community files, CI/CD workflows, and had inconsistent package metadata across Cargo.toml and package.json.

## Decision
We implemented the following:

### 1. GitHub Actions CI Workflow
- Created `.github/workflows/ci.yml` with:
  - Rust build and test job
  - wasm-pack build
  - Playwright E2E tests on Chromium
  - Caching for dependencies

### 2. GitHub Community Files
- LICENSE (MIT)
- `.github/ISSUE_TEMPLATE/bug_report.yml`
- `.github/ISSUE_TEMPLATE/feature_request.yml`
- `.github/ISSUE_TEMPLATE/config.yml`
- `.github/PULL_REQUEST_TEMPLATE.md`
- `.github/SECURITY.md`
- `.github/CODEOWNERS`

### 3. Project Configuration Files
- `rustfmt.toml` - Code formatting rules
- `clippy.toml` - Linter configuration

### 4. Package Metadata Fixes
- **Cargo.toml**: Updated repository URL, added keywords, categories, authors, homepage, documentation
- **package.json**: Synced version (0.1.0), fixed license (MIT), added proper scripts and repository info

## Consequences
- GitHub Community Standards now shows 100%
- CI/CD pipeline will run on all PRs
- Package metadata is consistent and complete
- License is explicitly stated

## Files Created/Modified
- `.github/workflows/ci.yml` (new)
- `LICENSE` (new)
- `.github/ISSUE_TEMPLATE/` (new)
- `.github/PULL_REQUEST_TEMPLATE.md` (new)
- `.github/SECURITY.md` (new)
- `.github/CODEOWNERS` (new)
- `CONTRIBUTING.md` (new)
- `rustfmt.toml` (new)
- `clippy.toml` (new)
- `Cargo.toml` (modified)
- `package.json` (modified)
