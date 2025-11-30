use crate::config::HeaderFooterConfig;
use crate::error::Result;
use krilla::geom::Point;
use krilla::text::{Font, TextDirection};
use krilla::surface::Surface;
use std::sync::Arc;

/// Context for rendering headers/footers on a single page
pub struct HeaderFooterContext<'a> {
    /// Current page number (1-indexed)
    pub page_number: usize,
    /// Total number of pages in the document
    pub total_pages: usize,
    /// Current filename being rendered (may be empty for cover/TOC pages)
    pub current_filename: &'a str,
    /// Current date string (YYYY-MM-DD format)
    pub date: &'a str,
}

/// Render header on the given surface
///
/// Parameters:
/// - margin_left, margin_top, content_width: default page margins
/// - page_width: total page width (needed for margin override calculations)
#[allow(clippy::too_many_arguments)]
pub fn render_header(
    surface: &mut Surface,
    font: Arc<Font>,
    config: &HeaderFooterConfig,
    ctx: &HeaderFooterContext,
    margin_left: f32,
    margin_top: f32,
    _content_width: f32,
    page_width: f32,
    margin_right: f32,
) -> Result<()> {
    if !config.enabled {
        return Ok(());
    }

    // Apply margin overrides if specified
    let effective_margin_left = config.margins.as_ref()
        .and_then(|m| m.left.as_ref())
        .map(|v| v.as_points())
        .unwrap_or(margin_left);

    let effective_margin_top = config.margins.as_ref()
        .and_then(|m| m.top.as_ref())
        .map(|v| v.as_points())
        .unwrap_or(margin_top);

    let effective_margin_right = config.margins.as_ref()
        .and_then(|m| m.right.as_ref())
        .map(|v| v.as_points())
        .unwrap_or(margin_right);

    let effective_content_width = page_width - effective_margin_left - effective_margin_right;

    let font_size = config.font_size as f32;
    let line_height = font_size * 1.2;

    // Check if using full-width text mode
    if !config.text.is_empty() {
        let text = substitute_variables(&config.text, ctx);
        let lines = wrap_text(&text, effective_content_width, font_size);

        if lines.is_empty() {
            return Ok(());
        }

        // Calculate starting Y position - center the block vertically in the top margin
        let total_height = lines.len() as f32 * line_height;
        let start_y = (effective_margin_top - total_height) / 2.0 + font_size;

        // Full-width text always uses left alignment (justified left)
        for (i, line) in lines.iter().enumerate() {
            let y = start_y + i as f32 * line_height;
            surface.draw_text(
                Point::from_xy(effective_margin_left, y),
                font.as_ref().clone(),
                font_size,
                line,
                false,
                TextDirection::Auto,
            );
        }

        return Ok(());
    }

    // Fall back to left/center/right mode
    if config.left.is_empty() && config.center.is_empty() && config.right.is_empty() {
        return Ok(());
    }

    // Calculate available width for each section (divide into thirds)
    let section_width = effective_content_width / 3.0;

    // Wrap text for each section
    let left_text = substitute_variables(&config.left, ctx);
    let center_text = substitute_variables(&config.center, ctx);
    let right_text = substitute_variables(&config.right, ctx);

    let left_lines = wrap_text(&left_text, section_width, font_size);
    let center_lines = wrap_text(&center_text, section_width, font_size);
    let right_lines = wrap_text(&right_text, section_width, font_size);

    // Find maximum number of lines to determine total height
    let max_lines = left_lines.len().max(center_lines.len()).max(right_lines.len());
    if max_lines == 0 {
        return Ok(());
    }

    // Calculate starting Y position - center the block vertically in the top margin
    let total_height = max_lines as f32 * line_height;
    let start_y = (effective_margin_top - total_height) / 2.0 + font_size;

    // Render left text (left-aligned)
    for (i, line) in left_lines.iter().enumerate() {
        let y = start_y + i as f32 * line_height;
        surface.draw_text(
            Point::from_xy(effective_margin_left, y),
            font.as_ref().clone(),
            font_size,
            line,
            false,
            TextDirection::Auto,
        );
    }

    // Render center text (center-aligned)
    let center_x = effective_margin_left + section_width;
    for (i, line) in center_lines.iter().enumerate() {
        let y = start_y + i as f32 * line_height;
        let text_width = estimate_text_width(line, font_size);
        let x = center_x + (section_width - text_width) / 2.0;
        surface.draw_text(
            Point::from_xy(x.max(center_x), y),
            font.as_ref().clone(),
            font_size,
            line,
            false,
            TextDirection::Auto,
        );
    }

    // Render right text (right-aligned)
    let right_section_start = effective_margin_left + 2.0 * section_width;
    for (i, line) in right_lines.iter().enumerate() {
        let y = start_y + i as f32 * line_height;
        let text_width = estimate_text_width(line, font_size);
        let x = right_section_start + section_width - text_width;
        surface.draw_text(
            Point::from_xy(x.max(right_section_start), y),
            font.as_ref().clone(),
            font_size,
            line,
            false,
            TextDirection::Auto,
        );
    }

    Ok(())
}

