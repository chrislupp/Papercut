# Usage Guide

This guide covers how to use Papercut from the command line.

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/chrislupp/Papercut.git
cd Papercut

# Build and install
cargo install --path .
```

### Build Options

Papercut supports optional features:

```bash
# Build with all features (default)
cargo build --release

# Build without syntax highlighting
cargo build --release --no-default-features
```

## Basic Usage

The simplest way to use Papercut is to provide a configuration file:

Relative paths in the configuration are resolved from the configuration file's directory, not
from the shell's current working directory.

```bash
papercut --config config.yaml
```

This will:
1. Read the configuration from `config.yaml`
2. Process all files specified in the configuration
3. Generate PDF(s) according to the output mode
4. Save the PDF(s) to the specified output directory

## Command-Line Options

### `--config` / `-c`

Specify the path to the YAML configuration file.

```bash
papercut --config my-config.yaml
papercut -c /path/to/config.yaml
```

**Required**: Yes

### `--verbose` / `-v`

Enable verbose output to see detailed progress information.

```bash
papercut --config config.yaml --verbose
papercut -c config.yaml -v
```

**Required**: No

Example output with verbose mode:

```
Loading configuration from: config.yaml
Configuration loaded successfully
Output mode: Single
Files to process: 3
Output directory: ./output
Generating single PDF with 3 files
  Processing file 1/3: src/main.rs
  Processing file 2/3: src/lib.rs
  Processing file 3/3: README.md
Rendering PDF to: ./output/combined.pdf
✓ Generated: ./output/combined.pdf
PDF generation completed successfully!
```

### `--force` / `-f`

Force overwrite existing files without prompting.

```bash
papercut --config config.yaml --force
papercut -c config.yaml -f
```

**Required**: No

Use this flag in non-interactive environments (CI/CD pipelines, scripts) where prompts cannot be answered.

### `--quiet` / `-q`

Suppress all warning messages.

```bash
papercut --config config.yaml --quiet
papercut -c config.yaml -q
```

**Required**: No

Overrides the `warnings.enabled` setting in the configuration file. Useful for clean output in automated pipelines.

### `--list-themes`

List all available syntax highlighting themes (requires `syntax-highlighting` feature).

```bash
papercut --list-themes
```

Example output:

```
Available syntax highlighting themes:

Built-in presets:
  - vscode-light (default)
  - vscode-dark
  - jetbrains-light
  - jetbrains-darcula

Additional themes:
  - InspiredGitHub
  - Solarized (dark)
  - Solarized (light)
  - base16-ocean.dark
  - base16-ocean.light
  ...

Use these theme names in your configuration file.
```

### `--help` / `-h`

Display help information.

```bash
papercut --help
```

### `--version` / `-V`

Display version information.

```bash
papercut --version
```

## Common Workflows

### Quick Start: Single File

Create a minimal configuration file `quick.yaml`:

```yaml
output:
  mode: single
  directory: ./output
  filename: code.pdf

files:
  - path: main.rs
```

Generate the PDF:

```bash
papercut -c quick.yaml
```

### Converting Multiple Files to Separate PDFs

Configuration file `multi.yaml`:

```yaml
output:
  mode: multiple
  directory: ./pdfs

files:
  - path: src/main.rs
  - path: src/lib.rs
  - path: src/utils.rs
```

Generate PDFs:

```bash
papercut -c multi.yaml -v
```

This creates:
- `./pdfs/main.pdf`
- `./pdfs/lib.pdf`
- `./pdfs/utils.pdf`

### Pattern Matching and Directory Scanning

**NEW**: Papercut supports glob patterns, directory scanning, and file filtering.

#### Scan an entire directory

Configuration file `scan_dir.yaml`:

```yaml
output:
  mode: single
  directory: ./output
  filename: all_code.pdf

files:
  # Scan src/ directory for all Rust files
  - path: src/
    include_types: ["rs"]
    exclude: ["*.test.rs", "target/**"]
```

Generate:

```bash
papercut -c scan_dir.yaml -v
```

This will:
1. Scan the `src/` directory recursively
2. Include only `.rs` files
3. Exclude test files and target directory

#### Use glob patterns

Configuration file `patterns.yaml`:

```yaml
output:
  mode: single

