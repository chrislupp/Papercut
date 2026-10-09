//! Translate the existing configuration to an embedded Typst document.
use crate::config::{Config, FontFamily, HeaderFooterConfig, PageSize};
use crate::error::{PapercutError, Result};
use crate::warnings::WarningManager;
use std::fmt::Write;
use std::fs;
use typst::layout::PagedDocument;
use typst_as_lib::{typst_kit_options::TypstKitFontOptions, TypstEngine};

type Assets = Vec<(String, Vec<u8>)>;

/// JSON string literals are also valid Typst strings, except JSON's control
/// escapes use a different Unicode syntax. Escape explicitly to cover all input.
pub(super) fn quote(value: &str) -> String {
    let mut out = String::from("\"");
    for c in value.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => {
                write!(out, "\\u{{{:x}}}", c as u32).unwrap();
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn header_footer(hf: &HeaderFooterConfig, config: &Config, date: &str, header: bool) -> String {
    if !hf.enabled {
        return "none".into();
    }
    let text = |s: &str| format!("hf-text({}, {})", quote(s), quote(date));
    let content = if !hf.text.is_empty() {
        text(&hf.text)
    } else {
        format!(
            "grid(columns: (1fr, 1fr, 1fr), align(left, {}), align(center, {}), align(right, {}))",
            text(&hf.left),
            text(&hf.center),
            text(&hf.right)
        )
    };
    let margins = &config.page.margins;
    let (left, right, offset) = hf
        .margins
        .as_ref()
        .map(|m| {
            (
                m.left
                    .map(|v| margins.left.as_points() - v.as_points())
                    .unwrap_or(0.0),
                m.right
                    .map(|v| margins.right.as_points() - v.as_points())
                    .unwrap_or(0.0),
                if header {
                    m.top.map(|v| v.as_points() - margins.top.as_points() / 2.0)
                } else {
                    m.bottom
                        .map(|v| margins.bottom.as_points() / 2.0 - v.as_points())
                }
                .unwrap_or(0.0),
            )
        })
        .unwrap_or((0.0, 0.0, 0.0));
    format!(
        "context move(dy: {offset}pt, pad(left: {}pt, right: {}pt, text(size: {}pt, {content})))",
        -left, -right, hf.font_size
    )
}

pub(super) fn document_source(
    config: &Config,
    warnings: &WarningManager,
) -> Result<(String, Assets)> {
    let mut out = include_str!("document.typ").to_string();
    let mut assets = Vec::new();
    let date = chrono::Local::now().format("%Y-%m-%d").to_string();
    let metadata = config.effective_metadata();
    writeln!(
        out,
        "\n#set document(title: {}, author: {}, description: {}, keywords: ({}))",
        quote(&metadata.title),
        quote(&metadata.author),
        quote(&metadata.subject),
        metadata
            .keywords
            .iter()
            .map(|k| format!("{},", quote(k)))
            .collect::<String>()
    )
    .unwrap();
    let paper = match config.page.size {
        PageSize::A4 => "a4",
        PageSize::Letter => "us-letter",
        PageSize::Legal => "us-legal",
    };
    let m = &config.page.margins;
    writeln!(out, "#set page({}, margin: (top: {}pt, bottom: {}pt, left: {}pt, right: {}pt), header-ascent: 50%, footer-descent: 50%)", quote(paper), m.top.as_points(), m.bottom.as_points(), m.left.as_points(), m.right.as_points()).unwrap();
    out.push_str("#set text(font: (\"Arial\", \"Libertinus Serif\"), size: 11pt)\n#set heading(numbering: none)\n");
    let hf = |header, footer| {
        format!(
            "#set page(header: {}, footer: {})\n",
            header_footer(header, config, &date, true),
            header_footer(footer, config, &date, false)
        )
    };
    let cover = &config.cover_page;
    if cover.enabled {
        out.push_str(&hf(
            cover.header.as_ref().unwrap_or(&config.header),
            cover.footer.as_ref().unwrap_or(&config.footer),
        ));
        writeln!(
            out,
            "#text(font: ({}, \"Libertinus Serif\"), size: {}pt)[",
            quote(&cover.font_family),
            cover.text_font_size
        )
        .unwrap();
        writeln!(out, "#align(center)[#text(size: {}pt, weight: \"bold\", {})\n#parbreak()\n#plain({})]\n#v(18pt)", cover.title_font_size, quote(&cover.title), quote(&cover.authors)).unwrap();
        // Descriptions have always been plain text, including paragraph breaks.
        for paragraph in cover.description.split("\n\n") {
            writeln!(out, "#text({})\n#parbreak()", quote(paragraph)).unwrap();
        }
        writeln!(
            out,
            "#v(18pt)\n#align(center)[#text({})\n#parbreak()\n#text({})]\n]\n#pagebreak()",
            quote(&cover.location),
            quote(if cover.date.is_empty() {
                &date
            } else {
                &cover.date
            })
        )
        .unwrap();
    }
    out.push_str(&hf(&config.header, &config.footer));
    if cover.enabled && cover.include_toc && !config.expanded_files.is_empty() {
        out.push_str(
            "#outline(title: [Table of Contents], target: heading.where(level: 1))\n#pagebreak()\n",
        );
    }
    if config.markdown_report.enabled {
        let path = &config.markdown_report.path;
        let markdown = fs::read_to_string(path).map_err(|source| PapercutError::FileRead {
            path: path.display().to_string(),
            source,
        })?;
        out.push_str(&super::markdown::convert(
            &markdown,
            path.parent().unwrap_or(std::path::Path::new(".")),
            &mut assets,
            warnings,
        )?);
        out.push_str("\n#pagebreak(weak: true)\n");
    }
    let default_font = match config.styling.font_family {
        FontFamily::Courier => "Courier New",
        _ => "DejaVu Sans Mono",
    };
    let font = config.page.font_family.as_deref().unwrap_or(default_font);
    for file in &config.expanded_files {
        let name = file.display_name();
        writeln!(
            out,
            "#filename.update({})\n#metadata((filename: {}))\n#heading(level: 1, {})",
            quote(&name),
            quote(&name),
            quote(&name)
        )
        .unwrap();
        let code = fs::read_to_string(&file.path).map_err(|source| PapercutError::FileRead {
            path: file.path.display().to_string(),
            source,
        })?;
        writeln!(out, "#[\n#set text(font: ({}, \"DejaVu Sans Mono\"), size: {}pt, ligatures: false, hyphenate: false)\n#set par(leading: {}pt, spacing: 0pt)\n#set block(spacing: 0pt)", quote(font), config.page.font_size, config.page.font_size as f32 * (config.page.line_spacing - 1.0).max(0.0)).unwrap();
        #[cfg(feature = "syntax-highlighting")]
        let highlighted = if config.syntax_highlighting.enabled {
            match crate::highlighting::highlight_code_styled(
                &code,
                &file.path,
                &config.syntax_highlighting.theme,
                &config.syntax_highlighting.custom_syntaxes,
                warnings,
            ) {
                Ok(lines) => Some(lines),
                Err(e) => {
                    warnings.warnf(crate::warnings::WarningCategory::Highlighting, e);
                    None
                }
            }
        } else {
            None
        };
        for (index, line) in code.lines().enumerate() {
            let mut spans = String::new();
            #[cfg(feature = "syntax-highlighting")]
            if let Some(segments) = highlighted.as_ref().and_then(|l| l.get(index)) {
                for span in segments {
                    let value = span
                        .text
                        .trim_end_matches(['\r', '\n'])
                        .replace('\t', "    ");
                    write!(spans, "(text: {}, color: \"#{:02x}{:02x}{:02x}\", bold: {}, italic: {}, underline: {}),", quote(&value), span.foreground.r, span.foreground.g, span.foreground.b, span.bold, span.italic, span.underline).unwrap();
                }
            }
            if spans.is_empty() {
                write!(
                    spans,
                    "(text: {}, color: {}, bold: false, italic: false, underline: false),",
                    quote(&line.replace('\t', "    ")),
                    quote(&config.styling.text_color)
                )
                .unwrap();
            }
            writeln!(
                out,
                "#source-line(({spans}), {}, {}, {}, {}, {}, {}, {}, {}, {}, {}pt)",
                index + 1,
                config.page.line_numbers,
                config.page.line_number_separator,
                config.page.vertical_borders,
                quote(&config.styling.background_color),
                quote(&config.styling.text_color),
                quote(&config.styling.line_number_color),
                config.page.wrap_long_lines,
                config.page.wrap_indent,
                config.page.font_size as f32 * config.page.line_spacing
            )
            .unwrap();
        }
        out.push_str("]\n#v(10pt)\n");
    }
    Ok((out, assets))
}

pub fn render(config: &Config, warnings: &WarningManager) -> Result<Vec<u8>> {
    let (source, assets) = document_source(config, warnings)?;
    let engine = TypstEngine::builder()
        .main_file(source)
        .search_fonts_with(TypstKitFontOptions::default())
        .with_static_file_resolver(
            assets
                .iter()
                .map(|(name, bytes)| (name.as_str(), bytes.clone())),
        )
        .build();
    let result = engine.compile::<PagedDocument>();
    for diagnostic in &result.warnings {
        let category = if diagnostic.message.contains("font") {
            crate::warnings::WarningCategory::Fonts
        } else {
            crate::warnings::WarningCategory::Rendering
        };
        warnings.warnf(category, format!("Typst: {}", diagnostic.message));
    }
    let document = result
        .output
        .map_err(|e| PapercutError::PdfGeneration(format!("Typst compilation failed: {e}")))?;
    typst_pdf::pdf(&document, &Default::default())
        .map_err(|e| PapercutError::PdfGeneration(format!("Typst PDF export failed: {e:?}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use typst::layout::{Frame, FrameItem};

    fn frame_text(frame: &Frame) -> String {
        frame
            .items()
            .map(|(_, item)| match item {
                FrameItem::Text(text) => text.text.to_string(),
                FrameItem::Group(group) => frame_text(&group.frame),
                _ => String::new(),
            })
            .collect()
    }

    #[test]
    fn typesets_markdown_and_code_with_real_page_totals() {
        let dir = tempfile::tempdir().unwrap();
        let marker = "#panic(\"must remain literal\") [brackets] \\ slashes";
        let mut code = format!("{marker}\n\n");
        for i in 0..180 {
            writeln!(code, "let value_{i} = \"{}\";", "long_token".repeat(12)).unwrap();
        }
        code.push_str("END_OF_SOURCE\n");
        fs::write(dir.path().join("source.rs"), code).unwrap();
        fs::write(dir.path().join("image.svg"), "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"20\" height=\"20\"><rect width=\"20\" height=\"20\" fill=\"red\"/></svg>").unwrap();
        fs::write(dir.path().join("report.md"), "# Report\n\n**Bold** and *italic*, ~~deleted~~, `code`.\n\n1. First\n2. Second\n   - Nested\n\n> Quoted\n\n[Link](https://example.com)\n\n```rust\nfn main() {}\n```\n\n| A | B |\n|---|---|\n| C | D |\n\n![Image](image.svg)\n\nEND_OF_REPORT\n").unwrap();
        fs::write(
            dir.path().join("config.yaml"),
            r#"
output:
  mode: single
files:
  - path: source.rs
    title: Source Listing
cover_page:
  enabled: true
  title: Test Document
  description: '#panic("literal description")'
markdown_report:
  enabled: true
  path: report.md
header:
  enabled: true
  left: '{filename}'
footer:
  enabled: true
  center: 'Page {page} of {total}'
"#,
        )
        .unwrap();
        let config =
            Config::from_file_with_warnings(&dir.path().join("config.yaml"), false).unwrap();
        let (source, assets) = document_source(&config, &WarningManager::new(false)).unwrap();
        let source = typst::syntax::Source::detached(source);
        let engine = TypstEngine::builder()
            .main_file(source.clone())
            .search_fonts_with(TypstKitFontOptions::default().include_system_fonts(false))
            .with_static_file_resolver(
                assets
                    .iter()
                    .map(|(name, bytes)| (name.as_str(), bytes.clone())),
            )
            .build();
        let result = engine.compile::<PagedDocument>();
        for diagnostic in &result.warnings {
            if !diagnostic.message.contains("font") {
                panic!(
                    "{}: {:?}",
                    diagnostic.message,
                    source.range(diagnostic.span).map(|r| &source.text()[r])
                );
            }
        }
        let doc = result.output.unwrap();
        assert!(doc.pages.len() > 5);
        let pages: Vec<_> = doc.pages.iter().map(|p| frame_text(&p.frame)).collect();
        let all = pages.join("\n");
        assert!(all.contains("literal description"));
        assert!(all.contains("must remain literal"));
        assert!(all.contains("END_OF_REPORT"));
        assert!(all.contains("END_OF_SOURCE"));
        for (i, page) in pages.iter().enumerate() {
            assert!(
                page.contains(&format!("Page {} of {}", i + 1, pages.len())),
                "incorrect footer on page {}: {page}",
                i + 1
            );
        }
        let pdf = typst_pdf::pdf(&doc, &Default::default()).unwrap();
        assert!(pdf.starts_with(b"%PDF-"));
    }
}
