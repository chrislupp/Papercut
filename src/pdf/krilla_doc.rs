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

use crate::config::{Config, EffectiveMetadata, PageSize};
use crate::error::{PapercutError, Result};
use crate::pdf::fonts::FontManager;
use crate::warnings::WarningManager;
use krilla::Document;
use krilla::metadata::Metadata;
use krilla::page::PageSettings;
use std::path::Path;
use std::sync::Arc;

/// Simplified wrapper - krilla 0.5 manages pages differently
/// We'll just provide helper functions and let the generator manage pages directly
pub struct PdfContext {
    pub document: Document,
    pub font_manager: FontManager,
    pub page_width_mm: f32,
    pub page_height_mm: f32,
    pub margin_top: f32,
    pub margin_left: f32,
    pub margin_right: f32,
    pub margin_bottom: f32,
    pub content_width: f32,
    pub content_height: f32,
}

impl PdfContext {
    /// Create a new PDF context
    pub fn new(config: Config, warning_manager: Arc<WarningManager>) -> Result<Self> {
        let document = Document::new();
        let mut font_manager = FontManager::new(warning_manager);

        // Pre-load the monospace font
        font_manager.get_monospace_font()?;

        // Calculate page dimensions and margins (all in points)
        let (page_width_pt, page_height_pt) = get_page_size_points(&config.page.size);
        let margin_top = config.page.margins.top.as_points();
        let margin_right = config.page.margins.right.as_points();
        let margin_bottom = config.page.margins.bottom.as_points();
        let margin_left = config.page.margins.left.as_points();

        let content_width = page_width_pt - margin_left - margin_right;
        let content_height = page_height_pt - margin_top - margin_bottom;

        Ok(Self {
            document,
            font_manager,
            page_width_mm: page_width_pt,
            page_height_mm: page_height_pt,
            margin_top,
            margin_left,
            margin_right,
            margin_bottom,
            content_width,
            content_height,
        })
    }

    /// Get page settings for creating a new page
    pub fn page_settings(&self) -> PageSettings {
        PageSettings::new(self.page_width_mm, self.page_height_mm)
    }

    /// Set PDF metadata from effective metadata (merged config + cover page)
    pub fn set_metadata(&mut self, effective: &EffectiveMetadata) {
        let mut metadata = Metadata::new();

        if !effective.title.is_empty() {
            metadata = metadata.title(effective.title.clone());
        }

        if !effective.author.is_empty() {
            metadata = metadata.authors(vec![effective.author.clone()]);
        }

        if !effective.subject.is_empty() {
            metadata = metadata.description(effective.subject.clone());
        }

        if !effective.keywords.is_empty() {
            metadata = metadata.keywords(effective.keywords.clone());
        }

        self.document.set_metadata(metadata);
    }

    /// Finish the document and write to file
    pub fn save(self, path: &Path) -> Result<()> {
        // Finish and get PDF bytes
        let pdf_bytes = self.document
            .finish()
            .map_err(|e| PapercutError::PdfGeneration(
                format!("Failed to finish PDF document: {:?}", e)
            ))?;

        // Write to file
        std::fs::write(path, pdf_bytes)
            .map_err(|e| PapercutError::PdfGeneration(
                format!("Failed to write PDF to '{}': {}", path.display(), e)
            ))?;

        Ok(())
    }
}

/// Get page size in points (PDF standard: 1 point = 1/72 inch)
fn get_page_size_points(size: &PageSize) -> (f32, f32) {
    match size {
        PageSize::A4 => (595.28, 841.89),      // 210mm x 297mm
        PageSize::Letter => (612.0, 792.0),     // 8.5" x 11"
        PageSize::Legal => (612.0, 1008.0),     // 8.5" x 14"
    }
}

