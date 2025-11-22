use thiserror::Error;

#[derive(Error, Debug)]
pub enum PapercutError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("YAML parsing error: {0}")]
    YamlParse(#[from] serde_yaml::Error),

    #[error("PDF generation error: {0}")]
    PdfGeneration(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[cfg(feature = "syntax-highlighting")]
    #[error("Syntax highlighting error: {0}")]
    SyntaxHighlighting(String),
}

pub type Result<T> = std::result::Result<T, PapercutError>;
