#!/bin/bash
# pforge Release Script
# Automates version bumping and release tagging
#
# Usage:
#   ./scripts/release.sh <version>
#   ./scripts/release.sh 0.2.0

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if version argument provided
if [ -z "$1" ]; then
    echo -e "${RED}Error: Version argument required${NC}"
    echo "Usage: ./scripts/release.sh <version>"
    echo "Example: ./scripts/release.sh 0.2.0"
    exit 1
fi

NEW_VERSION="$1"

echo -e "${GREEN}╔═══════════════════════════════════════╗${NC}"
echo -e "${GREEN}║   pforge Release Automation          ║${NC}"
echo -e "${GREEN}║   Version: ${NEW_VERSION}                 ║${NC}"
echo -e "${GREEN}╚═══════════════════════════════════════╝${NC}"
echo ""

# Validate version format
if ! echo "$NEW_VERSION" | grep -qE '^[0-9]+\.[0-9]+\.[0-9]+$'; then
    echo -e "${RED}Error: Invalid version format${NC}"
    echo "Version must be in format: X.Y.Z (e.g., 0.2.0)"
    exit 1
fi

# Check if working directory is clean
if [ -n "$(git status --porcelain)" ]; then
    echo -e "${RED}Error: Working directory not clean${NC}"
    echo "Please commit or stash changes before releasing"
    git status --short
    exit 1
fi

# Check if on main branch
CURRENT_BRANCH=$(git branch --show-current)
if [ "$CURRENT_BRANCH" != "main" ]; then
    echo -e "${YELLOW}Warning: Not on main branch (current: $CURRENT_BRANCH)${NC}"
    read -p "Continue anyway? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

echo -e "${GREEN}✓${NC} Working directory clean"
echo -e "${GREEN}✓${NC} On branch: $CURRENT_BRANCH"
echo ""

# Update version in Cargo.toml files
echo -e "${YELLOW}→${NC} Updating version in Cargo.toml files..."

# Workspace Cargo.toml
sed -i "s/^version = \".*\"/version = \"$NEW_VERSION\"/" Cargo.toml

# Update all crate Cargo.toml files
for crate_toml in crates/*/Cargo.toml; do
    sed -i "s/^version = \".*\"/version = \"$NEW_VERSION\"/" "$crate_toml"
    echo "  Updated: $crate_toml"
done

echo -e "${GREEN}✓${NC} Version updated to $NEW_VERSION"
echo ""

# Update Cargo.lock
echo -e "${YELLOW}→${NC} Updating Cargo.lock..."
cargo check --quiet
echo -e "${GREEN}✓${NC} Cargo.lock updated"
echo ""

# Run tests
echo -e "${YELLOW}→${NC} Running tests..."
if cargo test --all --quiet; then
    echo -e "${GREEN}✓${NC} All tests passed"
else
    echo -e "${RED}✗${NC} Tests failed"
    echo "Please fix tests before releasing"
    exit 1
fi
echo ""

# Run quality gates
echo -e "${YELLOW}→${NC} Running quality gates..."
if cargo clippy --all-targets --all-features --quiet -- -D warnings 2>&1 | grep -q "warning:"; then
    echo -e "${RED}✗${NC} Clippy warnings found"
    cargo clippy --all-targets --all-features -- -D warnings
    exit 1
fi
echo -e "${GREEN}✓${NC} Clippy passed"

if ! cargo fmt --all -- --check &>/dev/null; then
    echo -e "${RED}✗${NC} Formatting check failed"
    exit 1
fi
echo -e "${GREEN}✓${NC} Formatting passed"
echo ""

# Create release commit
echo -e "${YELLOW}→${NC} Creating release commit..."
git add Cargo.toml Cargo.lock crates/*/Cargo.toml

git commit -m "chore: bump version to $NEW_VERSION

Release preparation for v$NEW_VERSION

This commit bumps all crate versions to $NEW_VERSION in preparation
for the release. All quality gates have passed.

🤖 Generated with release script"

echo -e "${GREEN}✓${NC} Release commit created"
echo ""

# Create git tag
echo -e "${YELLOW}→${NC} Creating git tag v$NEW_VERSION..."
git tag -a "v$NEW_VERSION" -m "Release v$NEW_VERSION

pforge v$NEW_VERSION

See GitHub release notes for full changelog and installation instructions.

🤖 Generated with release script"

echo -e "${GREEN}✓${NC} Tag v$NEW_VERSION created"
echo ""

# Summary
echo -e "${GREEN}╔═══════════════════════════════════════╗${NC}"
echo -e "${GREEN}║   Release Preparation Complete!      ║${NC}"
echo -e "${GREEN}╚═══════════════════════════════════════╝${NC}"
echo ""
echo "Version: $NEW_VERSION"
echo "Tag: v$NEW_VERSION"
echo ""
echo "Next steps:"
echo "  1. Review changes: git show HEAD"
echo "  2. Push commit: git push origin $CURRENT_BRANCH"
echo "  3. Push tag: git push origin v$NEW_VERSION"
echo ""
echo "The GitHub Actions release workflow will automatically:"
echo "  - Run quality gates"
echo "  - Build binaries for all platforms"
echo "  - Create GitHub release with changelog"
echo "  - Publish to crates.io"
echo ""
echo -e "${YELLOW}Note: You can also trigger a manual release via GitHub Actions${NC}"
