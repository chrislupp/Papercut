use crate::config::Config;
use crate::error::Result;
use crate::pdf::fonts::FontManager;
use crate::pdf::colors::rgb_to_paint;
use krilla::geom::{PathBuilder, Point};
use krilla::paint::Stroke;
use krilla::text::TextDirection;
use krilla::surface::Surface;
use chrono::Local;

/// Renders a cover page on the given surface
/// The caller should finish the page and start a new one after calling this
pub fn render_cover_page(
    font_manager: &mut FontManager,
    config: &Config,
    surface: &mut Surface,
    margin_left: f32,
    margin_top: f32,
    content_width: f32,
    content_height: f32,
) -> Result<()> {
    let font = font_manager.get_monospace_font()?;

    // Calculate center X position
    let center_x = margin_left + content_width / 2.0;
    let mut current_y = margin_top + 100.0; // Start with some top padding

    // Render title (large, centered)
    if !config.cover_page.title.is_empty() {
        surface.draw_text(
            Point::from_xy(center_x, current_y),
            font.as_ref().clone(),
            config.cover_page.title_font_size as f32,
            &config.cover_page.title,
            false,
            TextDirection::Auto,
        );
        current_y += config.cover_page.title_font_size as f32 + 30.0;
    }

    // Render description (wrapped if needed)
    if !config.cover_page.description.is_empty() {
        let text_font_size = config.cover_page.text_font_size as f32;
        let line_height = text_font_size * 1.5;

        // Simple line wrapping - split by newlines and long lines
        let max_line_width = content_width * 0.8; // Use 80% of content width for description
        let approx_char_width = text_font_size * 0.6; // Approximate monospace character width
        let max_chars_per_line = (max_line_width / approx_char_width) as usize;

        for paragraph in config.cover_page.description.split('\n') {
            if paragraph.is_empty() {
                current_y += line_height / 2.0;
                continue;
            }

            // Wrap long lines
            let words: Vec<&str> = paragraph.split_whitespace().collect();
            let mut line = String::new();

            for word in words {
                let test_line = if line.is_empty() {
                    word.to_string()
                } else {
                    format!("{} {}", line, word)
                };

                if test_line.len() > max_chars_per_line && !line.is_empty() {
                    // Draw current line
                    surface.draw_text(
                        Point::from_xy(center_x, current_y),
                        font.as_ref().clone(),
                        text_font_size,
                        &line,
                        false,
                        TextDirection::Auto,
                    );
                    current_y += line_height;
                    line = word.to_string();
                } else {
                    line = test_line;
                }
            }

            // Draw remaining line
            if !line.is_empty() {
                surface.draw_text(
                    Point::from_xy(center_x, current_y),
                    font.as_ref().clone(),
                    text_font_size,
                    &line,
                    false,
                    TextDirection::Auto,
                );
                current_y += line_height;
            }
        }

        current_y += 20.0; // Extra spacing after description
    }

    // Render location/URL
    if !config.cover_page.location.is_empty() {
        let text_font_size = config.cover_page.text_font_size as f32;

        surface.draw_text(
            Point::from_xy(center_x, current_y),
            font.as_ref().clone(),
            text_font_size,
            &format!("Location: {}", config.cover_page.location),
            false,
            TextDirection::Auto,
        );
        current_y += text_font_size + 20.0;
    }

    // Render date (auto-generate if empty)
    let date_text = if config.cover_page.date.is_empty() {
        Local::now().format("%Y-%m-%d").to_string()
    } else {
        config.cover_page.date.clone()
    };

    let text_font_size = config.cover_page.text_font_size as f32;
    surface.draw_text(
        Point::from_xy(center_x, current_y),
        font.as_ref().clone(),
        text_font_size,
        &format!("Date: {}", date_text),
        false,
        TextDirection::Auto,
    );
    current_y += text_font_size + 40.0;

    // Render table of contents if enabled
    if config.cover_page.include_toc && !config.expanded_files.is_empty() {
        // Draw TOC header
        let toc_font_size = config.cover_page.text_font_size as f32 + 2.0;
        surface.draw_text(
            Point::from_xy(center_x, current_y),
            font.as_ref().clone(),
            toc_font_size,
            "Table of Contents",
            false,
            TextDirection::Auto,
        );
        current_y += toc_font_size + 15.0;

        // Draw separator line
        let separator_y = current_y;
        let separator_start_x = center_x - 100.0;
        let separator_end_x = center_x + 100.0;

        let mut path_builder = PathBuilder::new();
        path_builder.move_to(separator_start_x, separator_y);
        path_builder.line_to(separator_end_x, separator_y);
        if let Some(separator_path) = path_builder.finish() {
            surface.set_stroke(Some(Stroke {
                paint: rgb_to_paint(100, 100, 100),
                width: 0.5,
                ..Default::default()
            }));
            surface.draw_path(&separator_path);
            surface.set_stroke(None);
        }

        current_y += 20.0;

        // List files (limit to avoid overflowing the page)
        let max_toc_entries = 30;
        let list_font_size = config.cover_page.text_font_size as f32 - 1.0;
        let line_height = list_font_size * 1.4;

        for (idx, file_entry) in config.expanded_files.iter().take(max_toc_entries).enumerate() {
            let filename = file_entry.path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Unknown");

            let toc_entry = format!("{}. {}", idx + 1, filename);

            // Left-aligned for TOC entries
            let toc_x = margin_left + 50.0;
            surface.draw_text(
                Point::from_xy(toc_x, current_y),
                font.as_ref().clone(),
                list_font_size,
                &toc_entry,
                false,
                TextDirection::Auto,
            );
            current_y += line_height;

            // Check if we're running out of space
            if current_y > margin_top + content_height - 50.0 {
                break;
            }
        }

        // If there are more files than we showed
        if config.expanded_files.len() > max_toc_entries {
            let remaining = config.expanded_files.len() - max_toc_entries;
            let toc_x = margin_left + 50.0;
            surface.draw_text(
                Point::from_xy(toc_x, current_y),
                font.as_ref().clone(),
                list_font_size,
                &format!("... and {} more files", remaining),
                false,
                TextDirection::Auto,
            );
        }
    }

    Ok(())
}
