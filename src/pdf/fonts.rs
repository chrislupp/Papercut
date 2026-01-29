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

use crate::error::{PapercutError, Result};
use crate::warnings::{WarningManager, WarningCategory};
use fontdb::{Database, Query, Source};
use krilla::text::Font;
use std::sync::Arc;

/// Manages font loading and caching for PDF generation
pub struct FontManager {
    db: Database,
    monospace_font: Option<Arc<Font>>,
    cover_font: Option<Arc<Font>>,
    cover_bold_font: Option<Arc<Font>>,
    warning_manager: Arc<WarningManager>,
}

impl FontManager {
    /// Create a new font manager and load system fonts
    pub fn new(warning_manager: Arc<WarningManager>) -> Self {
        let mut db = Database::new();

        // Load system fonts
        db.load_system_fonts();

        Self {
            db,
            monospace_font: None,
            cover_font: None,
            cover_bold_font: None,
            warning_manager,
        }
    }

    /// Get or load a monospace font suitable for code display
    pub fn get_monospace_font(&mut self) -> Result<Arc<Font>> {
        if let Some(font) = &self.monospace_font {
            return Ok(Arc::clone(font));
        }

        // Try to find a good monospace font in order of preference
        let font_families = vec![
            "Consolas",
            "Monaco",
            "Menlo",
            "DejaVu Sans Mono",
            "Liberation Mono",
            "Courier New",
            "Courier",
            "monospace",
        ];

        for family in font_families {
            if let Some(font) = self.try_load_font(family) {
                let font_arc = Arc::new(font);
                self.monospace_font = Some(Arc::clone(&font_arc));
                return Ok(font_arc);
            }
        }

        Err(PapercutError::InvalidConfig(
            "Could not find any suitable monospace font. Please install a monospace font like \
             Consolas, Monaco, or DejaVu Sans Mono.".to_string()
        ))
    }

    /// Get or load a cover page font (sans-serif font like Arial)
    pub fn get_cover_font(&mut self, font_family: &str) -> Result<Arc<Font>> {
        if let Some(font) = &self.cover_font {
            return Ok(Arc::clone(font));
        }

        // Try the specified font first, then fallbacks
        let font_families = if font_family.is_empty() {
            vec![
                "Arial",
                "Helvetica Neue",
                "Helvetica",
                "DejaVu Sans",
                "Liberation Sans",
                "sans-serif",
            ]
        } else {
            vec![
                font_family,
                "Arial",
                "Helvetica Neue",
                "Helvetica",
                "DejaVu Sans",
                "Liberation Sans",
                "sans-serif",
            ]
        };

        for family in font_families {
            if let Some(font) = self.try_load_font(family) {
                let font_arc = Arc::new(font);
                self.cover_font = Some(Arc::clone(&font_arc));
                return Ok(font_arc);
            }
        }

        // Fall back to monospace if no serif font available
        self.get_monospace_font()
    }

    /// Get or load a font for headers/footers (reuses cover font)
    pub fn get_header_footer_font(&mut self) -> Result<Arc<Font>> {
        self.get_cover_font("Arial")
    }

    /// Get or load a bold cover page font
    pub fn get_cover_bold_font(&mut self, font_family: &str) -> Result<Arc<Font>> {
        if let Some(font) = &self.cover_bold_font {
            return Ok(Arc::clone(font));
        }

        // Build list of fonts to try for bold variant
        // Always include common fonts with reliable bold variants
        let mut font_families = Vec::new();

        // First try the user-specified font if provided
        if !font_family.is_empty() {
            font_families.push(font_family);
        }

        // Then try fonts known to have good bold variants
        let fallbacks = [
            "Arial",           // Windows/macOS - has proper bold
            "Helvetica Neue",  // macOS - has proper bold
            "DejaVu Sans",     // Linux - has proper bold
            "Liberation Sans", // Linux - has proper bold
            "Helvetica",       // Try last as it may not have bold
            "sans-serif",
        ];

        for fallback in fallbacks {
            if fallback != font_family {
                font_families.push(fallback);
            }
        }

        for family in font_families {
            if let Some(font) = self.try_load_bold_font(family) {
                let font_arc = Arc::new(font);
                self.cover_bold_font = Some(Arc::clone(&font_arc));
                return Ok(font_arc);
            }
        }

        // Fall back to regular cover font if no bold available anywhere
        self.get_cover_font(font_family)
    }

    /// Try to load a font by family name
    fn try_load_font(&self, family: &str) -> Option<Font> {
        self.try_load_font_with_weight(family, fontdb::Weight::NORMAL)
    }

    /// Try to load a bold font by family name
    fn try_load_bold_font(&self, family: &str) -> Option<Font> {
        self.try_load_font_with_weight(family, fontdb::Weight::BOLD)
    }

    /// Try to load a font by family name and weight
    fn try_load_font_with_weight(&self, family: &str, weight: fontdb::Weight) -> Option<Font> {
        let query = Query {
            families: &[fontdb::Family::Name(family)],
            weight,
            stretch: fontdb::Stretch::Normal,
            style: fontdb::Style::Normal,
        };

        let id = self.db.query(&query)?;
        let face = self.db.face(id)?;

        // Load the font data, using the correct index for TTC (TrueType Collection) files
        let font_index = face.index;
        match &face.source {
            Source::Binary(data) => {
                // Try to create a Font from the binary data
                let vec_data = data.as_ref().as_ref().to_vec();
                Font::new(vec_data.into(), font_index)
            }
            Source::File(path) => {
                // Read font file and create Font
                match std::fs::read(path) {
                    Ok(data) => Font::new(data.into(), font_index),
                    Err(e) => {
                        self.warning_manager.warnf(
                            WarningCategory::Fonts,
                            format!("Failed to read font file '{}': {}", path.display(), e)
                        );
                        None
                    }
                }
            }
            Source::SharedFile(path, _) => {
                // Read font file and create Font
                match std::fs::read(path) {
                    Ok(data) => Font::new(data.into(), font_index),
                    Err(e) => {
                        self.warning_manager.warnf(
                            WarningCategory::Fonts,
                            format!("Failed to read shared font file '{}': {}", path.display(), e)
                        );
                        None
                    }
                }
            }
        }
    }

    /// List all available monospace fonts (for debugging)
    #[allow(dead_code)]
    pub fn list_monospace_fonts(&self) -> Vec<String> {
        let mut fonts = Vec::new();

        for face in self.db.faces() {
            if face.monospaced {
                for family in &face.families {
                    fonts.push(family.0.clone());
                }
            }
        }

        fonts.sort();
        fonts.dedup();
        fonts
    }
}

impl Default for FontManager {
    fn default() -> Self {
        Self {
            db: {
                let mut db = Database::new();
                db.load_system_fonts();
                db
            },
            monospace_font: None,
            cover_font: None,
            cover_bold_font: None,
            warning_manager: Arc::new(WarningManager::new(false)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_font_manager_creation() {
        let manager = FontManager::new(Arc::new(WarningManager::new(false)));
        assert!(manager.db.faces().count() > 0, "Should load system fonts");
    }

    #[test]
    fn test_find_monospace_font() {
        let mut manager = FontManager::new(Arc::new(WarningManager::new(false)));
        let result = manager.get_monospace_font();
        assert!(result.is_ok(), "Should find at least one monospace font");
    }
}
