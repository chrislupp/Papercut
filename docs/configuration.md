# Configuration Reference

This document provides a complete reference for all configuration options available in Papercut's YAML configuration files.

## Configuration Structure

The configuration file is organized into the following sections:

- `output` - Output settings (mode, directory, filename)
- `files` - List of source files to convert
- `syntax_highlighting` - Syntax highlighting options
- `page` - Page layout and formatting
- `header` - Header configuration
- `footer` - Footer configuration
- `styling` - Visual styling options
- `metadata` - PDF metadata

## Output Configuration

Controls how PDFs are generated and where they are saved.

```yaml
output:
  mode: single              # "single" or "multiple"
  directory: ./output       # Output directory path
  filename: output.pdf      # Filename (for single mode only)
```

### Fields

- **mode** (required): Output mode
  - `single`: Combine all files into one PDF
  - `multiple`: Create one PDF per source file

- **directory** (optional, default: `./output`): Directory where PDFs will be saved
  - Can be relative or absolute path
  - Will be created if it doesn't exist

- **filename** (optional, default: `output.pdf`): Output filename for single mode
  - Only used when `mode: single`
  - For multiple mode, filenames are derived from source files

## Files Configuration

Specify which files to convert to PDF. Supports individual files, glob patterns, and directory scanning with filters.

```yaml
files:
  # Individual file
  - path: src/main.rs
    title: "Main Entry Point"

  # Glob pattern
  - path: "src/**/*.rs"
    include_types: ["rs"]
    exclude: ["*.test.rs"]

  # Directory with filters
  - path: src/
    include_types: ["rs", "toml"]
    exclude: ["target/**", "**/tests/**"]
```

### Fields

- **path** (required): Path, pattern, or directory
  - **Individual file**: `src/main.rs`
  - **Glob pattern**: `src/**/*.rs` (recursive), `src/*.rs` (non-recursive)
  - **Directory**: `src/` (scans recursively)
  - Can be relative or absolute
  - Use quotes for patterns with wildcards

- **title** (optional): Custom title for this file/group in the PDF
  - If not specified, the filename will be used
  - Displayed as a separator in single-file mode
  - When using patterns, applies to all matched files

- **include_types** (optional, default: all types): File extensions to include
  - List of extensions without dots: `["rs", "py", "js"]`
  - Only files with these extensions will be included
  - If empty or omitted, all file types are included
  - Works with directories and glob patterns

- **exclude** (optional, default: none): Exclusion patterns
  - List of glob patterns: `["*.test.rs", "target/**"]`
  - Files matching these patterns will be excluded
  - Useful for excluding test files, build artifacts, etc.
  - Patterns are matched against the full file path

### Pattern Syntax

**Glob Wildcards:**
- `*` - Matches any number of characters (excluding `/`)
- `**` - Matches any number of directories
- `?` - Matches a single character
- `[abc]` - Matches any character in brackets

**Examples:**
- `*.rs` - All Rust files in current directory
- `**/*.py` - All Python files recursively
- `src/**/*.{rs,toml}` - All Rust and TOML files in src/
- `test_*.rs` - Files starting with `test_`

### Examples

**Example 1: All Rust files in a directory (excluding tests)**
```yaml
files:
  - path: src/
    include_types: ["rs"]
    exclude: ["*.test.rs", "**/tests/**"]
```

**Example 2: Specific file patterns**
```yaml
files:
  - path: "src/**/*.rs"
  - path: "examples/*.yaml"
  - path: "*.md"
```

**Example 3: Multiple directories with different filters**
```yaml
files:
  - path: src/
    include_types: ["rs"]
    exclude: ["target/**"]
    title: "Source Code"

  - path: examples/
    include_types: ["rs", "yaml"]
    title: "Examples"
```

## Syntax Highlighting Configuration

Controls syntax highlighting for source code.

```yaml
syntax_highlighting:
  enabled: true
  theme: base16-ocean.dark
```

