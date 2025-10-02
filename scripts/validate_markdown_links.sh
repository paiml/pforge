#!/bin/bash
# Validate markdown links in repository
# Usage: ./scripts/validate_markdown_links.sh

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

echo "🔗 Validating markdown links..."

# Find all markdown files
md_files=$(find . -name "*.md" -type f \
    -not -path "./target/*" \
    -not -path "./pforge-book/book/*" \
    -not -path "./.git/*")

errors=0
warnings=0

# Extract and validate links
for file in $md_files; do
    # Extract markdown links: [text](path)
    # Match local file links (not URLs)
    links=$(grep -oP '\]\(\K[^)]+' "$file" 2>/dev/null | grep -v '^http' | grep -v '^#' || true)

    for link in $links; do
        # Skip anchors
        if [[ "$link" == \#* ]]; then
            continue
        fi

        # Strip anchor from link if present
        link_no_anchor="${link%#*}"

        # Get directory of current file for relative path resolution
        dir=$(dirname "$file")

        # Resolve relative path
        if [[ "$link_no_anchor" == /* ]]; then
            # Absolute path from repo root
            full_path="${link_no_anchor#/}"
        else
            # Relative path
            full_path="$dir/$link_no_anchor"
        fi

        # Normalize path (remove ./ and ../)
        full_path=$(realpath -m "$full_path" 2>/dev/null || echo "$full_path")

        # Check if file exists
        if [ ! -e "$full_path" ]; then
            # Check if it's a directory
            if [ ! -d "$full_path" ]; then
                echo -e "${RED}✗${NC} Broken link in $file:"
                echo "    Link: $link"
                echo "    Expected: $full_path"
                errors=$((errors + 1))
            fi
        fi
    done
done

echo ""
if [ $errors -eq 0 ]; then
    echo -e "${GREEN}✓${NC} All markdown links valid"
    exit 0
else
    echo -e "${RED}✗${NC} Found $errors broken link(s)"
    exit 1
fi
