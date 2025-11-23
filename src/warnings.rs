use std::collections::HashSet;
use std::io::Write;
use std::sync::Mutex;

/// Categories of warnings that can be selectively enabled/disabled
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WarningCategory {
    Fonts,
    Themes,
    Highlighting,
    Filesystem,
}

impl WarningCategory {
    /// Parse a category from a string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "fonts" => Some(Self::Fonts),
            "themes" => Some(Self::Themes),
            "highlighting" => Some(Self::Highlighting),
            "filesystem" => Some(Self::Filesystem),
            _ => None,
        }
    }

    /// Get the string representation of a category
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Fonts => "fonts",
            Self::Themes => "themes",
            Self::Highlighting => "highlighting",
            Self::Filesystem => "filesystem",
        }
    }
}

/// Manages warning output with configurable categories
pub struct WarningManager {
    enabled: bool,
    silenced_categories: Mutex<HashSet<WarningCategory>>,
}

impl WarningManager {
    /// Create a new warning manager with warnings enabled
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            silenced_categories: Mutex::new(HashSet::new()),
        }
    }

    /// Silence a specific category of warnings
    pub fn silence_category(&self, category: WarningCategory) {
        if let Ok(mut silenced) = self.silenced_categories.lock() {
            silenced.insert(category);
        }
    }

    /// Silence multiple categories at once
    pub fn silence_categories(&self, categories: &[WarningCategory]) {
        if let Ok(mut silenced) = self.silenced_categories.lock() {
            for category in categories {
                silenced.insert(*category);
            }
        }
    }

    /// Check if a category is silenced
    fn is_category_silenced(&self, category: WarningCategory) -> bool {
        if let Ok(silenced) = self.silenced_categories.lock() {
            silenced.contains(&category)
        } else {
            false
        }
    }

    /// Emit a warning to stderr if enabled and not silenced
    pub fn warn(&self, category: WarningCategory, message: &str) {
        if !self.enabled || self.is_category_silenced(category) {
            return;
        }

        let category_str = category.as_str();
        let _ = writeln!(
            std::io::stderr(),
            "\x1b[33mWarning\x1b[0m [\x1b[36m{}\x1b[0m]: {}",
            category_str, message
        );
    }

    /// Emit a warning with formatted arguments
    pub fn warnf(&self, category: WarningCategory, message: String) {
        self.warn(category, &message);
    }
}

impl Default for WarningManager {
    fn default() -> Self {
        Self::new(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_category_from_str() {
        assert_eq!(WarningCategory::from_str("fonts"), Some(WarningCategory::Fonts));
        assert_eq!(WarningCategory::from_str("Fonts"), Some(WarningCategory::Fonts));
        assert_eq!(WarningCategory::from_str("themes"), Some(WarningCategory::Themes));
        assert_eq!(WarningCategory::from_str("highlighting"), Some(WarningCategory::Highlighting));
        assert_eq!(WarningCategory::from_str("filesystem"), Some(WarningCategory::Filesystem));
        assert_eq!(WarningCategory::from_str("invalid"), None);
    }

    #[test]
    fn test_warning_manager_enabled() {
        let manager = WarningManager::new(true);
        // Can't test actual stderr output in unit tests easily,
        // but we can verify the manager is created
        assert!(manager.enabled);
    }

    #[test]
    fn test_warning_manager_disabled() {
        let manager = WarningManager::new(false);
        assert!(!manager.enabled);
    }

    #[test]
    fn test_silence_category() {
        let manager = WarningManager::new(true);
        manager.silence_category(WarningCategory::Fonts);
        assert!(manager.is_category_silenced(WarningCategory::Fonts));
        assert!(!manager.is_category_silenced(WarningCategory::Themes));
    }

    #[test]
    fn test_silence_multiple_categories() {
        let manager = WarningManager::new(true);
        manager.silence_categories(&[WarningCategory::Fonts, WarningCategory::Themes]);
        assert!(manager.is_category_silenced(WarningCategory::Fonts));
        assert!(manager.is_category_silenced(WarningCategory::Themes));
        assert!(!manager.is_category_silenced(WarningCategory::Highlighting));
    }
}
