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

use crate::config::Config;
use crate::error::{PapercutError, Result};
use crate::pdf::colors::rgb_to_paint;
use crate::pdf::fonts::FontManager;
use crate::pdf::header_footer::{self, HeaderFooterContext};
use krilla::action::{Action, LinkAction};
use krilla::annotation::{Annotation, LinkAnnotation, Target};
use krilla::geom::{PathBuilder, Point, Rect, Size, Transform};
use krilla::image::Image;
use krilla::num::NormalizedF32;
use krilla::page::PageSettings;
use krilla::paint::{Fill, Stroke};
use krilla::surface::Surface;
use krilla::text::TextDirection;
use krilla::Document;
use krilla_svg::{SurfaceExt, SvgSettings};
use parley::layout::{Alignment, AlignmentOptions, PositionedLayoutItem};
use parley::style::{FontStack, LineHeight, StyleProperty};
use parley::{FontContext, Layout, LayoutContext};
use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use std::borrow::Cow;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use usvg::{Options as UsvgOptions, Tree as SvgTree};

/// Link annotation data for markdown links
struct MarkdownLink {
    rect: Rect,
    url: String,
}

/// Context for rendering markdown pages
pub struct MarkdownRenderContext {
    pub base_dir: PathBuf,
    pub page_settings: PageSettings,
    pub margin_left: f32,
    pub margin_top: f32,
    pub margin_right: f32,
    pub margin_bottom: f32,
    pub content_width: f32,
    #[allow(dead_code)]
    pub content_height: f32,
    pub page_width: f32,
    pub page_height: f32,
    pub start_page_num: usize,
    pub total_pages: usize,
    pub date_str: String,
}

/// Text style state for rich text rendering
#[derive(Clone, Default)]
struct TextStyle {
    bold: bool,
    #[allow(dead_code)]
    italic: bool,
    #[allow(dead_code)]
    code: bool,
}

/// A segment of styled text
struct StyledText {
    text: String,
    style: TextStyle,
}

