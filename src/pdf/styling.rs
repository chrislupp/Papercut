use crate::config::{PageSize, FontFamily};

/// Get page dimensions in millimeters based on page size
pub fn get_page_size(size: &PageSize) -> (f64, f64) {
    match size {
        PageSize::A4 => (210.0, 297.0),      // A4: 210mm x 297mm
        PageSize::Letter => (215.9, 279.4),  // Letter: 8.5" x 11"
        PageSize::Legal => (215.9, 355.6),   // Legal: 8.5" x 14"
    }
}

/// Get the font name based on font family
pub fn get_font_name(family: &FontFamily) -> &'static str {
    match family {
        FontFamily::Monospace => "monospace",
        FontFamily::Courier => "courier",
        FontFamily::DejaVu => "dejavu",
    }
}

/// Parse hex color to RGB tuple (0.0-1.0 range)
pub fn parse_hex_color(hex: &str) -> Result<(f32, f32, f32), String> {
    let hex = hex.trim_start_matches('#');

    if hex.len() != 6 {
        return Err(format!("Invalid hex color: {}", hex));
    }

    let r = u8::from_str_radix(&hex[0..2], 16)
        .map_err(|_| format!("Invalid hex color: {}", hex))?;
    let g = u8::from_str_radix(&hex[2..4], 16)
        .map_err(|_| format!("Invalid hex color: {}", hex))?;
    let b = u8::from_str_radix(&hex[4..6], 16)
        .map_err(|_| format!("Invalid hex color: {}", hex))?;

    Ok((
        r as f32 / 255.0,
        g as f32 / 255.0,
        b as f32 / 255.0,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hex_color() {
        assert_eq!(parse_hex_color("#ffffff"), Ok((1.0, 1.0, 1.0)));
        assert_eq!(parse_hex_color("#000000"), Ok((0.0, 0.0, 0.0)));
        assert_eq!(parse_hex_color("#ff0000"), Ok((1.0, 0.0, 0.0)));
        assert!(parse_hex_color("#fff").is_err());
        assert!(parse_hex_color("notahex").is_err());
    }
}