/// Render footer on the given surface
#[allow(clippy::too_many_arguments)]
pub fn render_footer(
    surface: &mut Surface,
    font: Arc<Font>,
    config: &HeaderFooterConfig,
    ctx: &HeaderFooterContext,
    margin_left: f32,
    page_height: f32,
    margin_bottom: f32,
    _content_width: f32,
    page_width: f32,
    margin_right: f32,
) -> Result<()> {
    if !config.enabled {
        return Ok(());
    }

    // Apply margin overrides if specified
    let effective_margin_left = config.margins.as_ref()
        .and_then(|m| m.left.as_ref())
        .map(|v| v.as_points())
        .unwrap_or(margin_left);

    let effective_margin_bottom = config.margins.as_ref()
        .and_then(|m| m.bottom.as_ref())
        .map(|v| v.as_points())
        .unwrap_or(margin_bottom);

    let effective_margin_right = config.margins.as_ref()
        .and_then(|m| m.right.as_ref())
        .map(|v| v.as_points())
        .unwrap_or(margin_right);

    let effective_content_width = page_width - effective_margin_left - effective_margin_right;

    let font_size = config.font_size as f32;
    let line_height = font_size * 1.2;

    // Check if using full-width text mode
    if !config.text.is_empty() {
        let text = substitute_variables(&config.text, ctx);
        let lines = wrap_text(&text, effective_content_width, font_size);

        if lines.is_empty() {
            return Ok(());
        }

        // Calculate starting Y position - center the block vertically in the bottom margin
        let total_height = lines.len() as f32 * line_height;
        let footer_area_start = page_height - effective_margin_bottom;
        let start_y = footer_area_start + (effective_margin_bottom - total_height) / 2.0 + font_size;

        // Full-width text always uses left alignment (justified left)
        for (i, line) in lines.iter().enumerate() {
            let y = start_y + i as f32 * line_height;
            surface.draw_text(
                Point::from_xy(effective_margin_left, y),
                font.as_ref().clone(),
                font_size,
                line,
                false,
                TextDirection::Auto,
            );
        }

        return Ok(());
    }

    // Fall back to left/center/right mode
    if config.left.is_empty() && config.center.is_empty() && config.right.is_empty() {
        return Ok(());
    }

    // Calculate available width for each section (divide into thirds)
    let section_width = effective_content_width / 3.0;

    // Wrap text for each section
    let left_text = substitute_variables(&config.left, ctx);
    let center_text = substitute_variables(&config.center, ctx);
    let right_text = substitute_variables(&config.right, ctx);

    let left_lines = wrap_text(&left_text, section_width, font_size);
    let center_lines = wrap_text(&center_text, section_width, font_size);
    let right_lines = wrap_text(&right_text, section_width, font_size);

    // Find maximum number of lines to determine total height
    let max_lines = left_lines.len().max(center_lines.len()).max(right_lines.len());
    if max_lines == 0 {
        return Ok(());
    }

    // Calculate starting Y position - center the block vertically in the bottom margin
    let total_height = max_lines as f32 * line_height;
    let footer_area_start = page_height - effective_margin_bottom;
    let start_y = footer_area_start + (effective_margin_bottom - total_height) / 2.0 + font_size;

    // Render left text (left-aligned)
    for (i, line) in left_lines.iter().enumerate() {
        let y = start_y + i as f32 * line_height;
        surface.draw_text(
            Point::from_xy(effective_margin_left, y),
            font.as_ref().clone(),
            font_size,
            line,
            false,
            TextDirection::Auto,
        );
    }

    // Render center text (center-aligned)
    let center_x = effective_margin_left + section_width;
    for (i, line) in center_lines.iter().enumerate() {
        let y = start_y + i as f32 * line_height;
        let text_width = estimate_text_width(line, font_size);
        let x = center_x + (section_width - text_width) / 2.0;
        surface.draw_text(
            Point::from_xy(x.max(center_x), y),
            font.as_ref().clone(),
            font_size,
            line,
            false,
            TextDirection::Auto,
        );
    }

    // Render right text (right-aligned)
    let right_section_start = effective_margin_left + 2.0 * section_width;
    for (i, line) in right_lines.iter().enumerate() {
        let y = start_y + i as f32 * line_height;
        let text_width = estimate_text_width(line, font_size);
        let x = right_section_start + section_width - text_width;
        surface.draw_text(
            Point::from_xy(x.max(right_section_start), y),
            font.as_ref().clone(),
            font_size,
            line,
            false,
            TextDirection::Auto,
        );
    }

    Ok(())
}

