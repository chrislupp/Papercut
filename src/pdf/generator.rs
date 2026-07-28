// Papercut - Source code to PDF converter
// Copyright (C) 2025-2026 Christopher A. Lupp
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.
//
// Distribution A: This work has been cleared for public release,
// distribution unlimited, case number: AFRL-2026-0405. The views expressed
// are those of the authors and do not reflect the official guidance or
// position of the United States Government, the Department of Defense or of
// the United States Air Force.
//
// Statement from DoD: The Appearance of external hyperlinks does not
// constitute endorsement by the United States Department of Defense (DoD) of
// the linked websites, of the information, products, or services contained
// therein. The DoD does not exercise any editorial, security, or other
// control over the information you may find at these locations.

use crate::config::{Config, OutputMode};
use crate::error::{PapercutError, Result};
use crate::pdf::colors::rgb_to_paint;
#[cfg(feature = "syntax-highlighting")]
use crate::pdf::colors::syntect_to_paint;
use crate::pdf::cover_page;
use crate::pdf::header_footer::{self, HeaderFooterContext};
use crate::pdf::krilla_doc::PdfContext;
use crate::pdf::markdown_renderer::{render_markdown, MarkdownRenderContext};
use crate::warnings::WarningManager;
use chrono::Local;
use indicatif::{ProgressBar, ProgressStyle};
use krilla::geom::{PathBuilder, Point};
use krilla::num::NormalizedF32;
use krilla::paint::{Fill, Stroke};
use krilla::text::TextDirection;
use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
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
                max_chars.saturating_sub(current_line_chars)
            } else {
                max_chars
                    .saturating_sub(wrap_indent)
                    .saturating_sub(current_line_chars)
            };
            let available = available.max(1);

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
        return Err(PapercutError::InvalidConfig(format!(
            "File '{}' already exists. Use --force to overwrite files in non-interactive mode.",
            path.display()
        )));
    }

    // Interactive mode: prompt user
    print!(
        "File '{}' already exists. Overwrite? [y/N]: ",
        path.display()
    );
    io::stdout().flush().map_err(PapercutError::Io)?;

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(PapercutError::Io)?;

    Ok(input.trim().eq_ignore_ascii_case("y"))
}

fn multiple_output_paths(config: &Config) -> Vec<PathBuf> {
    let mut totals = HashMap::new();
    for file in &config.expanded_files {
        let stem = file
            .path
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or("output")
            .to_string();
        *totals.entry(stem).or_insert(0usize) += 1;
    }

    let mut occurrences = HashMap::new();
    config
        .expanded_files
        .iter()
        .map(|file| {
            let stem = file
                .path
                .file_stem()
                .and_then(|value| value.to_str())
                .unwrap_or("output")
                .to_string();
            let occurrence = occurrences.entry(stem.clone()).or_insert(0usize);
            *occurrence += 1;
            let filename = if totals[&stem] > 1 {
                format!("{}-{}.pdf", stem, occurrence)
            } else {
                format!("{}.pdf", stem)
            };
            config.output.directory.join(filename)
        })
        .collect()
}

/// Count the number of wrapped lines for a file's content
fn count_wrapped_lines(content: &str, config: &Config, ctx: &PdfContext) -> usize {
    let font_size = config.page.font_size as f32;
    let line_num_width = if config.page.line_numbers {
        font_size * 3.5
    } else {
        0.0
    };
    let available_width = ctx.content_width - line_num_width;
    let char_width = font_size * 0.6;
    let max_chars = (available_width / char_width).floor() as usize;

    let mut total_lines = 0;
    for line in content.lines() {
        if config.page.wrap_long_lines {
            let line_chars = line.chars().count();
            if line_chars == 0 {
                total_lines += 1;
            } else {
                // First line uses full width, continuation lines lose wrap_indent
                let mut remaining = line_chars;
                let mut is_first = true;
                while remaining > 0 {
                    let available = if is_first {
                        max_chars
                    } else {
                        max_chars.saturating_sub(config.page.wrap_indent)
                    };
                    if available == 0 {
                        break; // Prevent infinite loop
                    }
                    total_lines += 1;
                    remaining = remaining.saturating_sub(available);
                    is_first = false;
                }
            }
        } else {
            total_lines += 1;
        }
    }
    total_lines
}

