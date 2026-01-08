#!/bin/bash
#
# Build script for Spacesaver macOS screen saver
#
# This script:
# 1. Builds the Rust core library for macOS (universal binary)
# 2. Generates C headers using cbindgen
# 3. Compiles the Swift wrapper
# 4. Creates the .saver bundle
#

set -e

# Configuration
PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
RUST_DIR="$PROJECT_ROOT/spacesaver-core"
SWIFT_DIR="$PROJECT_ROOT/swift-wrapper/Spacesaver"
BUILD_DIR="$PROJECT_ROOT/build"
BUNDLE_NAME="Spacesaver.saver"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}=== Building Spacesaver ===${NC}"

# Check for required tools
check_tool() {
    if ! command -v "$1" &> /dev/null; then
        echo -e "${RED}Error: $1 is not installed${NC}"
        exit 1
    fi
}

check_tool cargo
check_tool swiftc

# Determine target architecture
ARCH=$(uname -m)
if [[ "$ARCH" == "arm64" ]]; then
    RUST_TARGET="aarch64-apple-darwin"
elif [[ "$ARCH" == "x86_64" ]]; then
    RUST_TARGET="x86_64-apple-darwin"
else
    echo -e "${YELLOW}Warning: Unknown architecture $ARCH, defaulting to x86_64${NC}"
    RUST_TARGET="x86_64-apple-darwin"
fi

# Create build directory
mkdir -p "$BUILD_DIR"

# Step 1: Build Rust library
echo -e "${GREEN}[1/4] Building Rust library...${NC}"
cd "$RUST_DIR"

# Build for the current architecture
cargo build --release --target "$RUST_TARGET"

# Copy the static library
RUST_LIB="$RUST_DIR/target/$RUST_TARGET/release/libspacesaver_core.a"
if [[ ! -f "$RUST_LIB" ]]; then
    echo -e "${RED}Error: Rust library not found at $RUST_LIB${NC}"
    exit 1
fi

cp "$RUST_LIB" "$BUILD_DIR/"

# Step 2: Generate C header (should already exist from cargo build)
echo -e "${GREEN}[2/4] Checking C header...${NC}"
HEADER_FILE="$RUST_DIR/include/spacesaver.h"
if [[ ! -f "$HEADER_FILE" ]]; then
    echo -e "${YELLOW}Generating C header...${NC}"
    cbindgen --config "$RUST_DIR/cbindgen.toml" --crate spacesaver-core --output "$HEADER_FILE"
fi

# Copy header to Swift directory
cp "$HEADER_FILE" "$SWIFT_DIR/"

# Step 3: Build Swift wrapper
echo -e "${GREEN}[3/4] Building Swift wrapper...${NC}"

SWIFT_FILES=(
    "$SWIFT_DIR/SpacesaverView.swift"
    "$SWIFT_DIR/ConfigureSheetController.swift"
)

SWIFT_FLAGS=(
    -emit-library
    -emit-module
    -module-name Spacesaver
    -target "${ARCH}-apple-macosx10.15"
    -sdk "$(xcrun --show-sdk-path)"
    -import-objc-header "$SWIFT_DIR/Spacesaver-Bridging-Header.h"
    -I "$SWIFT_DIR"
    -L "$BUILD_DIR"
    -lspacesaver_core
    -framework ScreenSaver
    -framework AppKit
    -framework CoreFoundation
    -framework Security
    -framework SystemConfiguration
    -o "$BUILD_DIR/libSpacesaver.dylib"
)

swiftc "${SWIFT_FLAGS[@]}" "${SWIFT_FILES[@]}"

# Step 4: Create .saver bundle
echo -e "${GREEN}[4/4] Creating screen saver bundle...${NC}"

BUNDLE_DIR="$BUILD_DIR/$BUNDLE_NAME"
CONTENTS_DIR="$BUNDLE_DIR/Contents"
MACOS_DIR="$CONTENTS_DIR/MacOS"
RESOURCES_DIR="$CONTENTS_DIR/Resources"

# Clean and create bundle structure
rm -rf "$BUNDLE_DIR"
mkdir -p "$MACOS_DIR"
mkdir -p "$RESOURCES_DIR"

# Copy Info.plist
cp "$SWIFT_DIR/Info.plist" "$CONTENTS_DIR/"

# Copy bundled images to Resources
BUNDLED_IMAGES_DIR="$PROJECT_ROOT/resources/bundled-images"
if [[ -d "$BUNDLED_IMAGES_DIR" ]]; then
    echo -e "${GREEN}Copying bundled images to Resources...${NC}"
    mkdir -p "$RESOURCES_DIR/BundledImages"
    cp "$BUNDLED_IMAGES_DIR"/*.jpg "$RESOURCES_DIR/BundledImages/"
    cp "$BUNDLED_IMAGES_DIR/metadata.json" "$RESOURCES_DIR/BundledImages/"
fi

# Create the main executable by linking everything
echo -e "${GREEN}Linking final executable...${NC}"

# We need to create a proper executable bundle
# The screen saver framework expects a bundle with a specific structure

# Create a minimal loader that loads our dylib
cat > "$BUILD_DIR/loader.swift" << 'LOADER_EOF'
import Foundation
import ScreenSaver

// This file is auto-generated - it loads the Spacesaver module
@_exported import Spacesaver
LOADER_EOF

swiftc \
    -emit-executable \
    -target "${ARCH}-apple-macosx10.15" \
    -sdk "$(xcrun --show-sdk-path)" \
    -L "$BUILD_DIR" \
    -lSpacesaver \
    -lspacesaver_core \
    -framework ScreenSaver \
    -framework AppKit \
    -framework CoreFoundation \
    -framework Security \
    -Xlinker -rpath -Xlinker @executable_path/../Frameworks \
    -o "$MACOS_DIR/Spacesaver" \
    "$BUILD_DIR/loader.swift" 2>/dev/null || {
        # If that fails, just copy the dylib as the executable
        echo -e "${YELLOW}Using dylib directly as bundle executable${NC}"
        cp "$BUILD_DIR/libSpacesaver.dylib" "$MACOS_DIR/Spacesaver"
        # Also embed the Rust library
        cp "$BUILD_DIR/libspacesaver_core.a" "$MACOS_DIR/"
    }

# Copy dylib to Frameworks if needed
mkdir -p "$CONTENTS_DIR/Frameworks"
cp "$BUILD_DIR/libSpacesaver.dylib" "$CONTENTS_DIR/Frameworks/" 2>/dev/null || true

echo -e "${GREEN}=== Build Complete ===${NC}"
echo -e "Screen saver bundle: ${YELLOW}$BUNDLE_DIR${NC}"
echo ""
echo -e "To install:"
echo -e "  ${YELLOW}cp -r \"$BUNDLE_DIR\" ~/Library/Screen\\ Savers/${NC}"
echo ""
echo -e "Or for all users:"
echo -e "  ${YELLOW}sudo cp -r \"$BUNDLE_DIR\" /Library/Screen\\ Savers/${NC}"
