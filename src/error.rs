use thiserror::Error;

#[derive(Error, Debug)]
pub enum PapercutError {
    #[error("Configuration error: {0}\n  Tip: Check your YAML configuration file for syntax errors and required fields.")]
    Config(String),

    #[error("File not found: {0}\n  Tip: Verify the file path exists and is accessible.")]
    FileNotFound(String),

    #[error("Failed to read file '{path}': {source}\n  Tip: Check that the file exists and you have read permissions.")]
    FileRead {
        path: String,
        source: std::io::Error,
    },

    #[error("IO error: {0}\n  Tip: Check file permissions and available disk space.")]
    Io(#[from] std::io::Error),

    #[error("YAML parsing error: {0}\n  Tip: Validate your YAML syntax - check for proper indentation and valid field names.")]
    YamlParse(#[from] serde_yaml::Error),

    #[error("PDF generation error: {0}\n  Tip: Check that the output directory is writable and you have sufficient disk space.")]
    PdfGeneration(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[cfg(feature = "syntax-highlighting")]
    #[error("Syntax highlighting error: {0}\n  Tip: Try using a different theme or disabling syntax highlighting.")]
    SyntaxHighlighting(String),
}

pub type Result<T> = std::result::Result<T, PapercutError>;
