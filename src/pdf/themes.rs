use syntect::highlighting::{Theme, ThemeSet, ThemeSettings, Color, StyleModifier, ScopeSelectors};
use std::str::FromStr;

/// Built-in theme presets
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ThemePreset {
    VsCodeDark,
    VsCodeLight,
    JetBrainsDarcula,
    JetBrainsLight,
}

impl ThemePreset {
    /// Parse theme preset from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "vscode-dark" | "vscode_dark" => Some(Self::VsCodeDark),
            "vscode-light" | "vscode_light" => Some(Self::VsCodeLight),
            "jetbrains-darcula" | "jetbrains_darcula" | "darcula" => Some(Self::JetBrainsDarcula),
            "jetbrains-light" | "jetbrains_light" => Some(Self::JetBrainsLight),
            _ => None,
        }
    }

    /// Get the theme name as a string
    pub fn name(&self) -> &'static str {
        match self {
            Self::VsCodeDark => "vscode-dark",
            Self::VsCodeLight => "vscode-light",
            Self::JetBrainsDarcula => "jetbrains-darcula",
            Self::JetBrainsLight => "jetbrains-light",
        }
    }

    /// Load the theme
    pub fn load_theme(&self) -> Option<Theme> {
        match self {
            Self::VsCodeLight => Some(create_vscode_light_modern_theme()),
            _ => {
                // Load from syntect defaults for other themes
                let ts = ThemeSet::load_defaults();
                let theme_name = match self {
                    Self::VsCodeDark => "base16-ocean.dark",
                    Self::JetBrainsDarcula => "base16-mocha.dark",
                    Self::JetBrainsLight => "Solarized (light)",
                    _ => return None,
                };
                ts.themes.get(theme_name).cloned()
            }
        }
    }
}

