# ADR 007: Fix E2E Tests in GitHub Actions CI

## Status
Accepted

## Date
2026-02-26

## Context
The E2E tests were failing in GitHub Actions CI due to:
1. Vite dev server not being accessible (vite not found)
2. Port conflicts between webServer and manual server startup
3. Missing npm dependencies in web directory
4. Complex server management in CI environment

## Research Findings (Playwright Best Practices)

Based on official Playwright documentation:

1. **Use Playwright Docker Container**: Microsoft provides a pre-built Docker image (`mcr.microsoft.com/playwright:v1.58.2-noble`) with all dependencies including Xvfb for headed mode

2. **Proper webServer Configuration**: Playwright's `webServer` option should handle server lifecycle automatically

3. **Workers Setting**: Set `workers: 1` in CI for stability

4. **CI Environment Variable**: Set `CI: true` to disable parallel execution

## Decision

### Solution: Use Playwright Docker Container (Recommended by Playwright)

The most reliable approach is to use Microsoft's official Playwright Docker container which:
- Has all browser dependencies pre-installed
- Includes Xvfb for headless browsing
- Provides consistent environment

### Alternative: Manual Setup Fixes

If Docker is not preferred:
1. Install all npm dependencies (root + web)
2. Use proper `webServer` configuration in playwright.config.ts
3. Set `CI=true` environment variable
4. Use `workers: 1` for CI

## Implementation Plan

1. Update `.github/workflows/ci.yml` to use Playwright container approach
2. OR update playwright.config.ts with proper CI settings
3. Add CI-specific environment variables

## Consequences

### Benefits
- More reliable E2E tests in CI
- Faster test execution (no manual browser installation)
- Consistent environment across runs
- Recommended approach by Playwright team

### Trade-offs
- Docker container approach requires container support
- May need to update if switching Playwright versions

## Files to Modify
- `.github/workflows/ci.yml`
- `playwright.config.ts`

## References
- [Playwright CI Documentation](https://playwright.dev/docs/ci)
- [Playwright GitHub Actions Guide](https://playwright.dev/docs/ci#github-actions)
