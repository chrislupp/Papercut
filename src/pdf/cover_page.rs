use crate::config::Config;
use crate::error::Result;
use crate::pdf::fonts::FontManager;
use crate::pdf::colors::rgb_to_paint;
use krilla::annotation::{Annotation, LinkAnnotation, Target};
use krilla::destination::{Destination, XyzDestination};
use krilla::geom::{PathBuilder, Point, Rect};
use krilla::page::Page;
use krilla::paint::Stroke;
use krilla::text::TextDirection;
use krilla::surface::Surface;
use chrono::Local;

/// Renders the main cover page (title, description, location, date)
/// Returns Ok(()) - the TOC should be rendered on a separate page
pub fn render_cover_page(
    font_manager: &mut FontManager,
    config: &Config,
    surface: &mut Surface,
    margin_left: f32,
    margin_top: f32,
    content_width: f32,
    _content_height: f32,
) -> Result<()> {
    let font_family = &config.cover_page.font_family;
    let font = font_manager.get_cover_font(font_family)?;
    let bold_font = font_manager.get_cover_bold_font(font_family)?;

    let mut current_y = margin_top + 80.0; // Start with some top padding
    let text_font_size = config.cover_page.text_font_size as f32;
    let line_height = text_font_size * 1.5;

    // Render title (large, centered, bold)
    if !config.cover_page.title.is_empty() {
        let title_font_size = config.cover_page.title_font_size as f32;
        let title_width = estimate_text_width(&config.cover_page.title, title_font_size);
        let title_x = margin_left + (content_width - title_width) / 2.0;

        surface.draw_text(
            Point::from_xy(title_x.max(margin_left), current_y),
            bold_font.as_ref().clone(),
            title_font_size,
            &config.cover_page.title,
            false,
            TextDirection::Auto,
        );
        current_y += title_font_size + 20.0;
    }

    // Render date on same line as heading (form style)
    {
        let date_text = if config.cover_page.date.is_empty() {
            Local::now().format("%Y-%m-%d").to_string()
        } else {
            config.cover_page.date.clone()
        };

        // Draw "Date <value>" on same line
        let label = "Date ";
        let label_width = estimate_text_width(label, text_font_size);

        surface.draw_text(
            Point::from_xy(margin_left, current_y),
            bold_font.as_ref().clone(),
            text_font_size,
            label,
            false,
            TextDirection::Auto,
        );

        surface.draw_text(
            Point::from_xy(margin_left + label_width, current_y),
            font.as_ref().clone(),
            text_font_size,
            &date_text,
            false,
            TextDirection::Auto,
        );

        current_y += line_height + 20.0; // Spacing after date
    }

    // Render authors with bold heading
    if !config.cover_page.authors.is_empty() {
        // Draw "Author(s)" heading in bold
        surface.draw_text(
            Point::from_xy(margin_left, current_y),
            bold_font.as_ref().clone(),
            text_font_size,
            "Author(s)",
            false,
            TextDirection::Auto,
        );
        current_y += line_height + 5.0;

        // Draw authors text (supports paragraph breaks with double newlines)
        let lines = wrap_text(&config.cover_page.authors, content_width, text_font_size);
        for line in lines {
            surface.draw_text(
                Point::from_xy(margin_left, current_y),
                font.as_ref().clone(),
                text_font_size,
                &line,
                false,
                TextDirection::Auto,
            );
            current_y += line_height;
        }

        current_y += 30.0; // Extra spacing after authors
    }

    // Render description with bold heading
    if !config.cover_page.description.is_empty() {
        // Draw "Description" heading in bold
        surface.draw_text(
            Point::from_xy(margin_left, current_y),
            bold_font.as_ref().clone(),
            text_font_size,
            "Description",
            false,
            TextDirection::Auto,
        );
        current_y += line_height + 5.0;

        // Draw description text
        let lines = wrap_text(&config.cover_page.description, content_width, text_font_size);
        for line in lines {
            surface.draw_text(
                Point::from_xy(margin_left, current_y),
                font.as_ref().clone(),
                text_font_size,
                &line,
                false,
                TextDirection::Auto,
            );
            current_y += line_height;
        }

        current_y += 30.0; // Extra spacing after description
    }

    // Render location/URL with bold heading
    if !config.cover_page.location.is_empty() {
        // Draw "Location" heading in bold
        surface.draw_text(
            Point::from_xy(margin_left, current_y),
            bold_font.as_ref().clone(),
            text_font_size,
            "Location",
            false,
            TextDirection::Auto,
        );
        current_y += line_height + 5.0;

        // Draw location text
        let lines = wrap_text(&config.cover_page.location, content_width, text_font_size);
        for line in lines {
            surface.draw_text(
                Point::from_xy(margin_left, current_y),
                font.as_ref().clone(),
                text_font_size,
                &line,
                false,
                TextDirection::Auto,
            );
            current_y += line_height;
        }
    }

    Ok(())
}

