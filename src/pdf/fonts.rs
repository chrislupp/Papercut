use crate::error::{PapercutError, Result};
use fontdb::{Database, Query, Source};
use krilla::text::Font;
use std::sync::Arc;

/// Manages font loading and caching for PDF generation
pub struct FontManager {
    db: Database,
    monospace_font: Option<Arc<Font>>,
}

impl FontManager {
    /// Create a new font manager and load system fonts
    pub fn new() -> Self {
        let mut db = Database::new();

        // Load system fonts
        db.load_system_fonts();

        Self {
            db,
            monospace_font: None,
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
                self.monospace_font = Some(Arc::new(font));
                return Ok(Arc::clone(self.monospace_font.as_ref().unwrap()));
            }
        }

        Err(PapercutError::InvalidConfig(
            "Could not find any suitable monospace font. Please install a monospace font like \
             Consolas, Monaco, or DejaVu Sans Mono.".to_string()
        ))
    }

    /// Try to load a font by family name
    fn try_load_font(&self, family: &str) -> Option<Font> {
        // Query for a regular (non-bold, non-italic) version of the font
        let query = Query {
            families: &[fontdb::Family::Name(family)],
            weight: fontdb::Weight::NORMAL,
            stretch: fontdb::Stretch::Normal,
            style: fontdb::Style::Normal,
        };

        let id = self.db.query(&query)?;
        let face = self.db.face(id)?;

        // Load the font data
        match &face.source {
            Source::Binary(data) => {
                // Try to create a Font from the binary data
                let vec_data = data.as_ref().as_ref().to_vec();
                Font::new(vec_data.into(), 0)
            }
            Source::File(path) => {
                // Read font file and create Font
                if let Ok(data) = std::fs::read(path) {
                    Font::new(data.into(), 0)
                } else {
                    None
                }
            }
            Source::SharedFile(path, _) => {
                // Read font file and create Font
                if let Ok(data) = std::fs::read(path) {
                    Font::new(data.into(), 0)
                } else {
                    None
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
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_font_manager_creation() {
        let manager = FontManager::new();
        assert!(manager.db.faces().count() > 0, "Should load system fonts");
    }

    #[test]
    fn test_find_monospace_font() {
        let mut manager = FontManager::new();
        let result = manager.get_monospace_font();
        assert!(result.is_ok(), "Should find at least one monospace font");
    }
}
