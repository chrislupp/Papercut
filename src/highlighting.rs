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

use crate::pdf::themes::ThemePreset;
use crate::warnings::{WarningCategory, WarningManager};
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use syntect::easy::HighlightLines;
use syntect::highlighting::{Color, Style, Theme, ThemeSet};
use syntect::parsing::{SyntaxDefinition, SyntaxSet, SyntaxSetBuilder};
use syntect::util::LinesWithEndings;

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

/// Build SyntaxSet with defaults + embedded CMake + config syntaxes + convention dirs
fn build_syntax_set(config_syntaxes: &[PathBuf], warning_manager: &WarningManager) -> SyntaxSet {
    let mut builder = SyntaxSet::load_defaults_newlines().into_builder();

    // 1. Add embedded CMake syntax
    let cmake_syntax_yaml = include_str!("../assets/syntaxes/CMake.sublime-syntax");
    match SyntaxDefinition::load_from_str(cmake_syntax_yaml, true, None) {
        Ok(syntax) => builder.add(syntax),
        Err(e) => warning_manager.warnf(
            WarningCategory::Highlighting,
            format!("Failed to load embedded CMake syntax: {}", e),
        ),
    }

    // 2. Add syntaxes specified in config file (directories or files)
    for path in config_syntaxes {
        // Expand ~ to home directory
        let expanded_path = expand_tilde(path);

        if !expanded_path.exists() {
            warning_manager.warnf(
                WarningCategory::Highlighting,
                format!("Custom syntax path not found: {}", path.display()),
            );
            continue;
        }

        if expanded_path.is_dir() {
            // Load all .sublime-syntax files from directory
            if let Err(e) = builder.add_from_folder(&expanded_path, true) {
                warning_manager.warnf(
                    WarningCategory::Highlighting,
                    format!(
                        "Failed to load syntaxes from '{}': {}",
                        expanded_path.display(),
                        e
                    ),
                );
            }
        } else {
            // Load individual file
            load_syntax_file(&mut builder, &expanded_path, warning_manager);
        }
    }

    // 3. Add syntaxes from convention directories (.papercut/syntaxes/)
    for dir in get_convention_syntax_dirs() {
        if let Err(e) = builder.add_from_folder(&dir, true) {
            warning_manager.warnf(
                WarningCategory::Highlighting,
                format!("Failed to load syntaxes from '{}': {}", dir.display(), e),
            );
        }
    }

    builder.build()
}

/// Load a single syntax file
fn load_syntax_file(builder: &mut SyntaxSetBuilder, path: &Path, warning_manager: &WarningManager) {
    match fs::read_to_string(path) {
        Ok(content) => match SyntaxDefinition::load_from_str(&content, true, None) {
            Ok(syntax) => builder.add(syntax),
            Err(e) => warning_manager.warnf(
                WarningCategory::Highlighting,
                format!("Failed to parse syntax '{}': {}", path.display(), e),
            ),
        },
        Err(e) => warning_manager.warnf(
            WarningCategory::Highlighting,
            format!("Failed to read syntax file '{}': {}", path.display(), e),
        ),
    }
}

/// Expand ~ to home directory in path
fn expand_tilde(path: &Path) -> PathBuf {
    if let Some(path_str) = path.to_str() {
        if let Some(relative_path) = path_str.strip_prefix("~/") {
            if let Some(home) = dirs::home_dir() {
                return home.join(relative_path);
            }
        }
    }
    path.to_path_buf()
}

/// Get convention syntax directories (project-level first, then user home)
fn get_convention_syntax_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    let project_dir = PathBuf::from("./.papercut/syntaxes");
    if project_dir.exists() && project_dir.is_dir() {
        dirs.push(project_dir);
    }

    if let Some(home) = dirs::home_dir() {
        let home_dir = home.join(".papercut/syntaxes");
        if home_dir.exists() && home_dir.is_dir() {
            dirs.push(home_dir);
        }
    }

    dirs
}

