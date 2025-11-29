use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::error::{PapercutError, Result};

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
    pub title: Option<String>,
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
    pub top: f32,
    #[serde(default = "default_margin_bottom")]
    pub bottom: f32,
    #[serde(default = "default_margin_left")]
    pub left: f32,
    #[serde(default = "default_margin_right")]
    pub right: f32,
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
}

impl Default for HeaderFooterConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            left: String::new(),
            center: String::new(),
            right: String::new(),
            font_size: default_header_footer_font_size(),
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

#[derive(Debug, Deserialize, Serialize, Clone)]
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

impl Default for MetadataConfig {
    fn default() -> Self {
        Self {
            title: String::new(),
            author: String::new(),
            subject: String::new(),
            keywords: Vec::new(),
        }
    }
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
    /// Authors to display on the cover page (supports single string or list)
    #[serde(default, deserialize_with = "deserialize_string_or_vec")]
    pub authors: Vec<String>,
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
}

impl Default for CoverPageConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            title: String::new(),
            authors: Vec::new(),
            description: String::new(),
            location: String::new(),
            date: String::new(),
            include_toc: true,
            title_font_size: default_cover_title_font_size(),
            text_font_size: default_cover_text_font_size(),
            font_family: default_cover_font_family(),
        }
    }
}

/// Deserialize a field that can be either a single string or a list of strings
fn deserialize_string_or_vec<'de, D>(deserializer: D) -> std::result::Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{self, SeqAccess, Visitor};
    use std::fmt;

    struct StringOrVec;

    impl<'de> Visitor<'de> for StringOrVec {
        type Value = Vec<String>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string or a list of strings")
        }

        fn visit_str<E>(self, value: &str) -> std::result::Result<Vec<String>, E>
        where
            E: de::Error,
        {
            if value.is_empty() {
                Ok(Vec::new())
            } else {
                Ok(vec![value.to_string()])
            }
        }

        fn visit_seq<A>(self, mut seq: A) -> std::result::Result<Vec<String>, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let mut vec = Vec::new();
            while let Some(item) = seq.next_element()? {
                vec.push(item);
            }
            Ok(vec)
        }

        fn visit_none<E>(self) -> std::result::Result<Vec<String>, E>
        where
            E: de::Error,
        {
            Ok(Vec::new())
        }

        fn visit_unit<E>(self) -> std::result::Result<Vec<String>, E>
        where
            E: de::Error,
        {
            Ok(Vec::new())
        }
    }

    deserializer.deserialize_any(StringOrVec)
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

fn default_margin_top() -> f32 {
    2.5
}

fn default_margin_bottom() -> f32 {
    2.5
}

fn default_margin_left() -> f32 {
    2.0
}

fn default_margin_right() -> f32 {
    2.0
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

            // Add each expanded file with the original title (if provided)
            for file_path in files {
                expanded_files.push(ExpandedFileEntry {
                    path: file_path,
                    title: file_entry.title.clone(),
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

        // Validate margin values
        if self.page.margins.top < 0.0 || self.page.margins.bottom < 0.0 ||
           self.page.margins.left < 0.0 || self.page.margins.right < 0.0 {
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

        // Validate that content area is positive
        // Convert margins from cm to points (1 cm = 28.35 points)
        let margin_top_pt = self.page.margins.top * 28.35;
        let margin_bottom_pt = self.page.margins.bottom * 28.35;
        let margin_left_pt = self.page.margins.left * 28.35;
        let margin_right_pt = self.page.margins.right * 28.35;

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
                    self.page.margins.left, self.page.margins.right
                )
            ));
        }

        if content_height <= 0.0 {
            return Err(PapercutError::InvalidConfig(
                format!(
                    "Top and bottom margins ({:.2} cm + {:.2} cm) exceed page height. Content area has no height.",
                    self.page.margins.top, self.page.margins.bottom
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
                self.cover_page.authors.join(", ")
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
