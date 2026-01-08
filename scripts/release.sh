#!/bin/bash
#
# Release script for Spacesaver macOS screen saver
#
# This script:
# 1. Validates the version number
# 2. Updates version in Cargo.toml and Info.plist
# 3. Builds the release
# 4. Creates a distributable zip
# 5. Optionally tags and pushes
#
# Usage: ./scripts/release.sh <version> [--tag] [--push]
#   version: Semantic version (e.g., 0.1.0, 1.0.0-beta.1)
#   --tag:   Create git tag after build
#   --push:  Push commits and tags to remote
#
# Example: ./scripts/release.sh 0.2.0 --tag --push

set -e

# Configuration
PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
CARGO_TOML="$PROJECT_ROOT/spacesaver-core/Cargo.toml"
INFO_PLIST="$PROJECT_ROOT/swift-wrapper/Spacesaver/Info.plist"
BUILD_DIR="$PROJECT_ROOT/build"
DIST_DIR="$PROJECT_ROOT/dist"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Parse arguments
VERSION=""
DO_TAG=false
DO_PUSH=false

for arg in "$@"; do
    case $arg in
        --tag)
            DO_TAG=true
            ;;
        --push)
            DO_PUSH=true
            ;;
        -h|--help)
            echo "Usage: $0 <version> [--tag] [--push]"
            echo ""
            echo "Arguments:"
            echo "  version    Semantic version (e.g., 0.1.0, 1.0.0-beta.1)"
            echo "  --tag      Create git tag after successful build"
            echo "  --push     Push commits and tags to remote"
            echo ""
            echo "Example: $0 0.2.0 --tag --push"
            exit 0
            ;;
        *)
            if [[ -z "$VERSION" ]]; then
                VERSION="$arg"
            else
                echo -e "${RED}Error: Unknown argument '$arg'${NC}"
                exit 1
            fi
            ;;
    esac
done

# Validate version
if [[ -z "$VERSION" ]]; then
    echo -e "${RED}Error: Version number required${NC}"
    echo "Usage: $0 <version> [--tag] [--push]"
    exit 1
fi

# Validate semantic version format
if ! [[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.]+)?$ ]]; then
    echo -e "${RED}Error: Invalid version format '$VERSION'${NC}"
    echo "Expected semantic version like: 0.1.0, 1.0.0, 2.0.0-beta.1"
    exit 1
fi

echo -e "${BLUE}=== Spacesaver Release v$VERSION ===${NC}"
echo ""

# Check for uncommitted changes
if [[ -n "$(git status --porcelain)" ]]; then
    echo -e "${YELLOW}Warning: You have uncommitted changes${NC}"
    git status --short
    echo ""
    read -p "Continue anyway? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# Check if tag already exists
if git rev-parse "v$VERSION" >/dev/null 2>&1; then
    echo -e "${RED}Error: Tag v$VERSION already exists${NC}"
    exit 1
fi

# Step 1: Update version in Cargo.toml
echo -e "${GREEN}[1/6] Updating version in Cargo.toml...${NC}"
if [[ "$(uname)" == "Darwin" ]]; then
    sed -i '' "s/^version = \".*\"/version = \"$VERSION\"/" "$CARGO_TOML"
else
    sed -i "s/^version = \".*\"/version = \"$VERSION\"/" "$CARGO_TOML"
fi
echo "  Updated Cargo.toml to version $VERSION"

# Step 2: Update version in Info.plist
echo -e "${GREEN}[2/6] Updating version in Info.plist...${NC}"
/usr/libexec/PlistBuddy -c "Set :CFBundleShortVersionString $VERSION" "$INFO_PLIST"
/usr/libexec/PlistBuddy -c "Set :CFBundleVersion $VERSION" "$INFO_PLIST"
echo "  Updated Info.plist to version $VERSION"

# Step 3: Build the release
echo -e "${GREEN}[3/6] Building release...${NC}"
"$PROJECT_ROOT/scripts/build.sh"

# Step 4: Create distribution
echo -e "${GREEN}[4/6] Creating distribution...${NC}"
mkdir -p "$DIST_DIR"

# Create zip file
ZIP_NAME="Spacesaver-v$VERSION.zip"
cd "$BUILD_DIR"
zip -r "$DIST_DIR/$ZIP_NAME" Spacesaver.saver
cd "$PROJECT_ROOT"

echo "  Created $DIST_DIR/$ZIP_NAME"

# Calculate checksum
CHECKSUM=$(shasum -a 256 "$DIST_DIR/$ZIP_NAME" | cut -d' ' -f1)
echo "$CHECKSUM  $ZIP_NAME" > "$DIST_DIR/$ZIP_NAME.sha256"
echo "  SHA256: $CHECKSUM"

# Step 5: Commit version changes
echo -e "${GREEN}[5/6] Committing version changes...${NC}"
git add "$CARGO_TOML" "$INFO_PLIST"
git commit -m "Bump version to $VERSION" || echo "  No changes to commit"

# Step 6: Tag release
if $DO_TAG; then
    echo -e "${GREEN}[6/6] Creating git tag...${NC}"
    git tag -a "v$VERSION" -m "Release v$VERSION"
    echo "  Created tag v$VERSION"
else
    echo -e "${YELLOW}[6/6] Skipping git tag (use --tag to create)${NC}"
fi

# Push if requested
if $DO_PUSH; then
    echo -e "${GREEN}Pushing to remote...${NC}"
    git push origin HEAD
    if $DO_TAG; then
        git push origin "v$VERSION"
    fi
    echo "  Pushed to remote"
fi

echo ""
echo -e "${GREEN}=== Release Complete ===${NC}"
echo ""
echo -e "Distribution: ${YELLOW}$DIST_DIR/$ZIP_NAME${NC}"
echo -e "SHA256:       ${YELLOW}$CHECKSUM${NC}"
echo ""
if ! $DO_TAG; then
    echo -e "To tag this release:   ${BLUE}git tag -a v$VERSION -m 'Release v$VERSION'${NC}"
fi
if ! $DO_PUSH; then
    echo -e "To push to remote:     ${BLUE}git push origin HEAD && git push origin v$VERSION${NC}"
fi
echo ""
echo -e "To create a GitHub release:"
echo -e "  ${BLUE}gh release create v$VERSION $DIST_DIR/$ZIP_NAME --title 'Spacesaver v$VERSION' --notes 'Release notes here'${NC}"
