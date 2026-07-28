#!/bin/bash
#
# Papercut - Source code to PDF converter
# Copyright (C) 2025-2026 Christopher A. Lupp
#
# This program is free software: you can redistribute it and/or modify
# it under the terms of the GNU General Public License as published by
# the Free Software Foundation, either version 3 of the License, or
# (at your option) any later version.
#
# This program is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
# GNU General Public License for more details.
#
# You should have received a copy of the GNU General Public License
# along with this program.  If not, see <https://www.gnu.org/licenses/>.
#
# Distribution A: This work has been cleared for public release,
# distribution unlimited, case number: AFRL-2026-0405. The views expressed
# are those of the authors and do not reflect the official guidance or
# position of the United States Government, the Department of Defense or of
# the United States Air Force.
#
# Statement from DoD: The Appearance of external hyperlinks does not
# constitute endorsement by the United States Department of Defense (DoD) of
# the linked websites, of the information, products, or services contained
# therein. The DoD does not exercise any editorial, security, or other
# control over the information you may find at these locations.

set -e

# Create DMG installer for Papercut
# Usage: ./create-dmg.sh
#
# Requirements:
#   - create-dmg tool (install via: brew install create-dmg)
#   - Build artifacts from build-macos.sh

echo "========================================"
echo "Creating Papercut DMG Installer"
echo "========================================"
echo ""

# Get directories
PROJECT_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
PACKAGING_DIR="$PROJECT_ROOT/packaging/macos"
BUILD_DIR="$PACKAGING_DIR/build"
DIST_DIR="$PROJECT_ROOT/dist"

# Check if build directory exists
if [ ! -d "$BUILD_DIR/Papercut.app" ]; then
    echo "❌ Error: Build artifacts not found!"
    echo "Please run ./build-macos.sh first"
    exit 1
fi

# Check if create-dmg is installed
if ! command -v create-dmg &> /dev/null; then
    echo "❌ Error: create-dmg not found!"
    echo ""
    echo "Please install create-dmg using one of these methods:"
    echo ""
    echo "  Option 1 - Homebrew (recommended):"
    echo "    brew install create-dmg"
    echo ""
    echo "  Option 2 - Clone from GitHub:"
    echo "    git clone https://github.com/create-dmg/create-dmg.git"
    echo "    cd create-dmg"
    echo "    # Add to PATH or use directly"
    echo ""
    exit 1
fi

# Get version
VERSION=$(grep '^version' "$PROJECT_ROOT/Cargo.toml" | head -1 | sed 's/.*"\(.*\)".*/\1/')
DMG_NAME="Papercut-$VERSION.dmg"

# Create dist directory
mkdir -p "$DIST_DIR"

# Remove old DMG if exists
if [ -f "$DIST_DIR/$DMG_NAME" ]; then
    echo "🗑️  Removing old DMG..."
    rm "$DIST_DIR/$DMG_NAME"
fi

echo "📦 Creating DMG: $DMG_NAME"
echo ""

# Create a staging directory with only the app
STAGING_DIR="$BUILD_DIR/dmg-staging"
rm -rf "$STAGING_DIR"
mkdir -p "$STAGING_DIR"
cp -R "$BUILD_DIR/Papercut.app" "$STAGING_DIR/"

# Build create-dmg arguments
DMG_ARGS=(
  --volname "Papercut"
  --window-pos 200 120
  --window-size 500 350
  --icon-size 100
  --icon "Papercut.app" 125 150
  --hide-extension "Papercut.app"
  --app-drop-link 375 150
  --text-size 12
)

# Add volume icon if it exists
if [ -f "$BUILD_DIR/Papercut.app/Contents/Resources/AppIcon.icns" ]; then
  DMG_ARGS+=(--volicon "$BUILD_DIR/Papercut.app/Contents/Resources/AppIcon.icns")
fi

# Create DMG from staging directory
create-dmg "${DMG_ARGS[@]}" "$DIST_DIR/$DMG_NAME" "$STAGING_DIR/" 2>/dev/null || true

# Clean up staging
rm -rf "$STAGING_DIR"

# Note: create-dmg returns non-zero even on success sometimes, so we ignore errors
# and check if the DMG was created instead

if [ -f "$DIST_DIR/$DMG_NAME" ]; then
    echo ""
    echo "✅ DMG created: $DIST_DIR/$DMG_NAME"

    # Sign the DMG if signing identity is available
    if [ -n "$APPLE_SIGNING_IDENTITY" ]; then
        echo ""
        echo "🔏 Signing DMG..."
        codesign --force --sign "$APPLE_SIGNING_IDENTITY" --timestamp "$DIST_DIR/$DMG_NAME"
        echo "✓ DMG signed"
    fi

    echo ""
    echo "========================================"
    echo "✅ DMG packaging completed!"
    echo "========================================"
    echo ""
    echo "Output: $DIST_DIR/$DMG_NAME"

    # Get file size
    SIZE=$(du -h "$DIST_DIR/$DMG_NAME" | cut -f1)
    echo "Size: $SIZE"
    echo ""
    echo "To test the DMG:"
    echo "  open '$DIST_DIR/$DMG_NAME'"
    echo ""
else
    echo "❌ Error: DMG creation failed!"
    exit 1
fi
