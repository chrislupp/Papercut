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

### Automatic (CI)

The GitHub Actions release workflow automatically signs and notarizes the DMG when these repository secrets are configured:

| Secret | Description |
|--------|-------------|
| `APPLE_CERTIFICATE_BASE64` | Base64-encoded .p12 certificate |
| `APPLE_CERTIFICATE_PASSWORD` | Password for the .p12 file |
| `APPLE_TEAM_ID` | Apple Developer Team ID (e.g., "ABC123XYZ") |
| `APPLE_ID` | Apple ID email for notarization |
| `APPLE_ID_PASSWORD` | App-specific password for notarization |

**Setup steps:**

1. Export your Developer ID Application certificate from Keychain Access as .p12
2. Base64 encode it: `base64 -i certificate.p12 | pbcopy`
3. Create an app-specific password at https://appleid.apple.com/account/manage
4. Add all secrets to GitHub (Settings → Secrets → Actions)

Without these secrets, the workflow falls back to ad-hoc signing.

### Manual (Local)

For local distribution, sign and notarize manually:

```bash
# Set your signing identity
export APPLE_SIGNING_IDENTITY="Developer ID Application: Name (TEAM_ID)"

# Build and sign (scripts detect the env var)
./build-macos.sh
./create-dmg.sh

# Notarize
xcrun notarytool submit ../../dist/Papercut-X.X.X.dmg \
  --apple-id "email" --team-id "TEAM_ID" --password "app-specific-password" --wait

# Staple
xcrun stapler staple ../../dist/Papercut-X.X.X.dmg
```

### Verification

```bash
# Check app signature
codesign -dv --verbose=4 /Applications/Papercut.app

# Verify notarization
spctl -a -v /Applications/Papercut.app
# Output: "Papercut.app: accepted source=Notarized Developer ID"
```

## Troubleshooting

| Issue | Solution |
|-------|----------|
| `create-dmg: command not found` | `brew install create-dmg` |
| `rsvg-convert not found` | `brew install librsvg` |
| Security warning on unsigned app | Right-click → Open, or sign the app |
