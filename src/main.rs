mod config;
mod error;
mod file_scanner;
mod pdf;

#[cfg(feature = "syntax-highlighting")]
mod highlighting;

use clap::Parser;
use std::path::PathBuf;
use config::Config;
use error::Result;

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
    /// Path to the YAML configuration file
    #[arg(short, long, value_name = "FILE")]
    config: PathBuf,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,

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

    // Load configuration
    if args.verbose {
        println!("Loading configuration from: {}", args.config.display());
    }

    let config = Config::from_file(&args.config)?;

    if args.verbose {
        println!("Configuration loaded successfully");
        println!("Output mode: {:?}", config.output.mode);
        println!("File patterns specified: {}", config.files.len());
        println!("Files matched: {}", config.expanded_files.len());
    }

    // Create output directory if it doesn't exist
    std::fs::create_dir_all(&config.output.directory)?;

    if args.verbose {
        println!("Output directory: {}", config.output.directory.display());
    }

    // Generate PDF(s)
    pdf::generate(config, args.verbose)?;

    if args.verbose {
        println!("PDF generation completed successfully!");
    }

    Ok(())
}
