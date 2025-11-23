use syntect::highlighting::{Theme, ThemeSet};

/// Built-in theme presets that map to syntect's default themes
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

    /// Get the underlying syntect theme name that this preset maps to
    fn syntect_theme_name(&self) -> &'static str {
        match self {
            Self::VsCodeDark => "base16-ocean.dark",
            Self::VsCodeLight => "InspiredGitHub",
            Self::JetBrainsDarcula => "base16-mocha.dark",
            Self::JetBrainsLight => "Solarized (light)",
        }
    }

    /// Load the theme from syntect's built-in themes
    pub fn load_theme(&self) -> Option<Theme> {
        let ts = ThemeSet::load_defaults();
        ts.themes.get(self.syntect_theme_name()).cloned()
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
        let theme = ThemePreset::VsCodeDark.load_theme();
        assert!(theme.is_some());
        let theme = theme.unwrap();
        assert_eq!(theme.name, Some("Base16 Ocean Dark".to_string()));
    }
}
