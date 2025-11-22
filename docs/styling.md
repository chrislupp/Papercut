# Styling Guide

This guide covers advanced styling and formatting options for customizing the appearance of your PDFs.

## Overview

Papercut provides several ways to customize the visual appearance of generated PDFs:

- **Page Layout**: Size, margins, orientation
- **Typography**: Fonts, sizes, spacing
- **Colors**: Background, text, line numbers
- **Syntax Highlighting**: Themes and color schemes
- **Headers/Footers**: Position, content, formatting

## Page Layout

### Paper Sizes

Papercut supports three standard paper sizes:

| Size   | Dimensions (mm) | Dimensions (in) | Common Use      |
|--------|----------------|-----------------|-----------------|
| A4     | 210 × 297      | 8.27 × 11.69   | International   |
| Letter | 215.9 × 279.4  | 8.5 × 11       | North America   |
| Legal  | 215.9 × 355.6  | 8.5 × 14       | Legal documents |

Configuration:

```yaml
page:
  size: A4  # or Letter, Legal
```

### Margins

Margins are specified in centimeters and can be customized for each edge:

```yaml
page:
  margins:
    top: 2.5     # Top margin in cm
    bottom: 2.5  # Bottom margin in cm
    left: 2.0    # Left margin in cm
    right: 2.0   # Right margin in cm
```

**Tips:**
- Use larger margins (3.0-3.5 cm) for documents that will be printed and bound
- Use smaller margins (1.5-2.0 cm) for screen-only documents to maximize content area
- Consider header/footer space when setting top/bottom margins

### Line Spacing

Control the vertical spacing between lines:

```yaml
page:
  line_spacing: 1.2  # Multiplier
```

Common values:
- `1.0`: Single spacing (compact)
- `1.2`: Slight spacing (default, good readability)
- `1.5`: Medium spacing (very readable)
- `2.0`: Double spacing (maximum readability, uses more pages)

## Typography

### Font Families

Papercut supports three monospace font families for code:

```yaml
styling:
  font_family: monospace  # or courier, dejavu
```

Font characteristics:

| Font      | Style       | Best For                    |
|-----------|-------------|------------------------------|
| monospace | Clean, modern | General purpose, modern look |
| courier   | Classic     | Traditional code listings    |
| dejavu    | Professional | Professional documentation   |

### Font Sizes

Configure font sizes for different elements:

```yaml
page:
  font_size: 10  # Code content

header:
  font_size: 8   # Header text

footer:
  font_size: 8   # Footer text
```

**Guidelines:**
- **Code content**: 9-11 points (10 is default)
  - Use 9 for dense code
  - Use 11-12 for presentations or accessibility
- **Headers/footers**: 7-9 points (8 is default)

### Line Numbers

Enable or disable line numbers:

```yaml
page:
  line_numbers: true  # or false
```

Line numbers appear on the left side of code with the format:

```
   1 | fn main() {
   2 |     println!("Hello, world!");
   3 | }
```

## Colors

### Color Format

All colors are specified in hexadecimal format: `#RRGGBB`

Examples:
- `#ffffff` - White
- `#000000` - Black
- `#f5f5f5` - Light gray
- `#2e3440` - Dark gray
- `#888888` - Medium gray

### Color Options

```yaml
styling:
  background_color: "#ffffff"      # Page/code background
  text_color: "#000000"            # Text color (when syntax highlighting disabled)
  line_number_color: "#888888"     # Line number color
```

### Color Schemes

#### High Contrast (Default)

Best for printing and accessibility:

```yaml
styling:
  background_color: "#ffffff"
  text_color: "#000000"
  line_number_color: "#888888"
```

#### Low Contrast

Easier on the eyes for screen reading:

```yaml
styling:
  background_color: "#f5f5f5"
  text_color: "#2e3440"
  line_number_color: "#999999"
```

#### Dark Mode

For presentations or dark-themed documents:

```yaml
styling:
  background_color: "#2e3440"
  text_color: "#d8dee9"
  line_number_color: "#4c566a"
```

**Note:** When syntax highlighting is enabled, `text_color` is overridden by the syntax theme colors.

## Syntax Highlighting

### Enabling Syntax Highlighting

```yaml
syntax_highlighting:
  enabled: true
  theme: base16-ocean.dark
```

### Available Themes

To see all available themes:

```bash
papercut --list-themes
```

### Popular Themes

#### Light Themes

Good for printing and professional documents:

- **InspiredGitHub**: Clean, GitHub-style highlighting
- **Solarized (light)**: Carefully designed color palette
- **base16-ocean.light**: Soft, ocean-inspired colors

```yaml
syntax_highlighting:
  enabled: true
  theme: InspiredGitHub
```

