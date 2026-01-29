// Papercut - Source code to PDF converter
// Copyright (C) 2026 Papercut Contributors
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

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::{self, Visitor};
use std::fmt;
use std::path::PathBuf;
use crate::error::{PapercutError, Result};

/// A margin value that can be specified in cm or inches.
/// Internally stored as points (1 inch = 72 points, 1 cm = 28.3465 points).
#[derive(Debug, Clone, Copy)]
pub struct MarginValue {
    points: f32,
}

impl MarginValue {
    /// Create a margin value from centimeters
    pub fn from_cm(cm: f32) -> Self {
        Self { points: cm * 28.3465 }
    }

    /// Create a margin value from inches
    pub fn from_inches(inches: f32) -> Self {
        Self { points: inches * 72.0 }
    }

    /// Get the value in points
    pub fn as_points(&self) -> f32 {
        self.points
    }

    /// Get the value in centimeters
    pub fn as_cm(&self) -> f32 {
        self.points / 28.3465
    }
}

impl<'de> Deserialize<'de> for MarginValue {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct MarginValueVisitor;

        impl<'de> Visitor<'de> for MarginValueVisitor {
            type Value = MarginValue;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a number (in cm) or a string like '1in' or '2.5cm'")
            }

            fn visit_i64<E>(self, value: i64) -> std::result::Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(MarginValue::from_cm(value as f32))
            }

            fn visit_u64<E>(self, value: u64) -> std::result::Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(MarginValue::from_cm(value as f32))
            }

            fn visit_f64<E>(self, value: f64) -> std::result::Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(MarginValue::from_cm(value as f32))
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: de::Error,
            {
                let value = value.trim();

                // Try parsing as inches
                if let Some(num_str) = value.strip_suffix("in") {
                    let num: f32 = num_str.trim().parse().map_err(|_| {
                        de::Error::custom(format!(
                            "Invalid inch value '{}'. Expected a number followed by 'in'",
                            value
                        ))
                    })?;
                    return Ok(MarginValue::from_inches(num));
                }

                // Try parsing as centimeters
                if let Some(num_str) = value.strip_suffix("cm") {
                    let num: f32 = num_str.trim().parse().map_err(|_| {
                        de::Error::custom(format!(
                            "Invalid centimeter value '{}'. Expected a number followed by 'cm'",
                            value
                        ))
                    })?;
                    return Ok(MarginValue::from_cm(num));
                }

                // Try parsing as a plain number (assume cm for backward compatibility)
                if let Ok(num) = value.parse::<f32>() {
                    return Ok(MarginValue::from_cm(num));
                }

                Err(de::Error::custom(format!(
                    "Invalid margin value '{}'. Use a number (in cm) or a string like '1in' or '2.5cm'",
                    value
                )))
            }
        }

        deserializer.deserialize_any(MarginValueVisitor)
    }
}

impl Serialize for MarginValue {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Serialize as centimeters (the default unit)
        serializer.serialize_f32(self.as_cm())
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub output: OutputConfig,
    pub files: Vec<FileEntry>,
    /// Expanded file list (populated after pattern expansion)
    #[serde(skip)]
    pub expanded_files: Vec<ExpandedFileEntry>,
    #[serde(default)]
    pub syntax_highlighting: SyntaxHighlightingConfig,
    #[serde(default)]
    pub page: PageConfig,
    #[serde(default)]
    pub header: HeaderFooterConfig,
    #[serde(default)]
    pub footer: HeaderFooterConfig,
    #[serde(default)]
    pub styling: StylingConfig,
    #[serde(default)]
    pub metadata: MetadataConfig,
    #[serde(default)]
    pub warnings: WarningsConfig,
    #[serde(default)]
    pub cover_page: CoverPageConfig,
}

