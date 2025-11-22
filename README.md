# Papercut

Convert source code files to PDF with configurable headers, footers, and formatting.

## Quick Start

### Installation

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

- **Single or Multiple PDFs**: Combine all files or create separate PDFs
- **Syntax Highlighting**: Optional syntax highlighting with multiple themes
- **Customizable Headers/Footers**: Add page numbers, dates, filenames, and custom text
- **Flexible Formatting**: Configure page size, margins, fonts, and colors
- **PDF Metadata**: Set title, author, subject, and keywords

### Example Configurations

See the `examples/` directory:

- `minimal_config.yaml` - Bare minimum configuration
- `full_config.yaml` - All available options with comments
- `release_config.yaml` - Public release template

### Documentation

Complete documentation is available in the `docs/` directory:

- [Configuration Reference](docs/configuration.md) - All configuration options
- [Usage Guide](docs/usage.md) - CLI usage and workflows
- [Styling Guide](docs/styling.md) - Advanced styling options
- [Examples](docs/examples.md) - Real-world use cases

### List Available Themes

```bash
papercut --list-themes
```

### Help

```bash
papercut --help
```

## License

This project is licensed under the GNU General Public License v3.0 - see the [LICENSE](LICENSE) file for details.