files:
  # All Rust files in src/ and subdirectories
  - path: "src/**/*.rs"

  # All YAML files in examples/
  - path: "examples/*.yaml"

  # All markdown files in root
  - path: "*.md"
```

Generate:

```bash
papercut -c patterns.yaml
```

#### Complex filtering

Configuration file `filtered.yaml`:

```yaml
output:
  mode: single

files:
  # Source code (Rust and TOML only, no tests)
  - path: src/
    include_types: ["rs", "toml"]
    exclude:
      - "*.test.rs"
      - "**/tests/**"
      - "target/**"
    title: "Source Code"

  # Documentation
  - path: "docs/**/*.md"
    title: "Documentation"

  # Examples (only Rust files)
  - path: examples/
    include_types: ["rs"]
    title: "Examples"
```

Generate:

```bash
papercut -c filtered.yaml -v
```

### Release Documentation with Headers/Footers

Configuration file `release.yaml`:

```yaml
output:
  mode: single
  directory: ./releases
  filename: source_release_2024.pdf

files:
  - path: src/main.rs
  - path: LICENSE
  - path: README.md

header:
  enabled: true
  left: "APPROVED FOR PUBLIC RELEASE"
  right: "Page {page}/{total}"

footer:
  enabled: true
  left: "Distribution Unlimited"
  center: "{filename}"
  right: "{date}"

metadata:
  title: "Software Source Code - Public Release"
  author: "Engineering Division"
```

Generate release PDF:

```bash
papercut -c release.yaml -v
```

### Syntax-Highlighted Code with Custom Theme

Configuration file `highlighted.yaml`:

```yaml
output:
  mode: single
  directory: ./output

files:
  - path: src/main.rs
  - path: src/parser.rs

syntax_highlighting:
  enabled: true
  theme: InspiredGitHub

page:
  size: Letter
  font_size: 9
  line_numbers: true
```

First, check available themes:

```bash
papercut --list-themes
```

Then generate:

```bash
papercut -c highlighted.yaml
```

### Legal/Compliance Documents

Configuration for legal compliance `compliance.yaml`:

```yaml
output:
  mode: single
  directory: ./compliance
  filename: source_code_audit.pdf

files:
  - path: src/main.rs
    title: "Main Application Code"
  - path: src/security.rs
    title: "Security Module"
  - path: LICENSE
    title: "Software License"

page:
  size: Legal
  margins:
    top: 2.5
    bottom: 2.5
    left: 3.0
    right: 3.0
  line_numbers: true

header:
  enabled: true
  left: "CONFIDENTIAL - ATTORNEY WORK PRODUCT"
  right: "Page {page} of {total}"
  font_size: 8

footer:
  enabled: true
  left: "Generated: {date}"
  center: "Source Code Audit"
  right: "Case #2024-001"
  font_size: 8

metadata:
  title: "Source Code Audit - Case #2024-001"
  author: "Legal Department"
  subject: "Compliance Review"
  keywords:
    - audit
    - compliance
    - confidential
```

Generate:

```bash
papercut -c compliance.yaml -v
```

## Environment Variables

Currently, Papercut does not use environment variables. All configuration is done through the YAML configuration file and command-line options.

## Exit Codes

- `0`: Success
- `1`: Error (configuration error, file not found, PDF generation failed, etc.)

## Troubleshooting

### File Not Found

If you see an error like:

```
Error: File not found: src/main.rs
```

Check that:
1. The file path in the configuration is correct
2. The file exists
3. You're running papercut from the correct directory

### Invalid Configuration

If you see:

```
Error: Invalid configuration: No files specified in configuration
```

Ensure your configuration file has at least one file in the `files` list.

### Theme Not Found

If you see:

```
Error: Syntax highlighting error: Theme 'xyz' not found
```

Run `papercut --list-themes` to see available themes and use one of those names.

### PDF Generation Failed

If PDF generation fails, try running with verbose mode to see more details:

```bash
papercut -c config.yaml -v
```

## See Also

- [Configuration Reference](configuration.md) - Complete configuration options
- [Styling Guide](styling.md) - Advanced styling options
- [Examples](examples.md) - Real-world examples
