# ADR-008: Automated Dependency Updates with Dependabot

## Status

**Accepted** - 2026-02-27

## Context

The project uses multiple package ecosystems:
- **Cargo** (Rust) - 14 production dependencies, 1 dev dependency
- **npm** (root) - 2 dev dependencies (Playwright)
- **npm** (/web) - 4 dev dependencies (Vite, TypeScript, Vitest, Playwright)

Manually tracking updates across all ecosystems is time-consuming and risks security vulnerabilities in dependencies.

### Current State

| Ecosystem | Update Method | Status |
|-----------|---------------|--------|
| Cargo | Manual (`cargo outdated`) | No automation |
| npm | Manual (`npm outdated`) | No automation |

### Industry Best Practices (2026)

1. **Dependabot** is GitHub's recommended solution for automated dependency updates
2. Weekly schedules are optimal - monthly is too infrequent, daily is too noisy
3. Grouping minor/patch updates separately from major prevents breaking changes
4. PR limits prevent overwhelming maintainers

## Decision

Implement Dependabot with version 2 configuration:

### Configuration Details

```yaml
version: 2
updates:
  # Rust/Cargo
  - package-ecosystem: "cargo"
    directory: "/"
    schedule:
      interval: "weekly"
      day: "monday"
      time: "06:00"
      timezone: "UTC"
    open-pull-requests-limit: 10
    versioning-strategy: "increase"
    groups:
      minor-updates:
        patterns: ["*"]
        update-types: ["minor", "patch"]
      major-updates:
        patterns: ["*"]
        update-types: ["major"]

  # npm (root)
  - package-ecosystem: "npm"
    directory: "/"
    schedule:
      interval: "weekly"
      day: "monday"
      time: "06:00"
      timezone: "UTC"
    open-pull-requests-limit: 10

  # npm (/web)
  - package-ecosystem: "npm"
    directory: "/web"
    schedule:
      interval: "weekly"
      day: "monday"
      time: "06:00"
      timezone: "UTC"
    open-pull-requests-limit: 10
```

### Configuration Choices

| Setting | Value | Rationale |
|---------|-------|------------|
| `version` | 2 | Latest Dependabot config format |
| `schedule` | weekly (Monday 6am UTC) | Balance between frequent updates and low noise |
| `open-pull-requests-limit` | 10 | Prevent PR flood |
| `versioning-strategy` | increase | Semver-compliant for Rust |
| `commit-message.prefix` | chore | Conventional commits |
| `labels` | dependencies | Categorization |

## Consequences

### Benefits

1. **Security**: Automated CVEs addressed promptly
2. **Maintainability**: Dependencies stay up-to-date
3. **Reduced toil**: No manual `cargo outdated` / `npm outdated`
4. **Predictable**: Weekly schedule is reliable
5. **Safe**: Major updates grouped separately to avoid breaking changes

### Risks

1. **Noise**: 10 PRs per week could overwhelm notifications
   - **Mitigation**: Labels allow filtering, groups reduce PR count
2. **Breaking changes**: Even minor updates can break
   - **Mitigation**: CI runs tests on each PR
3. **Lock file conflicts**: Multiple PRs could conflict
   - **Mitigation**: 10 PR limit reduces this risk

### Effort Required

| Task | Time |
|------|------|
| Initial configuration | 30 min |
| Ongoing review | ~5 min/week (reviewing PRs) |
| Merging | ~10 min/week (if tests pass) |

## References

- [Dependabot Configuration Options](https://docs.github.com/en/code-security/dependabot/dependabot-version-updates/configuration-options-for-the-dependabotyml-file)
- [Dependabot ALM Best Practices](https://dependabot.com/blog/best-practices-with-dependabot/)