/// Render a markdown file to the document
/// Returns (pages_created, final_page_number)
pub fn render_markdown(
    document: &mut Document,
    font_manager: &mut FontManager,
    config: &Config,
    markdown_path: &Path,
    ctx: MarkdownRenderContext,
) -> Result<(usize, usize)> {
    // Read markdown content
    let content = fs::read_to_string(markdown_path).map_err(|e| PapercutError::FileRead {
        path: markdown_path.display().to_string(),
        source: e,
    })?;

    // Parse markdown with common extensions
    let options =
        Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH | Options::ENABLE_FOOTNOTES;
    let parser = Parser::new_ext(&content, options);
    let events: Vec<Event> = parser.collect();

    // Get fonts for krilla drawing
    let font = font_manager.get_cover_font("Arial")?;
    let bold_font = font_manager.get_cover_bold_font("Arial")?;
    let mono_font = font_manager.get_monospace_font(None)?;
    let hf_font = font_manager.get_header_footer_font()?;

    // Initialize parley for text layout
    let mut font_cx = FontContext::new();
    let mut layout_cx = LayoutContext::new();

    // Font sizes
    let base_font_size = config.cover_page.text_font_size as f32;
    let line_height = base_font_size * 1.5;

    // State tracking
    let mut current_page_num = ctx.start_page_num;
    let mut current_y = ctx.margin_top;
    let mut pending_links: Vec<MarkdownLink> = Vec::new();

    // Markdown state
    let mut current_heading_level: Option<HeadingLevel> = None;
    let mut in_blockquote = false;
    let mut in_code_block = false;
    let mut code_block_content = String::new();
    let mut list_stack: Vec<Option<u64>> = Vec::new();
    let mut current_link_url: Option<String> = None;
    let mut link_start_x: f32 = 0.0;
    let mut link_start_y: f32 = 0.0;
    let mut current_style = TextStyle::default();
    let mut styled_segments: Vec<StyledText> = Vec::new();

    // Start first page
    let mut page = document.start_page_with(ctx.page_settings.clone());
    {
        let mut surface = page.surface();

        // Render header on first page
        render_header_helper(
            &mut surface,
            &hf_font,
            config,
            current_page_num,
            ctx.total_pages,
            &ctx.date_str,
            ctx.margin_left,
            ctx.margin_top,
            ctx.content_width,
            ctx.page_width,
            ctx.margin_right,
        )?;

        // Process all events
        for event in &events {
            match event {
                Event::Start(tag) => match tag {
                    Tag::Heading { level, .. } => {
                        // Flush any accumulated text first
                        if !styled_segments.is_empty() {
                            current_y = render_styled_text(
                                &mut surface,
                                &styled_segments,
                                &mut font_cx,
                                &mut layout_cx,
                                &font,
                                &bold_font,
                                base_font_size,
                                ctx.margin_left,
                                current_y,
                                ctx.content_width,
                            );
                            styled_segments.clear();
                        }
                        current_heading_level = Some(*level);
                        current_y += line_height * 0.5;
                    }
                    Tag::Paragraph => {
                        // Flush any accumulated text
                        if !styled_segments.is_empty() {
                            current_y = render_styled_text(
                                &mut surface,
                                &styled_segments,
                                &mut font_cx,
                                &mut layout_cx,
                                &font,
                                &bold_font,
                                base_font_size,
                                ctx.margin_left,
                                current_y,
                                ctx.content_width,
                            );
                            styled_segments.clear();
                        }
                    }
                    Tag::BlockQuote(_) => {
                        in_blockquote = true;
                    }
                    Tag::CodeBlock(_) => {
                        in_code_block = true;
                        code_block_content.clear();
                        current_y += line_height * 0.5;
                    }
                    Tag::List(start) => {
                        list_stack.push(*start);
                    }
                    Tag::Item => {
                        // Flush any accumulated text
                        if !styled_segments.is_empty() {
                            current_y = render_styled_text(
                                &mut surface,
                                &styled_segments,
                                &mut font_cx,
                                &mut layout_cx,
                                &font,
                                &bold_font,
                                base_font_size,
                                ctx.margin_left,
                                current_y,
                                ctx.content_width,
                            );
                            styled_segments.clear();
                        }

                        // Check for page break
                        if current_y + line_height > ctx.page_height - ctx.margin_bottom {
                            render_footer_helper(
                                &mut surface,
                                &hf_font,
                                config,
                                current_page_num,
                                ctx.total_pages,
                                &ctx.date_str,
                                ctx.margin_left,
                                ctx.page_height,
                                ctx.margin_bottom,
                                ctx.content_width,
                                ctx.page_width,
                                ctx.margin_right,
                            )?;
                            surface.finish();
                            add_link_annotations(&mut page, &mut pending_links);
                            page.finish();
                            current_page_num += 1;

                            page = document.start_page_with(ctx.page_settings.clone());
                            surface = page.surface();
                            current_y = ctx.margin_top;
                            render_header_helper(
                                &mut surface,
                                &hf_font,
                                config,
                                current_page_num,
                                ctx.total_pages,
                                &ctx.date_str,
                                ctx.margin_left,
                                ctx.margin_top,
                                ctx.content_width,
                                ctx.page_width,
                                ctx.margin_right,
                            )?;
                        }

                        let indent = (list_stack.len() - 1) as f32 * 20.0;
                        let bullet_x = ctx.margin_left + indent;

                        // Draw bullet or number
                        if let Some(Some(num)) = list_stack.last() {
                            let number_text = format!("{}.", num);
                            surface.draw_text(
                                Point::from_xy(bullet_x, current_y + base_font_size),
                                font.as_ref().clone(),
                                base_font_size,
                                &number_text,
                                false,
                                TextDirection::Auto,
                            );
                            if let Some(Some(ref mut n)) = list_stack.last_mut() {
                                *n += 1;
                            }
                        } else {
                            surface.draw_text(
                                Point::from_xy(bullet_x, current_y + base_font_size),
                                font.as_ref().clone(),
                                base_font_size,
                                "\u{2022}",
                                false,
                                TextDirection::Auto,
                            );
                        }
                    }
                    Tag::Emphasis => {
                        current_style.italic = true;
                    }
                    Tag::Strong => {
                        current_style.bold = true;
                    }
                    Tag::Link { dest_url, .. } => {
                        // Flush text before link
                        if !styled_segments.is_empty() {
                            current_y = render_styled_text(
                                &mut surface,
                                &styled_segments,
                                &mut font_cx,
                                &mut layout_cx,
                                &font,
                                &bold_font,
                                base_font_size,
                                ctx.margin_left,
                                current_y,
                                ctx.content_width,
                            );
                            styled_segments.clear();
                        }
                        current_link_url = Some(dest_url.to_string());
                        link_start_x = ctx.margin_left;
                        link_start_y = current_y;
                    }
                    Tag::Image { dest_url, .. } => {
                        let image_path = ctx.base_dir.join(dest_url.as_ref());
                        let extension = image_path
                            .extension()
                            .and_then(|e| e.to_str())
                            .map(|s| s.to_lowercase())
                            .unwrap_or_default();

                        if extension == "svg" {
                            // Handle SVG as vector graphics
                            if let Ok(svg_data) = fs::read(&image_path) {
                                // Parse SVG with usvg
                                let opts = UsvgOptions::default();
                                if let Ok(svg_tree) = SvgTree::from_data(&svg_data, &opts) {
                                    let max_width = ctx.content_width;
                                    let max_height = 300.0;

                                    // Get SVG dimensions from the tree
                                    let svg_size = svg_tree.size();
                                    let svg_width = svg_size.width();
                                    let svg_height = svg_size.height();

                                    let scale = (max_width / svg_width)
                                        .min(max_height / svg_height)
                                        .min(1.0);

                                    let display_width = svg_width * scale;
                                    let display_height = svg_height * scale;

                                    // Check for page break
                                    if current_y + display_height
                                        > ctx.page_height - ctx.margin_bottom
                                    {
                                        render_footer_helper(
                                            &mut surface,
                                            &hf_font,
                                            config,
                                            current_page_num,
                                            ctx.total_pages,
                                            &ctx.date_str,
                                            ctx.margin_left,
                                            ctx.page_height,
                                            ctx.margin_bottom,
                                            ctx.content_width,
                                            ctx.page_width,
                                            ctx.margin_right,
                                        )?;
                                        surface.finish();
                                        add_link_annotations(&mut page, &mut pending_links);
                                        page.finish();
                                        current_page_num += 1;

                                        page = document.start_page_with(ctx.page_settings.clone());
                                        surface = page.surface();
                                        current_y = ctx.margin_top;
                                        render_header_helper(
                                            &mut surface,
                                            &hf_font,
                                            config,
                                            current_page_num,
                                            ctx.total_pages,
                                            &ctx.date_str,
                                            ctx.margin_left,
                                            ctx.margin_top,
                                            ctx.content_width,
                                            ctx.page_width,
                                            ctx.margin_right,
                                        )?;
                                    }

                                    // Center the SVG
                                    let image_x =
                                        ctx.margin_left + (ctx.content_width - display_width) / 2.0;

                                    // Draw SVG as vector graphics
                                    // First translate to position, then apply scale
                                    let translate_transform =
                                        Transform::from_translate(image_x, current_y);
                                    surface.push_transform(&translate_transform);

                                    // Draw at scaled size
                                    if let Some(size) = Size::from_wh(display_width, display_height)
                                    {
                                        let settings = SvgSettings::default();
                                        let _ = surface.draw_svg(&svg_tree, size, settings);
                                    }
                                    surface.pop();

                                    current_y += display_height + line_height;
                                }
                            }
                        } else if let Some(image) = load_image(&image_path) {
                            // Handle raster images (PNG, JPEG, GIF, WebP)
                            let (img_width, img_height) = image.size();
                            let max_width = ctx.content_width;
                            let max_height = 300.0;

                            let scale = (max_width / img_width as f32)
                                .min(max_height / img_height as f32)
                                .min(1.0);

                            let display_width = img_width as f32 * scale;
                            let display_height = img_height as f32 * scale;

                            // Check for page break
                            if current_y + display_height > ctx.page_height - ctx.margin_bottom {
                                render_footer_helper(
                                    &mut surface,
                                    &hf_font,
                                    config,
                                    current_page_num,
                                    ctx.total_pages,
                                    &ctx.date_str,
                                    ctx.margin_left,
                                    ctx.page_height,
                                    ctx.margin_bottom,
                                    ctx.content_width,
                                    ctx.page_width,
                                    ctx.margin_right,
                                )?;
                                surface.finish();
                                add_link_annotations(&mut page, &mut pending_links);
                                page.finish();
                                current_page_num += 1;

                                page = document.start_page_with(ctx.page_settings.clone());
                                surface = page.surface();
                                current_y = ctx.margin_top;
                                render_header_helper(
                                    &mut surface,
                                    &hf_font,
                                    config,
                                    current_page_num,
                                    ctx.total_pages,
                                    &ctx.date_str,
                                    ctx.margin_left,
                                    ctx.margin_top,
                                    ctx.content_width,
                                    ctx.page_width,
                                    ctx.margin_right,
                                )?;
                            }

                            // Center the image
                            let image_x =
                                ctx.margin_left + (ctx.content_width - display_width) / 2.0;

                            // Draw image
                            let transform = Transform::from_translate(image_x, current_y);
                            surface.push_transform(&transform);
                            if let Some(size) = Size::from_wh(display_width, display_height) {
                                surface.draw_image(image, size);
                            }
                            surface.pop();

                            current_y += display_height + line_height;
                        }
                    }
                    _ => {}
                },
                Event::End(tag_end) => match tag_end {
                    TagEnd::Heading(_) => {
                        let heading_font_size = match current_heading_level {
                            Some(HeadingLevel::H1) => base_font_size * 2.0,
                            Some(HeadingLevel::H2) => base_font_size * 1.7,
                            Some(HeadingLevel::H3) => base_font_size * 1.4,
                            Some(HeadingLevel::H4) => base_font_size * 1.2,
                            Some(HeadingLevel::H5) => base_font_size * 1.1,
                            _ => base_font_size,
                        };
                        let heading_line_height = heading_font_size * 1.5;

                        // Check for page break
                        if current_y + heading_line_height > ctx.page_height - ctx.margin_bottom {
                            render_footer_helper(
                                &mut surface,
                                &hf_font,
                                config,
                                current_page_num,
                                ctx.total_pages,
                                &ctx.date_str,
                                ctx.margin_left,
                                ctx.page_height,
                                ctx.margin_bottom,
                                ctx.content_width,
                                ctx.page_width,
                                ctx.margin_right,
                            )?;
                            surface.finish();
                            add_link_annotations(&mut page, &mut pending_links);
                            page.finish();
                            current_page_num += 1;

                            page = document.start_page_with(ctx.page_settings.clone());
                            surface = page.surface();
                            current_y = ctx.margin_top;
                            render_header_helper(
                                &mut surface,
                                &hf_font,
                                config,
                                current_page_num,
                                ctx.total_pages,
                                &ctx.date_str,
                                ctx.margin_left,
                                ctx.margin_top,
                                ctx.content_width,
                                ctx.page_width,
                                ctx.margin_right,
                            )?;
                        }

                        // Combine all styled segments into plain text for heading
                        let heading_text: String =
                            styled_segments.iter().map(|s| s.text.as_str()).collect();

                        surface.draw_text(
                            Point::from_xy(ctx.margin_left, current_y + heading_font_size),
                            bold_font.as_ref().clone(),
                            heading_font_size,
                            &heading_text,
                            false,
                            TextDirection::Auto,
                        );

                        current_y += heading_line_height;
                        styled_segments.clear();
                        current_heading_level = None;
                    }
                    TagEnd::Paragraph => {
                        if !styled_segments.is_empty() {
                            let draw_x = if in_blockquote {
                                ctx.margin_left + 20.0
                            } else {
                                ctx.margin_left
                            };
                            let available_width = if in_blockquote {
                                ctx.content_width - 20.0
                            } else {
                                ctx.content_width
                            };

                            // Combine segments into full text
                            let full_text: String =
                                styled_segments.iter().map(|s| s.text.as_str()).collect();

                            // Use parley for proper line breaking
                            let lines = calculate_line_breaks(
                                &full_text,
                                &mut font_cx,
                                &mut layout_cx,
                                base_font_size,
                                available_width,
                            );

                            // Render each line
                            for line_text in lines {
                                // Check for page break
                                if current_y + line_height > ctx.page_height - ctx.margin_bottom {
                                    render_footer_helper(
                                        &mut surface,
                                        &hf_font,
                                        config,
                                        current_page_num,
                                        ctx.total_pages,
                                        &ctx.date_str,
                                        ctx.margin_left,
                                        ctx.page_height,
                                        ctx.margin_bottom,
                                        ctx.content_width,
                                        ctx.page_width,
                                        ctx.margin_right,
                                    )?;
                                    surface.finish();
                                    add_link_annotations(&mut page, &mut pending_links);
                                    page.finish();
                                    current_page_num += 1;

                                    page = document.start_page_with(ctx.page_settings.clone());
                                    surface = page.surface();
                                    current_y = ctx.margin_top;
                                    render_header_helper(
                                        &mut surface,
                                        &hf_font,
                                        config,
                                        current_page_num,
                                        ctx.total_pages,
                                        &ctx.date_str,
                                        ctx.margin_left,
                                        ctx.margin_top,
                                        ctx.content_width,
                                        ctx.page_width,
                                        ctx.margin_right,
                                    )?;
                                }

                                surface.draw_text(
                                    Point::from_xy(draw_x, current_y + base_font_size),
                                    font.as_ref().clone(),
                                    base_font_size,
                                    &line_text,
                                    false,
                                    TextDirection::Auto,
                                );
                                current_y += line_height;
                            }

                            styled_segments.clear();
                        }
                        current_y += line_height * 0.5;
                    }
                    TagEnd::BlockQuote(_) => {
                        in_blockquote = false;
                    }
                    TagEnd::CodeBlock => {
                        let code_font_size = base_font_size * 0.9;
                        let code_line_height = code_font_size * 1.3;
                        let code_lines: Vec<&str> = code_block_content.lines().collect();
                        let block_height = (code_lines.len() as f32 * code_line_height) + 10.0;

                        // Check for page break
                        if current_y + block_height > ctx.page_height - ctx.margin_bottom {
                            render_footer_helper(
                                &mut surface,
                                &hf_font,
                                config,
                                current_page_num,
                                ctx.total_pages,
                                &ctx.date_str,
                                ctx.margin_left,
                                ctx.page_height,
                                ctx.margin_bottom,
                                ctx.content_width,
                                ctx.page_width,
                                ctx.margin_right,
                            )?;
                            surface.finish();
                            add_link_annotations(&mut page, &mut pending_links);
                            page.finish();
                            current_page_num += 1;

                            page = document.start_page_with(ctx.page_settings.clone());
                            surface = page.surface();
                            current_y = ctx.margin_top;
                            render_header_helper(
                                &mut surface,
                                &hf_font,
                                config,
                                current_page_num,
                                ctx.total_pages,
                                &ctx.date_str,
                                ctx.margin_left,
                                ctx.margin_top,
                                ctx.content_width,
                                ctx.page_width,
                                ctx.margin_right,
                            )?;
                        }

                        // Draw code block background
                        if let Some(rect) = Rect::from_xywh(
                            ctx.margin_left,
                            current_y,
                            ctx.content_width,
                            block_height,
                        ) {
                            let mut path_builder = PathBuilder::new();
                            path_builder.push_rect(rect);
                            if let Some(path) = path_builder.finish() {
                                surface.set_fill(Some(Fill {
                                    paint: rgb_to_paint(245, 245, 245),
                                    opacity: NormalizedF32::ONE,
                                    rule: Default::default(),
                                }));
                                surface.draw_path(&path);
                                surface.set_fill(None);
                            }
                        }

                        current_y += 5.0;
                        for line in code_lines {
                            surface.draw_text(
                                Point::from_xy(ctx.margin_left + 10.0, current_y + code_font_size),
                                mono_font.as_ref().clone(),
                                code_font_size,
                                line,
                                false,
                                TextDirection::Auto,
                            );
                            current_y += code_line_height;
                        }

                        current_y += 5.0 + line_height * 0.5;
                        in_code_block = false;
                        code_block_content.clear();
                    }
                    TagEnd::List(_) => {
                        list_stack.pop();
                        if list_stack.is_empty() {
                            current_y += line_height * 0.5;
                        }
                    }
                    TagEnd::Item => {
                        if !styled_segments.is_empty() {
                            let indent = (list_stack.len() - 1) as f32 * 20.0;
                            let item_x = ctx.margin_left + indent + 20.0;
                            let available_width = ctx.content_width - indent - 20.0;

                            // Combine segments into full text
                            let full_text: String =
                                styled_segments.iter().map(|s| s.text.as_str()).collect();

                            // Use parley for proper line breaking
                            let lines = calculate_line_breaks(
                                &full_text,
                                &mut font_cx,
                                &mut layout_cx,
                                base_font_size,
                                available_width,
                            );

                            // Render each line
                            for line_text in lines {
                                // Check for page break
                                if current_y + line_height > ctx.page_height - ctx.margin_bottom {
                                    render_footer_helper(
                                        &mut surface,
                                        &hf_font,
                                        config,
                                        current_page_num,
                                        ctx.total_pages,
                                        &ctx.date_str,
                                        ctx.margin_left,
                                        ctx.page_height,
                                        ctx.margin_bottom,
                                        ctx.content_width,
                                        ctx.page_width,
                                        ctx.margin_right,
                                    )?;
                                    surface.finish();
                                    add_link_annotations(&mut page, &mut pending_links);
                                    page.finish();
                                    current_page_num += 1;

                                    page = document.start_page_with(ctx.page_settings.clone());
                                    surface = page.surface();
                                    current_y = ctx.margin_top;
                                    render_header_helper(
                                        &mut surface,
                                        &hf_font,
                                        config,
                                        current_page_num,
                                        ctx.total_pages,
                                        &ctx.date_str,
                                        ctx.margin_left,
                                        ctx.margin_top,
                                        ctx.content_width,
                                        ctx.page_width,
                                        ctx.margin_right,
                                    )?;
                                }

                                surface.draw_text(
                                    Point::from_xy(item_x, current_y + base_font_size),
                                    font.as_ref().clone(),
                                    base_font_size,
                                    &line_text,
                                    false,
                                    TextDirection::Auto,
                                );
                                current_y += line_height;
                            }

                            styled_segments.clear();
                        } else {
                            // Even if no text, advance past the bullet line
                            current_y += line_height;
                        }
                    }
                    TagEnd::Emphasis => {
                        current_style.italic = false;
                    }
                    TagEnd::Strong => {
                        current_style.bold = false;
                    }
                    TagEnd::Link => {
                        if let Some(url) = &current_link_url {
                            // Render link text
                            let link_text: String =
                                styled_segments.iter().map(|s| s.text.as_str()).collect();
                            let text_width = estimate_text_width(&link_text, base_font_size);

                            // Draw link text in blue
                            surface.set_fill(Some(Fill {
                                paint: rgb_to_paint(0, 102, 204),
                                opacity: NormalizedF32::ONE,
                                rule: Default::default(),
                            }));
                            surface.draw_text(
                                Point::from_xy(link_start_x, link_start_y + base_font_size),
                                font.as_ref().clone(),
                                base_font_size,
                                &link_text,
                                false,
                                TextDirection::Auto,
                            );
                            surface.set_fill(None);

                            // Draw underline
                            let underline_y = link_start_y + base_font_size + 2.0;
                            let mut path_builder = PathBuilder::new();
                            path_builder.move_to(link_start_x, underline_y);
                            path_builder.line_to(link_start_x + text_width, underline_y);
                            if let Some(path) = path_builder.finish() {
                                surface.set_stroke(Some(Stroke {
                                    paint: rgb_to_paint(0, 102, 204),
                                    width: 0.5,
                                    ..Default::default()
                                }));
                                surface.draw_path(&path);
                                surface.set_stroke(None);
                            }

                            // Store link annotation
                            if let Some(rect) =
                                Rect::from_xywh(link_start_x, link_start_y, text_width, line_height)
                            {
                                pending_links.push(MarkdownLink {
                                    rect,
                                    url: url.clone(),
                                });
                            }

                            styled_segments.clear();
                        }
                        current_link_url = None;
                    }
                    _ => {}
                },
                Event::Text(text) => {
                    if in_code_block {
                        code_block_content.push_str(text);
                    } else {
                        styled_segments.push(StyledText {
                            text: text.to_string(),
                            style: current_style.clone(),
                        });
                    }
                }
                Event::Code(code) => {
                    styled_segments.push(StyledText {
                        text: format!("`{}`", code),
                        style: TextStyle {
                            code: true,
                            ..current_style.clone()
                        },
                    });
                }
                Event::SoftBreak => {
                    styled_segments.push(StyledText {
                        text: " ".to_string(),
                        style: current_style.clone(),
                    });
                }
                Event::HardBreak => {
                    styled_segments.push(StyledText {
                        text: "\n".to_string(),
                        style: current_style.clone(),
                    });
                }
                Event::Rule => {
                    current_y += line_height * 0.5;

                    if current_y + 10.0 > ctx.page_height - ctx.margin_bottom {
                        render_footer_helper(
                            &mut surface,
                            &hf_font,
                            config,
                            current_page_num,
                            ctx.total_pages,
                            &ctx.date_str,
                            ctx.margin_left,
                            ctx.page_height,
                            ctx.margin_bottom,
                            ctx.content_width,
                            ctx.page_width,
                            ctx.margin_right,
                        )?;
                        surface.finish();
                        add_link_annotations(&mut page, &mut pending_links);
                        page.finish();
                        current_page_num += 1;

                        page = document.start_page_with(ctx.page_settings.clone());
                        surface = page.surface();
                        current_y = ctx.margin_top;
                        render_header_helper(
                            &mut surface,
                            &hf_font,
                            config,
                            current_page_num,
                            ctx.total_pages,
                            &ctx.date_str,
                            ctx.margin_left,
                            ctx.margin_top,
                            ctx.content_width,
                            ctx.page_width,
                            ctx.margin_right,
                        )?;
                    }

                    let mut path_builder = PathBuilder::new();
                    path_builder.move_to(ctx.margin_left, current_y);
                    path_builder.line_to(ctx.margin_left + ctx.content_width, current_y);
                    if let Some(path) = path_builder.finish() {
                        surface.set_stroke(Some(Stroke {
                            paint: rgb_to_paint(180, 180, 180),
                            width: 1.0,
                            ..Default::default()
                        }));
                        surface.draw_path(&path);
                        surface.set_stroke(None);
                    }
                    current_y += line_height * 0.5;
                }
                _ => {}
            }
        }

        // Finish the last page
        render_footer_helper(
            &mut surface,
            &hf_font,
            config,
            current_page_num,
            ctx.total_pages,
            &ctx.date_str,
            ctx.margin_left,
            ctx.page_height,
            ctx.margin_bottom,
            ctx.content_width,
            ctx.page_width,
            ctx.margin_right,
        )?;
        surface.finish();
    }

    add_link_annotations(&mut page, &mut pending_links);
    page.finish();

    let pages_created = current_page_num - ctx.start_page_num + 1;
    Ok((pages_created, current_page_num))
}

