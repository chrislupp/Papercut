use crate::config::Config;
use crate::pdf::text_layout::MonospaceMetrics;
use krilla::geom::Point;
use krilla::surface::Surface;
use krilla::text::Font;
use std::path::Path;

/// Represents a code block with lstlistings-style formatting
///
/// NOTE: Currently renders plain text with line numbers.
/// TODO: Add borders, backgrounds, and colored syntax highlighting
pub struct CodeBlock {
    pub(crate) code: String,
    _file_path: Option<std::path::PathBuf>,
    show_line_numbers: bool,
    font_size: f32,
    line_spacing: f32,
    padding: f32,
}

impl CodeBlock {
    /// Create a new code block
    pub fn new(code: String, file_path: Option<&Path>) -> Self {
        Self {
            code,
            _file_path: file_path.map(|p| p.to_path_buf()),
            show_line_numbers: true,
            font_size: 10.0,
            line_spacing: 1.2,
            padding: 12.0,
        }
    }

    /// Configure from Config struct
    pub fn from_config(code: String, file_path: Option<&Path>, config: &Config) -> Self {
        Self {
            code,
            _file_path: file_path.map(|p| p.to_path_buf()),
            show_line_numbers: config.page.line_numbers,
            font_size: config.page.font_size as f32,
            line_spacing: config.page.line_spacing,
            padding: 12.0,
        }
    }

    /// Render the code block to the surface at the given position
    /// Returns the height used by the code block
    pub fn render(
        &self,
        surface: &mut Surface,
        font: &Font,
        x: f32,
        y: f32,
    ) -> f32 {
        let metrics = MonospaceMetrics::new(self.font_size, self.line_spacing);

        // Get lines
        let lines: Vec<&str> = self.code.lines().collect();
        let line_count = lines.len().max(1);

        // Calculate dimensions
        let line_num_width = if self.show_line_numbers {
            let max_line_num_digits = (line_count as f64).log10().floor() as usize + 1;
            (max_line_num_digits as f32 + 2.0) * metrics.char_width()
        } else {
            0.0
        };

        let gutter_width = if self.show_line_numbers {
            line_num_width + metrics.char_width()
        } else {
            0.0
        };

        // Render code lines
        for (i, line) in lines.iter().enumerate() {
            let line_y = y + self.padding + (i as f32) * metrics.line_height() + self.font_size;

            // Draw line number
            if self.show_line_numbers {
                let line_num_text = format!("{:>4} ", i + 1);

                surface.draw_text(
                    Point::from_xy(x + self.padding, line_y),
                    font.clone(),
                    self.font_size,
                    &line_num_text,
                    false,
                    krilla::text::TextDirection::Auto,
                );
            }

            // Draw code text
            surface.draw_text(
                Point::from_xy(x + self.padding + gutter_width, line_y),
                font.clone(),
                self.font_size,
                line,
                false,
                krilla::text::TextDirection::Auto,
            );
        }

        let content_height = metrics.total_height(line_count);
        content_height + 2.0 * self.padding
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_block_creation() {
        let code = "fn main() {\n    println!(\"Hello\");\n}".to_string();
        let block = CodeBlock::new(code, None);
        assert_eq!(block.font_size, 10.0);
        assert_eq!(block.show_line_numbers, true);
    }
}
