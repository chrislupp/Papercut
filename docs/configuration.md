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
- `warnings` - Warning configuration
- `cover_page` - Cover page configuration
- `markdown_report` - Markdown report configuration

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
  theme: vscode-light
  custom_syntaxes:
    - /path/to/syntaxes/  # Directory of .sublime-syntax files
    - /path/to/Custom.sublime-syntax  # Individual syntax file
```

### Fields

- **enabled** (optional, default: `true`): Enable syntax highlighting
  - Requires the `syntax-highlighting` feature to be enabled

- **theme** (optional, default: `vscode-light`): Theme name
  - Run `papercut --list-themes` to see available themes
  - **Built-in presets**: `vscode-light`, `vscode-dark`, `jetbrains-light`, `jetbrains-darcula`
  - **Popular themes**: `InspiredGitHub`, `Solarized (dark)`, `Solarized (light)`, `base16-ocean.dark`

- **custom_syntaxes** (optional): List of custom syntax definition paths
  - Can be directories (scanned for all `.sublime-syntax` files) or individual files
  - Useful for languages not included in the default set (e.g., CMake)
  - Syntax files use the Sublime Text `.sublime-syntax` format

## Page Configuration

Controls page layout and formatting.

```yaml
page:
  size: A4
  margins:
    top: 2.5          # 2.5 cm (plain number = centimeters)
    bottom: "1in"     # 1 inch (string with unit)
    left: "2.0cm"     # 2.0 cm (explicit unit)
    right: 2.0
  font_family: "JetBrains Mono"  # Optional custom font
  font_size: 10
  line_numbers: true
  line_number_separator: true
  vertical_borders: true
  line_spacing: 1.2
  wrap_long_lines: true
  wrap_indent: 4
```

### Fields

- **size** (optional, default: `A4`): Paper size
  - `A4`: 210mm × 297mm (common in Europe, Asia)
  - `Letter`: 8.5" × 11" (common in North America)
  - `Legal`: 8.5" × 14" (legal documents)

- **margins** (optional): Page margins (supports centimeters and inches)
  - **top** (default: `2.5`): Top margin
  - **bottom** (default: `2.5`): Bottom margin
  - **left** (default: `2.0`): Left margin
  - **right** (default: `2.0`): Right margin

  **Margin value formats:**
  - Plain number: Treated as centimeters (e.g., `2.5` = 2.5 cm)
  - String with `cm` suffix: Centimeters (e.g., `"2.5cm"`)
  - String with `in` suffix: Inches (e.g., `"1in"`, `"0.75in"`)

- **font_family** (optional): Preferred monospace font for source code
  - If not specified, auto-detects from: Consolas, Monaco, Menlo, DejaVu Sans Mono, etc.
  - Common options: `"JetBrains Mono"`, `"Fira Code"`, `"Source Code Pro"`, `"Consolas"`, `"Monaco"`
  - If the specified font is not found, a warning is emitted and falls back to system defaults

- **font_size** (optional, default: `10`): Font size for code content in points
  - Typical values: 8-12

- **line_numbers** (optional, default: `true`): Show line numbers

- **line_number_separator** (optional, default: `true`): Show vertical line between line numbers and code

- **vertical_borders** (optional, default: `true`): Show vertical borders at left and right edges of content

- **line_spacing** (optional, default: `1.2`): Line spacing multiplier
  - 1.0 = single spacing
  - 1.5 = 1.5× spacing
  - 2.0 = double spacing

- **wrap_long_lines** (optional, default: `true`): Enable line wrapping for long lines
  - When enabled, lines exceeding the page width wrap to the next line
  - When disabled, long lines are truncated

- **wrap_indent** (optional, default: `4`): Indentation for wrapped continuation lines
  - Number of spaces to indent wrapped lines
  - Helps visually distinguish continuation lines from new source lines

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

PDF document metadata. Title and author automatically fall back to cover page values if not specified.

```yaml
metadata:
  # title: "..."      # Falls back to cover_page.title if empty
  # author: "..."     # Falls back to cover_page.authors (joined) if empty
  subject: "Code Review"
  keywords:
    - source code
    - review
```

### Fields

- **title** (optional): PDF document title. Falls back to `cover_page.title` if not specified.

- **author** (optional): PDF document author. Falls back to `cover_page.authors` (comma-separated) if not specified.

- **subject** (optional): PDF document subject

- **keywords** (optional): List of keywords for searchability

## Warnings Configuration

Controls warning output during PDF generation.

```yaml
warnings:
  enabled: true
  silence_categories:
    - fonts        # Silence font-related warnings
    - themes       # Silence theme-related warnings
