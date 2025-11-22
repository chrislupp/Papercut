use crate::config::{Config, OutputMode};
use crate::error::{PapercutError, Result};
use crate::pdf::krilla_doc::PdfContext;
use crate::pdf::colors::{rgb_to_paint, syntect_to_paint};
use krilla::geom::{PathBuilder, Point};
use krilla::num::NormalizedF32;
use krilla::paint::{Fill, Stroke};
use krilla::text::TextDirection;
use std::fs;

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
        println!("Generating single PDF with {} files", config.expanded_files.len());
    }

    let output_path = config.output.directory.join(&config.output.filename);

    let mut ctx = PdfContext::new(config.clone())?;
    let font = ctx.font_manager.get_monospace_font()?;

    // Start first page
    let mut page = ctx.document.start_page_with(ctx.page_settings());
    let mut surface = page.surface();
    let mut current_y = ctx.margin_top;

    // Add document title if present
    if !config.metadata.title.is_empty() {
        let title_x = ctx.margin_left + ctx.content_width / 2.0;
        let title_y = current_y + 14.0;

        surface.draw_text(
            Point::from_xy(title_x, title_y),
            font.as_ref().clone(),
            14.0,
            &config.metadata.title,
            false,
            TextDirection::Auto,
        );

        current_y += 25.0;
    }

    // Process each file
    for (idx, file_entry) in config.expanded_files.iter().enumerate() {
        if verbose {
            println!("  Processing file {}/{}: {}",
                idx + 1,
                config.expanded_files.len(),
                file_entry.path.display()
            );
        }

        // Read file content
        let content = fs::read_to_string(&file_entry.path)
            .map_err(|e| PapercutError::Io(e))?;

        // Get file title
        let default_title = file_entry.path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown")
            .to_string();
        let title = file_entry.title.as_ref().unwrap_or(&default_title);

        // Add separator line above file header
        let mut path_builder = PathBuilder::new();
        path_builder.move_to(ctx.margin_left, current_y);
        path_builder.line_to(ctx.margin_left + ctx.content_width, current_y);
        if let Some(separator_path) = path_builder.finish() {
            surface.set_stroke(Some(Stroke {
                paint: rgb_to_paint(180, 180, 180),
                width: 0.5,
                ..Default::default()
            }));
            surface.draw_path(&separator_path);
            surface.set_stroke(None);
        }

        current_y += 8.0;

        // Add file header
        surface.set_fill(Some(Fill {
            paint: rgb_to_paint(50, 50, 50),
            opacity: NormalizedF32::ONE,
            rule: Default::default(),
        }));
        surface.draw_text(
            Point::from_xy(ctx.margin_left, current_y + 12.0),
            font.as_ref().clone(),
            12.0,
            &format!("FILE: {}", title),
            false,
            TextDirection::Auto,
        );
        surface.set_fill(None);

        current_y += 25.0;

        // Add separator line below file header
        let mut path_builder = PathBuilder::new();
        path_builder.move_to(ctx.margin_left, current_y);
        path_builder.line_to(ctx.margin_left + ctx.content_width, current_y);
        if let Some(separator_path) = path_builder.finish() {
            surface.set_stroke(Some(Stroke {
                paint: rgb_to_paint(180, 180, 180),
                width: 0.5,
                ..Default::default()
            }));
            surface.draw_path(&separator_path);
            surface.set_stroke(None);
        }

        current_y += 10.0;

        // Render code lines with syntax highlighting if enabled
        let font_size = config.page.font_size as f32;
        let line_height = font_size * config.page.line_spacing;

        // Calculate available width for code (accounting for line numbers)
        let line_num_width = if config.page.line_numbers {
            font_size * 3.5 // Space for "999 " format
        } else {
            0.0
        };
        let code_x = ctx.margin_left + line_num_width;

        // Try syntax highlighting if enabled
        #[cfg(feature = "syntax-highlighting")]
        let highlighted = if config.syntax_highlighting.enabled {
            highlighting::highlight_code_styled(
                &content,
                &file_entry.path,
                &config.syntax_highlighting.theme
            ).ok()
        } else {
            None
        };

        #[cfg(not(feature = "syntax-highlighting"))]
        let highlighted: Option<Vec<Vec<highlighting::StyledSegment>>> = None;

        if let Some(styled_lines) = highlighted {
            // Render with syntax highlighting
            for (i, line_segments) in styled_lines.iter().enumerate() {
                let line_y = current_y + font_size;

                // Draw line number if enabled
                if config.page.line_numbers {
                    let line_num = format!("{:>4} ", i + 1);
                    surface.draw_text(
                        Point::from_xy(ctx.margin_left, line_y),
                        font.as_ref().clone(),
                        font_size,
                        &line_num,
                        false,
                        TextDirection::Auto,
                    );
                }

                // Draw colored segments
                let mut current_x = code_x;
                for segment in line_segments {
                    let paint = syntect_to_paint(segment.foreground);

                    // Set the fill color for this segment
                    surface.set_fill(Some(Fill {
                        paint,
                        opacity: NormalizedF32::ONE,
                        rule: Default::default(),
                    }));

                    // Draw the text segment with the current fill color
                    surface.draw_text(
                        Point::from_xy(current_x, line_y),
                        font.as_ref().clone(),
                        font_size,
                        &segment.text,
                        false,
                        TextDirection::Auto,
                    );

                    // Advance x position (approximate)
                    current_x += segment.text.len() as f32 * font_size * 0.6;
                }

                // Reset fill to None after rendering the line
                surface.set_fill(None);

                current_y += line_height;

                // Check if we need a new page
                if current_y + line_height > ctx.page_height_mm - ctx.margin_bottom {
                    surface.finish();
                    page.finish();
                    page = ctx.document.start_page_with(ctx.page_settings());
                    surface = page.surface();
                    current_y = ctx.margin_top;
                }
            }
        } else {
            // Fallback to plain text rendering
            let lines: Vec<&str> = content.lines().collect();
            let available_width = ctx.content_width - line_num_width;
            let char_width = font_size * 0.6;
            let max_chars = (available_width / char_width).floor() as usize;

            for (i, line) in lines.iter().enumerate() {
                let line_y = current_y + font_size;

                // Draw line number if enabled
                if config.page.line_numbers {
                    let line_num = format!("{:>4} ", i + 1);
                    surface.draw_text(
                        Point::from_xy(ctx.margin_left, line_y),
                        font.as_ref().clone(),
                        font_size,
                        &line_num,
                        false,
                        TextDirection::Auto,
                    );
                }

                // Draw code line (truncate if too long, respecting UTF-8 boundaries)
                let display_line: String = if line.chars().count() > max_chars {
                    line.chars().take(max_chars.saturating_sub(3)).collect()
                } else {
                    line.to_string()
                };

                surface.draw_text(
                    Point::from_xy(code_x, line_y),
                    font.as_ref().clone(),
                    font_size,
                    &display_line,
                    false,
                    TextDirection::Auto,
                );

                current_y += line_height;

                // Check if we need a new page
                if current_y + line_height > ctx.page_height_mm - ctx.margin_bottom {
                    surface.finish();
                    page.finish();
                    page = ctx.document.start_page_with(ctx.page_settings());
                    surface = page.surface();
                    current_y = ctx.margin_top;
                }
            }
        }

        // Add separator line at end of file
        current_y += 10.0;
        let mut path_builder = PathBuilder::new();
        path_builder.move_to(ctx.margin_left, current_y);
        path_builder.line_to(ctx.margin_left + ctx.content_width, current_y);
        if let Some(separator_path) = path_builder.finish() {
            surface.set_stroke(Some(Stroke {
                paint: rgb_to_paint(180, 180, 180),
                width: 0.5,
                ..Default::default()
            }));
            surface.draw_path(&separator_path);
            surface.set_stroke(None);
        }

        current_y += 15.0; // Space after file
    }

    // Finish last page
    surface.finish();
    page.finish();

    // Save the document
    if verbose {
        println!("Rendering PDF to: {}", output_path.display());
    }

    ctx.save(&output_path)?;

    println!("✓ Generated: {}", output_path.display());

    Ok(())
}

