// Papercut - Source code to PDF converter
// Copyright (C) 2025-2026 Christopher A. Lupp
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
        let color = SyntectColor {
            r: 255,
            g: 0,
            b: 0,
            a: 255,
        };
        let _paint = syntect_to_paint(color);
        // Paint doesn't expose internal structure for testing
    }
}
