use crate::config::{Config, OutputMode};
use crate::error::{PapercutError, Result};
use crate::pdf::krilla_doc::PdfContext;
use crate::pdf::colors::{rgb_to_paint, syntect_to_paint};
use crate::warnings::WarningManager;
use krilla::geom::{PathBuilder, Point};
use krilla::num::NormalizedF32;
use krilla::paint::{Fill, Stroke};
use krilla::text::TextDirection;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::sync::Arc;

#[cfg(feature = "syntax-highlighting")]
use crate::highlighting;

#[cfg(feature = "syntax-highlighting")]
/// Wrap styled segments to fit within available width
/// Returns a Vec of wrapped lines, each containing styled segments
fn wrap_styled_line(
    segments: &[highlighting::StyledSegment],
    max_chars: usize,
    wrap_indent: usize,
) -> Vec<Vec<highlighting::StyledSegment>> {
    let mut wrapped_lines = Vec::new();
    let mut current_line_segments = Vec::new();
    let mut current_line_chars = 0;
    let mut is_first_line = true;

    for segment in segments {
        let segment_chars: Vec<char> = segment.text.chars().collect();
        let mut remaining_chars = &segment_chars[..];

        while !remaining_chars.is_empty() {
            let available = if is_first_line {
                max_chars - current_line_chars
            } else {
                max_chars - wrap_indent - current_line_chars
            };

            if remaining_chars.len() <= available {
                // Entire remaining segment fits on current line
                current_line_segments.push(highlighting::StyledSegment {
                    text: remaining_chars.iter().collect(),
                    foreground: segment.foreground,
                    background: segment.background,
                    bold: segment.bold,
                    italic: segment.italic,
                    underline: segment.underline,
                });
                current_line_chars += remaining_chars.len();
                break;
            } else {
                // Need to split the segment
                let take_chars = available;
                if take_chars > 0 {
                    current_line_segments.push(highlighting::StyledSegment {
                        text: remaining_chars[..take_chars].iter().collect(),
                        foreground: segment.foreground,
                        background: segment.background,
                        bold: segment.bold,
                        italic: segment.italic,
                        underline: segment.underline,
                    });
                }

                // Move to next line
                wrapped_lines.push(current_line_segments);
                current_line_segments = Vec::new();
                current_line_chars = 0;
                is_first_line = false;
                remaining_chars = &remaining_chars[take_chars..];

                // Add indentation for wrapped line
                if wrap_indent > 0 && !remaining_chars.is_empty() {
                    current_line_segments.push(highlighting::StyledSegment {
                        text: " ".repeat(wrap_indent),
                        foreground: segment.foreground,
                        background: segment.background,
                        bold: false,
                        italic: false,
                        underline: false,
                    });
                    current_line_chars = wrap_indent;
                }
            }
        }
    }

    // Add the last line if it has content
    if !current_line_segments.is_empty() {
        wrapped_lines.push(current_line_segments);
    }

    // Return at least one empty line if input was empty
    if wrapped_lines.is_empty() {
        wrapped_lines.push(Vec::new());
    }

    wrapped_lines
}

/// Check if we should proceed with writing to a file that already exists
/// Returns Ok(true) if we should proceed, Ok(false) if we should skip, or Err on failure
fn should_overwrite_file(path: &Path, force: bool) -> Result<bool> {
    if !path.exists() {
        return Ok(true);
    }

    if force {
        return Ok(true);
    }

    // Check if stdout is a TTY (interactive terminal)
    if !is_terminal::IsTerminal::is_terminal(&io::stdout()) {
        return Err(PapercutError::InvalidConfig(
            format!(
                "File '{}' already exists. Use --force to overwrite files in non-interactive mode.",
                path.display()
            )
        ));
    }

    // Interactive mode: prompt user
    print!("File '{}' already exists. Overwrite? [y/N]: ", path.display());
    io::stdout()
        .flush()
        .map_err(|e| PapercutError::Io(e))?;

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|e| PapercutError::Io(e))?;

    Ok(input.trim().eq_ignore_ascii_case("y"))
}

/// Main entry point for PDF generation
pub fn generate(config: Config, verbose: bool, force: bool, warning_manager: Arc<WarningManager>) -> Result<()> {
    match config.output.mode {
        OutputMode::Single => generate_single_pdf(config, verbose, force, warning_manager),
        OutputMode::Multiple => generate_multiple_pdfs(config, verbose, force, warning_manager),
    }
}

