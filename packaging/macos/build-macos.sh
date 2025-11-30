#!/bin/bash
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
