use krilla::color::rgb;
use krilla::paint::Paint;

#[cfg(feature = "syntax-highlighting")]
use syntect::highlighting::Color as SyntectColor;

/// Convert a syntect Color to a krilla Paint
#[cfg(feature = "syntax-highlighting")]
pub fn syntect_to_paint(color: SyntectColor) -> Paint {
    rgb::Color::new(color.r, color.g, color.b).into()
}

/// Convert RGB values (0-255) to a krilla Paint
pub fn rgb_to_paint(r: u8, g: u8, b: u8) -> Paint {
    rgb::Color::new(r, g, b).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rgb_to_paint() {
        let _paint = rgb_to_paint(255, 0, 0);
        // Paint doesn't expose internal structure for testing
    }

    #[cfg(feature = "syntax-highlighting")]
    #[test]
    fn test_syntect_to_paint() {
        let color = SyntectColor { r: 255, g: 0, b: 0, a: 255 };
        let _paint = syntect_to_paint(color);
        // Paint doesn't expose internal structure for testing
    }
}
