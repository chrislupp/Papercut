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
}

pub type Result<T> = std::result::Result<T, PapercutError>;