/// Data for a TOC link annotation to be added after rendering
pub struct TocLink {
    pub rect: Rect,
    pub target_page: usize,
    pub target_y: f32,
}

/// Renders the table of contents on its own page(s)
/// Returns (files_rendered, annotations) - the annotations should be added to the page after surface.finish()
///
/// Parameters:
/// - `file_pages`: Pre-calculated page indices for each file (for hyperlinks)
pub fn render_toc_page(
    font_manager: &mut FontManager,
    config: &Config,
    surface: &mut Surface,
    margin_left: f32,
    margin_top: f32,
    content_width: f32,
    content_height: f32,
    start_index: usize,
    file_pages: &[usize],
) -> Result<(usize, Vec<TocLink>)> {
    if !config.cover_page.include_toc || config.expanded_files.is_empty() {
        return Ok((0, Vec::new()));
    }

    let font_family = &config.cover_page.font_family;
    let font = font_manager.get_cover_font(font_family)?;
    let bold_font = font_manager.get_cover_bold_font(font_family)?;
    let mut current_y = margin_top;
    let max_y = margin_top + content_height - 20.0;
    let mut toc_links = Vec::new();

    // Draw TOC header (only on first TOC page)
    if start_index == 0 {
        let toc_font_size = config.cover_page.text_font_size as f32 + 4.0;
        let header_text = "Table of Contents";
        let header_width = estimate_text_width(header_text, toc_font_size);
        let header_x = margin_left + (content_width - header_width) / 2.0;

        surface.draw_text(
            Point::from_xy(header_x.max(margin_left), current_y),
            bold_font.as_ref().clone(),
            toc_font_size,
            header_text,
            false,
            TextDirection::Auto,
        );
        current_y += toc_font_size + 15.0;

        // Draw separator line
        draw_separator_line(surface, margin_left, margin_left + content_width, current_y);
        current_y += 25.0;
    }

    // List files
    let list_font_size = config.cover_page.text_font_size as f32;
    let line_height = list_font_size * 1.6;
    let mut files_rendered = 0;

    for (idx, file_entry) in config.expanded_files.iter().enumerate().skip(start_index) {
        // Check if we're running out of space
        if current_y + line_height > max_y {
            // Return number of files we rendered - caller will create new page
            return Ok((files_rendered, toc_links));
        }

        // Get the display path - use relative path if available, otherwise filename
        let display_path = file_entry.path
            .to_string_lossy()
            .to_string();

        let toc_entry = format!("{:>3}. {}", idx + 1, display_path);

        // Truncate if too long for the page
        let max_chars = estimate_max_chars(content_width, list_font_size);
        let truncated_entry = if toc_entry.len() > max_chars {
            format!("{}...", &toc_entry[..max_chars.saturating_sub(3)])
        } else {
            toc_entry
        };

        surface.draw_text(
            Point::from_xy(margin_left, current_y),
            font.as_ref().clone(),
            list_font_size,
            &truncated_entry,
            false,
            TextDirection::Auto,
        );

        // Collect link annotation data for this TOC entry
        if let Some(&target_page) = file_pages.get(idx) {
            let text_width = estimate_text_width(&truncated_entry, list_font_size);
            // Create rectangle for clickable area (x, y is top-left in krilla coordinates)
            if let Some(rect) = Rect::from_xywh(
                margin_left,
                current_y - list_font_size,  // Top of text line
                text_width.min(content_width),
                line_height,
            ) {
                toc_links.push(TocLink {
                    rect,
                    target_page,
                    target_y: margin_top,
                });
            }
        }

        current_y += line_height;
        files_rendered += 1;
    }

    Ok((files_rendered, toc_links))
}

