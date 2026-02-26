# ADR-004: GitHub Repository Configuration

## Status

**Proposed** - 2026-02-26

## Context

The repository lacks standard GitHub community files and templates that improve project discoverability, contributor experience, and maintainability.

### Current State

| File | Status | Impact |
|------|--------|--------|
| `.github/ISSUE_TEMPLATE/` | Missing | No structured bug reports/feature requests |
| `.github/PULL_REQUEST_TEMPLATE.md` | Missing | Inconsistent PR descriptions |
| `.github/SECURITY.md` | Missing | No security policy |
| `.github/CODEOWNERS` | Missing | No automatic review assignments |
| `LICENSE` | Missing | License unclear despite Cargo.toml saying MIT |
| `CONTRIBUTING.md` | Missing | Contributors lack guidance |
| `CHANGELOG.md` | Missing | No version history tracking |
| `rustfmt.toml` | Missing | Inconsistent formatting config |
| `clippy.toml` | Missing | No lint configuration |

### Industry Best Practices (2026)

Per GitHub's community standards and open-source best practices:

1. **Issue Templates**: Structured forms reduce back-and-forth, ensure reproducible bugs
2. **PR Templates**: Checklist ensures code quality, tests run, documentation updated
3. **SECURITY.md**: Critical for responsible disclosure; GitHub shows security banner
4. **LICENSE**: Required for open-source; without it, code is technically proprietary
5. **CONTRIBUTING.md**: Reduces maintainer burden, sets expectations

## Decision

Create the following GitHub configuration files:

### 1. Issue Templates (`.github/ISSUE_TEMPLATE/`)

```
.github/
├── ISSUE_TEMPLATE/
│   ├── bug_report.yml      # Structured bug report
│   ├── feature_request.yml # Feature request form
│   └── config.yml          # Template config (links to docs)
```

### 2. Pull Request Template (`.github/PULL_REQUEST_TEMPLATE.md`)

Include checklist for:
- [ ] Tests pass
- [ ] Code formatted (`cargo fmt`)
- [ ] Clippy passes
- [ ] Documentation updated

### 3. Security Policy (`.github/SECURITY.md`)

- Supported versions
- Reporting vulnerabilities (email/security advisory)
- Response timeline expectations

### 4. License File (`LICENSE`)

MIT License text (matching Cargo.toml)

### 5. Contributing Guide (`CONTRIBUTING.md`)

- Development setup
- Code style expectations
- PR process
- Testing requirements

### 6. Rust Configuration

**`rustfmt.toml`**:
```toml
edition = "2021"
max_width = 100
use_small_heuristics = "Default"
```

**`clippy.toml`**:
```toml
msrv = "1.70"
```

## Consequences

### Benefits

1. **Discoverability**: GitHub Community Standards checklist shows 100%
2. **Contributor Experience**: Clear guidelines reduce friction
3. **Maintainability**: Structured issues/PRs save maintainer time
4. **Security**: Clear disclosure process protects users
5. **Legal Clarity**: LICENSE file makes MIT status explicit

### Effort Required

| Task | Time | Priority |
|------|------|----------|
| LICENSE | 1 min | Critical |
| Issue Templates | 15 min | High |
| PR Template | 5 min | High |
| SECURITY.md | 10 min | High |
| CONTRIBUTING.md | 20 min | Medium |
| rustfmt.toml | 2 min | Medium |
| clippy.toml | 1 min | Low |

## References

- [GitHub Community Standards](https://docs.github.com/en/communities/setting-up-your-project-for-healthy-contributions)
- [Open Source Guides - Your Code of Conduct](https://opensource.guide/code-of-conduct/)
- [Rust API Guidelines - Tooling](https://rust-lang.github.io/api-guidelines/about.html)