/// Try to load a custom theme from .papercut folders
/// Search order: ./.papercut/themes/ → ~/.papercut/themes/
fn load_custom_theme(theme_name: &str, warning_manager: &WarningManager) -> Option<Theme> {
    // Try project-level .papercut folder first
    let project_theme_path =
        PathBuf::from("./.papercut/themes").join(format!("{}.tmTheme", theme_name));
    if project_theme_path.exists() {
        match fs::read_to_string(&project_theme_path) {
            Ok(theme_data) => {
                let mut cursor = Cursor::new(theme_data.as_bytes());
                match ThemeSet::load_from_reader(&mut cursor) {
                    Ok(theme) => return Some(theme),
                    Err(e) => {
                        warning_manager.warnf(
                            WarningCategory::Themes,
                            format!(
                                "Failed to parse custom theme '{}': {:?}",
                                project_theme_path.display(),
                                e
                            ),
                        );
                    }
                }
            }
            Err(e) => {
                warning_manager.warnf(
                    WarningCategory::Themes,
                    format!(
                        "Failed to read custom theme file '{}': {}",
                        project_theme_path.display(),
                        e
                    ),
                );
            }
        }
    }

    // Try user home .papercut folder
    if let Some(home_dir) = dirs::home_dir() {
        let home_theme_path = home_dir
            .join(".papercut/themes")
            .join(format!("{}.tmTheme", theme_name));
        if home_theme_path.exists() {
            match fs::read_to_string(&home_theme_path) {
                Ok(theme_data) => {
                    let mut cursor = Cursor::new(theme_data.as_bytes());
                    match ThemeSet::load_from_reader(&mut cursor) {
                        Ok(theme) => return Some(theme),
                        Err(e) => {
                            warning_manager.warnf(
                                WarningCategory::Themes,
                                format!(
                                    "Failed to parse custom theme '{}': {:?}",
                                    home_theme_path.display(),
                                    e
                                ),
                            );
                        }
                    }
                }
                Err(e) => {
                    warning_manager.warnf(
                        WarningCategory::Themes,
                        format!(
                            "Failed to read custom theme file '{}': {}",
                            home_theme_path.display(),
                            e
                        ),
                    );
                }
            }
        }
    }

    None
}

/// Highlight code and return styled segments for PDF rendering
pub fn highlight_code_styled(
    code: &str,
    file_path: &Path,
    theme_name: &str,
    custom_syntaxes: &[PathBuf],
    warning_manager: &WarningManager,
) -> Result<Vec<Vec<StyledSegment>>, String> {
    // Load syntax set with custom syntaxes
    let ss = build_syntax_set(custom_syntaxes, warning_manager);

    // Get the syntax definition based on file extension
    let syntax = ss
        .find_syntax_for_file(file_path)
        .map_err(|e| format!("Failed to determine syntax: {}", e))?
        .unwrap_or_else(|| ss.find_syntax_plain_text());

    // Theme loading order: built-in presets → custom .papercut themes → syntect defaults
    let theme = if let Some(preset) = ThemePreset::from_str(theme_name) {
        // 1. Use built-in theme preset
        preset
            .load_theme()
            .ok_or_else(|| format!("Failed to load built-in theme preset '{}'", theme_name))?
    } else if let Some(custom_theme) = load_custom_theme(theme_name, warning_manager) {
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
        let ranges: Vec<(Style, &str)> = h
            .highlight_line(line, &ss)
            .map_err(|e| format!("Highlighting error: {}", e))?;

        let mut segments = Vec::new();
        for (style, text) in ranges {
            segments.push(StyledSegment {
                text: text.to_string(),
                foreground: style.foreground,
                background: style.background,
                bold: style
                    .font_style
                    .contains(syntect::highlighting::FontStyle::BOLD),
                italic: style
                    .font_style
                    .contains(syntect::highlighting::FontStyle::ITALIC),
                underline: style
                    .font_style
                    .contains(syntect::highlighting::FontStyle::UNDERLINE),
            });
        }
        lines.push(segments);
    }

    Ok(lines)
}

/// List all available themes
pub fn list_themes() {
    // First show built-in presets
    println!("Built-in Presets:");
    println!();
    let presets = [
        ThemePreset::VsCodeDark,
        ThemePreset::VsCodeLight,
        ThemePreset::JetBrainsDarcula,
        ThemePreset::JetBrainsLight,
    ];
    for preset in &presets {
        println!("  - {}", preset.name());
    }

    // Then show syntect default themes
    println!();
    println!("Syntect Default Themes:");
    println!();

    let ts = ThemeSet::load_defaults();
    let mut themes: Vec<&String> = ts.themes.keys().collect();
    themes.sort();

    for theme in themes {
        println!("  - {}", theme);
    }

    println!();
    println!("You can also place custom .tmTheme files in .papercut/themes/");
}

