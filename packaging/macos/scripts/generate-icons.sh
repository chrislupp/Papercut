#!/bin/bash
set -e

# Generate macOS app icon (.icns) from SVG logo
# Usage: ./generate-icons.sh
#
# Requirements:
#   - rsvg-convert (install via: brew install librsvg)
#   - iconutil (built-in on macOS)

echo "Generating macOS app icon..."

# Get directories
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
PACKAGING_DIR="$PROJECT_ROOT/packaging/macos"
ASSETS_DIR="$PROJECT_ROOT/assets/logos"
OUTPUT_DIR="$PACKAGING_DIR/build"

# Source SVG
SVG_FILE="$ASSETS_DIR/papercut_logo.svg"

if [ ! -f "$SVG_FILE" ]; then
    echo "Error: SVG logo not found at $SVG_FILE"
    exit 1
fi

# Check for rsvg-convert
if ! command -v rsvg-convert &> /dev/null; then
    echo "Error: rsvg-convert not found!"
    echo ""
    echo "Please install librsvg using Homebrew:"
    echo "  brew install librsvg"
    echo ""
    exit 1
fi

# Create temporary iconset directory
ICONSET_DIR="$OUTPUT_DIR/AppIcon.iconset"
mkdir -p "$ICONSET_DIR"

echo "Converting SVG to PNG at various sizes..."

# Generate PNG files at required sizes for macOS icons
# macOS requires these specific sizes for .icns
SIZES=(16 32 128 256 512)

for SIZE in "${SIZES[@]}"; do
    echo "  Generating ${SIZE}x${SIZE}..."
    rsvg-convert -w "$SIZE" -h "$SIZE" "$SVG_FILE" -o "$ICONSET_DIR/icon_${SIZE}x${SIZE}.png"

    # Also generate @2x versions (except for 512 which becomes 1024)
    DOUBLE=$((SIZE * 2))
    echo "  Generating ${SIZE}x${SIZE}@2x (${DOUBLE}x${DOUBLE})..."
    rsvg-convert -w "$DOUBLE" -h "$DOUBLE" "$SVG_FILE" -o "$ICONSET_DIR/icon_${SIZE}x${SIZE}@2x.png"
done

echo "Creating .icns file..."

# Use iconutil to create the .icns file
iconutil -c icns "$ICONSET_DIR" -o "$OUTPUT_DIR/AppIcon.icns"

# Clean up iconset directory
rm -rf "$ICONSET_DIR"

echo "Icon generated: $OUTPUT_DIR/AppIcon.icns"
