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

# Build macOS application bundle and DMG installer for Papercut
# Usage: ./build-macos.sh

echo "========================================"
echo "Building Papercut for macOS"
echo "========================================"
echo ""

# Get the project root directory
PROJECT_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
PACKAGING_DIR="$PROJECT_ROOT/packaging/macos"
BUILD_DIR="$PACKAGING_DIR/build"
ASSETS_DIR="$PROJECT_ROOT/assets/logos"

# Clean previous build
echo "🧹 Cleaning previous build artifacts..."
rm -rf "$BUILD_DIR"
mkdir -p "$BUILD_DIR"

# Get version from Cargo.toml
VERSION=$(grep '^version' "$PROJECT_ROOT/Cargo.toml" | head -1 | sed 's/.*"\(.*\)".*/\1/')
echo "📦 Version: $VERSION"
echo ""

# Build the Rust binary
echo "🔨 Building Rust binary (release mode)..."
cd "$PROJECT_ROOT"
cargo build --release
echo "✓ Binary built successfully"
echo ""

# Create app bundle structure
echo "📁 Creating app bundle structure..."
APP_BUNDLE="$BUILD_DIR/Papercut.app"
mkdir -p "$APP_BUNDLE/Contents/"{MacOS,Resources}
echo "✓ Bundle structure created"
echo ""

# Copy binary
echo "📋 Copying binary to app bundle..."
cp "$PROJECT_ROOT/target/release/papercut" "$APP_BUNDLE/Contents/MacOS/"
chmod +x "$APP_BUNDLE/Contents/MacOS/papercut"
echo "✓ Binary copied"
echo ""

# Generate app icon from SVG
echo "🎨 Generating app icon..."
SVG_FILE="$ASSETS_DIR/papercut_logo.svg"
ICONSET_DIR="$BUILD_DIR/AppIcon.iconset"
ICNS_FILE="$BUILD_DIR/AppIcon.icns"

if [ ! -f "$SVG_FILE" ]; then
    echo "⚠️  Warning: SVG logo not found at $SVG_FILE"
    echo "   Skipping icon generation"
else
    # Check for rsvg-convert
    if ! command -v rsvg-convert &> /dev/null; then
        echo "⚠️  Warning: rsvg-convert not found"
        echo "   Install with: brew install librsvg"
        echo "   Skipping icon generation"
    else
        mkdir -p "$ICONSET_DIR"

        # Generate PNG files at required sizes for macOS icons
        for SIZE in 16 32 128 256 512; do
            rsvg-convert -w "$SIZE" -h "$SIZE" "$SVG_FILE" -o "$ICONSET_DIR/icon_${SIZE}x${SIZE}.png"
            DOUBLE=$((SIZE * 2))
            rsvg-convert -w "$DOUBLE" -h "$DOUBLE" "$SVG_FILE" -o "$ICONSET_DIR/icon_${SIZE}x${SIZE}@2x.png"
        done

        # Create .icns file
        iconutil -c icns "$ICONSET_DIR" -o "$ICNS_FILE"
        rm -rf "$ICONSET_DIR"

        # Copy icon to app bundle Resources
        cp "$ICNS_FILE" "$APP_BUNDLE/Contents/Resources/AppIcon.icns"
        echo "✓ App icon generated and copied"
    fi
fi
echo ""

# Generate Info.plist from template
echo "📝 Generating Info.plist..."
sed "s/{{VERSION}}/$VERSION/g" "$PACKAGING_DIR/Info.plist.template" > "$APP_BUNDLE/Contents/Info.plist"
echo "✓ Info.plist created"
echo ""

# Sign the app bundle
if [ -n "$APPLE_SIGNING_IDENTITY" ]; then
    echo "🔏 Signing app bundle with Developer ID..."
    codesign --force --options=runtime --deep \
        --sign "$APPLE_SIGNING_IDENTITY" \
        --timestamp \
        "$APP_BUNDLE"
    echo "✓ App bundle signed with Developer ID"
else
    echo "🔏 Signing app bundle (ad-hoc)..."
    codesign --force --deep --sign - "$APP_BUNDLE"
    echo "✓ App bundle signed (ad-hoc)"
fi
echo ""

echo "========================================"
echo "✅ Build completed successfully!"
echo "========================================"
echo ""
echo "Output: $APP_BUNDLE"
echo ""
echo "Next steps:"
echo "  1. Run ./create-dmg.sh to create a DMG installer"
echo "  2. Or test directly: open '$APP_BUNDLE'"
echo ""