```

### Fields

- **enabled** (optional, default: `true`): Enable or disable all warnings
  - When `false`, all warnings are suppressed
  - Can also be overridden with `--quiet` CLI flag

- **silence_categories** (optional): List of warning categories to silence
  - `fonts`: Font file read failures and fallback behavior
  - `themes`: Custom theme loading and parsing errors
  - `highlighting`: Syntax highlighting failures (falls back to plain text)
  - `filesystem`: Directory walking errors, permission denied, non-UTF-8 paths

## Cover Page Configuration

Controls the optional cover page that appears before the printed code.

```yaml
cover_page:
  enabled: true
  title: "Project Source Code"
  authors: "Development Team"           # Single author
  # Or multiple authors:
  # authors:
  #   - "Jane Doe"
  #   - "John Smith"
  description: "Complete source code listing for the project review."
  location: "https://github.com/user/project"
  date: ""                    # Empty = auto-generate current date
  include_toc: true           # Table of contents (on separate page)
  title_font_size: 24
  text_font_size: 12
  font_family: "Arial"                  # Cover page font
  # Optional: Override header/footer for cover page only
  header:
    enabled: false
  footer:
    enabled: true
    center: "Cover Page Footer"
```

### Fields

- **enabled** (optional, default: `false`): Enable the cover page
  - When `true`, a cover page will be added before the code content
  - Works in both single and multiple PDF modes
  - Each PDF gets its own cover page in multiple mode

- **title** (optional): Main title displayed on the cover page
  - Displayed prominently at the top of the cover page
  - If empty, no title is shown
  - Uses the `title_font_size` setting

- **authors** (optional): Author name(s) displayed below the title
  - Can be a single string or a list of strings
  - Multiple authors are displayed one per line, centered
  - Uses the `text_font_size` setting

- **description** (optional): Descriptive text about the code
  - Can be multiple lines (use YAML multiline syntax)
  - Automatically wraps to fit the page width
  - Displayed below the authors

- **location** (optional): URL or location where the code is kept
  - Typically a GitHub URL, Git repository, or file path
  - Displayed as "Location: [value]"
  - Useful for referencing the source of the code

- **date** (optional): Date to display on the cover page
  - If empty, automatically generates current date (YYYY-MM-DD format)
  - Can be any custom date string
  - Displayed as "Date: [value]"

- **include_toc** (optional, default: `true`): Include table of contents
  - Lists all files that will be printed
  - Rendered on a separate page after the cover page
  - Supports multiple pages for large file lists
  - Only shown if cover page is enabled

- **title_font_size** (optional, default: `24`): Font size for the title in points

- **text_font_size** (optional, default: `12`): Font size for description and other text in points

- **font_family** (optional, default: `"Arial"`): Font family for the cover page
  - Common options: "Arial", "Helvetica", "Times New Roman", "Georgia"
  - Falls back to Arial if the specified font is not available
  - The title and "Description" heading are rendered in bold

- **header** (optional): Override header configuration for cover page only
  - If not specified, uses the main `header` configuration
  - Accepts the same fields as the main header configuration

- **footer** (optional): Override footer configuration for cover page only
  - If not specified, uses the main `footer` configuration
  - Accepts the same fields as the main footer configuration

### Examples

**Example 1: Simple cover page with single author**
```yaml
cover_page:
  enabled: true
  title: "My Project"
  authors: "John Doe"
  description: "Source code documentation"
  location: "https://github.com/user/myproject"
```

**Example 2: Cover page with multiple authors**
```yaml
cover_page:
  enabled: true
  title: "Code Review Package"
  authors:
    - "Jane Smith"
    - "John Doe"
    - "Engineering Team"
  description: |
    This package contains all source code files for the Q4 2024 review.

    Please review all changes and provide feedback by the deadline.
  location: "/mnt/shared/code-reviews/q4-2024"
  date: "2024-12-15"
  include_toc: true
```

**Example 3: Cover page without table of contents**
```yaml
cover_page:
  enabled: true
  title: "Confidential Source Code"
  authors: "Security Team"
  description: "Internal use only"
  location: "Internal Repository"
  include_toc: false
```

**Example 4: Large title for presentation**
```yaml
cover_page:
  enabled: true
  title: "Company Project Alpha"
  authors:
    - "Alpha Development Team"
    - "Documentation Team"
  title_font_size: 32
  text_font_size: 14
  description: "Complete technical documentation"
  location: "https://internal.company.com/alpha"
```

## Markdown Report Configuration

Include a markdown document before the source code listing. Useful for adding documentation, analysis, or reports.

```yaml
markdown_report:
  enabled: true
  path: docs/report.md
```

### Fields

- **enabled** (optional, default: `false`): Enable the markdown report
  - When `true`, the markdown file is rendered before source code content

- **path** (required when enabled): Path to the markdown file
  - Can be relative to the config file or absolute
  - Supports standard markdown formatting
  - Images are embedded (supports PNG, JPEG, SVG)
  - Code blocks are syntax highlighted

### Supported Markdown Features

- **Headings**: `#`, `##`, `###`, etc.
- **Text formatting**: Bold, italic, strikethrough
- **Lists**: Ordered and unordered
- **Code blocks**: Fenced code blocks with syntax highlighting
- **Images**: Local images are embedded in the PDF
- **Links**: Displayed as text (PDFs don't support clickable links in body)
- **Block quotes**: Indented quote blocks
- **Horizontal rules**: Page separators

### Example

```yaml
markdown_report:
  enabled: true
  path: analysis/code_review.md
```

Where `code_review.md` might contain:

```markdown
# Code Review Summary

## Overview
This document provides an analysis of the authentication module.

## Key Findings
- Input validation is comprehensive
- Session handling follows best practices

![Architecture Diagram](diagrams/auth_flow.svg)
```

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
