# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.1.0] - 2026-07-28

### Added
- Custom font family selection for source code
- Markdown reports with vector and raster image support
- Custom syntax definitions and bundled CMake syntax highlighting
- Built-in VS Code and JetBrains theme presets
- Configurable long-line wrapping, line-number separators, and vertical borders
- Per-cover-page header and footer overrides
- Multiple authors in cover-page configuration
- Strict validation for unknown configuration fields and unusable wrapping layouts
- Deterministic numbering for colliding filenames in multiple-output mode
- CLI regression tests for config-relative paths and overwrite protection

### Changed
- Document Homebrew installation on macOS and Linux
- Default syntax highlighting theme to `vscode-light`
- Expanded configuration, styling, usage, and example documentation
- Resolve relative paths from the configuration file's directory
- Preserve custom titles and deduplicate files selected by overlapping entries
- Replace deprecated `serde_yaml` with `serde-saphyr`
- Check overwrite decisions before rendering PDFs

### Fixed
- Release workflow now synchronizes the package version in `Cargo.lock`
- Repository URLs in documentation
- Build failures when syntax highlighting is disabled
- Potential unsigned underflow while wrapping highlighted source lines
- Suppressed filesystem warnings during input scanning

## [1.0.0] - 2026-01-29

### Added
- PDF generation from source code files with syntax highlighting
- Configurable page headers and footers with variable substitution
- Cover page with table of contents and clickable hyperlinks
- Directory scanning with glob patterns and file filtering
- Flexible margin configuration (mm, cm, inches)
- macOS DMG installer with app icon