/// Use parley to calculate line breaks and return wrapped lines
fn calculate_line_breaks(
    text: &str,
    font_cx: &mut FontContext,
    layout_cx: &mut LayoutContext,
    font_size: f32,
    max_width: f32,
) -> Vec<String> {
    if text.trim().is_empty() {
        return vec![];
    }

    // Build layout with parley
    let mut builder = layout_cx.ranged_builder(font_cx, text, 1.0, false);

    // Push default styles
    builder.push_default(StyleProperty::FontSize(font_size));
    builder.push_default(StyleProperty::LineHeight(LineHeight::FontSizeRelative(1.5)));
    builder.push_default(StyleProperty::FontStack(FontStack::Source(Cow::Borrowed(
        "Arial",
    ))));

    // Build the layout
    let mut layout: Layout<[u8; 4]> = builder.build(text);
    layout.break_all_lines(Some(max_width));
    layout.align(
        Some(max_width),
        Alignment::Start,
        AlignmentOptions::default(),
    );

    // Extract line texts
    let mut lines = Vec::new();
    for line in layout.lines() {
        let mut line_text = String::new();
        for item in line.items() {
            if let PositionedLayoutItem::GlyphRun(run) = item {
                let text_range = run.run().text_range();
                line_text.push_str(&text[text_range]);
            }
        }
        if !line_text.is_empty() {
            lines.push(line_text);
        }
    }

    if lines.is_empty() && !text.trim().is_empty() {
        // Fallback if parley didn't produce lines
        lines.push(text.to_string());
    }

    lines
}

