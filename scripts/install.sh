#!/bin/bash
#
# Install script for Spacesaver macOS screen saver
#

set -e

PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BUILD_DIR="$PROJECT_ROOT/build"
BUNDLE_NAME="Spacesaver.saver"
USER_INSTALL_DIR="$HOME/Library/Screen Savers"
SYSTEM_INSTALL_DIR="/Library/Screen Savers"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${GREEN}=== Spacesaver Installer ===${NC}"
echo ""

# Check if bundle exists
if [[ ! -d "$BUILD_DIR/$BUNDLE_NAME" ]]; then
    echo -e "${RED}Error: Screen saver bundle not found.${NC}"
    echo "Please run 'make build' first."
    exit 1
fi

# Ask for installation type
echo "Where would you like to install Spacesaver?"
echo "  1) Current user only ($USER_INSTALL_DIR)"
echo "  2) All users ($SYSTEM_INSTALL_DIR) [requires sudo]"
echo ""
read -p "Choice [1]: " choice
choice=${choice:-1}

case $choice in
    1)
        INSTALL_DIR="$USER_INSTALL_DIR"
        ;;
    2)
        INSTALL_DIR="$SYSTEM_INSTALL_DIR"
        ;;
    *)
        echo -e "${RED}Invalid choice${NC}"
        exit 1
        ;;
esac

# Create install directory if needed
if [[ ! -d "$INSTALL_DIR" ]]; then
    if [[ "$choice" == "2" ]]; then
        sudo mkdir -p "$INSTALL_DIR"
    else
        mkdir -p "$INSTALL_DIR"
    fi
fi

# Remove old installation if exists
if [[ -d "$INSTALL_DIR/$BUNDLE_NAME" ]]; then
    echo -e "${YELLOW}Removing existing installation...${NC}"
    if [[ "$choice" == "2" ]]; then
        sudo rm -rf "$INSTALL_DIR/$BUNDLE_NAME"
    else
        rm -rf "$INSTALL_DIR/$BUNDLE_NAME"
    fi
fi

# Install
echo -e "${GREEN}Installing to $INSTALL_DIR...${NC}"
if [[ "$choice" == "2" ]]; then
    sudo cp -r "$BUILD_DIR/$BUNDLE_NAME" "$INSTALL_DIR/"
else
    cp -r "$BUILD_DIR/$BUNDLE_NAME" "$INSTALL_DIR/"
fi

echo ""
echo -e "${GREEN}Installation complete!${NC}"
echo ""
echo "To select Spacesaver as your screen saver:"
echo "  1. Open System Preferences (or System Settings on macOS Ventura+)"
echo "  2. Go to 'Screen Saver' (or 'Lock Screen' > 'Screen Saver')"
echo "  3. Select 'Spacesaver' from the list"
echo ""
echo "The first time Spacesaver runs, it will download NASA images."
echo "You can configure settings by clicking 'Screen Saver Options...'."
echo ""
echo -e "${YELLOW}Tip: Get a free NASA API key at https://api.nasa.gov for higher rate limits.${NC}"
