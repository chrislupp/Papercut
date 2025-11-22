use parley::{FontContext, LayoutContext};

/// Helper for text layout and measurement using parley
pub struct TextLayoutHelper {
    font_cx: FontContext,
    layout_cx: LayoutContext,
}

impl TextLayoutHelper {
    pub fn new() -> Self {
        Self {
            font_cx: FontContext::new(),
            layout_cx: LayoutContext::new(),
        }
    }

    /// Measure the width of a text string
    /// For monospace fonts, this is approximately: char_count * char_width
    /// But parley can give us more accurate measurements
    pub fn measure_text_width(&mut self, text: &str, font_size: f32) -> f32 {
        // For now, use a simple monospace approximation
        // In monospace fonts, each character is roughly 0.6 * font_size wide
        text.len() as f32 * font_size * 0.6
    }

    /// Get the height for a line of text (line height = font_size * line_spacing)
    pub fn get_line_height(&self, font_size: f32, line_spacing: f32) -> f32 {
        font_size * line_spacing
    }

    /// Calculate how many characters fit in a given width (for monospace)
    pub fn chars_per_line(&self, width: f32, font_size: f32) -> usize {
        let char_width = font_size * 0.6;
        (width / char_width).floor() as usize
    }
}

impl Default for TextLayoutHelper {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple text measurements for monospace fonts
/// This provides a lightweight alternative to full parley layouts
pub struct MonospaceMetrics {
    char_width: f32,
    line_height: f32,
}

impl MonospaceMetrics {
    /// Create metrics for a monospace font
    pub fn new(font_size: f32, line_spacing: f32) -> Self {
        Self {
            char_width: font_size * 0.6, // Approximate monospace width
            line_height: font_size * line_spacing,
        }
    }

    /// Get the width of a string
    pub fn text_width(&self, text: &str) -> f32 {
        text.len() as f32 * self.char_width
    }

    /// Get line height
    pub fn line_height(&self) -> f32 {
        self.line_height
    }

    /// Get character width
    pub fn char_width(&self) -> f32 {
        self.char_width
    }

    /// Calculate number of characters that fit in width
    pub fn chars_per_line(&self, width: f32) -> usize {
        (width / self.char_width).floor() as usize
    }

    /// Calculate total height for multiple lines
    pub fn total_height(&self, line_count: usize) -> f32 {
        line_count as f32 * self.line_height
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monospace_metrics() {
        let metrics = MonospaceMetrics::new(12.0, 1.5);

        // Character width should be approx 0.6 * font_size
        assert!((metrics.char_width() - 7.2).abs() < 0.1);

        // Line height should be font_size * line_spacing
        assert!((metrics.line_height() - 18.0).abs() < 0.1);
    }

    #[test]
    fn test_text_width() {
        let metrics = MonospaceMetrics::new(10.0, 1.0);
        let width = metrics.text_width("hello");

        // 5 characters * (10 * 0.6) = 30
        assert!((width - 30.0).abs() < 0.1);
    }

    #[test]
    fn test_chars_per_line() {
        let metrics = MonospaceMetrics::new(10.0, 1.0);
        let chars = metrics.chars_per_line(100.0);

        // 100 / (10 * 0.6) = 16.666... -> floor = 16
        assert_eq!(chars, 16);
    }
}
