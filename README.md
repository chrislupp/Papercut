# Papercut

Convert source code files to PDF with configurable headers, footers, and formatting.

## Quick Start

### Installation

#### macOS (DMG Installer)

1. Download the latest `Papercut-X.X.X.dmg` from the [releases page](https://github.com/chrislupp/papercut/releases)
2. Open the DMG and drag `Papercut.app` to your Applications folder
3. (Optional) Add to PATH for terminal access:
   ```bash
   sudo ln -sf /Applications/Papercut.app/Contents/MacOS/papercut /usr/local/bin/papercut
   ```
4. Run from terminal:
   ```bash
   papercut --config config.yaml
   ```

To uninstall: Delete Papercut.app and run `sudo rm /usr/local/bin/papercut`

#### Build from Source

```bash
git clone https://github.com/yourusername/papercut.git
cd papercut
cargo install --path .
```

### Basic Usage

1. Create a configuration file `config.yaml`:

```yaml
output:
  mode: single
  directory: ./output
  filename: code.pdf

files:
  - path: src/main.rs
  - path: src/lib.rs
```

2. Generate PDF:

```bash
papercut --config config.yaml
```

### Features

- **Cover Pages**: Optional cover page with title, description, location, date, and table of contents
- **Directory Scanning & Glob Patterns**: Use wildcards (`*.rs`, `src/**/*.py`) and scan entire directories
- **File Filtering**: Include/exclude specific file types and patterns
- **Single or Multiple PDFs**: Combine all files or create separate PDFs
- **Syntax Highlighting**: Optional syntax highlighting with multiple themes
- **Customizable Headers/Footers**: Add page numbers, dates, filenames, and custom text
- **Flexible Formatting**: Configure page size, margins, fonts, and colors
- **PDF Metadata**: Set title, author, subject, and keywords
- **Robust Error Handling**: Detailed error messages with actionable suggestions
- **Configurable Warnings**: Control warning output by category
- **Progress Indicators**: Visual progress bars for multi-file processing
- **Non-Interactive Mode**: Safe operation in CI/CD environments with `--force` flag

### Example Configurations

See the `examples/` directory:

- `minimal_config.yaml` - Bare minimum configuration
- `full_config.yaml` - All available options with comments
- `release_config.yaml` - Public release template
- `patterns_config.yaml` - Directory scanning and pattern matching examples

### Documentation

Complete documentation is available in the `docs/` directory:

- [Configuration Reference](docs/configuration.md) - All configuration options
- [Usage Guide](docs/usage.md) - CLI usage and workflows
- [Styling Guide](docs/styling.md) - Advanced styling options
- [Examples](docs/examples.md) - Real-world use cases

### CLI Flags

```bash
# Basic usage
papercut --config config.yaml

# Verbose output (shows detailed processing information)
papercut --config config.yaml --verbose

# Suppress all warnings
papercut --config config.yaml --quiet

# Non-interactive mode (skip prompts, requires --force to overwrite files)
papercut --config config.yaml --force

# List available syntax highlighting themes
papercut --list-themes

# Show help
papercut --help
```

### Warning Configuration

Papercut provides detailed warnings for non-critical issues. You can control warnings in your `config.yaml`:

```yaml
warnings:
  # Enable or disable all warnings (default: true)
  enabled: true

  # Selectively silence specific warning categories
  silence_categories:
    - fonts        # Font loading issues
    - themes       # Theme loading issues
    - highlighting # Syntax highlighting failures
    - filesystem   # File access and permission issues
```

**Warning Categories:**
- `fonts`: Font file read failures and fallback behavior
- `themes`: Custom theme loading and parsing errors
- `highlighting`: Syntax highlighting failures (falls back to plain text)
- `filesystem`: Directory walking errors, permission denied, non-UTF-8 paths

**CLI Override:**
Use `--quiet` to suppress all warnings regardless of config settings.

## Building for Distribution

### macOS DMG Installer

To build a macOS DMG installer:

```bash
# 1. Build the app bundle
cd packaging/macos
./build-macos.sh

# 2. Create the DMG (requires create-dmg)
brew install create-dmg
./create-dmg.sh
```

The DMG will be created in the `dist/` directory.

**What's included:**
- `Papercut.app` - Application bundle containing the CLI tool

**Optional: Code Signing and Notarization**

For distribution outside the App Store, you should sign and notarize the app:

```bash
# Sign the app
codesign --force --options=runtime \
  --sign "Developer ID Application: Your Name (TEAM_ID)" \
  --timestamp packaging/macos/build/Papercut.app

# Sign the DMG
codesign --sign "Developer ID Application: Your Name" \
  dist/Papercut-X.X.X.dmg

# Notarize (requires Apple Developer account)
xcrun notarytool submit dist/Papercut-X.X.X.dmg \
  --apple-id "your@email.com" \
  --team-id "TEAM_ID" \
  --password "app-specific-password" \
  --wait

# Staple the notarization ticket
xcrun stapler staple dist/Papercut-X.X.X.dmg
```

## License

This project is licensed under the GNU General Public License v3.0 - see the [LICENSE](LICENSE) file for details.