/// Expanded file entry after pattern matching
#[derive(Debug, Clone)]
pub struct ExpandedFileEntry {
    pub path: PathBuf,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OutputConfig {
    /// "single" for one combined PDF, "multiple" for one PDF per file
    pub mode: OutputMode,
    /// Output directory path
    #[serde(default = "default_output_dir")]
    pub directory: PathBuf,
    /// Filename for single mode (optional, defaults to "output.pdf")
    #[serde(default = "default_output_filename")]
    pub filename: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum OutputMode {
    Single,
    Multiple,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FileEntry {
    pub path: PathBuf,
    /// Optional custom title for this file in the PDF
    #[serde(default)]
    pub title: Option<String>,
    /// File types to include (by extension, e.g., ["rs", "py", "js"])
    /// If empty, all file types are included
    #[serde(default)]
    pub include_types: Vec<String>,
    /// Patterns to exclude (e.g., ["*.test.rs", "target/**"])
    /// Uses glob pattern syntax
    #[serde(default)]
    pub exclude: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SyntaxHighlightingConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Theme name (e.g., "base16-ocean.dark", "InspiredGitHub")
    #[serde(default = "default_theme")]
    pub theme: String,
}

impl Default for SyntaxHighlightingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            theme: default_theme(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PageConfig {
    /// Page size: "A4", "Letter", "Legal"
    #[serde(default = "default_page_size")]
    pub size: PageSize,
    /// Margins in centimeters
    #[serde(default)]
    pub margins: MarginsConfig,
    /// Font size for code content
    #[serde(default = "default_font_size")]
    pub font_size: u8,
    /// Show line numbers
    #[serde(default = "default_true")]
    pub line_numbers: bool,
    /// Show vertical line separator between line numbers and code
    #[serde(default = "default_true")]
    pub line_number_separator: bool,
    /// Show vertical borders at left and right edges of content
    #[serde(default = "default_true")]
    pub vertical_borders: bool,
    /// Line spacing multiplier
    #[serde(default = "default_line_spacing")]
    pub line_spacing: f32,
    /// Enable line wrapping for long lines
    #[serde(default = "default_true")]
    pub wrap_long_lines: bool,
    /// Indentation for wrapped continuation lines (in characters)
    #[serde(default = "default_wrap_indent")]
    pub wrap_indent: usize,
}

impl Default for PageConfig {
    fn default() -> Self {
        Self {
            size: default_page_size(),
            margins: MarginsConfig::default(),
            font_size: default_font_size(),
            line_numbers: true,
            line_number_separator: true,
            vertical_borders: true,
            line_spacing: default_line_spacing(),
            wrap_long_lines: true,
            wrap_indent: default_wrap_indent(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum PageSize {
    A4,
    Letter,
    Legal,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MarginsConfig {
    #[serde(default = "default_margin_top")]
    pub top: MarginValue,
    #[serde(default = "default_margin_bottom")]
    pub bottom: MarginValue,
    #[serde(default = "default_margin_left")]
    pub left: MarginValue,
    #[serde(default = "default_margin_right")]
    pub right: MarginValue,
}

impl Default for MarginsConfig {
    fn default() -> Self {
        Self {
            top: default_margin_top(),
            bottom: default_margin_bottom(),
            left: default_margin_left(),
            right: default_margin_right(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct HeaderFooterConfig {
    #[serde(default = "default_false")]
    pub enabled: bool,
    /// Full-width text (supports variables: {page}, {total}, {filename}, {date})
    /// When set, this takes precedence over left/center/right and wraps across the full width
    /// Respects paragraph breaks (double newlines) and is left-justified
    #[serde(default)]
    pub text: String,
    /// Text to display on the left (supports variables: {page}, {total}, {filename}, {date})
    #[serde(default)]
    pub left: String,
    /// Text to display in the center
    #[serde(default)]
    pub center: String,
    /// Text to display on the right
    #[serde(default)]
    pub right: String,
    /// Font size for header/footer
    #[serde(default = "default_header_footer_font_size")]
    pub font_size: u8,
    /// Optional margin overrides for this header/footer
    #[serde(default)]
    pub margins: Option<HeaderFooterMargins>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct HeaderFooterMargins {
    /// Top margin for header positioning (in mm or with unit suffix like "1 in")
    #[serde(default)]
    pub top: Option<MarginValue>,
    /// Bottom margin for footer positioning (in mm or with unit suffix like "1 in")
    #[serde(default)]
    pub bottom: Option<MarginValue>,
    /// Left margin for header/footer (in mm or with unit suffix like "1 in")
    #[serde(default)]
    pub left: Option<MarginValue>,
    /// Right margin for header/footer (in mm or with unit suffix like "1 in")
    #[serde(default)]
    pub right: Option<MarginValue>,
}

impl Default for HeaderFooterConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            text: String::new(),
            left: String::new(),
            center: String::new(),
            right: String::new(),
            font_size: default_header_footer_font_size(),
            margins: None,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StylingConfig {
    /// Font family for code: "monospace", "courier", "dejavu"
    #[serde(default = "default_font_family")]
    pub font_family: FontFamily,
    /// Background color for code blocks (hex format, e.g., "#ffffff")
    #[serde(default = "default_background_color")]
    pub background_color: String,
    /// Text color (hex format, e.g., "#000000")
    #[serde(default = "default_text_color")]
    pub text_color: String,
    /// Line number color (hex format)
    #[serde(default = "default_line_number_color")]
    pub line_number_color: String,
}

impl Default for StylingConfig {
    fn default() -> Self {
        Self {
            font_family: default_font_family(),
            background_color: default_background_color(),
            text_color: default_text_color(),
            line_number_color: default_line_number_color(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum FontFamily {
    #[serde(rename = "monospace")]
    Monospace,
    #[serde(rename = "courier")]
    Courier,
    #[serde(rename = "dejavu")]
    DejaVu,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct MetadataConfig {
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub subject: String,
    #[serde(default)]
    pub keywords: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WarningsConfig {
    /// Enable or disable all warnings
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// List of warning categories to silence: fonts, themes, highlighting, filesystem
    #[serde(default)]
    pub silence_categories: Vec<String>,
}

impl Default for WarningsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            silence_categories: Vec::new(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CoverPageConfig {
    /// Enable or disable the cover page
    #[serde(default = "default_false")]
    pub enabled: bool,
    /// Title to display on the cover page
    #[serde(default)]
    pub title: String,
    /// Authors text for the cover page (supports multiple lines with double newlines)
    #[serde(default)]
    pub authors: String,
    /// Description text for the cover page
    #[serde(default)]
    pub description: String,
    /// Location/URL where the code is kept
    #[serde(default)]
    pub location: String,
    /// Date to display (auto-generates current date if empty)
    #[serde(default)]
    pub date: String,
    /// Include table of contents listing all files
    #[serde(default = "default_true")]
    pub include_toc: bool,
    /// Font size for the title
    #[serde(default = "default_cover_title_font_size")]
    pub title_font_size: u8,
    /// Font size for the description and other text
    #[serde(default = "default_cover_text_font_size")]
    pub text_font_size: u8,
    /// Font family for the cover page (default: Arial)
    #[serde(default = "default_cover_font_family")]
    pub font_family: String,
    /// Optional header config for cover page (overrides main header if set)
    #[serde(default)]
    pub header: Option<HeaderFooterConfig>,
    /// Optional footer config for cover page (overrides main footer if set)
    #[serde(default)]
    pub footer: Option<HeaderFooterConfig>,
}

impl Default for CoverPageConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            title: String::new(),
            authors: String::new(),
            description: String::new(),
            location: String::new(),
            date: String::new(),
            include_toc: true,
            title_font_size: default_cover_title_font_size(),
            text_font_size: default_cover_text_font_size(),
            font_family: default_cover_font_family(),
            header: None,
            footer: None,
        }
    }
}

// Default value functions
fn default_output_dir() -> PathBuf {
    PathBuf::from("./output")
}

fn default_output_filename() -> String {
    "output.pdf".to_string()
}

fn default_false() -> bool {
    false
}

fn default_true() -> bool {
    true
}

fn default_theme() -> String {
    "vscode-light".to_string()
}

fn default_page_size() -> PageSize {
    PageSize::A4
}

fn default_font_size() -> u8 {
    10
}

fn default_line_spacing() -> f32 {
    1.2
}

fn default_wrap_indent() -> usize {
    4
}

fn default_margin_top() -> MarginValue {
    MarginValue::from_cm(2.5)
}

fn default_margin_bottom() -> MarginValue {
    MarginValue::from_cm(2.5)
}

fn default_margin_left() -> MarginValue {
    MarginValue::from_cm(2.0)
}

fn default_margin_right() -> MarginValue {
    MarginValue::from_cm(2.0)
}

fn default_header_footer_font_size() -> u8 {
    8
}

fn default_font_family() -> FontFamily {
    FontFamily::Monospace
}

fn default_background_color() -> String {
    "#ffffff".to_string()
}

fn default_text_color() -> String {
    "#000000".to_string()
}

fn default_line_number_color() -> String {
    "#888888".to_string()
}

fn default_cover_title_font_size() -> u8 {
    24
}

fn default_cover_text_font_size() -> u8 {
    12
}

fn default_cover_font_family() -> String {
    "Arial".to_string()
}

impl Config {
    /// Load configuration from a YAML file
    pub fn from_file(path: &PathBuf) -> Result<Self> {
        let contents = std::fs::read_to_string(path)
            .map_err(|e| PapercutError::Config(format!("Failed to read config file: {}", e)))?;

        let mut config: Config = serde_yaml::from_str(&contents)?;

        // Expand file patterns before validation
        config.expand_file_patterns()?;
        config.validate()?;

        Ok(config)
    }

    /// Expand file patterns and populate expanded_files
    fn expand_file_patterns(&mut self) -> Result<()> {
        use crate::file_scanner;
        use crate::warnings::WarningManager;
        use std::sync::Arc;

        // Create a disabled warning manager for config loading
        // (warnings config hasn't been parsed yet)
        let warning_manager = Arc::new(WarningManager::new(false));

        let mut expanded_files = Vec::new();

        for file_entry in &self.files {
            let files = file_scanner::expand_file_patterns(
                &file_entry.path,
                &file_entry.include_types,
                &file_entry.exclude,
                &warning_manager,
            )?;

            // Add each expanded file
            for file_path in files {
                expanded_files.push(ExpandedFileEntry {
                    path: file_path,
                });
            }
        }

        self.expanded_files = expanded_files;
        Ok(())
    }

    /// Validate the configuration
    fn validate(&self) -> Result<()> {
        // Validate that we have at least one file specification
        if self.files.is_empty() {
            return Err(PapercutError::InvalidConfig(
                "No files specified in configuration".to_string()
            ));
        }

        // Validate that pattern expansion resulted in at least one file
        if self.expanded_files.is_empty() {
            return Err(PapercutError::InvalidConfig(
                "No files matched the specified patterns".to_string()
            ));
        }

        // Validate that all expanded files exist and are readable
        for file_entry in &self.expanded_files {
            if !file_entry.path.exists() {
                return Err(PapercutError::FileNotFound(
                    file_entry.path.display().to_string()
                ));
            }
            if !file_entry.path.is_file() {
                return Err(PapercutError::InvalidConfig(
                    format!("{} is not a regular file", file_entry.path.display())
                ));
            }
        }

        // Validate margin values (using points, which are always non-negative from valid input)
        let margin_top_pt = self.page.margins.top.as_points();
        let margin_bottom_pt = self.page.margins.bottom.as_points();
        let margin_left_pt = self.page.margins.left.as_points();
        let margin_right_pt = self.page.margins.right.as_points();

        if margin_top_pt < 0.0 || margin_bottom_pt < 0.0 ||
           margin_left_pt < 0.0 || margin_right_pt < 0.0 {
            return Err(PapercutError::InvalidConfig(
                "Margins must be non-negative".to_string()
            ));
        }

        // Validate font sizes
        if self.page.font_size == 0 {
            return Err(PapercutError::InvalidConfig(
                "Font size must be greater than 0".to_string()
            ));
        }

        // Get page size in points
        let (page_width_pt, page_height_pt) = match self.page.size {
            PageSize::A4 => (595.28, 841.89),      // 210mm x 297mm
            PageSize::Letter => (612.0, 792.0),     // 8.5" x 11"
            PageSize::Legal => (612.0, 1008.0),     // 8.5" x 14"
        };

        // Calculate content area
        let content_width = page_width_pt - margin_left_pt - margin_right_pt;
        let content_height = page_height_pt - margin_top_pt - margin_bottom_pt;

        if content_width <= 0.0 {
            return Err(PapercutError::InvalidConfig(
                format!(
                    "Left and right margins ({:.2} cm + {:.2} cm) exceed page width. Content area has no width.",
                    self.page.margins.left.as_cm(), self.page.margins.right.as_cm()
                )
            ));
        }

        if content_height <= 0.0 {
            return Err(PapercutError::InvalidConfig(
                format!(
                    "Top and bottom margins ({:.2} cm + {:.2} cm) exceed page height. Content area has no height.",
                    self.page.margins.top.as_cm(), self.page.margins.bottom.as_cm()
                )
            ));
        }

        // Warn if content area is very small (less than 50% of page)
        let width_ratio = content_width / page_width_pt;
        let height_ratio = content_height / page_height_pt;

        if width_ratio < 0.5 || height_ratio < 0.5 {
            eprintln!(
                "Warning: Margins are very large. Content area is only {:.0}% x {:.0}% of the page.",
                width_ratio * 100.0, height_ratio * 100.0
            );
        }

        Ok(())
    }

    /// Get effective metadata, falling back to cover page values when metadata fields are empty
    pub fn effective_metadata(&self) -> EffectiveMetadata {
        EffectiveMetadata {
            title: if self.metadata.title.is_empty() {
                self.cover_page.title.clone()
            } else {
                self.metadata.title.clone()
            },
            author: if self.metadata.author.is_empty() {
                self.cover_page.authors.clone()
            } else {
                self.metadata.author.clone()
            },
            subject: self.metadata.subject.clone(),
            keywords: self.metadata.keywords.clone(),
        }
    }
}

/// Effective metadata after merging with cover page values
#[derive(Debug, Clone)]
pub struct EffectiveMetadata {
    pub title: String,
    pub author: String,
    pub subject: String,
    pub keywords: Vec<String>,
}
