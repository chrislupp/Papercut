# macOS Packaging for Papercut

This directory contains scripts and resources for creating a macOS DMG installer for Papercut.

## Quick Start

```bash
# 1. Build the app bundle
./build-macos.sh

# 2. Install create-dmg (if not already installed)
brew install create-dmg

# 3. Create the DMG
./create-dmg.sh
```

The DMG will be created in `../../dist/Papercut-X.X.X.dmg`

## Directory Structure

```
packaging/macos/
├── README.md                           # This file
├── Info.plist.template                 # macOS app bundle metadata template
├── dmg-readme.txt                      # Instructions included in DMG
├── build-macos.sh                      # Main build script
├── create-dmg.sh                       # DMG creation script
├── scripts/
│   └── install-cli-tool.applescript   # CLI installer helper source
├── assets/                             # Optional icons and backgrounds
├── build/                              # Build output (gitignored)
│   ├── Papercut.app
│   ├── Install CLI Tool.app
│   └── README.txt
└── dmg-contents/                       # Additional DMG contents
```

## What Gets Built

### Papercut.app
A macOS application bundle containing:
- `Contents/MacOS/papercut` - The CLI binary
- `Contents/Info.plist` - Bundle metadata
- `Contents/Resources/` - (Future: icons, resources)

### Install CLI Tool.app
A helper application that:
- Requests administrator privileges
- Creates a symlink: `/usr/local/bin/papercut → /Applications/Papercut.app/Contents/MacOS/papercut`
- Shows success/failure dialog

### DMG Installer
A disk image containing:
- Papercut.app
- Install CLI Tool.app
- Applications folder shortcut (drag-to-install)
- README.txt with instructions

## Scripts

### build-macos.sh
Builds the complete app bundle:
1. Runs `cargo build --release`
2. Creates app bundle structure
3. Copies binary to bundle
4. Generates Info.plist with version from Cargo.toml
5. Compiles AppleScript installer to .app

**Options:**
- Automatically extracts version from Cargo.toml
- Creates clean build directory each run

### create-dmg.sh
Creates the DMG installer:
1. Validates build artifacts exist
2. Checks for create-dmg tool
3. Creates DMG with custom layout
4. Names DMG with version number

**Requirements:**
- create-dmg tool (install via Homebrew)
- Completed build from build-macos.sh

## Customization

### Adding an App Icon

1. Create an icon file in ICNS format (512x512, 256x256, 128x128, etc.)
2. Save as `assets/AppIcon.icns`
3. Update `build-macos.sh` to copy it:
   ```bash
   cp "$PACKAGING_DIR/assets/AppIcon.icns" "$APP_BUNDLE/Contents/Resources/"
   ```

### Custom DMG Background

1. Create a background image (PNG, 660x450 recommended)
2. Save as `assets/dmg-background.png`
3. Update `create-dmg.sh`:
   ```bash
   --background "$PACKAGING_DIR/assets/dmg-background.png" \
   ```

### Changing Bundle Identifier

Edit `Info.plist.template` and change:
```xml
<key>CFBundleIdentifier</key>
<string>com.yourcompany.papercut</string>
```

## Code Signing & Notarization

For public distribution, you should sign and notarize the app:

```bash
# Sign the app bundle
codesign --force --options=runtime \
  --sign "Developer ID Application: Your Name (TEAM_ID)" \
  --timestamp build/Papercut.app

# Sign the DMG
codesign --sign "Developer ID Application: Your Name" \
  ../../dist/Papercut-0.1.0.dmg

# Notarize (requires Apple Developer account)
xcrun notarytool submit ../../dist/Papercut-0.1.0.dmg \
  --apple-id "your@email.com" \
  --team-id "TEAM_ID" \
  --password "app-specific-password" \
  --wait

# Staple the notarization ticket
xcrun stapler staple ../../dist/Papercut-0.1.0.dmg
```

**Requirements:**
- Apple Developer account ($99/year)
- Developer ID Application certificate
- App-specific password for notarization

## Testing

### Test the App Bundle
```bash
# Run directly
./build/Papercut.app/Contents/MacOS/papercut --help

# Open in Finder
open build/Papercut.app
```

### Test the DMG
```bash
# Mount the DMG
open ../../dist/Papercut-0.1.0.dmg

# Test installation
# 1. Drag Papercut.app to Applications
# 2. Double-click "Install CLI Tool"
# 3. Open Terminal and run: papercut --help
```

### Verify Installation
```bash
# Check if symlink was created
ls -la /usr/local/bin/papercut

# Should output:
# /usr/local/bin/papercut -> /Applications/Papercut.app/Contents/MacOS/papercut
```

## Troubleshooting

### "create-dmg: command not found"
Install create-dmg:
```bash
brew install create-dmg
```

### AppleScript Compilation Fails
Make sure you have Xcode Command Line Tools:
```bash
xcode-select --install
```

### "Build artifacts not found"
Run `build-macos.sh` before `create-dmg.sh`:
```bash
./build-macos.sh
./create-dmg.sh
```

### DMG Won't Mount on Other Macs
You may need to sign and notarize the DMG. See "Code Signing & Notarization" above.

### CLI Tool Installer Shows Security Warning
This is expected for unsigned apps. Users can:
1. Right-click → Open (instead of double-clicking)
2. Or: System Settings → Privacy & Security → "Open Anyway"

For public distribution, sign and notarize the app.

## Future Enhancements

Potential improvements:
- [ ] Add app icon (ICNS file)
- [ ] Custom DMG background image
- [ ] Automated code signing in build script
- [ ] GitHub Actions workflow for releases
- [ ] Support for other package managers (Homebrew, MacPorts)
- [ ] Universal binary (Intel + Apple Silicon)

## Resources

- [create-dmg documentation](https://github.com/create-dmg/create-dmg)
- [macOS App Bundle Structure](https://developer.apple.com/library/archive/documentation/CoreFoundation/Conceptual/CFBundles/BundleTypes/BundleTypes.html)
- [Code Signing Guide](https://developer.apple.com/support/code-signing/)
- [Notarization Guide](https://developer.apple.com/documentation/security/notarizing_macos_software_before_distribution)