fn create_vscode_light_modern_theme() -> Theme {
    use syntect::highlighting::ThemeItem;

    let settings = ThemeSettings {
        foreground: Some(Color { r: 0x3B, g: 0x3B, b: 0x3B, a: 0xFF }),
        background: Some(Color { r: 0xFF, g: 0xFF, b: 0xFF, a: 0xFF }),
        caret: Some(Color { r: 0x00, g: 0x00, b: 0x00, a: 0xFF }),
        line_highlight: Some(Color { r: 0x00, g: 0x00, b: 0x00, a: 0x14 }),
        selection: Some(Color { r: 0xAD, g: 0xD6, b: 0xFF, a: 0x80 }),
        ..Default::default()
    };

    let items = vec![
        // Comments - green
        ThemeItem {
            scope: ScopeSelectors::from_str("comment").expect("Built-in theme scope selector should be valid"),
            style: StyleModifier {
                foreground: Some(Color { r: 0x00, g: 0x80, b: 0x00, a: 0xFF }),
                ..Default::default()
            },
        },
        // Strings - red/maroon
        ThemeItem {
            scope: ScopeSelectors::from_str("string").expect("Built-in theme scope selector should be valid"),
            style: StyleModifier {
                foreground: Some(Color { r: 0xA3, g: 0x15, b: 0x15, a: 0xFF }),
                ..Default::default()
            },
        },
        // Numbers - teal
        ThemeItem {
            scope: ScopeSelectors::from_str("constant.numeric").expect("Built-in theme scope selector should be valid"),
            style: StyleModifier {
                foreground: Some(Color { r: 0x09, g: 0x88, b: 0x5A, a: 0xFF }),
                ..Default::default()
            },
        },
        // Functions - brown/gold
        ThemeItem {
            scope: ScopeSelectors::from_str("entity.name.function, support.function").expect("Built-in theme scope selector should be valid"),
            style: StyleModifier {
                foreground: Some(Color { r: 0x79, g: 0x5E, b: 0x26, a: 0xFF }),
                ..Default::default()
            },
        },
        // Types - teal
        ThemeItem {
            scope: ScopeSelectors::from_str("support.class, support.type, entity.name.type, storage.type").expect("Built-in theme scope selector should be valid"),
            style: StyleModifier {
                foreground: Some(Color { r: 0x26, g: 0x7F, b: 0x99, a: 0xFF }),
                ..Default::default()
            },
        },
        // Control keywords - blue
        ThemeItem {
            scope: ScopeSelectors::from_str("keyword.control").expect("Built-in theme scope selector should be valid"),
            style: StyleModifier {
                foreground: Some(Color { r: 0x00, g: 0x00, b: 0xFF, a: 0xFF }),
                ..Default::default()
            },
        },
        // Variables - dark blue
        ThemeItem {
            scope: ScopeSelectors::from_str("variable, entity.name.variable").expect("Built-in theme scope selector should be valid"),
            style: StyleModifier {
                foreground: Some(Color { r: 0x00, g: 0x10, b: 0x80, a: 0xFF }),
                ..Default::default()
            },
        },
        // Keywords (general) - blue
        ThemeItem {
            scope: ScopeSelectors::from_str("keyword, storage, storage.modifier").expect("Built-in theme scope selector should be valid"),
            style: StyleModifier {
                foreground: Some(Color { r: 0x00, g: 0x00, b: 0xFF, a: 0xFF }),
                ..Default::default()
            },
        },
        // keyword.operator - black
        ThemeItem {
            scope: ScopeSelectors::from_str("keyword.operator").expect("Built-in theme scope selector should be valid"),
            style: StyleModifier {
                foreground: Some(Color { r: 0x00, g: 0x00, b: 0x00, a: 0xFF }),
                ..Default::default()
            },
        },
        // constant.language - blue
        ThemeItem {
            scope: ScopeSelectors::from_str("constant.language").expect("Built-in theme scope selector should be valid"),
            style: StyleModifier {
                foreground: Some(Color { r: 0x00, g: 0x00, b: 0xFF, a: 0xFF }),
                ..Default::default()
            },
        },
        // constant.character - blue
        ThemeItem {
            scope: ScopeSelectors::from_str("constant.character").expect("Built-in theme scope selector should be valid"),
            style: StyleModifier {
                foreground: Some(Color { r: 0x00, g: 0x00, b: 0xFF, a: 0xFF }),
                ..Default::default()
            },
        },
        // variable.language - blue
        ThemeItem {
            scope: ScopeSelectors::from_str("variable.language").expect("Built-in theme scope selector should be valid"),
            style: StyleModifier {
                foreground: Some(Color { r: 0x00, g: 0x00, b: 0xFF, a: 0xFF }),
                ..Default::default()
            },
        },
        // entity.name.tag - maroon
        ThemeItem {
            scope: ScopeSelectors::from_str("entity.name.tag").expect("Built-in theme scope selector should be valid"),
            style: StyleModifier {
                foreground: Some(Color { r: 0x80, g: 0x00, b: 0x00, a: 0xFF }),
                ..Default::default()
            },
        },
        // entity.other.attribute-name - red
        ThemeItem {
            scope: ScopeSelectors::from_str("entity.other.attribute-name").expect("Built-in theme scope selector should be valid"),
            style: StyleModifier {
                foreground: Some(Color { r: 0xFF, g: 0x00, b: 0x00, a: 0xFF }),
                ..Default::default()
            },
        },
        // meta.preprocessor - blue
        ThemeItem {
            scope: ScopeSelectors::from_str("meta.preprocessor").expect("Built-in theme scope selector should be valid"),
            style: StyleModifier {
                foreground: Some(Color { r: 0x00, g: 0x00, b: 0xFF, a: 0xFF }),
                ..Default::default()
            },
        },
        // invalid - red
        ThemeItem {
            scope: ScopeSelectors::from_str("invalid").expect("Built-in theme scope selector should be valid"),
            style: StyleModifier {
                foreground: Some(Color { r: 0xCD, g: 0x31, b: 0x31, a: 0xFF }),
                ..Default::default()
            },
        },
    ];

    Theme {
        name: Some("VSCode Light Modern".to_string()),
        author: Some("Microsoft".to_string()),
        settings: settings,
        scopes: items,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_preset_parsing() {
        assert_eq!(ThemePreset::from_str("vscode-dark"), Some(ThemePreset::VsCodeDark));
        assert_eq!(ThemePreset::from_str("VsCode_Dark"), Some(ThemePreset::VsCodeDark));
        assert_eq!(ThemePreset::from_str("jetbrains-darcula"), Some(ThemePreset::JetBrainsDarcula));
        assert_eq!(ThemePreset::from_str("darcula"), Some(ThemePreset::JetBrainsDarcula));
        assert_eq!(ThemePreset::from_str("unknown"), None);
    }

    #[test]
    fn test_theme_loading() {
        let theme = ThemePreset::VsCodeLight.load_theme();
        assert!(theme.is_some());
    }
}
