//! Markdown is parsed, not evaluated as Typst. Images are loaded explicitly and
//! supplied as in-memory assets; the compiler needs no filesystem resolver.
use super::typst_renderer::quote;
use crate::error::Result;
use crate::warnings::{WarningCategory, WarningManager};
use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag, TagEnd};
use std::fmt::Write;
use std::path::Path;

pub(super) fn convert(
    markdown: &str,
    base: &Path,
    assets: &mut Vec<(String, Vec<u8>)>,
    warnings: &WarningManager,
) -> Result<String> {
    let mut out = String::new();
    let mut code: Option<(String, String)> = None;
    let mut image: Option<(String, String)> = None;
    let mut table_columns = 0;
    for event in Parser::new_ext(
        markdown,
        Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TABLES | Options::ENABLE_TASKLISTS,
    ) {
        if let Some((_, text)) = code.as_mut() {
            if let Event::Text(value) = &event {
                text.push_str(value);
                continue;
            }
        }
        if let Some((_, alt)) = image.as_mut() {
            match &event {
                Event::Text(text) | Event::Code(text) => {
                    alt.push_str(text);
                    continue;
                }
                Event::End(TagEnd::Image) => {}
                _ => continue,
            }
        }
        match event {
            Event::Start(tag) => {
                match tag {
                    Tag::Paragraph => {}
                    Tag::Heading { level, .. } => {
                        write!(out, "#heading(level: {}, outlined: false)[", level as u8).unwrap();
                    }
                    Tag::Emphasis => out.push_str("#emph["),
                    Tag::Strong => out.push_str("#strong["),
                    Tag::Strikethrough => out.push_str("#strike["),
                    Tag::BlockQuote(_) => out
                        .push_str("#block(inset: (left: 12pt), stroke: (left: 2pt + luma(180)))["),
                    Tag::CodeBlock(kind) => {
                        code = Some((
                            match kind {
                                CodeBlockKind::Fenced(lang) => {
                                    lang.split_whitespace().next().unwrap_or("").to_string()
                                }
                                _ => String::new(),
                            },
                            String::new(),
                        ));
                    }
                    Tag::List(start) => {
                        if let Some(start) = start {
                            writeln!(out, "#enum(start: {start},").unwrap();
                        } else {
                            out.push_str("#list(\n");
                        }
                    }
                    Tag::Item => out.push('['),
                    Tag::Link { dest_url, .. } => {
                        if dest_url.starts_with('#') {
                            out.push_str("#text[");
                        } else {
                            write!(out, "#link({})[", quote(&dest_url)).unwrap();
                        }
                    }
                    Tag::Image { dest_url, .. } => {
                        image = Some((dest_url.to_string(), String::new()))
                    }
                    Tag::Table(alignments) => {
                        table_columns = alignments.len();
                        writeln!(out, "#table(columns: {table_columns}, inset: 5pt, stroke: 0.4pt + luma(180),").unwrap();
                    }
                    Tag::TableHead => out.push_str("table.header("),
                    Tag::TableCell => out.push('['),
                    _ => {}
                }
            }
            Event::End(tag) => match tag {
                TagEnd::Paragraph => out.push_str("#parbreak()\n"),
                TagEnd::Heading(_) | TagEnd::BlockQuote(_) => out.push_str("]\n"),
                TagEnd::Emphasis | TagEnd::Strong | TagEnd::Strikethrough | TagEnd::Link => {
                    out.push(']')
                }
                TagEnd::List(_) => out.push_str(")\n"),
                TagEnd::Item | TagEnd::TableCell => out.push_str("],\n"),
                TagEnd::TableHead => out.push_str("),\n"),
                TagEnd::Table => {
                    if table_columns > 0 {
                        out.push_str(")\n");
                    }
                }
                TagEnd::CodeBlock => {
                    let (lang, text) = code.take().unwrap();
                    writeln!(out, "#block(width: 100%, inset: 8pt, fill: luma(245), breakable: true)[#raw({}, block: true, lang: {})]", quote(text.trim_end_matches('\n')), quote(&lang)).unwrap();
                }
                TagEnd::Image => {
                    let (path, alt) = image.take().unwrap();
                    let resolved = base.join(&path);
                    match std::fs::read(&resolved) {
                        Ok(bytes) => {
                            let extension = resolved
                                .extension()
                                .and_then(|e| e.to_str())
                                .unwrap_or("png");
                            let name = format!("image-{}.{}", assets.len(), extension);
                            writeln!(
                                out,
                                "#image({}, width: 100%, alt: {})",
                                quote(&name),
                                quote(&alt)
                            )
                            .unwrap();
                            assets.push((name, bytes));
                        }
                        Err(e) => {
                            warnings.warnf(
                                WarningCategory::Filesystem,
                                format!("Cannot read Markdown image '{}': {e}", resolved.display()),
                            );
                            write!(out, "#text({})", quote(&alt)).unwrap();
                        }
                    }
                }
                _ => {}
            },
            Event::Text(value) | Event::Html(value) | Event::InlineHtml(value) => {
                write!(out, "#text({})", quote(&value)).unwrap();
            }
            Event::Code(value) => {
                write!(out, "#raw({})", quote(&value)).unwrap();
            }
            Event::SoftBreak => out.push(' '),
            Event::HardBreak => out.push_str("#linebreak()"),
            Event::Rule => out.push_str("#line(length: 100%, stroke: 0.5pt + luma(180))\n"),
            Event::TaskListMarker(done) => out.push_str(if done {
                "#text(\"☑ \" )"
            } else {
                "#text(\"☐ \" )"
            }),
            _ => {}
        }
    }
    Ok(out)
}