/// Generate multiple PDFs, one per file
fn generate_multiple_pdfs(config: Config, verbose: bool) -> Result<()> {
    if verbose {
        println!("Generating {} separate PDFs", config.expanded_files.len());
    }

    for (idx, file_entry) in config.expanded_files.iter().enumerate() {
        if verbose {
            println!("  Processing file {}/{}: {}",
                idx + 1,
                config.expanded_files.len(),
                file_entry.path.display()
            );
        }

        // Create output filename
        let output_filename = file_entry.path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output");
        let output_path = config.output.directory.join(format!("{}.pdf", output_filename));

        let mut ctx = PdfContext::new(config.clone())?;
        let font = ctx.font_manager.get_monospace_font()?;

        // Start page
        let mut page = ctx.document.start_page_with(ctx.page_settings());
        let mut surface = page.surface();
        let mut current_y = ctx.margin_top;

        // Read file content
        let content = fs::read_to_string(&file_entry.path)
            .map_err(|e| PapercutError::Io(e))?;

        // Render code (simplified)
        let lines: Vec<&str> = content.lines().collect();
        let font_size = config.page.font_size as f32;
        let line_height = font_size * config.page.line_spacing;

        // Calculate available width for code (accounting for line numbers)
        let line_num_width = if config.page.line_numbers {
            font_size * 3.5 // Space for "999 " format
        } else {
            0.0
        };
        let available_width = ctx.content_width - line_num_width;
        let char_width = font_size * 0.6; // Monospace approximation
        let max_chars = (available_width / char_width).floor() as usize;

        for (i, line) in lines.iter().enumerate() {
            let line_y = current_y + font_size;

            if config.page.line_numbers {
                let line_num = format!("{:>4} ", i + 1);
                surface.draw_text(
                    Point::from_xy(ctx.margin_left, line_y),
                    font.as_ref().clone(),
                    font_size,
                    &line_num,
                    false,
                    TextDirection::Auto,
                );
            }

            // Draw code line (truncate if too long, respecting UTF-8 boundaries)
            let code_x = ctx.margin_left + line_num_width;
            let display_line: String = if line.chars().count() > max_chars {
                line.chars().take(max_chars.saturating_sub(3)).collect()
            } else {
                line.to_string()
            };

            surface.draw_text(
                Point::from_xy(code_x, line_y),
                font.as_ref().clone(),
                font_size,
                &display_line,
                false,
                TextDirection::Auto,
            );

            current_y += line_height;

            if current_y + line_height > ctx.page_height_mm - ctx.margin_bottom {
                surface.finish();
                page.finish();
                page = ctx.document.start_page_with(ctx.page_settings());
                surface = page.surface();
                current_y = ctx.margin_top;
            }
        }

        // Finish page
        surface.finish();
        page.finish();

        // Save
        ctx.save(&output_path)?;

        println!("✓ Generated: {}", output_path.display());
    }

    Ok(())
}