/// Render styled text (simplified version for headings)
#[allow(clippy::too_many_arguments)]
fn render_styled_text(
    surface: &mut Surface,
    segments: &[StyledText],
    _font_cx: &mut FontContext,
    _layout_cx: &mut LayoutContext,
    regular_font: &Arc<krilla::text::Font>,
    bold_font: &Arc<krilla::text::Font>,
    font_size: f32,
    x: f32,
    y: f32,
    _max_width: f32,
) -> f32 {
    let mut current_x = x;

    for segment in segments {
        let font = if segment.style.bold {
            bold_font.as_ref().clone()
        } else {
            regular_font.as_ref().clone()
        };

        surface.draw_text(
            Point::from_xy(current_x, y + font_size),
            font,
            font_size,
            &segment.text,
            false,
            TextDirection::Auto,
        );

        current_x += estimate_text_width(&segment.text, font_size);
    }

    y + font_size * 1.5
}

/// Helper function to render header
#[allow(clippy::too_many_arguments)]
fn render_header_helper(
    surface: &mut Surface,
    hf_font: &Arc<krilla::text::Font>,
    config: &Config,
    page_number: usize,
    total_pages: usize,
    date_str: &str,
    margin_left: f32,
    margin_top: f32,
    content_width: f32,
    page_width: f32,
    margin_right: f32,
) -> Result<()> {
    let hf_ctx = HeaderFooterContext {
        page_number,
        total_pages,
        current_filename: "",
        date: date_str,
    };
    header_footer::render_header(
        surface,
        hf_font.clone(),
        &config.header,
        &hf_ctx,
        margin_left,
        margin_top,
        content_width,
        page_width,
        margin_right,
    )
}

