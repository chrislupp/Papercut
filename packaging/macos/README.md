# macOS Packaging

Build scripts for creating the Papercut macOS DMG installer.

## Requirements

- Xcode Command Line Tools
- [Homebrew](https://brew.sh)

```bash
brew install librsvg create-dmg
```

## Build

```bash
./build-macos.sh    # Build app bundle
./create-dmg.sh     # Create DMG installer
```

Output: `../../dist/Papercut-X.X.X.dmg`

## Contents

| File | Description |
|------|-------------|
| `build-macos.sh` | Builds Papercut.app bundle with icon |
| `create-dmg.sh` | Creates DMG installer |
| `Info.plist.template` | App bundle metadata |
| `scripts/generate-icons.sh` | Converts SVG logo to .icns |
| `scripts/install-cli-tool.applescript` | CLI symlink installer |

## Code Signing

For distribution, sign and notarize:

```bash
codesign --force --options=runtime \
  --sign "Developer ID Application: Name (TEAM_ID)" \
  --timestamp build/Papercut.app

xcrun notarytool submit ../../dist/Papercut-X.X.X.dmg \
  --apple-id "email" --team-id "TEAM_ID" --password "..." --wait

xcrun stapler staple ../../dist/Papercut-X.X.X.dmg
```

## Troubleshooting

| Issue | Solution |
|-------|----------|
| `create-dmg: command not found` | `brew install create-dmg` |
| `rsvg-convert not found` | `brew install librsvg` |
| Security warning on unsigned app | Right-click → Open, or sign the app |