#### Dark Themes

Good for screen reading and presentations:

- **base16-ocean.dark**: Popular dark theme with blue tones
- **Solarized (dark)**: Professional dark theme
- **Monokai**: High contrast, vibrant colors

```yaml
syntax_highlighting:
  enabled: true
  theme: base16-ocean.dark
```

### Choosing a Theme

Consider:
- **Printing**: Use light themes with good contrast
- **Screen reading**: Dark or light themes both work
- **Accessibility**: Themes with high contrast
- **Company branding**: Choose colors that match your brand

## Headers and Footers

### Layout

Headers and footers support three alignment zones:

```
┌─────────────────────────────────────┐
│ LEFT          CENTER          RIGHT │  ← Header
├─────────────────────────────────────┤
│                                     │
│         Page Content                │
│                                     │
├─────────────────────────────────────┤
│ LEFT          CENTER          RIGHT │  ← Footer
└─────────────────────────────────────┘
```

### Configuration

```yaml
header:
  enabled: true
  left: "Company Confidential"
  center: "Source Code Review"
  right: "Page {page} of {total}"
  font_size: 8

footer:
  enabled: true
  left: "{date}"
  center: "{filename}"
  right: "© 2024 Company"
  font_size: 8
```

### Variables

Use these variables for dynamic content:

| Variable     | Description          | Example Output    |
|--------------|---------------------|-------------------|
| `{page}`     | Current page number | `5`               |
| `{total}`    | Total pages         | `23`              |
| `{filename}` | Current file name   | `main.rs`         |
| `{date}`     | Current date        | `2024-11-22`      |

### Common Patterns

#### Professional Document

```yaml
header:
  enabled: true
  left: "Project Name"
  center: "Code Documentation"
  right: "Page {page}/{total}"

footer:
  enabled: true
  left: "Generated {date}"
  center: "{filename}"
  right: "Engineering Team"
```

#### Confidential Release

```yaml
header:
  enabled: true
  left: "CONFIDENTIAL - INTERNAL USE ONLY"
  right: "Page {page} of {total}"

footer:
  enabled: true
  left: "Distribution Restricted"
  right: "{date}"
```

#### Legal Compliance

```yaml
header:
  enabled: true
  left: "ATTORNEY WORK PRODUCT"
  center: "Source Code Audit"
  right: "Page {page}/{total}"

footer:
  enabled: true
  left: "Case #{case_number}"
  center: "{filename}"
  right: "Privileged & Confidential"
```

## Complete Styling Examples

### Example 1: Clean Professional

Modern, clean look for technical documentation:

```yaml
page:
  size: Letter
  margins: { top: 2.5, bottom: 2.5, left: 2.5, right: 2.5 }
  font_size: 10
  line_numbers: true
  line_spacing: 1.3

syntax_highlighting:
  enabled: true
  theme: InspiredGitHub

styling:
  font_family: monospace
  background_color: "#ffffff"
  text_color: "#000000"
  line_number_color: "#888888"

header:
  enabled: true
  left: "Technical Documentation"
  right: "Page {page}/{total}"
  font_size: 8

footer:
  enabled: true
  center: "{filename}"
  right: "{date}"
  font_size: 8
```

### Example 2: Compact Code Listing

Maximize content on each page:

```yaml
page:
  size: A4
  margins: { top: 1.5, bottom: 1.5, left: 1.5, right: 1.5 }
  font_size: 8
  line_numbers: true
  line_spacing: 1.0

syntax_highlighting:
  enabled: true
  theme: base16-ocean.dark

styling:
  font_family: courier
  line_number_color: "#666666"
```

### Example 3: Presentation Style

Large, readable for presentations:

```yaml
page:
  size: Letter
  margins: { top: 3.0, bottom: 3.0, left: 3.0, right: 3.0 }
  font_size: 12
  line_numbers: false
  line_spacing: 1.5

syntax_highlighting:
  enabled: true
  theme: Solarized (light)

styling:
  font_family: dejavu

header:
  enabled: true
  center: "Code Review Meeting"
  right: "{date}"
  font_size: 10
```

## Best Practices

1. **Consistency**: Use the same styling across all documents in a project
2. **Readability**: Prioritize readability over compactness
3. **Purpose**: Match styling to document purpose (formal, internal, presentation)
4. **Testing**: Generate sample PDFs to test appearance before bulk generation
5. **Accessibility**: Use high-contrast themes and adequate font sizes
6. **Branding**: Incorporate company colors and fonts where appropriate

## See Also

- [Configuration Reference](configuration.md) - Complete configuration options
- [Usage Guide](usage.md) - CLI usage instructions
- [Examples](examples.md) - Real-world configuration examples