/// Generate a single PDF containing all files
fn generate_single_pdf(config: Config, verbose: bool, force: bool, warning_manager: Arc<WarningManager>) -> Result<()> {
    if verbose {
        println!("Generating single PDF with {} files", config.expanded_files.len());
    }

    let output_path = config.output.directory.join(&config.output.filename);

    let mut ctx = PdfContext::new(config.clone(), Arc::clone(&warning_manager))?;
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
            .map_err(|e| PapercutError::FileRead {
                path: file_entry.path.display().to_string(),
                source: e,
            })?;

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

        // Calculate vertical line positions (left border, middle separator, right border)
        let vertical_lines: Vec<(f32, (u8, u8, u8), f32)> = {
            let mut lines = Vec::new();

            // Left border (at left edge of line numbers)
            if config.page.vertical_borders {
                lines.push((ctx.margin_left, (0, 0, 0), 0.5));
            }

            // Middle separator (between line numbers and code)
            if config.page.line_numbers && config.page.line_number_separator {
                let sep_x = ctx.margin_left + line_num_width - (font_size * 0.5);
                lines.push((sep_x, (0, 0, 0), 0.3));
            }

            // Right border (at right edge of content)
            if config.page.vertical_borders {
                let right_x = ctx.margin_left + ctx.content_width;
                lines.push((right_x, (0, 0, 0), 0.5));
            }

            lines
        };
        let mut page_segment_start_y = current_y;

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
            // Calculate max chars for wrapping
            let available_width = ctx.content_width - line_num_width;
            let char_width = font_size * 0.6;
            let max_chars = (available_width / char_width).floor() as usize;

            // Render with syntax highlighting
            for (source_line_num, line_segments) in styled_lines.iter().enumerate() {
                // Wrap the line if needed
                let wrapped_lines = if config.page.wrap_long_lines {
                    wrap_styled_line(line_segments, max_chars, config.page.wrap_indent)
                } else {
                    vec![line_segments.clone()]
                };

                // Render each wrapped line
                for (wrapped_idx, wrapped_segments) in wrapped_lines.iter().enumerate() {
                    let line_y = current_y + font_size;
                    let is_first_wrapped_line = wrapped_idx == 0;

                    // Draw line number only for the first wrapped line of each source line
                    if config.page.line_numbers && is_first_wrapped_line {
                        let line_num = format!("{:>4} ", source_line_num + 1);
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
                    for segment in wrapped_segments {
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
                        // Draw vertical lines for current page segment before finishing
                        for (line_x, color, width) in &vertical_lines {
                            let mut path_builder = PathBuilder::new();
                            path_builder.move_to(*line_x, page_segment_start_y);
                            path_builder.line_to(*line_x, current_y);
                            if let Some(path) = path_builder.finish() {
                                surface.set_stroke(Some(Stroke {
                                    paint: rgb_to_paint(color.0, color.1, color.2),
                                    width: *width,
                                    ..Default::default()
                                }));
                                surface.draw_path(&path);
                                surface.set_stroke(None);
                            }
                        }

                        surface.finish();
                        page.finish();
                        page = ctx.document.start_page_with(ctx.page_settings());
                        surface = page.surface();
                        current_y = ctx.margin_top;
                        page_segment_start_y = ctx.margin_top;
                    }
                }
            }
        } else {
            // Fallback to plain text rendering
            let lines: Vec<&str> = content.lines().collect();
            let available_width = ctx.content_width - line_num_width;
            let char_width = font_size * 0.6;
            let max_chars = (available_width / char_width).floor() as usize;

            for (source_line_num, line) in lines.iter().enumerate() {
                // Wrap or truncate the line
                let line_parts: Vec<String> = if config.page.wrap_long_lines {
                    // Wrap the line
                    let chars: Vec<char> = line.chars().collect();
                    let mut parts = Vec::new();
                    let mut remaining = &chars[..];
                    let mut is_first = true;

                    while !remaining.is_empty() {
                        let available = if is_first {
                            max_chars
                        } else {
                            max_chars.saturating_sub(config.page.wrap_indent)
                        };

                        if remaining.len() <= available {
                            parts.push(remaining.iter().collect());
                            break;
                        } else {
                            parts.push(remaining[..available].iter().collect());
                            remaining = &remaining[available..];
                            is_first = false;
                        }
                    }

                    if parts.is_empty() {
                        parts.push(String::new());
                    }

                    parts
                } else {
                    // Truncate if too long
                    let display_line: String = if line.chars().count() > max_chars {
                        line.chars().take(max_chars.saturating_sub(3)).collect()
                    } else {
                        line.to_string()
                    };
                    vec![display_line]
                };

                // Render each part (wrapped or single line)
                for (part_idx, part) in line_parts.iter().enumerate() {
                    let line_y = current_y + font_size;
                    let is_first_part = part_idx == 0;

                    // Draw line number only for the first part
                    if config.page.line_numbers && is_first_part {
                        let line_num = format!("{:>4} ", source_line_num + 1);
                        surface.draw_text(
                            Point::from_xy(ctx.margin_left, line_y),
                            font.as_ref().clone(),
                            font_size,
                            &line_num,
                            false,
                            TextDirection::Auto,
                        );
                    }

                    // Add indentation for wrapped lines
                    let display_text = if !is_first_part && config.page.wrap_indent > 0 {
                        format!("{}{}", " ".repeat(config.page.wrap_indent), part)
                    } else {
                        part.clone()
                    };

                    surface.draw_text(
                        Point::from_xy(code_x, line_y),
                        font.as_ref().clone(),
                        font_size,
                        &display_text,
                        false,
                        TextDirection::Auto,
                    );

                    current_y += line_height;

                    // Check if we need a new page
                    if current_y + line_height > ctx.page_height_mm - ctx.margin_bottom {
                        // Draw vertical lines for current page segment before finishing
                        for (line_x, color, width) in &vertical_lines {
                            let mut path_builder = PathBuilder::new();
                            path_builder.move_to(*line_x, page_segment_start_y);
                            path_builder.line_to(*line_x, current_y);
                            if let Some(path) = path_builder.finish() {
                                surface.set_stroke(Some(Stroke {
                                    paint: rgb_to_paint(color.0, color.1, color.2),
                                    width: *width,
                                    ..Default::default()
                                }));
                                surface.draw_path(&path);
                                surface.set_stroke(None);
                            }
                        }

                        surface.finish();
                        page.finish();
                        page = ctx.document.start_page_with(ctx.page_settings());
                        surface = page.surface();
                        current_y = ctx.margin_top;
                        page_segment_start_y = ctx.margin_top;
                    }
                }
            }
        }

        // Draw all vertical lines for the final page segment
        for (line_x, color, width) in &vertical_lines {
            let mut path_builder = PathBuilder::new();
            path_builder.move_to(*line_x, page_segment_start_y);
            path_builder.line_to(*line_x, current_y);
            if let Some(path) = path_builder.finish() {
                surface.set_stroke(Some(Stroke {
                    paint: rgb_to_paint(color.0, color.1, color.2),
                    width: *width,
                    ..Default::default()
                }));
                surface.draw_path(&path);
                surface.set_stroke(None);
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

    // Check if file exists and prompt user (or fail if non-interactive)
    if !should_overwrite_file(&output_path, force)? {
        println!("Skipping file.");
        return Ok(());
    }

    ctx.save(&output_path)?;

    println!("✓ Generated: {}", output_path.display());

    Ok(())
}

/// Generate multiple PDFs, one per file
fn generate_multiple_pdfs(config: Config, verbose: bool, force: bool, warning_manager: Arc<WarningManager>) -> Result<()> {
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

        let mut ctx = PdfContext::new(config.clone(), Arc::clone(&warning_manager))?;
        let font = ctx.font_manager.get_monospace_font()?;

        // Start page
        let mut page = ctx.document.start_page_with(ctx.page_settings());
        let mut surface = page.surface();
        let mut current_y = ctx.margin_top;

        // Read file content
        let content = fs::read_to_string(&file_entry.path)
            .map_err(|e| PapercutError::FileRead {
                path: file_entry.path.display().to_string(),
                source: e,
            })?;

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

        let code_x = ctx.margin_left + line_num_width;

        // Calculate vertical line positions (left border, middle separator, right border)
        let vertical_lines: Vec<(f32, (u8, u8, u8), f32)> = {
            let mut lines = Vec::new();

            // Left border (at left edge of line numbers)
            if config.page.vertical_borders {
                lines.push((ctx.margin_left, (0, 0, 0), 0.5));
            }

            // Middle separator (between line numbers and code)
            if config.page.line_numbers && config.page.line_number_separator {
                let sep_x = ctx.margin_left + line_num_width - (font_size * 0.5);
                lines.push((sep_x, (0, 0, 0), 0.3));
            }

            // Right border (at right edge of content)
            if config.page.vertical_borders {
                let right_x = ctx.margin_left + ctx.content_width;
                lines.push((right_x, (0, 0, 0), 0.5));
            }

            lines
        };
        let mut page_segment_start_y = current_y;

        for (source_line_num, line) in lines.iter().enumerate() {
            // Wrap or truncate the line
            let line_parts: Vec<String> = if config.page.wrap_long_lines {
                // Wrap the line
                let chars: Vec<char> = line.chars().collect();
                let mut parts = Vec::new();
                let mut remaining = &chars[..];
                let mut is_first = true;

                while !remaining.is_empty() {
                    let available = if is_first {
                        max_chars
                    } else {
                        max_chars.saturating_sub(config.page.wrap_indent)
                    };

                    if remaining.len() <= available {
                        parts.push(remaining.iter().collect());
                        break;
                    } else {
                        parts.push(remaining[..available].iter().collect());
                        remaining = &remaining[available..];
                        is_first = false;
                    }
                }

                if parts.is_empty() {
                    parts.push(String::new());
                }

                parts
            } else {
                // Truncate if too long
                let display_line: String = if line.chars().count() > max_chars {
                    line.chars().take(max_chars.saturating_sub(3)).collect()
                } else {
                    line.to_string()
                };
                vec![display_line]
            };

            // Render each part (wrapped or single line)
            for (part_idx, part) in line_parts.iter().enumerate() {
                let line_y = current_y + font_size;
                let is_first_part = part_idx == 0;

                // Draw line number only for the first part
                if config.page.line_numbers && is_first_part {
                    let line_num = format!("{:>4} ", source_line_num + 1);
                    surface.draw_text(
                        Point::from_xy(ctx.margin_left, line_y),
                        font.as_ref().clone(),
                        font_size,
                        &line_num,
                        false,
                        TextDirection::Auto,
                    );
                }

                // Add indentation for wrapped lines
                let display_text = if !is_first_part && config.page.wrap_indent > 0 {
                    format!("{}{}", " ".repeat(config.page.wrap_indent), part)
                } else {
                    part.clone()
                };

                surface.draw_text(
                    Point::from_xy(code_x, line_y),
                    font.as_ref().clone(),
                    font_size,
                    &display_text,
                    false,
                    TextDirection::Auto,
                );

                current_y += line_height;

                if current_y + line_height > ctx.page_height_mm - ctx.margin_bottom {
                    // Draw vertical lines for current page segment before finishing
                    for (line_x, color, width) in &vertical_lines {
                        let mut path_builder = PathBuilder::new();
                        path_builder.move_to(*line_x, page_segment_start_y);
                        path_builder.line_to(*line_x, current_y);
                        if let Some(path) = path_builder.finish() {
                            surface.set_stroke(Some(Stroke {
                                paint: rgb_to_paint(color.0, color.1, color.2),
                                width: *width,
                                ..Default::default()
                            }));
                            surface.draw_path(&path);
                            surface.set_stroke(None);
                        }
                    }

                    surface.finish();
                    page.finish();
                    page = ctx.document.start_page_with(ctx.page_settings());
                    surface = page.surface();
                    current_y = ctx.margin_top;
                    page_segment_start_y = ctx.margin_top;
                }
            }
        }

        // Draw all vertical lines for the final page segment
        for (line_x, color, width) in &vertical_lines {
            let mut path_builder = PathBuilder::new();
            path_builder.move_to(*line_x, page_segment_start_y);
            path_builder.line_to(*line_x, current_y);
            if let Some(path) = path_builder.finish() {
                surface.set_stroke(Some(Stroke {
                    paint: rgb_to_paint(color.0, color.1, color.2),
                    width: *width,
                    ..Default::default()
                }));
                surface.draw_path(&path);
                surface.set_stroke(None);
            }
        }

        // Finish page
        surface.finish();
        page.finish();

        // Check if file exists and prompt user (or fail if non-interactive)
        if !should_overwrite_file(&output_path, force)? {
            println!("Skipping file: {}", output_path.display());
            continue;
        }

        // Save
        ctx.save(&output_path)?;

        println!("✓ Generated: {}", output_path.display());
    }

    Ok(())
}