/// Estimate how many TOC entries fit on a single page
fn estimate_toc_entries_per_page(config: &Config, ctx: &PdfContext) -> usize {
    let list_font_size = config.cover_page.text_font_size as f32;
    let line_height = list_font_size * 1.6;

    // First page has header and separator
    let toc_header_height = (config.cover_page.text_font_size as f32 + 4.0) + 15.0 + 25.0;
    let available_height = ctx.content_height - toc_header_height - 20.0;

    (available_height / line_height).floor() as usize
}

/// Document layout information for header/footer rendering
pub struct DocumentLayout {
    /// Page index where each file starts (0-indexed)
    pub file_pages: Vec<usize>,
    /// Total number of pages in the document
    pub total_pages: usize,
}

/// Calculate which page each file starts on (0-indexed) and total page count
fn calculate_document_layout(config: &Config, ctx: &PdfContext) -> Result<DocumentLayout> {
    let mut file_pages = Vec::new();
    let mut current_page: usize = 0;
    let mut current_y = ctx.margin_top;

    // Account for cover page and TOC
    if config.cover_page.enabled {
        current_page += 1; // Cover page

        // Calculate TOC pages
        if config.cover_page.include_toc && !config.expanded_files.is_empty() {
            let entries_first_page = estimate_toc_entries_per_page(config, ctx);
            // Subsequent pages have more room (no header)
            let list_font_size = config.cover_page.text_font_size as f32;
            let line_height = list_font_size * 1.6;
            let entries_per_subsequent_page =
                ((ctx.content_height - 20.0) / line_height).floor() as usize;

            let total_files = config.expanded_files.len();
            if total_files <= entries_first_page {
                current_page += 1;
            } else {
                current_page += 1; // First TOC page
                let remaining = total_files - entries_first_page;
                if entries_per_subsequent_page > 0 {
                    current_page += remaining.div_ceil(entries_per_subsequent_page);
                }
            }
        }

        current_y = ctx.margin_top; // Reset for content pages
    }

    let font_size = config.page.font_size as f32;
    let line_height = font_size * config.page.line_spacing;
    let file_header_height = 8.0 + 25.0 + 10.0; // separator spacing + header + separator spacing

    for file_entry in &config.expanded_files {
        // Check if we need a new page before this file
        // File header needs: separator + header text + separator + first line
        let min_space_needed = file_header_height + line_height;
        if current_y + min_space_needed > ctx.page_height_mm - ctx.margin_bottom {
            current_page += 1;
            current_y = ctx.margin_top;
        }

        // Record this file's start page
        file_pages.push(current_page);

        // Calculate space used by this file
        let content = fs::read_to_string(&file_entry.path).unwrap_or_default();
        let line_count = count_wrapped_lines(&content, config, ctx);

        // Add file header space
        current_y += file_header_height;

        // Process each line
        for _ in 0..line_count {
            current_y += line_height;
            if current_y + line_height > ctx.page_height_mm - ctx.margin_bottom {
                current_page += 1;
                current_y = ctx.margin_top;
            }
        }

        // End-of-file spacing
        current_y += 10.0 + 15.0; // separator + spacing after file
    }

    // Total pages is current_page + 1 (since current_page is 0-indexed)
    let total_pages = current_page + 1;

    Ok(DocumentLayout {
        file_pages,
        total_pages,
    })
}

/// Main entry point for PDF generation
pub fn generate(
    config: Config,
    verbose: bool,
    force: bool,
    warning_manager: Arc<WarningManager>,
) -> Result<()> {
    match config.output.mode {
        OutputMode::Single => generate_single_pdf(config, verbose, force, warning_manager),
        OutputMode::Multiple => generate_multiple_pdfs(config, verbose, force, warning_manager),
    }
}

