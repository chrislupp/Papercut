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

# Generate Info.plist from template
echo "📝 Generating Info.plist..."
sed "s/{{VERSION}}/$VERSION/g" "$PACKAGING_DIR/Info.plist.template" > "$APP_BUNDLE/Contents/Info.plist"
echo "✓ Info.plist created"
echo ""

# Create CLI installer helper app
echo "🛠️  Creating CLI installer helper..."
INSTALLER_APP="$BUILD_DIR/Install CLI Tool.app"
mkdir -p "$INSTALLER_APP/Contents/MacOS"

# Compile AppleScript to app
osacompile -o "$INSTALLER_APP" "$PACKAGING_DIR/scripts/install-cli-tool.applescript"
echo "✓ CLI installer created"
echo ""

# Copy README for DMG
if [ -f "$PACKAGING_DIR/dmg-readme.txt" ]; then
    echo "📄 Copying DMG readme..."
    cp "$PACKAGING_DIR/dmg-readme.txt" "$BUILD_DIR/README.txt"
    echo "✓ README copied"
    echo ""
fi

echo "========================================"
echo "✅ Build completed successfully!"
echo "========================================"
echo ""
echo "Output directory: $BUILD_DIR"
echo ""
echo "Contents:"
echo "  • Papercut.app - Main application bundle"
echo "  • Install CLI Tool.app - Optional CLI installer"
if [ -f "$BUILD_DIR/README.txt" ]; then
    echo "  • README.txt - Installation instructions"
fi
echo ""
echo "Next steps:"
echo "  1. Run ./create-dmg.sh to create a DMG installer"
echo "  2. Or test the app directly: open '$APP_BUNDLE'"
echo ""