/// Add TOC link annotations to a page
pub fn add_toc_annotations(page: &mut Page, toc_links: Vec<TocLink>, margin_left: f32) {
    for link_data in toc_links {
        let destination = XyzDestination::new(
            link_data.target_page,
            Point::from_xy(margin_left, link_data.target_y),
        );

        let link = LinkAnnotation::new(
            link_data.rect,
            Target::Destination(Destination::from(destination)),
        );
        let annotation = Annotation::new_link(link, None);
        page.add_annotation(annotation);
    }
}

/// Check if TOC should be rendered
pub fn should_render_toc(config: &Config) -> bool {
    config.cover_page.include_toc && !config.expanded_files.is_empty()
}

/// Get total number of files for TOC
pub fn get_toc_file_count(config: &Config) -> usize {
    config.expanded_files.len()
}

/// Estimate text width based on font size for proportional fonts
fn estimate_text_width(text: &str, font_size: f32) -> f32 {
    // For proportional fonts like Arial, average character width
    // is approximately 0.5 * font_size (varies by character)
    text.len() as f32 * font_size * 0.5
}

/// Estimate maximum characters that fit in a given width for proportional fonts
fn estimate_max_chars(width: f32, font_size: f32) -> usize {
    // Use a conservative estimate for proportional fonts
    // Average char width ~0.5 * font_size, but we use 0.45 to be safe
    (width / (font_size * 0.45)) as usize
}

/// Wrap text to fit within a given width
/// Treats double newlines as paragraph breaks, single newlines as spaces
fn wrap_text(text: &str, max_width: f32, font_size: f32) -> Vec<String> {
    let max_chars = estimate_max_chars(max_width, font_size);
    let mut lines = Vec::new();

    // Split by double newlines to get paragraphs, treating single newlines as spaces
    let paragraphs: Vec<&str> = text.split("\n\n").collect();

    for (para_idx, paragraph) in paragraphs.iter().enumerate() {
        // Replace single newlines with spaces within a paragraph
        let normalized = paragraph.replace('\n', " ");
        let trimmed = normalized.trim();

        if trimmed.is_empty() {
            if para_idx > 0 {
                lines.push(String::new()); // Empty line between paragraphs
            }
            continue;
        }

        let words: Vec<&str> = trimmed.split_whitespace().collect();
        let mut current_line = String::new();

        for word in words {
            let test_line = if current_line.is_empty() {
                word.to_string()
            } else {
                format!("{} {}", current_line, word)
            };

            if test_line.len() > max_chars && !current_line.is_empty() {
                lines.push(current_line);
                current_line = word.to_string();
            } else {
                current_line = test_line;
            }
        }

        if !current_line.is_empty() {
            lines.push(current_line);
        }

        // Add empty line after paragraph (except for last one)
        if para_idx < paragraphs.len() - 1 {
            lines.push(String::new());
        }
    }

    lines
}

/// Draw a horizontal separator line
fn draw_separator_line(surface: &mut Surface, start_x: f32, end_x: f32, y: f32) {
    let mut path_builder = PathBuilder::new();
    path_builder.move_to(start_x, y);
    path_builder.line_to(end_x, y);
    if let Some(separator_path) = path_builder.finish() {
        surface.set_stroke(Some(Stroke {
            paint: rgb_to_paint(150, 150, 150),
            width: 0.5,
            ..Default::default()
        }));
        surface.draw_path(&separator_path);
        surface.set_stroke(None);
    }
}
