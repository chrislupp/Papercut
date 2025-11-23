#!/bin/bash
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

# Create DMG
create-dmg \
  --volname "Papercut Installer" \
  --volicon "$BUILD_DIR/Papercut.app/Contents/Resources/AppIcon.icns" \
  --window-pos 200 120 \
  --window-size 660 450 \
  --icon-size 100 \
  --icon "Papercut.app" 180 180 \
  --hide-extension "Papercut.app" \
  --icon "Install CLI Tool.app" 480 180 \
  --hide-extension "Install CLI Tool.app" \
  --app-drop-link 180 320 \
  --text-size 12 \
  "$DIST_DIR/$DMG_NAME" \
  "$BUILD_DIR/" \
  2>/dev/null || true

# Note: create-dmg returns non-zero even on success sometimes, so we ignore errors
# and check if the DMG was created instead

if [ -f "$DIST_DIR/$DMG_NAME" ]; then
    echo ""
    echo "========================================"
    echo "✅ DMG created successfully!"
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
    echo "To distribute:"
    echo "  1. Test installation on a clean macOS system"
    echo "  2. (Optional) Code sign and notarize for distribution"
    echo "  3. Upload to your releases page"
    echo ""
else
    echo "❌ Error: DMG creation failed!"
    exit 1
fi
