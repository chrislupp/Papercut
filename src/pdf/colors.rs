use crate::error::Result;
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

/// Parse a hex color string (e.g., "#ff0000") to a krilla Paint
pub fn hex_to_paint(hex: &str) -> Result<Paint> {
    let hex = hex.trim_start_matches('#');

    let (r, g, b) = match hex.len() {
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16)
                .map_err(|e| crate::error::PapercutError::InvalidConfig(
                    format!("Invalid hex color: {}", e)
                ))?;
            let g = u8::from_str_radix(&hex[2..4], 16)
                .map_err(|e| crate::error::PapercutError::InvalidConfig(
                    format!("Invalid hex color: {}", e)
                ))?;
            let b = u8::from_str_radix(&hex[4..6], 16)
                .map_err(|e| crate::error::PapercutError::InvalidConfig(
                    format!("Invalid hex color: {}", e)
                ))?;
            (r, g, b)
        }
        _ => {
            return Err(crate::error::PapercutError::InvalidConfig(
                format!("Invalid hex color format: {}", hex)
            ));
        }
    };

    Ok(rgb_to_paint(r, g, b))
}

/// Common colors for code blocks
pub mod colors {
    use super::*;

    // Dark theme colors (base16-ocean.dark style)
    pub fn dark_background() -> Paint {
        rgb_to_paint(0x2b, 0x30, 0x3b) // #2b303b
    }

    pub fn dark_border() -> Paint {
        rgb_to_paint(0x4f, 0x5b, 0x66) // #4f5b66
    }

    pub fn dark_line_numbers() -> Paint {
        rgb_to_paint(0x65, 0x73, 0x7e) // #65737e
    }

    pub fn dark_text() -> Paint {
        rgb_to_paint(0xc0, 0xc5, 0xce) // #c0c5ce
    }

    // Light theme colors
    pub fn light_background() -> Paint {
        rgb_to_paint(0xfa, 0xfa, 0xfa) // #fafafa
    }

    pub fn light_border() -> Paint {
        rgb_to_paint(0xcc, 0xcc, 0xcc) // #cccccc
    }

    pub fn light_line_numbers() -> Paint {
        rgb_to_paint(0x88, 0x88, 0x88) // #888888
    }

    pub fn light_text() -> Paint {
        rgb_to_paint(0x00, 0x00, 0x00) // #000000
    }

    // Generic colors
    pub fn black() -> Paint {
        rgb_to_paint(0, 0, 0)
    }

    pub fn white() -> Paint {
        rgb_to_paint(255, 255, 255)
    }

    pub fn gray(value: u8) -> Paint {
        rgb_to_paint(value, value, value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rgb_to_paint() {
        let _paint = rgb_to_paint(255, 0, 0);
        // Paint doesn't expose internal structure for testing
    }

    #[test]
    fn test_hex_to_paint() {
        let result = hex_to_paint("#ff0000");
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_hex() {
        let result = hex_to_paint("#ff");
        assert!(result.is_err());
    }

    #[cfg(feature = "syntax-highlighting")]
    #[test]
    fn test_syntect_to_paint() {
        let color = SyntectColor { r: 255, g: 0, b: 0, a: 255 };
        let _paint = syntect_to_paint(color);
        // Paint doesn't expose internal structure for testing
    }
}
