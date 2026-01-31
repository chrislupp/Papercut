# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.1.0] - 2026-01-31

### Added
- Custom font family for source code (`page.font_family`) - specify preferred monospace font
- Markdown report support (`markdown_report`) - include documentation with images before source code
- Custom syntax highlighting definitions (`syntax_highlighting.custom_syntaxes`)
- CMake syntax highlighting (bundled)
- Built-in theme presets: `vscode-light`, `vscode-dark`, `jetbrains-light`, `jetbrains-darcula`
- Line wrapping options (`page.wrap_long_lines`, `page.wrap_indent`)
- Visual formatting options (`page.line_number_separator`, `page.vertical_borders`)
- Cover page header/footer overrides
- Support for authors as list in cover page configuration

### Changed
- Default syntax highlighting theme changed to `vscode-light`
- Improved documentation with comprehensive examples

### Fixed
- Repository URL in documentation

## [1.0.0] - 2026-01-29

### Added
- PDF generation from source code files with syntax highlighting
- Configurable page headers and footers with variable substitution
- Cover page with table of contents and clickable hyperlinks
- Directory scanning with glob patterns and file filtering
- Flexible margin configuration (mm, cm, inches)
- macOS DMG installer with app icon
