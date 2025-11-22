use crate::config::{Config, OutputMode};
use crate::error::{PapercutError, Result};
use crate::pdf::styling;
use genpdf::elements;
use genpdf::{Alignment, Document, SimplePageDecorator};
use std::fs;
use std::path::PathBuf;

#[cfg(feature = "syntax-highlighting")]
use crate::highlighting;

/// Main entry point for PDF generation
pub fn generate(config: Config, verbose: bool) -> Result<()> {
    match config.output.mode {
        OutputMode::Single => generate_single_pdf(config, verbose),
        OutputMode::Multiple => generate_multiple_pdfs(config, verbose),
    }
}

/// Generate a single PDF containing all files
fn generate_single_pdf(config: Config, verbose: bool) -> Result<()> {
    if verbose {
        println!("Generating single PDF with {} files", config.files.len());
    }

    let output_path = config
        .output
        .directory
        .join(&config.output.filename);

    let mut doc = create_document(&config)?;

    // Process each file and add to the document
    for (idx, file_entry) in config.files.iter().enumerate() {
        if verbose {
            println!("  Processing file {}/{}: {}",
                idx + 1,
                config.files.len(),
                file_entry.path.display()
            );
        }

        let content = fs::read_to_string(&file_entry.path)
            .map_err(|e| PapercutError::Io(e))?;

        // Add file title/separator
        let default_title = file_entry.path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown")
            .to_string();
        let title = file_entry.title.as_ref().unwrap_or(&default_title);

        add_file_separator(&mut doc, title, &config)?;

        // Add file content
        add_file_content(&mut doc, &content, &file_entry.path, &config)?;

        // Add page break between files (except for last file)
        if idx < config.files.len() - 1 {
            doc.push(elements::PageBreak::new());
        }
    }

    // Render the PDF
    if verbose {
        println!("Rendering PDF to: {}", output_path.display());
    }

    doc.render_to_file(&output_path)
        .map_err(|e| PapercutError::PdfGeneration(e.to_string()))?;

    println!("✓ Generated: {}", output_path.display());

    Ok(())
}

/// Generate multiple PDFs, one per file
fn generate_multiple_pdfs(config: Config, verbose: bool) -> Result<()> {
    if verbose {
        println!("Generating {} separate PDFs", config.files.len());
    }

    for (idx, file_entry) in config.files.iter().enumerate() {
        if verbose {
            println!("  Processing file {}/{}: {}",
                idx + 1,
                config.files.len(),
                file_entry.path.display()
            );
        }

        // Create output filename
        let output_filename = file_entry.path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output");
        let output_path = config.output.directory.join(format!("{}.pdf", output_filename));

        let mut doc = create_document(&config)?;

        // Read file content
        let content = fs::read_to_string(&file_entry.path)
            .map_err(|e| PapercutError::Io(e))?;

        // Add file content
        add_file_content(&mut doc, &content, &file_entry.path, &config)?;

        // Render the PDF
        doc.render_to_file(&output_path)
            .map_err(|e| PapercutError::PdfGeneration(e.to_string()))?;

        println!("✓ Generated: {}", output_path.display());
    }

    Ok(())
}

/// Create a new document with the configured settings
fn create_document(config: &Config) -> Result<Document> {
    // Get page size
    let (width_mm, height_mm) = styling::get_page_size(&config.page.size);

    // Load the default font family
    let default_font_family = genpdf::fonts::from_files("./fonts", "LiberationSans", None)
        .unwrap_or_else(|_| {
            // If that fails, try loading from common system font paths
            genpdf::fonts::from_files("/usr/share/fonts/truetype/liberation", "LiberationSans", None)
                .unwrap_or_else(|_| {
                    // If all else fails, use printpdf's default font
                    panic!("Unable to load fonts. Please ensure font files are available.")
                })
        });

    let mut doc = Document::new(default_font_family);

    // Set page size using genpdf::Size
    doc.set_paper_size(genpdf::Size::new(width_mm, height_mm));

    // Set line spacing
    doc.set_line_spacing(config.page.line_spacing as f64);

    // Add page decorator for headers/footers if enabled
    if config.header.enabled || config.footer.enabled {
        let decorator = create_page_decorator(config);
        doc.set_page_decorator(decorator);
    }

    // Set metadata
    if !config.metadata.title.is_empty() {
        doc.set_title(&config.metadata.title);
    }

    Ok(doc)
}

/// Create a page decorator for headers and footers
fn create_page_decorator(_config: &Config) -> SimplePageDecorator {
    // TODO: Implement proper headers/footers using custom decorator
    SimplePageDecorator::new()
}

/// Add a file separator/title to the document
fn add_file_separator(doc: &mut Document, title: &str, config: &Config) -> Result<()> {
    // Create styled text for the title
    let _title_size = config.page.font_size + 2;
    let separator = elements::Paragraph::new(title)
        .aligned(Alignment::Left);

    // Add the separator
    doc.push(separator);

    // Add some spacing
    doc.push(elements::Break::new(1.5));

    Ok(())
}

/// Add file content to the document
fn add_file_content(
    doc: &mut Document,
    content: &str,
    file_path: &PathBuf,
    config: &Config,
) -> Result<()> {
    #[cfg(feature = "syntax-highlighting")]
    if config.syntax_highlighting.enabled {
        return add_highlighted_content(doc, content, file_path, config);
    }

    // Add plain text content
    add_plain_content(doc, content, config)
}

/// Add plain text content without syntax highlighting
fn add_plain_content(doc: &mut Document, content: &str, config: &Config) -> Result<()> {
    let lines: Vec<&str> = content.lines().collect();

    for (line_num, line) in lines.iter().enumerate() {
        let line_text = if config.page.line_numbers {
            format!("{:4} | {}", line_num + 1, line)
        } else {
            line.to_string()
        };

        let para = elements::Paragraph::new(&line_text);
        doc.push(para);
    }

    Ok(())
}

/// Add syntax-highlighted content
#[cfg(feature = "syntax-highlighting")]
fn add_highlighted_content(
    doc: &mut Document,
    content: &str,
    file_path: &PathBuf,
    config: &Config,
) -> Result<()> {
    let highlighted = highlighting::highlight_code(content, file_path, &config.syntax_highlighting.theme)
        .map_err(|e| PapercutError::SyntaxHighlighting(e))?;

    let lines: Vec<&str> = highlighted.lines().collect();

    for (line_num, line) in lines.iter().enumerate() {
        let line_text = if config.page.line_numbers {
            format!("{:4} | {}", line_num + 1, line)
        } else {
            line.to_string()
        };

        let para = elements::Paragraph::new(&line_text);
        doc.push(para);
    }

    Ok(())
}