/// Helper function to render footer
#[allow(clippy::too_many_arguments)]
fn render_footer_helper(
    surface: &mut Surface,
    hf_font: &Arc<krilla::text::Font>,
    config: &Config,
    page_number: usize,
    total_pages: usize,
    date_str: &str,
    margin_left: f32,
    page_height: f32,
    margin_bottom: f32,
    content_width: f32,
    page_width: f32,
    margin_right: f32,
) -> Result<()> {
    let hf_ctx = HeaderFooterContext {
        page_number,
        total_pages,
        current_filename: "",
        date: date_str,
    };
    header_footer::render_footer(
        surface,
        hf_font.clone(),
        &config.footer,
        &hf_ctx,
        margin_left,
        page_height,
        margin_bottom,
        content_width,
        page_width,
        margin_right,
    )
}

/// Add link annotations to a page
fn add_link_annotations(page: &mut krilla::page::Page, pending_links: &mut Vec<MarkdownLink>) {
    for link in pending_links.drain(..) {
        if link.url.starts_with("http://") || link.url.starts_with("https://") {
            let link_action = LinkAction::new(link.url);
            let link_annotation =
                LinkAnnotation::new(link.rect, Target::Action(Action::Link(link_action)));
            page.add_annotation(Annotation::new_link(link_annotation, None));
        }
    }
}

/// Load an image from a file path
fn load_image(path: &Path) -> Option<Image> {
    let image_data = fs::read(path).ok()?;
    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase())
        .unwrap_or_default();

    match extension.as_str() {
        "png" => Image::from_png(image_data.into(), true).ok(),
        "jpg" | "jpeg" => Image::from_jpeg(image_data.into(), true).ok(),
        "gif" => Image::from_gif(image_data.into(), true).ok(),
        "webp" => Image::from_webp(image_data.into(), true).ok(),
        _ => None,
    }
}

/// Estimate text width based on font size
fn estimate_text_width(text: &str, font_size: f32) -> f32 {
    text.len() as f32 * font_size * 0.5
}
