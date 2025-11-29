mod config;
mod error;
mod file_scanner;
mod pdf;
mod warnings;

#[cfg(feature = "syntax-highlighting")]
mod highlighting;

use clap::Parser;
use std::path::PathBuf;
use std::sync::Arc;
use config::Config;
use error::Result;
use warnings::{WarningManager, WarningCategory};

/// Convert source code files to PDF with configurable headers and footers
#[derive(Parser, Debug)]
#[command(
    name = "papercut",
    version,
    about = "Convert source code files to PDF with configurable headers and footers",
    long_about = "Papercut is a CLI tool that converts source code files to PDFs based on a YAML configuration file. \
                  It supports customizable headers, footers, syntax highlighting, and various page formatting options."
)]
struct Args {
    /// Path to the YAML configuration file (default: .papercut.yaml)
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Suppress all warnings
    #[arg(short, long)]
    quiet: bool,

    /// Skip interactive prompts and overwrite files without asking
    #[arg(short, long)]
    force: bool,

    /// List available syntax highlighting themes (requires syntax-highlighting feature)
    #[cfg(feature = "syntax-highlighting")]
    #[arg(long)]
    list_themes: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    #[cfg(feature = "syntax-highlighting")]
    if args.list_themes {
        highlighting::list_themes();
        return Ok(());
    }

    // Determine config file path
    let default_config = PathBuf::from(".papercut.yaml");
    let config_path = args.config.as_ref().unwrap_or(&default_config);

    // Check if config file exists
    if !config_path.exists() {
        if args.config.is_some() {
            return Err(error::PapercutError::FileNotFound(
                config_path.display().to_string()
            ));
        } else {
            return Err(error::PapercutError::Config(
                "No configuration file found. Create a .papercut.yaml file or specify one with -c/--config".to_string()
            ));
        }
    }

    // Load configuration
    if args.verbose {
        println!("Loading configuration from: {}", config_path.display());
    }

    let config = Config::from_file(config_path)?;

    if args.verbose {
        println!("Configuration loaded successfully");
        println!("Output mode: {:?}", config.output.mode);
        println!("File patterns specified: {}", config.files.len());
        println!("Files matched: {}", config.expanded_files.len());
    }

    // Set up warning manager
    let warnings_enabled = config.warnings.enabled && !args.quiet;
    let warning_manager = Arc::new(WarningManager::new(warnings_enabled));

    // Configure warning categories based on config
    let (known_categories, unknown_categories): (Vec<_>, Vec<_>) = config
        .warnings
        .silence_categories
        .iter()
        .map(|s| (s, WarningCategory::from_str(s)))
        .partition(|(_, cat)| cat.is_some());

    let categories: Vec<_> = known_categories
        .into_iter()
        .filter_map(|(_, cat)| cat)
        .collect();
    warning_manager.silence_categories(&categories);

    if args.verbose {
        for (name, _) in unknown_categories {
            eprintln!("Warning: Unknown warning category '{}' in config", name);
        }
    }

    // Create output directory if it doesn't exist
    std::fs::create_dir_all(&config.output.directory)?;

    if args.verbose {
        println!("Output directory: {}", config.output.directory.display());
    }

    // Generate PDF(s)
    pdf::generate(config, args.verbose, args.force, warning_manager)?;

    if args.verbose {
        println!("PDF generation completed successfully!");
    }

    Ok(())
}