/// List all available syntax definitions
pub fn list_syntaxes(config_syntaxes: &[PathBuf]) {
    let warning_manager = WarningManager::new(true);
    let ss = build_syntax_set(config_syntaxes, &warning_manager);

    println!("Available Syntax Definitions:");
    println!();

    let mut syntaxes: Vec<_> = ss.syntaxes().iter().collect();
    syntaxes.sort_by_key(|syntax| syntax.name.to_lowercase());

    for syntax in syntaxes {
        let exts: Vec<_> = syntax.file_extensions.iter().map(|s| s.as_str()).collect();
        let exts_str = exts.join(", ");
        if exts_str.is_empty() {
            println!("  - {}", syntax.name);
        } else {
            println!("  - {} ({})", syntax.name, exts_str);
        }
    }

    println!();
    println!("Custom syntaxes can be added via:");
    println!("  - config: syntax_highlighting.custom_syntaxes");
    println!("  - directories: .papercut/syntaxes/ or ~/.papercut/syntaxes/");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_highlight_rust_code_styled() {
        let code = "fn main() {\n    println!(\"Hello, world!\");\n}\n";
        let path = PathBuf::from("test.rs");
        let warning_manager = WarningManager::new(false);
        let result = highlight_code_styled(code, &path, "base16-ocean.dark", &[], &warning_manager);

        assert!(result.is_ok());
        let lines = result.unwrap();
        assert!(!lines.is_empty());
        // Check that we got styled segments
        let first_line_text: String = lines[0].iter().map(|s| s.text.as_str()).collect();
        assert!(first_line_text.contains("fn"));
    }

    #[test]
    fn test_invalid_theme() {
        let code = "fn main() {}";
        let path = PathBuf::from("test.rs");
        let warning_manager = WarningManager::new(false);
        let result = highlight_code_styled(code, &path, "nonexistent-theme", &[], &warning_manager);

        assert!(result.is_err());
    }

    #[test]
    fn test_cmake_syntax_available() {
        let warning_manager = WarningManager::new(false);
        let ss = build_syntax_set(&[], &warning_manager);

        // Check that CMake syntax is available
        let cmake_syntax = ss.find_syntax_by_name("CMake");
        assert!(cmake_syntax.is_some(), "CMake syntax should be available");

        // Check that it matches CMakeLists.txt files
        let cmake_path = PathBuf::from("CMakeLists.txt");
        let syntax = ss.find_syntax_for_file(&cmake_path).ok().flatten();
        assert!(syntax.is_some(), "Should find syntax for CMakeLists.txt");
        assert_eq!(syntax.unwrap().name, "CMake");
    }

    #[test]
    fn test_highlight_cmake_code() {
        let code = r#"cmake_minimum_required(VERSION 3.10)
project(MyProject)

set(CMAKE_CXX_STANDARD 17)

add_executable(myapp main.cpp)
"#;
        let path = PathBuf::from("CMakeLists.txt");
        let warning_manager = WarningManager::new(false);
        let result = highlight_code_styled(code, &path, "base16-ocean.dark", &[], &warning_manager);

        assert!(result.is_ok());
        let lines = result.unwrap();
        assert!(!lines.is_empty());
        // Check that we got styled segments
        let first_line_text: String = lines[0].iter().map(|s| s.text.as_str()).collect();
        assert!(first_line_text.contains("cmake_minimum_required"));
    }

    #[test]
    fn test_expand_tilde() {
        // Test with a tilde path
        let path = PathBuf::from("~/test/path");
        let expanded = expand_tilde(&path);

        if let Some(home) = dirs::home_dir() {
            assert_eq!(expanded, home.join("test/path"));
        }

        // Test without tilde (should remain unchanged)
        let path_no_tilde = PathBuf::from("/absolute/path");
        let expanded_no_tilde = expand_tilde(&path_no_tilde);
        assert_eq!(expanded_no_tilde, path_no_tilde);
    }
}
