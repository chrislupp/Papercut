use std::path::{Path, PathBuf};
use syntect::easy::HighlightLines;
use syntect::highlighting::{ThemeSet, Style, Color, Theme};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;
use crate::pdf::themes::ThemePreset;
use std::fs;
use std::io::Cursor;

/// Represents a styled text segment
#[derive(Debug, Clone)]
pub struct StyledSegment {
    pub text: String,
    pub foreground: Color,
    pub background: Color,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
}

/// Try to load a custom theme from .papercut folders
/// Search order: ./.papercut/themes/ → ~/.papercut/themes/
fn load_custom_theme(theme_name: &str) -> Option<Theme> {
    // Try project-level .papercut folder first
    let project_theme_path = PathBuf::from("./.papercut/themes").join(format!("{}.tmTheme", theme_name));
    if project_theme_path.exists() {
        if let Ok(theme_data) = fs::read_to_string(&project_theme_path) {
            let mut cursor = Cursor::new(theme_data.as_bytes());
            if let Ok(theme) = ThemeSet::load_from_reader(&mut cursor) {
                return Some(theme);
            }
        }
    }

    // Try user home .papercut folder
    if let Some(home_dir) = dirs::home_dir() {
        let home_theme_path = home_dir.join(".papercut/themes").join(format!("{}.tmTheme", theme_name));
        if home_theme_path.exists() {
            if let Ok(theme_data) = fs::read_to_string(&home_theme_path) {
                let mut cursor = Cursor::new(theme_data.as_bytes());
                if let Ok(theme) = ThemeSet::load_from_reader(&mut cursor) {
                    return Some(theme);
                }
            }
        }
    }

    None
}

/// Highlight code and return styled segments for PDF rendering
pub fn highlight_code_styled(code: &str, file_path: &Path, theme_name: &str) -> Result<Vec<Vec<StyledSegment>>, String> {
    // Load syntax set
    let ss = SyntaxSet::load_defaults_newlines();

    // Get the syntax definition based on file extension
    let syntax = ss
        .find_syntax_for_file(file_path)
        .map_err(|e| format!("Failed to determine syntax: {}", e))?
        .unwrap_or_else(|| ss.find_syntax_plain_text());

    // Theme loading order: built-in presets → custom .papercut themes → syntect defaults
    let theme = if let Some(preset) = ThemePreset::from_str(theme_name) {
        // 1. Use built-in theme preset
        preset.load_theme()
            .ok_or_else(|| format!("Failed to load built-in theme preset '{}'", theme_name))?
    } else if let Some(custom_theme) = load_custom_theme(theme_name) {
        // 2. Use custom theme from .papercut folders
        custom_theme
    } else {
        // 3. Fall back to syntect's default themes by name
        let ts = ThemeSet::load_defaults();
        ts.themes.get(theme_name)
            .ok_or_else(|| format!("Theme '{}' not found in built-in presets, .papercut folders, or syntect defaults", theme_name))?
            .clone()
    };

    // Highlight the code line by line
    let mut h = HighlightLines::new(syntax, &theme);
    let mut lines = Vec::new();

    for line in LinesWithEndings::from(code) {
        let ranges: Vec<(Style, &str)> = h.highlight_line(line, &ss)
            .map_err(|e| format!("Highlighting error: {}", e))?;

        let mut segments = Vec::new();
        for (style, text) in ranges {
            segments.push(StyledSegment {
                text: text.to_string(),
                foreground: style.foreground,
                background: style.background,
                bold: style.font_style.contains(syntect::highlighting::FontStyle::BOLD),
                italic: style.font_style.contains(syntect::highlighting::FontStyle::ITALIC),
                underline: style.font_style.contains(syntect::highlighting::FontStyle::UNDERLINE),
            });
        }
        lines.push(segments);
    }

    Ok(lines)
}

/// Highlight code using syntect (returns plain text)
pub fn highlight_code(code: &str, file_path: &Path, theme_name: &str) -> Result<String, String> {
    // Load syntax set
    let ss = SyntaxSet::load_defaults_newlines();

    // Get the syntax definition based on file extension
    let syntax = ss
        .find_syntax_for_file(file_path)
        .map_err(|e| format!("Failed to determine syntax: {}", e))?
        .unwrap_or_else(|| ss.find_syntax_plain_text());

    // Theme loading order: built-in presets → custom .papercut themes → syntect defaults
    let theme = if let Some(preset) = ThemePreset::from_str(theme_name) {
        // 1. Use built-in theme preset
        preset.load_theme()
            .ok_or_else(|| format!("Failed to load built-in theme preset '{}'", theme_name))?
    } else if let Some(custom_theme) = load_custom_theme(theme_name) {
        // 2. Use custom theme from .papercut folders
        custom_theme
    } else {
        // 3. Fall back to syntect's default themes by name
        let ts = ThemeSet::load_defaults();
        ts.themes.get(theme_name)
            .ok_or_else(|| format!("Theme '{}' not found in built-in presets, .papercut folders, or syntect defaults", theme_name))?
            .clone()
    };

    // Highlight the code
    let mut h = HighlightLines::new(syntax, &theme);
    let mut result = String::new();

    for line in LinesWithEndings::from(code) {
        let ranges: Vec<(Style, &str)> = h.highlight_line(line, &ss)
            .map_err(|e| format!("Highlighting error: {}", e))?;

        // For now, we'll just append the text without color codes
        // since genpdf doesn't support colored text easily
        // In a future version, we could use styled text elements
        for (_, text) in ranges {
            result.push_str(text);
        }
    }

    Ok(result)
}

/// List all available themes
pub fn list_themes() {
    let ts = ThemeSet::load_defaults();

    println!("Available syntax highlighting themes:");
    println!();

    let mut themes: Vec<&String> = ts.themes.keys().collect();
    themes.sort();

    for theme in themes {
        println!("  - {}", theme);
    }

    println!();
    println!("Use these theme names in your configuration file.");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_highlight_rust_code() {
        let code = "fn main() {\n    println!(\"Hello, world!\");\n}\n";
        let path = PathBuf::from("test.rs");
        let result = highlight_code(code, &path, "base16-ocean.dark");

        assert!(result.is_ok());
        let highlighted = result.unwrap();
        assert!(highlighted.contains("main"));
        assert!(highlighted.contains("println!"));
    }

    #[test]
    fn test_invalid_theme() {
        let code = "fn main() {}";
        let path = PathBuf::from("test.rs");
        let result = highlight_code(code, &path, "nonexistent-theme");

        assert!(result.is_err());
    }
}