/// Substitute template variables in text
fn substitute_variables(text: &str, ctx: &HeaderFooterContext) -> String {
    text.replace("{page}", &ctx.page_number.to_string())
        .replace("{total}", &ctx.total_pages.to_string())
        .replace("{filename}", ctx.current_filename)
        .replace("{date}", ctx.date)
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
    (width / (font_size * 0.5)) as usize
}

/// Wrap text to fit within a given width
/// Respects paragraph breaks (double newlines) and treats single newlines as spaces
fn wrap_text(text: &str, max_width: f32, font_size: f32) -> Vec<String> {
    if text.is_empty() {
        return Vec::new();
    }

    let max_chars = estimate_max_chars(max_width, font_size);
    if max_chars == 0 {
        return vec![text.to_string()];
    }

    let mut lines = Vec::new();

    // Split by double newlines to get paragraphs
    let paragraphs: Vec<&str> = text.split("\n\n").collect();

    for (para_idx, paragraph) in paragraphs.iter().enumerate() {
        // Replace single newlines with spaces within a paragraph
        let normalized = paragraph.replace('\n', " ");
        let trimmed = normalized.trim();

        if trimmed.is_empty() {
            // Add empty line for paragraph break (except at start)
            if para_idx > 0 {
                lines.push(String::new());
            }
            continue;
        }

        let words: Vec<&str> = trimmed.split_whitespace().collect();
        if words.is_empty() {
            continue;
        }

        let mut current_line = String::new();

        for word in words {
            // If word itself is too long, break it
            if word.len() > max_chars {
                // Finish current line first
                if !current_line.is_empty() {
                    lines.push(current_line);
                    current_line = String::new();
                }
                // Break the long word
                let chars: Vec<char> = word.chars().collect();
                let mut i = 0;
                while i < chars.len() {
                    let end = (i + max_chars).min(chars.len());
                    lines.push(chars[i..end].iter().collect());
                    i = end;
                }
                continue;
            }

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