/// Generate a single PDF containing all files
fn generate_single_pdf(
    config: Config,
    verbose: bool,
    force: bool,
    warning_manager: Arc<WarningManager>,
) -> Result<()> {
    if verbose {
        println!(
            "Generating single PDF with {} files",
            config.expanded_files.len()
        );
    }

    let output_path = config.output.directory.join(&config.output.filename);
    if !should_overwrite_file(&output_path, force)? {
        println!("Skipping file.");
        return Ok(());
    }

    let mut ctx = PdfContext::new(config.clone(), Arc::clone(&warning_manager))?;

    // Set PDF metadata (falls back to cover page values)
    let effective_metadata = config.effective_metadata();
    ctx.set_metadata(&effective_metadata);

    // Pre-calculate document layout (page indices and total pages)
    let layout = calculate_document_layout(&config, &ctx)?;

    let font = ctx
        .font_manager
        .get_monospace_font(config.page.font_family.as_deref())?;
    let hf_font = ctx.font_manager.get_header_footer_font()?;

    // Initialize header/footer state
    let mut current_page_num: usize = 1;
    let total_pages = layout.total_pages;
    let date_str = Local::now().format("%Y-%m-%d").to_string();
    let mut current_filename = String::new();

    // Start first page
    let mut page = ctx.document.start_page_with(ctx.page_settings());
    let mut surface = page.surface();
    let mut current_y = ctx.margin_top;

    // Render cover page if enabled
    if config.cover_page.enabled {
        // Use cover page header/footer if specified, otherwise use main header/footer
        let cover_header = config.cover_page.header.as_ref().unwrap_or(&config.header);
        let cover_footer = config.cover_page.footer.as_ref().unwrap_or(&config.footer);

        // Render header on cover page
        let hf_ctx = HeaderFooterContext {
            page_number: current_page_num,
            total_pages,
            current_filename: &current_filename,
            date: &date_str,
        };
        header_footer::render_header(
            &mut surface,
            hf_font.clone(),
            cover_header,
            &hf_ctx,
            ctx.margin_left,
            ctx.margin_top,
            ctx.content_width,
            ctx.page_width_mm,
            ctx.margin_right,
        )?;

        cover_page::render_cover_page(
            &mut ctx.font_manager,
            &config,
            &mut surface,
            ctx.margin_left,
            ctx.margin_top,
            ctx.content_width,
            ctx.content_height,
        )?;

        // Render footer on cover page
        header_footer::render_footer(
            &mut surface,
            hf_font.clone(),
            cover_footer,
            &hf_ctx,
            ctx.margin_left,
            ctx.page_height_mm,
            ctx.margin_bottom,
            ctx.content_width,
            ctx.page_width_mm,
            ctx.margin_right,
        )?;

        // Finish cover page and start a new page
        surface.finish();
        page.finish();
        current_page_num += 1;

        // Render TOC on separate page(s) if enabled
        if cover_page::should_render_toc(&config) {
            let total_files = cover_page::get_toc_file_count(&config);
            let mut toc_start_index = 0;

            while toc_start_index < total_files {
                page = ctx.document.start_page_with(ctx.page_settings());
                surface = page.surface();

                // Render header on TOC page
                let hf_ctx = HeaderFooterContext {
                    page_number: current_page_num,
                    total_pages,
                    current_filename: &current_filename,
                    date: &date_str,
                };
                header_footer::render_header(
                    &mut surface,
                    hf_font.clone(),
                    &config.header,
                    &hf_ctx,
                    ctx.margin_left,
                    ctx.margin_top,
                    ctx.content_width,
                    ctx.page_width_mm,
                    ctx.margin_right,
                )?;

                let (files_rendered, toc_links) = cover_page::render_toc_page(
                    &mut ctx.font_manager,
                    &config,
                    &mut surface,
                    ctx.margin_left,
                    ctx.margin_top,
                    ctx.content_width,
                    ctx.content_height,
                    toc_start_index,
                    &layout.file_pages,
                )?;

                // Render footer on TOC page
                header_footer::render_footer(
                    &mut surface,
                    hf_font.clone(),
                    &config.footer,
                    &hf_ctx,
                    ctx.margin_left,
                    ctx.page_height_mm,
                    ctx.margin_bottom,
                    ctx.content_width,
                    ctx.page_width_mm,
                    ctx.margin_right,
                )?;

                // Finish surface before adding annotations to page
                surface.finish();

                // Add link annotations to the page
                cover_page::add_toc_annotations(&mut page, toc_links, ctx.margin_left);

                page.finish();
                current_page_num += 1;

                toc_start_index += files_rendered;

                // Safety check to prevent infinite loop
                if files_rendered == 0 {
                    break;
                }
            }
        }

        page = ctx.document.start_page_with(ctx.page_settings());
        surface = page.surface();
        current_y = ctx.margin_top;

        // Render header on first content page
        let hf_ctx = HeaderFooterContext {
            page_number: current_page_num,
            total_pages,
            current_filename: &current_filename,
            date: &date_str,
        };
        header_footer::render_header(
            &mut surface,
            hf_font.clone(),
            &config.header,
            &hf_ctx,
            ctx.margin_left,
            ctx.margin_top,
            ctx.content_width,
            ctx.page_width_mm,
            ctx.margin_right,
        )?;
    } else {
        // No cover page - render header on first page
        let hf_ctx = HeaderFooterContext {
            page_number: current_page_num,
            total_pages,
            current_filename: &current_filename,
            date: &date_str,
        };
        header_footer::render_header(
            &mut surface,
            hf_font.clone(),
            &config.header,
            &hf_ctx,
            ctx.margin_left,
            ctx.margin_top,
            ctx.content_width,
            ctx.page_width_mm,
            ctx.margin_right,
        )?;
    }

    // Render markdown report if enabled (after cover/TOC, before source code)
    if config.markdown_report.enabled && !config.markdown_report.path.as_os_str().is_empty() {
        // Finish current page before markdown rendering takes over
        header_footer::render_footer(
            &mut surface,
            hf_font.clone(),
            &config.footer,
            &HeaderFooterContext {
                page_number: current_page_num,
                total_pages,
                current_filename: &current_filename,
                date: &date_str,
            },
            ctx.margin_left,
            ctx.page_height_mm,
            ctx.margin_bottom,
            ctx.content_width,
            ctx.page_width_mm,
            ctx.margin_right,
        )?;
        surface.finish();
        page.finish();

        // Determine base directory for resolving relative image paths
        let base_dir = config
            .markdown_report
            .path
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

        if verbose {
            println!(
                "  Rendering markdown report: {}",
                config.markdown_report.path.display()
            );
        }

        // Create markdown render context
        let md_ctx = MarkdownRenderContext {
            base_dir,
            page_settings: ctx.page_settings(),
            margin_left: ctx.margin_left,
            margin_top: ctx.margin_top,
            margin_right: ctx.margin_right,
            margin_bottom: ctx.margin_bottom,
            content_width: ctx.content_width,
            content_height: ctx.content_height,
            page_width: ctx.page_width_mm,
            page_height: ctx.page_height_mm,
            start_page_num: current_page_num + 1,
            total_pages,
            date_str: date_str.clone(),
        };

        // Render markdown
        let (_pages_created, final_page) = render_markdown(
            &mut ctx.document,
            &mut ctx.font_manager,
            &config,
            &config.markdown_report.path,
            md_ctx,
        )?;
        current_page_num = final_page;

        // Start new page for source code
        page = ctx.document.start_page_with(ctx.page_settings());
        surface = page.surface();
        current_y = ctx.margin_top;
        current_page_num += 1;

        // Render header on first source code page
        let hf_ctx = HeaderFooterContext {
            page_number: current_page_num,
            total_pages,
            current_filename: &current_filename,
            date: &date_str,
        };
        header_footer::render_header(
            &mut surface,
            hf_font.clone(),
            &config.header,
            &hf_ctx,
            ctx.margin_left,
            ctx.margin_top,
            ctx.content_width,
            ctx.page_width_mm,
            ctx.margin_right,
        )?;
    }

    // Create progress bar if processing multiple files and not in verbose mode
    let progress = if !verbose && config.expanded_files.len() > 1 {
        let pb = ProgressBar::new(config.expanded_files.len() as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {msg}")
                .expect("Invalid progress bar template")
                .progress_chars("=>-"),
        );
        Some(pb)
    } else {
        None
    };

    // Process each file
    for (idx, file_entry) in config.expanded_files.iter().enumerate() {
        if verbose {
            println!(
                "  Processing file {}/{}: {}",
                idx + 1,
                config.expanded_files.len(),
                file_entry.path.display()
            );
        } else if let Some(ref pb) = progress {
            pb.set_message(format!(
                "{}",
                file_entry
                    .path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
            ));
        }

        // Check file size before reading
        const MAX_FILE_SIZE_WARNING: u64 = 100 * 1024 * 1024; // 100MB
        if let Ok(metadata) = fs::metadata(&file_entry.path) {
            let file_size = metadata.len();
            if file_size > MAX_FILE_SIZE_WARNING {
                warning_manager.warnf(
                    crate::warnings::WarningCategory::Filesystem,
                    format!(
                        "File '{}' is very large ({:.2} MB). Processing may be slow or use significant memory.",
                        file_entry.path.display(),
                        file_size as f64 / (1024.0 * 1024.0)
                    )
                );
            }
        }

        // Read file content
        let content =
            fs::read_to_string(&file_entry.path).map_err(|e| PapercutError::FileRead {
                path: file_entry.path.display().to_string(),
                source: e,
            })?;

        // Get file path for header and update current filename for header/footer
        let file_path_str = file_entry.display_name();
        current_filename = file_entry.path.display().to_string();

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
            &file_path_str,
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
            match highlighting::highlight_code_styled(
                &content,
                &file_entry.path,
                &config.syntax_highlighting.theme,
                &config.syntax_highlighting.custom_syntaxes,
                &warning_manager,
            ) {
                Ok(result) => Some(result),
                Err(e) => {
                    warning_manager.warnf(
                        crate::warnings::WarningCategory::Highlighting,
                        format!(
                            "Syntax highlighting failed for '{}': {}. Falling back to plain text.",
                            file_entry.path.display(),
                            e
                        ),
                    );
                    None
                }
            }
        } else {
            None
        };

        #[cfg(feature = "syntax-highlighting")]
        let rendered_highlighted = if let Some(styled_lines) = highlighted {
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

                        // Render footer before finishing page
                        let hf_ctx = HeaderFooterContext {
                            page_number: current_page_num,
                            total_pages,
                            current_filename: &current_filename,
                            date: &date_str,
                        };
                        header_footer::render_footer(
                            &mut surface,
                            hf_font.clone(),
                            &config.footer,
                            &hf_ctx,
                            ctx.margin_left,
                            ctx.page_height_mm,
                            ctx.margin_bottom,
                            ctx.content_width,
                            ctx.page_width_mm,
                            ctx.margin_right,
                        )?;

                        surface.finish();
                        page.finish();
                        current_page_num += 1;

                        page = ctx.document.start_page_with(ctx.page_settings());
                        surface = page.surface();
                        current_y = ctx.margin_top;
                        page_segment_start_y = ctx.margin_top;

                        // Render header on new page
                        let hf_ctx = HeaderFooterContext {
                            page_number: current_page_num,
                            total_pages,
                            current_filename: &current_filename,
                            date: &date_str,
                        };
                        header_footer::render_header(
                            &mut surface,
                            hf_font.clone(),
                            &config.header,
                            &hf_ctx,
                            ctx.margin_left,
                            ctx.margin_top,
                            ctx.content_width,
                            ctx.page_width_mm,
                            ctx.margin_right,
                        )?;
                    }
                }
            }
            true
        } else {
            false
        };

        #[cfg(not(feature = "syntax-highlighting"))]
        let rendered_highlighted = false;

        if !rendered_highlighted {
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

                        // Render footer before finishing page
                        let hf_ctx = HeaderFooterContext {
                            page_number: current_page_num,
                            total_pages,
                            current_filename: &current_filename,
                            date: &date_str,
                        };
                        header_footer::render_footer(
                            &mut surface,
                            hf_font.clone(),
                            &config.footer,
                            &hf_ctx,
                            ctx.margin_left,
                            ctx.page_height_mm,
                            ctx.margin_bottom,
                            ctx.content_width,
                            ctx.page_width_mm,
                            ctx.margin_right,
                        )?;

                        surface.finish();
                        page.finish();
                        current_page_num += 1;

                        page = ctx.document.start_page_with(ctx.page_settings());
                        surface = page.surface();
                        current_y = ctx.margin_top;
                        page_segment_start_y = ctx.margin_top;

                        // Render header on new page
                        let hf_ctx = HeaderFooterContext {
                            page_number: current_page_num,
                            total_pages,
                            current_filename: &current_filename,
                            date: &date_str,
                        };
                        header_footer::render_header(
                            &mut surface,
                            hf_font.clone(),
                            &config.header,
                            &hf_ctx,
                            ctx.margin_left,
                            ctx.margin_top,
                            ctx.content_width,
                            ctx.page_width_mm,
                            ctx.margin_right,
                        )?;
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

        // Update progress
        if let Some(ref pb) = progress {
            pb.inc(1);
        }
    }

    // Finish progress bar
    if let Some(pb) = progress {
        pb.finish_with_message("Processing complete");
    }

    // Render footer on last page
    let hf_ctx = HeaderFooterContext {
        page_number: current_page_num,
        total_pages,
        current_filename: &current_filename,
        date: &date_str,
    };
    header_footer::render_footer(
        &mut surface,
        hf_font.clone(),
        &config.footer,
        &hf_ctx,
        ctx.margin_left,
        ctx.page_height_mm,
        ctx.margin_bottom,
        ctx.content_width,
        ctx.page_width_mm,
        ctx.margin_right,
    )?;

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
fn generate_multiple_pdfs(
    config: Config,
    verbose: bool,
    force: bool,
    warning_manager: Arc<WarningManager>,
) -> Result<()> {
    if verbose {
        println!("Generating {} separate PDFs", config.expanded_files.len());
    }

    let output_paths = multiple_output_paths(&config);
    let mut selected_outputs = Vec::with_capacity(output_paths.len());
    for output_path in output_paths {
        if should_overwrite_file(&output_path, force)? {
            selected_outputs.push(Some(output_path));
        } else {
            println!("Skipping file: {}", output_path.display());
            selected_outputs.push(None);
        }
    }

    // Create progress bar if processing multiple files and not in verbose mode
    let progress = if !verbose && config.expanded_files.len() > 1 {
        let pb = ProgressBar::new(config.expanded_files.len() as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {msg}")
                .expect("Invalid progress bar template")
                .progress_chars("=>-"),
        );
        Some(pb)
    } else {
        None
    };

    for (idx, (file_entry, output_path)) in config
        .expanded_files
        .iter()
        .zip(selected_outputs)
        .enumerate()
    {
        let Some(output_path) = output_path else {
            continue;
        };
        if verbose {
            println!(
                "  Processing file {}/{}: {}",
                idx + 1,
                config.expanded_files.len(),
                file_entry.path.display()
            );
        } else if let Some(ref pb) = progress {
            pb.set_message(format!(
                "{}",
                file_entry
                    .path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
            ));
        }

        let mut ctx = PdfContext::new(config.clone(), Arc::clone(&warning_manager))?;

        // Set PDF metadata (falls back to cover page values)
        let effective_metadata = config.effective_metadata();
        ctx.set_metadata(&effective_metadata);

        let font = ctx
            .font_manager
            .get_monospace_font(config.page.font_family.as_deref())?;
        let hf_font = ctx.font_manager.get_header_footer_font()?;

        // For multiple mode, calculate pages for this single file
        // Create a temporary config with just this file to calculate layout
        let single_file_line_count = {
            let content = fs::read_to_string(&file_entry.path).unwrap_or_default();
            count_wrapped_lines(&content, &config, &ctx)
        };
        let font_size = config.page.font_size as f32;
        let line_height = font_size * config.page.line_spacing;
        let lines_per_page = ((ctx.content_height) / line_height).floor() as usize;
        let total_pages = if lines_per_page > 0 {
            let content_pages = single_file_line_count.div_ceil(lines_per_page);
            if config.cover_page.enabled {
                content_pages + 1
            } else {
                content_pages.max(1)
            }
        } else {
            1
        };

        // Initialize header/footer state for this file
        let mut current_page_num: usize = 1;
        let date_str = Local::now().format("%Y-%m-%d").to_string();
        let current_filename = file_entry.path.display().to_string();

        // Start page
        let mut page = ctx.document.start_page_with(ctx.page_settings());
        let mut surface = page.surface();
        let mut current_y = ctx.margin_top;

        // Render cover page if enabled (skip TOC in multiple mode - each file is separate)
        if config.cover_page.enabled {
            // Use cover page header/footer if specified, otherwise use main header/footer
            let cover_header = config.cover_page.header.as_ref().unwrap_or(&config.header);
            let cover_footer = config.cover_page.footer.as_ref().unwrap_or(&config.footer);

            // Render header on cover page
            let hf_ctx = HeaderFooterContext {
                page_number: current_page_num,
                total_pages,
                current_filename: &current_filename,
                date: &date_str,
            };
            header_footer::render_header(
                &mut surface,
                hf_font.clone(),
                cover_header,
                &hf_ctx,
                ctx.margin_left,
                ctx.margin_top,
                ctx.content_width,
                ctx.page_width_mm,
                ctx.margin_right,
            )?;

            cover_page::render_cover_page(
                &mut ctx.font_manager,
                &config,
                &mut surface,
                ctx.margin_left,
                ctx.margin_top,
                ctx.content_width,
                ctx.content_height,
            )?;

            // Render footer on cover page
            header_footer::render_footer(
                &mut surface,
                hf_font.clone(),
                cover_footer,
                &hf_ctx,
                ctx.margin_left,
                ctx.page_height_mm,
                ctx.margin_bottom,
                ctx.content_width,
                ctx.page_width_mm,
                ctx.margin_right,
            )?;

            // Finish cover page and start a new page for content
            surface.finish();
            page.finish();
            current_page_num += 1;

            page = ctx.document.start_page_with(ctx.page_settings());
            surface = page.surface();
            current_y = ctx.margin_top;

            // Render header on content page
            let hf_ctx = HeaderFooterContext {
                page_number: current_page_num,
                total_pages,
                current_filename: &current_filename,
                date: &date_str,
            };
            header_footer::render_header(
                &mut surface,
                hf_font.clone(),
                &config.header,
                &hf_ctx,
                ctx.margin_left,
                ctx.margin_top,
                ctx.content_width,
                ctx.page_width_mm,
                ctx.margin_right,
            )?;
        } else {
            // No cover page - render header on first page
            let hf_ctx = HeaderFooterContext {
                page_number: current_page_num,
                total_pages,
                current_filename: &current_filename,
                date: &date_str,
            };
            header_footer::render_header(
                &mut surface,
                hf_font.clone(),
                &config.header,
                &hf_ctx,
                ctx.margin_left,
                ctx.margin_top,
                ctx.content_width,
                ctx.page_width_mm,
                ctx.margin_right,
            )?;
        }

        // Check file size before reading
        const MAX_FILE_SIZE_WARNING: u64 = 100 * 1024 * 1024; // 100MB
        if let Ok(metadata) = fs::metadata(&file_entry.path) {
            let file_size = metadata.len();
            if file_size > MAX_FILE_SIZE_WARNING {
                warning_manager.warnf(
                    crate::warnings::WarningCategory::Filesystem,
                    format!(
                        "File '{}' is very large ({:.2} MB). Processing may be slow or use significant memory.",
                        file_entry.path.display(),
                        file_size as f64 / (1024.0 * 1024.0)
                    )
                );
            }
        }

        // Read file content
        let content =
            fs::read_to_string(&file_entry.path).map_err(|e| PapercutError::FileRead {
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

                    // Render footer before finishing page
                    let hf_ctx = HeaderFooterContext {
                        page_number: current_page_num,
                        total_pages,
                        current_filename: &current_filename,
                        date: &date_str,
                    };
                    header_footer::render_footer(
                        &mut surface,
                        hf_font.clone(),
                        &config.footer,
                        &hf_ctx,
                        ctx.margin_left,
                        ctx.page_height_mm,
                        ctx.margin_bottom,
                        ctx.content_width,
                        ctx.page_width_mm,
                        ctx.margin_right,
                    )?;

                    surface.finish();
                    page.finish();
                    current_page_num += 1;

                    page = ctx.document.start_page_with(ctx.page_settings());
                    surface = page.surface();
                    current_y = ctx.margin_top;
                    page_segment_start_y = ctx.margin_top;

                    // Render header on new page
                    let hf_ctx = HeaderFooterContext {
                        page_number: current_page_num,
                        total_pages,
                        current_filename: &current_filename,
                        date: &date_str,
                    };
                    header_footer::render_header(
                        &mut surface,
                        hf_font.clone(),
                        &config.header,
                        &hf_ctx,
                        ctx.margin_left,
                        ctx.margin_top,
                        ctx.content_width,
                        ctx.page_width_mm,
                        ctx.margin_right,
                    )?;
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

        // Render footer on last page
        let hf_ctx = HeaderFooterContext {
            page_number: current_page_num,
            total_pages,
            current_filename: &current_filename,
            date: &date_str,
        };
        header_footer::render_footer(
            &mut surface,
            hf_font.clone(),
            &config.footer,
            &hf_ctx,
            ctx.margin_left,
            ctx.page_height_mm,
            ctx.margin_bottom,
            ctx.content_width,
            ctx.page_width_mm,
            ctx.margin_right,
        )?;

        // Finish page
        surface.finish();
        page.finish();

        // Save
        ctx.save(&output_path)?;

        println!("✓ Generated: {}", output_path.display());

        // Update progress
        if let Some(ref pb) = progress {
            pb.inc(1);
        }
    }

    // Finish progress bar
    if let Some(pb) = progress {
        pb.finish_and_clear();
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disambiguates_duplicate_output_stems() {
        let mut config: Config = serde_saphyr::from_str(
            r#"
output:
  mode: multiple
  directory: output
files:
  - path: one/config.rs
"#,
        )
        .expect("config structure should deserialize");
        config.expanded_files = vec![
            crate::config::ExpandedFileEntry {
                path: PathBuf::from("one/config.rs"),
                title: None,
            },
            crate::config::ExpandedFileEntry {
                path: PathBuf::from("two/config.rs"),
                title: None,
            },
        ];

        assert_eq!(
            multiple_output_paths(&config),
            vec![
                PathBuf::from("output/config-1.pdf"),
                PathBuf::from("output/config-2.pdf")
            ]
        );
    }
}
