#!/bin/bash
# Release script with guard-rails
# Usage: ./scripts/release.sh [patch|minor|major|<version>]

set -e

VERSION="${1:-patch}"
DRY_RUN="${2:-false}"

echo "========================================"
echo "ASCII Canvas Release Script"
echo "========================================"
echo "Version strategy: $VERSION"
echo "Dry run: $DRY_RUN"
echo ""

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Guard-rails check function
check() {
    echo -n "Running $1... "
    if eval "$2"; then
        echo -e "${GREEN}✓${NC}"
        return 0
    else
        echo -e "${RED}✗${NC}"
        return 1
    fi
}

# Step 1: Guard Rails
echo "=== GUARD RAILS ==="

check "Unit Tests" "cargo test --lib"
check "Clippy" "cargo clippy -- -D warnings"
check "Format" "cargo fmt --check"

echo ""
echo "=== Guard Rails Passed ==="
echo ""

# Step 2: Version determination
echo "=== VERSION ==="

if [[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    NEW_VERSION="$VERSION"
else
    LAST_TAG=$(git describe --tags --abbrev=0 2>/dev/null || echo "v0.0.0")
    LAST_VERSION=${LAST_TAG#v}
    
    IFS='.' read -r MAJOR MINOR PATCH <<< "$LAST_VERSION"
    MAJOR=${MAJOR:-0}
    MINOR=${MINOR:-0}
    PATCH=${PATCH:-0}
    
    case "$VERSION" in
        major)
            MAJOR=$((MAJOR + 1))
            MINOR=0
            PATCH=0
            ;;
        minor)
            MINOR=$((MINOR + 1))
            PATCH=0
            ;;
        patch|*)
            PATCH=$((PATCH + 1))
            ;;
    esac
    
    NEW_VERSION="$MAJOR.$MINOR.$PATCH"
fi

echo "Releasing version: $NEW_VERSION"
echo ""

if [ "$DRY_RUN" == "true" ]; then
    echo -e "${YELLOW}DRY RUN - No actual release will be created${NC}"
    exit 0
fi

# Step 3: Build WASM
echo "=== BUILD WASM ==="
wasm-pack build --release --target web --out-dir web/pkg
echo ""

# Step 4: Create tag
echo "=== CREATE TAG ==="
TAG_NAME="v$NEW_VERSION"
git tag -a "$TAG_NAME" -m "Release $TAG_NAME"
echo "Created tag: $TAG_NAME"
echo ""

# Step 5: Create release
echo "=== CREATE RELEASE ==="
gh release create "$TAG_NAME" \
    --title "$TAG_NAME" \
    --notes-file /dev/stdin \
    --draft false \
    --prerelease false << 'RELEASE_NOTES'
## Quick Start
```bash
wasm-pack build --release --target web --out-dir web/pkg
cd web && npm run dev
```

See CHANGELOG.md for full details.
RELEASE_NOTES

echo ""
echo "=== PUSH TAGS ==="
git push origin "$TAG_NAME"
git push origin main

echo ""
echo -e "${GREEN}========================================"
echo "Release $NEW_VERSION Complete!"
echo "========================================${NC}"