### Fields

- **enabled** (optional, default: `false`): Enable syntax highlighting
  - Requires the `syntax-highlighting` feature to be enabled

- **theme** (optional, default: `base16-ocean.dark`): Theme name
  - Run `papercut --list-themes` to see available themes
  - Popular themes: `InspiredGitHub`, `Solarized (dark)`, `Solarized (light)`

## Page Configuration

Controls page layout and formatting.

```yaml
page:
  size: A4
  margins:
    top: 2.5
    bottom: 2.5
    left: 2.0
    right: 2.0
  font_size: 10
  line_numbers: true
  line_spacing: 1.2
```

### Fields

- **size** (optional, default: `A4`): Paper size
  - `A4`: 210mm × 297mm (common in Europe, Asia)
  - `Letter`: 8.5" × 11" (common in North America)
  - `Legal`: 8.5" × 14" (legal documents)

- **margins** (optional): Page margins in centimeters
  - **top** (default: `2.5`): Top margin
  - **bottom** (default: `2.5`): Bottom margin
  - **left** (default: `2.0`): Left margin
  - **right** (default: `2.0`): Right margin

- **font_size** (optional, default: `10`): Font size for code content in points
  - Typical values: 8-12

- **line_numbers** (optional, default: `true`): Show line numbers

- **line_spacing** (optional, default: `1.2`): Line spacing multiplier
  - 1.0 = single spacing
  - 1.5 = 1.5× spacing
  - 2.0 = double spacing

## Header/Footer Configuration

Controls headers and footers on each page.

```yaml
header:
  enabled: true
  left: "CONFIDENTIAL"
  center: ""
  right: "Page {page} of {total}"
  font_size: 8

footer:
  enabled: true
  left: "Generated: {date}"
  center: "{filename}"
  right: "© 2024 Company"
  font_size: 8
```

### Fields

- **enabled** (optional, default: `false`): Enable header/footer

- **left** (optional, default: `""`): Text for left alignment

- **center** (optional, default: `""`): Text for center alignment

- **right** (optional, default: `""`): Text for right alignment

- **font_size** (optional, default: `8`): Font size in points

### Variable Substitution

Headers and footers support the following variables:

- `{page}`: Current page number
- `{total}`: Total number of pages
- `{filename}`: Current source file name
- `{date}`: Current date (YYYY-MM-DD format)

Example: `"Page {page} of {total}"` → `"Page 5 of 23"`

## Styling Configuration

Controls visual appearance of the PDF.

```yaml
styling:
  font_family: monospace
  background_color: "#ffffff"
  text_color: "#000000"
  line_number_color: "#888888"
```

### Fields

- **font_family** (optional, default: `monospace`): Font for code rendering
  - `monospace`: Generic monospace font
  - `courier`: Courier New
  - `dejavu`: DejaVu Sans Mono

- **background_color** (optional, default: `"#ffffff"`): Background color (hex format)

- **text_color** (optional, default: `"#000000"`): Text color (hex format)
  - Note: Overridden when syntax highlighting is enabled

- **line_number_color** (optional, default: `"#888888"`): Line number color (hex format)

## Metadata Configuration

PDF document metadata.

```yaml
metadata:
  title: "Source Code Documentation"
  author: "Engineering Team"
  subject: "Code Review"
  keywords:
    - source code
    - review
```

### Fields

- **title** (optional): PDF document title

- **author** (optional): PDF document author

- **subject** (optional): PDF document subject

- **keywords** (optional): List of keywords for searchability

## Minimal Configuration Example

The absolute minimum required configuration:

```yaml
output:
  mode: single

files:
  - path: src/main.rs
```

All other fields have sensible defaults and can be omitted.

## See Also

- [Usage Guide](usage.md) - CLI usage and examples
- [Styling Guide](styling.md) - Advanced styling options
- [Examples](examples.md) - Real-world configuration examples
